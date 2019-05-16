#[macro_use]
extern crate tantivy;
extern crate diesel;

extern crate diesel_lib;

use std::fs;
use std::io::ErrorKind;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::Index;
use tantivy::ReloadPolicy;

use self::diesel::prelude::*;
use diesel_lib::models::*;

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

    let thread_id = schema_builder.add_i64_field("thread_id", FAST);
    let title = schema_builder.add_text_field("title", TEXT | STORED);
    let body = schema_builder.add_text_field("body", TEXT);
    let node_id = schema_builder.add_i64_field("node_id", FAST);
    let need_moder = schema_builder.add_i64_field("need_moder_id", FAST);
    let post_date = schema_builder.add_i64_field("post_date_id", FAST);

    let schema = schema_builder.build();
    let index = Index::create_in_dir(index_path, schema.clone())?;
    let mut index_writer = index.writer(50_000_000)?;

    // ### Adding documents
    //
    index_writer.add_document(doc!(
    title => "Of Mice and Men",
    body => "A few miles south of Soledad, the Salinas River drops in close to the hillside \
            bank and runs deep and green. The water is warm too, for it has slipped twinkling \
            over the yellow sands in the sunlight before reaching the narrow pool. On one \
            side of the river the golden foothill slopes curve up to the strong and rocky \
            Gabilan Mountains, but on the valley side the water is lined with trees—willows \
            fresh and green with every spring, carrying in their lower leaf junctures the \
            debris of the winter’s flooding; and sycamores with mottled, white, recumbent \
            limbs and branches that arch over the pool"
    ));

    // This call is blocking.
    index_writer.commit()?;

    // # Searching
    //
    // ### Searcher
    //
    // In the code below, we rely on the 'ON_COMMIT' policy: the reader
    // will reload the index automatically after each commit.
    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommit)
        .try_into()?;

    let searcher = reader.searcher();

    // ### Query

    let query_parser = QueryParser::for_index(&index, vec![title, body]);

    let query = query_parser.parse_query("sea whale")?;

    let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;

    for (_score, doc_address) in top_docs {
        let retrieved_doc = searcher.doc(doc_address)?;
        println!("{}", schema.to_json(&retrieved_doc));
    }

    Ok(())
}
