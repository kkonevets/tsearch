use super::register_tokenizer;
use std::sync::{Arc, Mutex};
use tantivy::query::QueryParser;
use tantivy::{Index, IndexReader, IndexWriter, ReloadPolicy};

#[derive(Clone)]
pub struct SearchState {
    pub index: Arc<Index>,
    pub reader: Arc<IndexReader>,
    pub writer: Arc<Mutex<IndexWriter>>,
    pub query_parser: Arc<QueryParser>,
    pub schema: Arc<tantivy::schema::Schema>,
}

pub fn load_index() -> Result<Index, tantivy::TantivyError> {
    let index_path = "./index";

    let index = Index::open_in_dir(index_path)?;
    register_tokenizer(&index);
    Ok(index)
}

impl SearchState {
    pub fn new() -> Result<SearchState, tantivy::TantivyError> {
        let index = load_index()?;

        let schema = index.schema();

        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommit)
            .try_into()?;

        let title_t = schema.get_field("title").unwrap();
        let text_t = schema.get_field("text").unwrap();

        let query_parser = QueryParser::for_index(&index, vec![title_t, text_t]);

        let writer = index.writer(50_000_000)?;

        let state = SearchState {
            index: Arc::new(index),
            reader: Arc::new(reader),
            writer: Arc::new(Mutex::new(writer)),
            query_parser: Arc::new(query_parser),
            schema: Arc::new(schema),
        };

        Ok(state)
    }
}
