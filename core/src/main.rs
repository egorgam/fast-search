#[macro_use]
extern crate tantivy;

use actix_web::{get, web, http, App, HttpServer, HttpResponse};
use actix_cors::Cors;
use serde::Deserialize;

mod search_engine;
mod etl;

struct SearchEngine {
    index: tantivy::Index,
    reader: tantivy::IndexReader,
    schema: tantivy::schema::Schema,
}

#[derive(Deserialize)]
struct SearchQuery {
    query: String,
}

#[get("/search")]
async fn search(search_state: web::Data<SearchEngine>, input_data: web::Query<SearchQuery>) -> HttpResponse {
    let search_result = format!("{}", search_engine::search(&search_state.index, 
                                                                        &search_state.reader, 
                                                                        &search_state.schema,
                                                                        &input_data.query));
    return HttpResponse::Ok()
            .content_type("application/json")
            .body(search_result)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let schema = etl::init_schema();
        let index = etl::init_index(&schema);
        let reader = search_engine::init_reader(&index);
        let search_state = SearchEngine{index, reader, schema};
        let cors = Cors::default()
              .allowed_origin("http://localhost:8081")
              .allowed_methods(vec!["GET"])
              .allowed_header(http::header::CONTENT_TYPE)
              .max_age(3600);
        App::new()
        .wrap(cors)
        .data(search_state)
        .service(search)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
