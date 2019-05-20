extern crate common;
extern crate tantivy;

use std::time::Instant;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::Index;
use tantivy::ReloadPolicy;

use common::{preprocess, register_tokenizer};

fn main() -> tantivy::Result<()> {
    let now = Instant::now();

    let index_path = "./index";

    let index = Index::open_in_dir(index_path)?;
    register_tokenizer(&index);

    let schema = index.schema();

    // # Searching

    // For a search server you will typically create one reader for the entire lifetime of your
    // program, and acquire a new searcher for every single request.

    // In the code below, we rely on the 'ON_COMMIT' policy: the reader
    // will reload the index automatically after each commit.
    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommit)
        .try_into()?;

    let title_t = schema.get_field("title").unwrap();
    let text_t = schema.get_field("text").unwrap();

    let query_parser = QueryParser::for_index(&index, vec![title_t, text_t]);

    println!(
        "index and metadata loaded in {} ms",
        now.elapsed().as_millis()
    );

    // ########## query start - client side ##############

    let now = Instant::now();

    let searcher = reader.searcher();

    let qtext = preprocess("Тренинг Kak Bлюбитьcя и нe пoтеpять ceбя");
    let query = query_parser.parse_query(qtext.as_str())?;
    let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;

    println!("query execution time {} ms", now.elapsed().as_millis());

    // ########## query finish ##############

    for (_score, doc_address) in top_docs {
        let retrieved_doc = searcher.doc(doc_address)?;
        println!("score: {}, {}", _score, schema.to_json(&retrieved_doc));
    }

    Ok(())
}
