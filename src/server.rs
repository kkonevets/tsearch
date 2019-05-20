extern crate actix;
extern crate actix_web;
extern crate common;
extern crate tantivy;

#[macro_use]
extern crate failure;
extern crate serde_derive;

use actix_web::{error, http, server::HttpServer, App, HttpResponse, Json, State};
use common::{preprocess, register_tokenizer};
use serde::Deserialize;
use std::sync::Arc;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::{Index, IndexReader, ReloadPolicy};

#[derive(Fail, Debug)]
#[fail(display = "Search engine error")]
struct SearchEngineError {
    name: String,
}

impl From<tantivy::Error> for SearchEngineError {
    fn from(e: tantivy::Error) -> Self {
        SearchEngineError {
            name: format!("{:?}", e),
        }
    }
}

impl From<tantivy::query::QueryParserError> for SearchEngineError {
    fn from(e: tantivy::query::QueryParserError) -> Self {
        SearchEngineError {
            name: format!("{:?}", e),
        }
    }
}

impl error::ResponseError for SearchEngineError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::InternalServerError()
            .content_type("text/html")
            .body(&self.name)
    }
}

struct SearchState {
    reader: Arc<IndexReader>,
    query_parser: Arc<QueryParser>,
    schema: Arc<tantivy::schema::Schema>,
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

        // let title_t = schema.get_field("title").unwrap();
        let text_t = schema.get_field("text").unwrap();

        let query_parser = QueryParser::for_index(&index, vec![text_t]);

        let state = SearchState {
            reader: Arc::new(reader),
            query_parser: Arc::new(query_parser),
            schema: Arc::new(schema),
        };

        Ok(state)
    }
}

#[derive(Deserialize)]
struct PostInfo {
    query: String,
    topk: usize,
}

fn index(
    (info, state): (Json<PostInfo>, State<SearchState>),
) -> Result<HttpResponse, SearchEngineError> {
    let searcher = state.reader.searcher();
    let qtext = preprocess(info.query.as_str());
    let query = match state.query_parser.parse_query(qtext.as_str()) {
        Ok(v) => v,
        Err(e) => {
            return Err(SearchEngineError::from(e));
        }
    };

    let top_docs = match searcher.search(&query, &TopDocs::with_limit(info.topk)) {
        Ok(v) => v,
        Err(e) => return Err(SearchEngineError::from(e)),
    };

    let schema = &state.schema;
    let mut docs = Vec::new();
    for (_score, doc_address) in top_docs {
        let retrieved_doc = match searcher.doc(doc_address) {
            Ok(v) => v,
            Err(e) => return Err(SearchEngineError::from(e)),
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

    HttpServer::new(|| {
        App::with_state(SearchState::new().unwrap())
            .resource("/", |r| r.method(http::Method::POST).with(index))
    })
    .bind("0.0.0.0:8080")
    .unwrap()
    .start();

    println!("Started http server: 0.0.0.0:8080");
    let _ = sys.run();
}
