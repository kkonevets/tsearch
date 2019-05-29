use super::models::Post;
use super::preprocess;
use tantivy::collector::TopDocs;
use tantivy::query::TermQuery;
use tantivy::schema::*;
use tantivy::{IndexReader, IndexWriter};

pub fn doc_by_id(reader: &IndexReader, term: &Term) -> tantivy::Result<Option<Document>> {
    let searcher = reader.searcher();

    // This is the simplest query you can think of.
    // It matches all of the documents containing a specific term.
    //
    // The second argument is here to tell we don't care about decoding positions,
    // or term frequencies.
    let term_query = TermQuery::new(term.clone(), IndexRecordOption::Basic);
    let top_docs = searcher.search(&term_query, &TopDocs::with_limit(1))?;

    if let Some((_score, doc_address)) = top_docs.first() {
        let doc = searcher.doc(*doc_address)?;
        Ok(Some(doc))
    } else {
        // no doc matching this ID.
        Ok(None)
    }
}

pub struct TantivyPost {
    pub thread_id_f: Field,
    pub title_f: Field,
    pub text_f: Field,
    pub node_id_f: Field,
    pub need_moder_f: Field,
    pub post_date_f: Field,
}

impl TantivyPost {
    pub fn new(schema: &Schema) -> TantivyPost {
        TantivyPost {
            thread_id_f: schema.get_field("thread_id").unwrap(),
            title_f: schema.get_field("title").unwrap(),
            text_f: schema.get_field("text").unwrap(),
            node_id_f: schema.get_field("node_id").unwrap(),
            need_moder_f: schema.get_field("needModer").unwrap(),
            post_date_f: schema.get_field("post_date").unwrap(),
        }
    }

    pub fn add(&self, post: &Post, writer: &mut IndexWriter) {
        // let mut all_text = post.title.clone();
        // all_text.push_str(" ");
        // all_text.push_str(&post.text);

        writer.add_document(doc!(
            self.thread_id_f => post.thread_id,
            self.title_f => preprocess(&post.title),
            self.text_f => preprocess(&post.text),
            self.node_id_f => post.node_id,
            self.need_moder_f => post.needModer,
            self.post_date_f => post.post_date
        ));
    }
}
