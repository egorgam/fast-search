use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::Index;
use tantivy::ReloadPolicy;
// use tantivy::tokenizer::NgramTokenizer;

use serde_json::{Map, Value};

use std::path::Path;
use std::fs::File;


pub fn init_reader(index: &tantivy::Index) -> tantivy::IndexReader {
    return index
    .reader_builder()
    .reload_policy(ReloadPolicy::OnCommit)
    .try_into()
    .unwrap();
}

pub fn init_schema() -> tantivy::schema::Schema {
    let mut schema_builder = Schema::builder();
    // let text_field_indexing = TextFieldIndexing::default()
    // .set_tokenizer("ngram3")
    // .set_index_option(IndexRecordOption::WithFreqsAndPositions);
    // let text_options = TextOptions::default()
    // .set_indexing_options(text_field_indexing)
    // .set_stored();
    schema_builder.add_text_field("track_id", TEXT | STORED);
    schema_builder.add_text_field("name", TEXT | STORED);
    return schema_builder.build()
}

pub fn init_index(schema: &tantivy::schema::Schema) -> tantivy::Index {
    let index_path = "./../store/index";
    if Path::new(&[index_path, "meta.json"].join("/")).exists(){
        return Index::open_in_dir(&index_path).unwrap();
    }
    else {
        let index = Index::create_in_dir(&index_path, schema.clone()).unwrap();
        // index
        // .tokenizers()
        // .register("ngram3", NgramTokenizer::new(3, 3, false));

        let mut index_writer = index.writer(50_000_000).unwrap();
        let track_id = schema.get_field("track_id").unwrap();
        let name = schema.get_field("name").unwrap();

        let file = File::open("./../store/raw/tracks.csv").unwrap();
        let mut rdr = csv::Reader::from_reader(file);
        for result in rdr.records() {
            let record = result.unwrap();
            let track_id_csv = &record[0];
            let name_csv = &record[2];
            index_writer.add_document(doc!(
                track_id => track_id_csv,
                name => name_csv
                ));
        }
        index_writer.commit().unwrap();
        return index
    }
}

pub fn search(index: &tantivy::Index, reader: &tantivy::IndexReader, schema: &tantivy::schema::Schema, input_data: &str) -> String {
    let track_id = schema.get_field("track_id").unwrap();
    let name = schema.get_field("name").unwrap();

    let searcher = &reader.searcher();
    let mut query_parser = QueryParser::for_index(&index, vec![track_id, name]);
    query_parser.set_conjunction_by_default();
    
    let query = query_parser.parse_query(&input_data).unwrap();
    let top_docs = searcher.search(&query, &TopDocs::with_limit(20)).unwrap();

    if top_docs.len() == 0 {
        return r#"{"result": null}"#.to_string()
    }
    else {
        let mut vec = Vec::new();
        for (score, doc_address) in top_docs {
            let mut map = Map::new();
            let retrieved_doc = searcher.doc(doc_address).unwrap();
            let score_text = score.to_string();
            let track_id_text = retrieved_doc.get_first(track_id).unwrap().text().unwrap();
            let name_text = retrieved_doc.get_first(name).unwrap().text().unwrap();
            map.insert("score".to_string(), Value::String(score_text.to_string()));
            map.insert("track_id".to_string(), Value::String(track_id_text.to_string()));
            map.insert("name".to_string(), Value::String(name_text.to_string()));
            vec.push(Value::Object(map))
        }
        let mut result = Map::new();
        result.insert("result".to_string(), Value::Array(vec));
        return serde_json::to_string(&result).unwrap()
    }
}