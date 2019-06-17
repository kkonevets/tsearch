extern crate diesel;
extern crate tantivy;
extern crate tsearch;

use std::fs;
use std::io::ErrorKind;
use tantivy::schema::*;
use tantivy::Index;

use diesel::prelude::*;
use tsearch::crud::TantivyPost;
use tsearch::models::*;
use tsearch::*;

fn main() -> tantivy::Result<()> {
    let index_path = "./index";
    match fs::remove_dir_all(index_path) {
        Ok(_) => (),
        Err(e) => match e.kind() {
            ErrorKind::NotFound => (),
            other_error => panic!(other_error),
        },
    }

    match fs::create_dir(index_path) {
        Ok(_) => (),
        Err(error) => panic!(error.to_string()),
    }

    let text_options = TextOptions::default().set_indexing_options(
        TextFieldIndexing::default()
            .set_tokenizer("ru_stem")
            .set_index_option(IndexRecordOption::WithFreqsAndPositions),
    );

    let mut schema_builder = Schema::builder();

    schema_builder.add_i64_field("thread_id", INDEXED | STORED);
    schema_builder.add_text_field("title", text_options.clone() | STORED);
    schema_builder.add_text_field("text", text_options.clone());
    schema_builder.add_i64_field("node_id", INDEXED);
    schema_builder.add_i64_field("needModer", INDEXED);
    schema_builder.add_i64_field("post_date", INDEXED);

    let schema = schema_builder.build();
    let index = Index::create_in_dir(index_path, schema.clone())?;
    let mut index_writer = index.writer(50_000_000)?;

    register_tokenizer(&index);

    let tpost = TantivyPost::new(&schema);

    // ### Adding documents
    use tsearch::schema::threads_message_extra::dsl::*;

    let connection = establish_connection();
    let results = threads_message_extra
        // .filter(published.eq(true))
        // .limit(5)
        .load::<Post>(&connection)
        .expect("Error loading posts");

    println!("Indexing {} docs", results.len());
    for post in results {
        tpost.add(&post, &mut index_writer);
    }

    // This call is blocking.
    index_writer.commit()?;
    index_writer.wait_merging_threads()
}
