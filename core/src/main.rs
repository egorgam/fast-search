#[macro_use]
extern crate tantivy;

use actix_web::{get, web, http, App, HttpServer, HttpResponse};
use actix_cors::Cors;
use serde::Deserialize;

mod search_engine;
mod etl;

struct SearchEngine {
    words: search_engine::Words,
    phrases: search_engine::Phrases
}

#[derive(Deserialize)]
struct SearchQuery {
    query: String,
}

#[get("/search")]
async fn search(search_state: web::Data<SearchEngine>, input_data: web::Query<SearchQuery>) -> HttpResponse {
    let search_result = format!("{}", search_engine::search(&search_state.words, 
                                                                        &search_state.phrases,
                                                                        &input_data.query));
    return HttpResponse::Ok()
            .content_type("application/json")
            .body(search_result)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let words_schema = etl::init_words_schema();
        let words_index = etl::init_words_index(&words_schema);
        let words = search_engine::Words{ reader: search_engine::init_words_reader(&words_index), schema: words_schema, index: words_index };

        let phrases_schema = etl::init_phrases_schema();
        let phrases_index = etl::init_phrases_index(&phrases_schema);
        let phrases = search_engine::Phrases{ reader: search_engine::init_phrases_reader(&phrases_index), schema: phrases_schema, index: phrases_index };

        let search_state = SearchEngine{words, phrases};

        let cors = Cors::default()
              .allowed_origin("http://localhost:8000")
              .allowed_methods(vec!["GET"])
              .allowed_header(http::header::CONTENT_TYPE)
              .max_age(3600);
        App::new()
        .wrap(cors)
        .data(search_state)
        .service(search)
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
