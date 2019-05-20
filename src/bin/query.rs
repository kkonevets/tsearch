extern crate tantivy;
extern crate tsearch;

use std::time::Instant;
use tantivy::collector::TopDocs;
use tsearch::preprocess;
use tsearch::state::SearchState;

fn main() -> tantivy::Result<()> {
    let now = Instant::now();

    let state = SearchState::new().unwrap();

    println!(
        "index and metadata loaded in {} ms",
        now.elapsed().as_millis()
    );

    // ########## query start - client side ##############

    let now = Instant::now();

    let searcher = state.reader.searcher();

    let qtext = preprocess("Оксана Лифенко");
    let query = state.query_parser.parse_query(qtext.as_str())?;
    let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;

    println!("query execution time {} ms", now.elapsed().as_millis());

    // ########## query finish ##############

    for (_score, doc_address) in top_docs {
        let retrieved_doc = searcher.doc(doc_address)?;
        println!(
            "score: {}, {}",
            _score,
            state.schema.to_json(&retrieved_doc)
        );
    }

    Ok(())
}
