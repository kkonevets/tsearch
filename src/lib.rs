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
        let stopwords = [
                "и",
                "в",
                "во",
                "не",
                "что",
                "он",
                "на",
                "я",
                "с",
                "со",
                "как",
                "а",
                "то",
                "все",
                "она",
                "так",
                "его",
                "но",
                "да",
                "ты",
                "к",
                "у",
                "же",
                "вы",
                "за",
                "бы",
                "по",
                "только",
                "ее",
                "мне",
                "было",
                "вот",
                "от",
                "меня",
                "еще",
                "нет",
                "о",
                "из",
                "ему",
                "теперь",
                "когда",
                "даже",
                "ну",
                "вдруг",
                "ли",
                "если",
                "уже",
                "или",
                "ни",
                "быть",
                "был",
                "него",
                "до",
                "вас",
                "нибудь",
                "опять",
                "уж",
                "вам",
                "ведь",
                "там",
                "потом",
                "себя",
                "ничего",
                "ей",
                "может",
                "они",
                "тут",
                "где",
                "есть",
                "надо",
                "ней",
                "для",
                "мы",
                "тебя",
                "их",
                "чем",
                "была",
                "сам",
                "чтоб",
                "без",
                "будто",
                "чего",
                "раз",
                "тоже",
                "себе",
                "под",
                "будет",
                "ж",
                "тогда",
                "кто",
                "этот",
                "того",
                "потому",
                "этого",
                "какой",
                "совсем",
                "ним",
                "здесь",
                "этом",
                "один",
                "почти",
                "мой",
                "тем",
                "чтобы",
                "нее",
                "сейчас",
                "были",
                "куда",
                "зачем",
                "всех",
                "никогда",
                "можно",
                "при",
                "наконец",
                "два",
                "об",
                "другой",
                "хоть",
                "после",
                "над",
                "больше",
                "тот",
                "через",
                "эти",
                "нас",
                "про",
                "всего",
                "них",
                "какая",
                "много",
                "разве",
                "три",
                "эту",
                "моя",
                "впрочем",
                "хорошо",
                "свою",
                "этой",
                "перед",
                "иногда",
                "лучше",
                "чуть",
                "том",
                "нельзя",
                "такой",
                "им",
                "более",
                "всегда",
                "конечно",
                "всю",
                "между",
        ];

        let stopwords_vec = stopwords.into_iter().map(|w| w.to_string()).collect();

        let ru_stem = SimpleTokenizer
                .filter(RemoveLongFilter::limit(40))
                .filter(LowerCaser)
                .filter(StopWordFilter::remove(stopwords_vec))
                .filter(Stemmer::new(Language::Russian));

        index.tokenizers().register("ru_stem", ru_stem);
}
