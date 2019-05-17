pub mod models;
pub mod schema;

use tantivy::tokenizer::*;
use tantivy::Index;

#[macro_use]
extern crate diesel;
extern crate dotenv;

use diesel::mysql::MysqlConnection;
use diesel::prelude::*;
use dotenv::dotenv;
use std::env;

pub fn establish_connection() -> MysqlConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    MysqlConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

pub fn preprocess(text: &str) -> String {
    return text.replace("-", " ");
}

pub fn register_tokenizer(index: &Index) {
    let ru_stem = SimpleTokenizer
        .filter(RemoveLongFilter::limit(40))
        .filter(LowerCaser)
        .filter(Stemmer::new(Language::Russian));

    index.tokenizers().register("ru_stem", ru_stem);
}
