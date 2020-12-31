use tantivy::{collector::TopDocs, LeasedItem, Searcher, DocAddress};
use tantivy::query::{FuzzyTermQuery, BooleanQuery, Query, Occur, QueryParser, TermQuery, PhraseQuery};
use tantivy::schema::IndexRecordOption;
use tantivy::Term;
use tantivy::ReloadPolicy;

use serde_json::{Map, Value};

pub struct Words {
    pub reader: tantivy::IndexReader,
    pub schema: tantivy::schema::Schema,
    pub index: tantivy::Index
}
pub struct Phrases {
    pub reader: tantivy::IndexReader,
    pub schema: tantivy::schema::Schema,
    pub index: tantivy::Index
}

pub fn init_words_reader(index: &tantivy::Index) -> tantivy::IndexReader {
    return index
    .reader_builder()
    .reload_policy(ReloadPolicy::OnCommit)
    .try_into()
    .unwrap();
}

pub fn init_phrases_reader(index: &tantivy::Index) -> tantivy::IndexReader {
    return index
    .reader_builder()
    .reload_policy(ReloadPolicy::OnCommit)
    .try_into()
    .unwrap();
}

fn process_result(phrases: &Phrases, searcher: &LeasedItem<Searcher>, top_docs: Vec<(f32, DocAddress)>) -> Vec<serde_json::Value> {

    // get indexed fields
    let track_id = phrases.schema.get_field("track_id").unwrap();
    let name = phrases.schema.get_field("name").unwrap();
    let raw_name = phrases.schema.get_field("raw_name").unwrap();
    let lang = phrases.schema.get_field("lang").unwrap();

    // collect search results to serde json structure
    let mut result = Vec::new();
    for (score, doc_address) in top_docs {
        let mut map = Map::new();
        let retrieved_doc = searcher.doc(doc_address).unwrap();
        let track_id_text = retrieved_doc.get_first(track_id).unwrap().text().unwrap();
        let name_text = retrieved_doc.get_first(name).unwrap().text().unwrap();
        let raw_name_text = retrieved_doc.get_first(raw_name).unwrap().text().unwrap();
        let lang_text = retrieved_doc.get_first(lang).unwrap().text().unwrap();

        map.insert("score".to_string(), Value::String(score.to_string()));
        map.insert("track_id".to_string(), Value::String(track_id_text.to_string()));
        map.insert("name".to_string(), Value::String(name_text.to_string()));
        map.insert("name_raw".to_string(), Value::String(raw_name_text.to_string()));
        map.insert("lang".to_string(), Value::String(lang_text.to_string()));

        result.push(serde_json::Value::Object(map))
    }
    return result
}

fn get_search_results(phrases: &Phrases, user_input: &Vec<&str>, suggestions: Vec<String>) -> Vec<serde_json::Value> {
    // Implemetation of mixed boolean logiic for user input data and suggestions. For example:
    // high AND (ho OR hopes) -> "high hopes" -> FOUND
    // high AND hopes AND (ol OR ololo) -> "high hopes ololo" -> NOT_FOUND

    // get indexed fields
    let name = phrases.schema.get_field("name").unwrap();

    // define searcher
    let searcher = &phrases.reader.searcher();
    if user_input.len() == 1 {
        let query = TermQuery::new( // ВОЗМОЖНО этой штуке нужно скормить полный совпад в колонке, поэтому оно не работает в режиме автодополнения
            Term::from_field_text(name, user_input[0]),
            IndexRecordOption::Basic,
        );
        let top_docs = searcher.search(&query, &TopDocs::with_limit(50)).unwrap();
        return process_result(phrases, searcher, top_docs)
    }
    else {
        // user input phrase query (AND for all terms)
        let mut user_input_phrase: Vec<Term> = Vec::new();
        for token in user_input {
            user_input_phrase.push(Term::from_field_text(name, token))
        }
        let phrase_query: Box<dyn Query> = Box::new(PhraseQuery::new(user_input_phrase));

        // suggestions terms occur (OR for all terms)
        let mut suggestions_terms_occur: Vec<(Occur, Box<dyn Query>)> = Vec::new();
        for token in suggestions {
            println!("{}", &token);
            let term_query = Box::new(TermQuery::new(
                Term::from_field_text(name, &token),
                IndexRecordOption::Basic,
            ));
            suggestions_terms_occur.push((Occur::Should, term_query))
        }
        let suggestions_query = BooleanQuery::from(suggestions_terms_occur);

        // define nested query with both occurs
        let query = BooleanQuery::from(vec![
            (Occur::Must, phrase_query),
            (Occur::Must, Box::new(suggestions_query))
        ]);
        let top_docs = searcher.search(&query, &TopDocs::with_limit(50)).unwrap();
        return process_result(phrases, searcher, top_docs)
    }
}

fn get_suggestions(words: &Words, input_data: &str) -> Vec<String> {
    let mut result = Vec::new();
    let word = words.schema.get_field("word").unwrap();
    let words_searcher = &words.reader.searcher();

    let query_parser = QueryParser::for_index(&words.index, vec![word]);
    let query = query_parser.parse_query(&input_data).unwrap();
    let mut top_docs = words_searcher.search(&query, &TopDocs::with_limit(10)).unwrap();

    if top_docs.len() == 0 {
        let term = Term::from_field_text(word, &input_data.to_lowercase());
        let query = FuzzyTermQuery::new(term, 1, true);
        top_docs = words_searcher.search(&query, &TopDocs::with_limit(10)).unwrap();
        if top_docs.len() == 0 {
            return result
        }
    }
    for (_, doc_address) in top_docs {
        let retrieved_doc = words_searcher.doc(doc_address).unwrap();
        result.push(retrieved_doc.get_first(word).unwrap().text().unwrap().to_string());
        
    }
    result.dedup();
    return result
}

fn remove_whitespace(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

pub fn search(words: &Words, phrases: &Phrases, raw_text: &str) -> String {
    let mut result = Vec::new();
    let user_input: Vec<&str> = raw_text.split(" ").collect();
    if remove_whitespace(raw_text).len() > 0 {
        if user_input.len() == 1 {
            result = get_search_results(phrases, &user_input, vec![]);
            if result.len() == 0 {
                println!("{}", raw_text);
                result = get_search_results(phrases, &user_input, get_suggestions(words, raw_text));
            }
        }
        else {
            let mut generated_phrase: Vec<String> = Vec::new();
            // for token in &user_input {
                let token = &user_input.last().copied().unwrap();
                if remove_whitespace(token).len() > 0 {
                    let suggestions = get_suggestions(words, token);
                    for s in suggestions{
                        generated_phrase.push(s)
                    }
                }
            // }
            result = get_search_results(phrases, &user_input, generated_phrase);
            // println!("{}", generated_phrase);
        }
    }
    let mut answer = Map::new();
    answer.insert("result".to_string(), serde_json::Value::Array(result));
    return serde_json::to_string(&answer).unwrap()
}
