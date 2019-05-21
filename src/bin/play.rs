#[macro_use]
extern crate tantivy;
use tantivy::schema::Term;

use tsearch::crud::doc_by_id;
use tsearch::state::SearchState;

fn main() -> tantivy::Result<()> {
    let state = SearchState::new().unwrap();
    let mut index_writer = state.index.writer(50_000_000)?;
    let schema = state.schema;

    let thread_id: i64 = 114215;
    let thread_id_f = schema.get_field("thread_id").unwrap();
    let title_f = schema.get_field("title").unwrap();
    let text_f = schema.get_field("text").unwrap();

    let thread_id_term = Term::from_field_i64(thread_id_f, thread_id);

    let retrieved_doc = doc_by_id(&state.reader, &thread_id_term)?.unwrap();
    println!("{}", schema.to_json(&retrieved_doc));

    index_writer.delete_term(thread_id_term.clone());
    index_writer.add_document(doc!(
           thread_id_f => thread_id,
           title_f => "modified",
           text_f => "Frankenstein",
    ));

    index_writer.commit()?;
    // We reload our searcher to make our change available to clients.
    state.reader.reload()?;

    let retrieved_doc = doc_by_id(&state.reader, &thread_id_term)?.unwrap();
    println!("{}", schema.to_json(&retrieved_doc));

    Ok(())
}
