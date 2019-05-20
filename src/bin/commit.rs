#[macro_use]
extern crate tantivy;
extern crate diesel;

extern crate tsearch;

use std::fs;
use std::io::ErrorKind;
use tantivy::schema::*;
use tantivy::Index;

use self::diesel::prelude::*;
use tsearch::models::*;
use tsearch::*;

fn main() -> tantivy::Result<()> {
    let index_path = "./index";
    match fs::create_dir(index_path) {
        Ok(_) => (),
        Err(error) => match error.kind() {
            ErrorKind::AlreadyExists => (),
            other_error => panic!(other_error),
        },
    }

    let text_options = TextOptions::default().set_indexing_options(
        TextFieldIndexing::default()
            .set_tokenizer("ru_stem")
            .set_index_option(IndexRecordOption::WithFreqsAndPositions),
    );

    let mut schema_builder = Schema::builder();

    let thread_id_t = schema_builder.add_i64_field("thread_id", INDEXED | STORED);
    let title_t = schema_builder.add_text_field("title", text_options.clone() | STORED);
    let text_t = schema_builder.add_text_field("text", text_options.clone());
    let node_id_t = schema_builder.add_i64_field("node_id", INDEXED);
    let need_moder_t = schema_builder.add_i64_field("need_moder_id", INDEXED);
    let post_date_t = schema_builder.add_i64_field("post_date_id", INDEXED);

    let schema = schema_builder.build();
    let index = Index::create_in_dir(index_path, schema.clone())?;
    let mut index_writer = index.writer(50_000_000)?;

    register_tokenizer(&index);

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
        let mut all_text = post.title.clone();
        all_text.push_str(" ");
        all_text.push_str(&post.text);

        index_writer.add_document(doc!(
            thread_id_t => post.thread_id,
            title_t => preprocess(&post.title),
            text_t => preprocess(&all_text),
            node_id_t => post.node_id,
            need_moder_t => post.needModer,
            post_date_t => post.post_date
        ));
    }

    // This call is blocking.
    index_writer.commit()?;

    Ok(())
}