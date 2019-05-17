extern crate rust_stemmers;
use rust_stemmers::{Algorithm, Stemmer};

fn main() {
    // Create a stemmer for the english language
    let en_stemmer = Stemmer::create(Algorithm::Russian);

    // Stemm the word "fruitlessly"
    // Please be aware that all algorithms expect their input to only contain lowercase characters.
    println!("{}", en_stemmer.stem("программистом"));
}
