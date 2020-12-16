use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::query::FuzzyTermQuery;
use tantivy::Term;

use tantivy::ReloadPolicy;

use serde_json::{Map, Value};

pub fn init_reader(index: &tantivy::Index) -> tantivy::IndexReader {
    return index
    .reader_builder()
    .reload_policy(ReloadPolicy::OnCommit)
    .try_into()
    .unwrap();
}

pub fn search(index: &tantivy::Index, reader: &tantivy::IndexReader, schema: &tantivy::schema::Schema, input_data: &str) -> String {
    let track_id = schema.get_field("track_id").unwrap();
    let name = schema.get_field("name").unwrap();

    let searcher = &reader.searcher();
    let mut query_parser = QueryParser::for_index(&index, vec![track_id, name]);
    query_parser.set_conjunction_by_default();
    
    let term = Term::from_field_text(name, &input_data.to_lowercase());
    let query = FuzzyTermQuery::new(term, 2, true);
    // let query = query_parser.parse_query(&input_data.to_lowercase()).unwrap();
    let top_docs = searcher.search(&query, &TopDocs::with_limit(50)).unwrap();

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