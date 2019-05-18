extern crate actix;
extern crate actix_web;
extern crate common;
extern crate tantivy;

#[macro_use]
extern crate failure;

use common::{preprocess, register_tokenizer};
use std::cell::RefCell;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::{Index, IndexReader, ReloadPolicy};

use actix_web::{error, http, server, App, HttpRequest, HttpResponse};

#[derive(Fail, Debug)]
#[fail(display = "Search engine error")]
struct SearchEngineError {
    name: String,
}

// Use default implementation for `error_response()` method
impl error::ResponseError for SearchEngineError {}

struct SearchState {
    reader: RefCell<IndexReader>,
    query_parser: RefCell<QueryParser>,
    schema: RefCell<tantivy::schema::Schema>,
}

impl SearchState {
    pub fn new() -> Result<SearchState, tantivy::TantivyError> {
        let index_path = "./index";

        let index = Index::open_in_dir(index_path)?;
        register_tokenizer(&index);

        let schema = index.schema();

        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommit)
            .try_into()?;

        let title_t = schema.get_field("title").unwrap();
        let text_t = schema.get_field("text").unwrap();

        let query_parser = QueryParser::for_index(&index, vec![title_t, text_t]);

        let state = SearchState {
            reader: RefCell::new(reader),
            query_parser: RefCell::new(query_parser),
            schema: RefCell::new(schema),
        };

        Ok(state)
    }
}

fn index(req: &HttpRequest<SearchState>) -> Result<HttpResponse, SearchEngineError> {
    let state = req.state();
    let searcher = state.reader.borrow().searcher();
    let qtext = preprocess("WOW-флорист: Сырный БУМ");
    let query = match state.query_parser.borrow().parse_query(qtext.as_str()) {
        Ok(v) => v,
        Err(_) => {
            return Err(SearchEngineError {
                name: "QueryParserError".to_string(),
            })
        }
    };

    let top_docs = match searcher.search(&query, &TopDocs::with_limit(10)) {
        Ok(v) => v,
        Err(e) => {
            return Err(SearchEngineError {
                name: e.to_string(),
            })
        }
    };

    let schema = state.schema.borrow();
    let mut docs = Vec::new();
    for (_score, doc_address) in top_docs {
        let retrieved_doc = match searcher.doc(doc_address) {
            Ok(v) => v,
            Err(e) => {
                return Err(SearchEngineError {
                    name: e.to_string(),
                })
            }
        };

        let mut doc_str = "[".to_string();
        doc_str.push_str(&_score.to_string());
        doc_str.push_str(",");
        doc_str.push_str(&schema.to_json(&retrieved_doc));
        doc_str.push_str("]");
        docs.push(doc_str);
    }

    let mut result = "[".to_string();
    result.push_str(&docs.join(","));
    result.push_str("]");

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(result))
}

fn main() {
    let sys = actix::System::new("searcher");

    server::new(|| {
        App::with_state(SearchState::new().unwrap())
            .resource("/", |r| r.method(http::Method::GET).f(index))
    })
    .bind("127.0.0.1:8088")
    .unwrap()
    .start();

    println!("Started http server: 127.0.0.1:8088");
    let _ = sys.run();
}
