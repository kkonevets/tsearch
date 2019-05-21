use tantivy::collector::TopDocs;
use tantivy::query::TermQuery;
use tantivy::schema::*;
use tantivy::Index;
use tantivy::IndexReader;

pub fn doc_by_id(reader: &IndexReader, isbn_term: &Term) -> tantivy::Result<Option<Document>> {
    let searcher = reader.searcher();

    // This is the simplest query you can think of.
    // It matches all of the documents containing a specific term.
    //
    // The second argument is here to tell we don't care about decoding positions,
    // or term frequencies.
    let term_query = TermQuery::new(isbn_term.clone(), IndexRecordOption::Basic);
    let top_docs = searcher.search(&term_query, &TopDocs::with_limit(1))?;

    if let Some((_score, doc_address)) = top_docs.first() {
        let doc = searcher.doc(*doc_address)?;
        Ok(Some(doc))
    } else {
        // no doc matching this ID.
        Ok(None)
    }
}

// fn add(post: &models::Post, rewrite: bool) {
//     let mut index_writer = index.writer(50_000_000)?;
//     index_writer.add_document(doc!(
//            title => "Frankenstein",
//            isbn => "978-9176370711",
//     ));
// }
