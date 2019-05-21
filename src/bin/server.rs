extern crate actix;
extern crate actix_web;
extern crate tantivy;
extern crate tsearch;

#[macro_use]
extern crate failure;
extern crate serde_derive;

use actix_web::{error, http, server::HttpServer, App, HttpResponse, Json, State};
use serde::Deserialize;
use tantivy::collector::TopDocs;
use tsearch::models::Post;
use tsearch::preprocess;
use tsearch::state::SearchState;

use std::sync::mpsc;

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
        Err(e) => return Err(SearchEngineError::from(e)),
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
    let host = "0.0.0.0:8080";

    let sys = actix::System::new("searcher");

    let (tx, rx) = mpsc::channel::<Post>();

    HttpServer::new(|| {
        App::with_state(SearchState::new().unwrap())
            .resource("/", |r| r.method(http::Method::POST).with(index))
    })
    .bind(host)
    .unwrap()
    .start();

    println!("Started http server: {}", host);
    let _ = sys.run();
}
