use super::register_tokenizer;
use std::sync::{Arc, RwLock};
use tantivy::query::QueryParser;
use tantivy::{Index, IndexReader, IndexWriter, ReloadPolicy};

#[derive(Clone)]
pub struct SearchState {
    pub index: Arc<Index>,
    pub reader: Arc<IndexReader>,
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

        // let title_t = schema.get_field("title").unwrap();
        let text_t = schema.get_field("text").unwrap();

        let query_parser = QueryParser::for_index(&index, vec![text_t]);

        let state = SearchState {
            index: Arc::new(index),
            reader: Arc::new(reader),
            query_parser: Arc::new(query_parser),
            schema: Arc::new(schema),
        };

        Ok(state)
    }
}

// #[derive(Clone)]
// pub struct ModifyState {
//     pub index: Arc<Index>,
//     pub writer: Arc<RwLock<IndexWriter>>,
//     pub schema: Arc<tantivy::schema::Schema>,
// }

// impl ModifyState {
//     pub fn new() -> Result<ModifyState, tantivy::TantivyError> {
//         let index = load_index()?;

//         let writer = index.writer_with_num_threads(1, 5_000_000)?;
//         let schema = index.schema();

//         let state = ModifyState {
//             index: Arc::new(index),
//             writer: Arc::new(RwLock::new(writer)),
//             schema: Arc::new(schema),
//         };

//         Ok(state)
//     }
// }
