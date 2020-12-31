use tantivy::schema::*;
use tantivy::tokenizer::*;
use tantivy::Index;
use whatlang::{Detector, Lang};

use std::path::Path;
use std::fs::File;

const STORE_PATH: &str = "/Users/egorgam/git/zvq/fast-search/store/";

fn apply_words_index_tokenizator(index: tantivy::Index) -> tantivy::Index {
    let tokenizer = TextAnalyzer::from(SimpleTokenizer)
            .filter(LowerCaser)
            .filter(Stemmer::new(Language::English))
            .filter(Stemmer::new(Language::Russian));
    
    index.tokenizers()
        .register("word", tokenizer);

    index.tokenizers()
        .register("ngram", NgramTokenizer::new(1, 10, true));

    return index
}

fn apply_phrases_index_tokenizator(index: tantivy::Index) -> tantivy::Index {
        let tokenizer = TextAnalyzer::from(SimpleTokenizer)
            .filter(LowerCaser);
            // .filter(Stemmer::new(Language::English))
            // .filter(Stemmer::new(Language::Russian));
        index.tokenizers().register("phrases", tokenizer);
    return index
}


pub fn init_words_schema() -> tantivy::schema::Schema {
    let mut schema_builder = Schema::builder();
    let text_field_indexing = TextFieldIndexing::default()
        .set_tokenizer("word")
        .set_tokenizer("ngram")
        .set_index_option(IndexRecordOption::WithFreqsAndPositions);

    let text_options = TextOptions::default()
        .set_indexing_options(text_field_indexing)
        .set_stored();

    schema_builder.add_text_field("word", text_options);
    return schema_builder.build()
}

pub fn init_phrases_schema() -> tantivy::schema::Schema {
    let mut schema_builder = Schema::builder();
    let text_field_indexing = TextFieldIndexing::default()
        .set_tokenizer("phrases")
        .set_index_option(IndexRecordOption::WithFreqsAndPositions);

    let text_options = TextOptions::default()
        .set_indexing_options(text_field_indexing)
        .set_stored();

    schema_builder.add_text_field("track_id", STRING | STORED);
    schema_builder.add_text_field("name", text_options);
    schema_builder.add_text_field("raw_name", TEXT | STORED);
    schema_builder.add_text_field("lang", STRING | STORED);
    return schema_builder.build()
}

fn get_text_lang(text: &str) -> &str {
    let whitelist = vec![Lang::Eng, Lang::Rus];
    let detector = Detector::with_whitelist(whitelist);
    let lang = detector.detect_lang(text);
    if lang != None {
        if lang.unwrap() == Lang::Eng {
            return "en"
        }
        else if lang.unwrap() == Lang::Rus {
            return "ru";
        }
    }
    return "unknown";
}

pub fn init_phrases_index(schema: &tantivy::schema::Schema) -> tantivy::Index {
    let index_path = &format!("{}{}",STORE_PATH, "index/tracks/phrases/v1");
    if Path::new(&[index_path, "meta.json"].join("/")).exists(){
        let index = Index::open_in_dir(&index_path).unwrap();
        return apply_phrases_index_tokenizator(index)
    }
    else {
        let index = apply_phrases_index_tokenizator(Index::create_in_dir(&index_path, schema.clone()).unwrap());
        let mut index_writer = index.writer(50_000_000).unwrap();
        let track_id = schema.get_field("track_id").unwrap();
        let name = schema.get_field("name").unwrap();
        let raw_name = schema.get_field("raw_name").unwrap();
        let lang = schema.get_field("lang").unwrap();

        let file = File::open("./../store/raw/tracks.csv").unwrap();
        let mut rdr = csv::Reader::from_reader(file);
        for result in rdr.records() {
            let record = result.unwrap();
            let track_id_csv = &record[0];
            let name_csv = &record[2];
            let raw_name_text = name_csv.clone();
            let text_lang = get_text_lang(name_csv);
            index_writer.add_document(doc!(
                track_id => track_id_csv,
                name => name_csv,
                raw_name => raw_name_text,
                lang => text_lang
                ));
        }
        index_writer.commit().unwrap();
        return index
    }
}

pub fn init_words_index(schema: &tantivy::schema::Schema) -> tantivy::Index {
    let index_path = &format!("{}{}",STORE_PATH, "index/tracks/words/v1");
    if Path::new(&[index_path, "meta.json"].join("/")).exists(){
        let index = Index::open_in_dir(&index_path).unwrap();
        return apply_words_index_tokenizator(index)
    }
    else {
        let index = apply_words_index_tokenizator(Index::create_in_dir(&index_path, schema.clone()).unwrap());
        let mut index_writer = index.writer(50_000_000).unwrap();
        let word = schema.get_field("word").unwrap();

        let file = File::open("./../store/raw/tracks.csv").unwrap();
        let mut rdr = csv::Reader::from_reader(file);
        for result in rdr.records() {
            let record = result.unwrap();
            let name_csv = &record[2].to_string();
            for w in name_csv.split(" ") {
                // println!("{}", w);
                index_writer.add_document(doc!(
                    word => w.to_lowercase()
                    ));
            }
            
        }
        index_writer.commit().unwrap();
        return index
    }
}