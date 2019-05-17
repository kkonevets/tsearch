use tantivy::tokenizer::*;

pub ru_stem = SimpleTokenizer
    .filter(RemoveLongFilter::limit(40))
    .filter(LowerCaser)
    .filter(Stemmer::new(Language::Russian));
