extern crate actix;
extern crate actix_web;
extern crate tantivy;
extern crate tsearch;

#[macro_use]
extern crate failure;
extern crate serde_derive;

use actix_web::{error, http, server::HttpServer, App, HttpRequest, HttpResponse, Json, State};
use serde::{Deserialize, Serialize};
use tantivy::collector::TopDocs;
use tantivy::schema::Term;
use tsearch::crud::{doc_by_id, TantivyPost};
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

#[derive(Deserialize, Serialize)]
struct ModifyInfo {
    overwrite: bool,
    delete: bool,
    doc: Post,
}

fn modify_index(
    (info, state): (Json<Vec<ModifyInfo>>, State<SearchState>),
) -> Result<HttpResponse, SearchEngineError> {
    let schema = &state.schema;

    let tpost = TantivyPost::new(&schema);

    let mut writer = state.writer.lock().unwrap();

    // std::thread::sleep(std::time::Duration::from_secs(20));

    for record in &info.into_inner() {
        let post = &record.doc;
        let thread_id_term = Term::from_field_i64(tpost.thread_id_f, post.thread_id);

        if record.delete {
            writer.delete_term(thread_id_term.clone());
        } else {
            match doc_by_id(&state.reader, &thread_id_term)? {
                Some(_) => {
                    if record.overwrite {
                        writer.delete_term(thread_id_term.clone());
                    } else {
                        // document already exists, do nothing
                        continue;
                    }
                }
                None => (),
            };

            tpost.add(&post, &mut writer);
        }
    }

    writer.commit()?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body("{}"))
}

fn drop_index(req: &HttpRequest<SearchState>) -> Result<HttpResponse, SearchEngineError> {
    let state = req.state();
    let schema = &state.schema;
    let tpost = TantivyPost::new(&schema);

    let query = match state.query_parser.parse_query("*") {
        Ok(v) => v,
        Err(e) => return Err(SearchEngineError::from(e)),
    };

    // iterate in chunks over all docs and delete them
    loop {
        let searcher = state.reader.searcher();

        let top_docs = match searcher.search(&query, &TopDocs::with_limit(10_000)) {
            Ok(v) => v,
            Err(e) => return Err(SearchEngineError::from(e)),
        };

        if top_docs.len() == 0 {
            break;
        }

        let mut writer = state.writer.lock().unwrap();

        for (_, doc_address) in top_docs {
            let doc = searcher.doc(doc_address)?;
            let thread_id = doc.get_first(tpost.thread_id_f).unwrap().i64_value();
            let thread_id_term = Term::from_field_i64(tpost.thread_id_f, thread_id);

            writer.delete_term(thread_id_term);
        }

        writer.commit()?;
        state.reader.reload()?;
    }

    // finally collect garbage
    let mut writer = state.writer.lock().unwrap();
    writer.garbage_collect_files()?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body("{}"))
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
                .resource("", |r| {
                    r.method(http::Method::POST)
                        .with_config(modify_index, |cfg| {
                            cfg.0 .0.limit(10_000_000_000); // <- limit size of the payload to 10Gb
                        })
                })
                .boxed(),
            App::with_state(search_state.clone())
                .prefix("/drop")
                .resource("", |r| r.method(http::Method::POST).f(drop_index))
                .boxed(),
        ]
    })
    .bind(host)
    .unwrap()
    .start();

    println!("Started http server: {}", host);
    let _ = sys.run();
}

#[cfg(test)]
mod tests {
    #[test]

    fn batch_insert() {
        extern crate hyper;

        use super::ModifyInfo;
        use diesel::prelude::*;
        use hyper::rt::{self, Future};
        use hyper::{Body, Client, Request};
        use tsearch::establish_connection;
        use tsearch::models::Post;
        use tsearch::schema::threads_message_extra::dsl::*;

        rt::run(rt::lazy(|| {
            let client = Client::new();

            let connection = establish_connection();
            let results = threads_message_extra
                // .filter(published.eq(true))
                // .limit(5)
                .load::<Post>(&connection)
                .expect("Error loading posts");

            let mut items = Vec::new();
            for post in results {
                let item = ModifyInfo {
                    doc: post,
                    overwrite: true,
                    delete: false,
                };

                items.push(item);
            }

            let body = serde_json::to_string(&items).unwrap();

            let req = Request::builder()
                .method("POST")
                .uri("http://188.120.254.29:8080/modify")
                .header("Content-Type", "application/json")
                .header("Accept-Encoding", "gzip")
                .body(Body::from(body))
                .expect("request builder");

            client
                .request(req)
                .map(|res| {
                    println!("Response: {}", res.status());
                })
                .map_err(|err| {
                    println!("Error: {}", err);
                })
        }));
    }
}
