#[macro_use]
extern crate tantivy;
extern crate diesel;

extern crate diesel_lib;

use std::fs;
use std::io::ErrorKind;
use tantivy::schema::*;
use tantivy::Index;

use self::diesel::prelude::*;
use diesel_lib::models::*;
use diesel_lib::*;

fn main() -> tantivy::Result<()> {
    let index_path = "./index";
    match fs::create_dir(index_path) {
        Ok(_) => (),
        Err(error) => match error.kind() {
            ErrorKind::AlreadyExists => (),
            other_error => panic!(other_error),
        },
    }

    let mut schema_builder = Schema::builder();

    let thread_id_t = schema_builder.add_i64_field("thread_id", FAST);
    let title_t = schema_builder.add_text_field("title", TEXT | STORED);
    let text_t = schema_builder.add_text_field("text", TEXT);
    let node_id_t = schema_builder.add_i64_field("node_id", FAST);
    let need_moder_t = schema_builder.add_i64_field("need_moder_id", FAST);
    let post_date_t = schema_builder.add_i64_field("post_date_id", FAST);

    let schema = schema_builder.build();
    let index = Index::create_in_dir(index_path, schema.clone())?;
    let mut index_writer = index.writer(50_000_000)?;

    // ### Adding documents
    use diesel_lib::schema::threads_message_extra::dsl::*;

    let connection = establish_connection();
    let results = threads_message_extra
        // .filter(published.eq(true))
        // .limit(5)
        .load::<Post>(&connection)
        .expect("Error loading posts");

    println!("Displaying {} posts", results.len());
    for post in results {
        index_writer.add_document(doc!(
            thread_id_t => post.thread_id,
            title_t => post.title,
            text_t => post.text,
            node_id_t => post.node_id,
            need_moder_t => post.needModer,
            post_date_t => post.post_date
        ));
    }

    // This call is blocking.
    index_writer.commit()?;

    Ok(())
}
