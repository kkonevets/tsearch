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
use tantivy::schema::Term;
use tsearch::models::Post;
use tsearch::preprocess;
use tsearch::state::SearchState;

#[derive(Fail, Debug)]
#[fail(display = "Search engine error")]
struct SearchEngineError {
    name: String,
}

impl From<tantivy::Error> for SearchEngineError {
    fn from(e: tantivy::Error) -> Self {
        SearchEngineError {
            name: e.to_string(),
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
struct SearchInfo {
    query: String,
    topk: usize,
}

fn search_index(
    (info, state): (Json<SearchInfo>, State<SearchState>),
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

#[derive(Deserialize)]
struct ModifyInfo {
    overwrite: bool,
    delete: bool,
    post: Post,
}

fn modify_index(
    (info, state): (Json<ModifyInfo>, State<SearchState>),
) -> Result<HttpResponse, SearchEngineError> {
    let schema = &state.schema;

    let thread_id_f = schema.get_field("thread_id").unwrap();
    let thread_id_term = Term::from_field_i64(thread_id_f, info.post.thread_id);

    let mut index_writer = match state.index.writer_with_num_threads(1, 5_000_000) {
        Ok(v) => v,
        Err(e) => return Err(SearchEngineError::from(e)),
    };

    std::thread::sleep(std::time::Duration::from_secs(30));

    if info.delete {
        index_writer.delete_term(thread_id_term.clone());
    } else {
        if info.overwrite {
        } else {
        }
    }

    index_writer.commit()?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body("{\"sucess\": true}"))
}

fn main() {
    let host = "0.0.0.0:8080";

    let sys = actix::System::new("searcher");

    HttpServer::new(|| {
        vec![
            App::with_state(SearchState::new().unwrap())
                .prefix("/search")
                .resource("", |r| r.method(http::Method::POST).with(search_index)),
            App::with_state(SearchState::new().unwrap())
                .prefix("/modify")
                .resource("", |r| r.method(http::Method::POST).with(modify_index)),
        ]
    })
    .bind(host)
    .unwrap()
    .start();

    println!("Started http server: {}", host);
    let _ = sys.run();
}
