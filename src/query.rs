extern crate tantivy;

use std::time::Instant;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::Index;
use tantivy::ReloadPolicy;

fn main() -> tantivy::Result<()> {
    let index_path = "./index";

    let index = Index::open_in_dir(index_path)?;
    let schema = index.schema();

    // # Searching
    //
    // In the code below, we rely on the 'ON_COMMIT' policy: the reader
    // will reload the index automatically after each commit.
    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommit)
        .try_into()?;

    let searcher = reader.searcher();

    // ### Query

    let title_t = schema.get_field("title").unwrap();
    let text_t = schema.get_field("text").unwrap();

    let query_parser = QueryParser::for_index(&index, vec![title_t, text_t]);

    // ########## query start ##############

    let now = Instant::now();

    let query = query_parser.parse_query("стать программистом")?;
    let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;

    println!("{} ms", now.elapsed().as_millis());

    // ########## query finish ##############

    for (_score, doc_address) in top_docs {
        let retrieved_doc = searcher.doc(doc_address)?;
        println!("{}", schema.to_json(&retrieved_doc));
    }

    Ok(())
}
