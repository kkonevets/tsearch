extern crate actix;
extern crate actix_web;
#[macro_use]
extern crate tantivy;
extern crate tsearch;

#[macro_use]
extern crate failure;
extern crate serde_derive;

use actix_web::{error, http, server::HttpServer, App, HttpResponse, Json, State};
use serde::Deserialize;
use tantivy::collector::TopDocs;
use tantivy::schema::Term;
use tsearch::crud::doc_by_id;
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
    posts: std::vec::Vec<Post>,
}

fn modify_index(
    (info, state): (Json<ModifyInfo>, State<SearchState>),
) -> Result<HttpResponse, SearchEngineError> {
    let schema = &state.schema;

    let mut writer = match state.index.writer(50_000_000) {
        Ok(v) => v,
        Err(e) => return Err(SearchEngineError::from(e)),
    };

    let thread_id_f = schema.get_field("thread_id").unwrap();
    let title_f = schema.get_field("title").unwrap();
    let text_f = schema.get_field("text").unwrap();
    let node_id_f = schema.get_field("node_id").unwrap();
    let need_moder_f = schema.get_field("need_moder_id").unwrap();
    let post_date_f = schema.get_field("post_date_id").unwrap();

    // std::thread::sleep(std::time::Duration::from_secs(30));

    for post in &info.posts {
        let thread_id_term = Term::from_field_i64(thread_id_f, post.thread_id);

        if info.delete {
            writer.delete_term(thread_id_term.clone());
        } else {
            match doc_by_id(&state.reader, &thread_id_term)? {
                Some(_) => {
                    if info.overwrite {
                        writer.delete_term(thread_id_term.clone());
                    } else {
                        // document already exists, do nothing
                        continue;
                    }
                }
                None => (),
            };

            writer.add_document(doc!(
                thread_id_f => post.thread_id,
                title_f => preprocess(&post.title),
                text_f => preprocess(&post.text),
                node_id_f => post.node_id,
                need_moder_f => post.needModer,
                post_date_f => post.post_date

            ));
        }
    }

    writer.commit()?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body("{\"sucess\": true}"))
}

fn main() {
    let host = "0.0.0.0:8080";

    let sys = actix::System::new("searcher");
    let search_state = SearchState::new().unwrap();

    HttpServer::new(move || {
        vec![
            App::with_state(search_state.clone())
                .prefix("/search")
                .resource("", |r| r.method(http::Method::POST).with(search_index))
                .boxed(),
            App::with_state(search_state.clone())
                .prefix("/modify")
                .resource("", |r| r.method(http::Method::POST).with(modify_index))
                .boxed(),
        ]
    })
    .bind(host)
    .unwrap()
    .start();

    println!("Started http server: {}", host);
    let _ = sys.run();
}
