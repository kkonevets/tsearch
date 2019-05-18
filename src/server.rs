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

use actix_web::{error, http, server, App, Error, HttpRequest, HttpResponse, Responder};

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
}

impl SearchState {
    pub fn new() -> Result<SearchState, SearchEngineError> {
        let index_path = "./index";

        let index = match Index::open_in_dir(index_path) {
            Ok(i) => i,
            Err(e) => {
                return Err(SearchEngineError {
                    name: e.to_string(),
                })
            }
        };
        register_tokenizer(&index);

        let schema = index.schema();

        let reader = match index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommit)
            .try_into()
        {
            Ok(r) => r,
            Err(e) => {
                return Err(SearchEngineError {
                    name: e.to_string(),
                })
            }
        };

        let title_t = schema.get_field("title").unwrap();
        let text_t = schema.get_field("text").unwrap();

        let query_parser = QueryParser::for_index(&index, vec![title_t, text_t]);

        let state = SearchState {
            reader: RefCell::new(reader),
            query_parser: RefCell::new(query_parser),
        };

        Ok(state)
    }
}

fn index(req: &HttpRequest<SearchState>) -> Result<String, SearchEngineError> {
    let state = req.state();
    let searcher = state.reader.borrow().searcher();
    let qtext = preprocess("WOW-флорист: Сырный БУМ");
    // let query = state.query_parser.parse_query(qtext.as_str())?;
    // let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;

    Ok("".to_string())
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
