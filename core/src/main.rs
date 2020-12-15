#[macro_use]
extern crate tantivy;
use actix_web::{get, web, App, HttpServer, HttpResponse};
use serde::Deserialize;

mod search_engine;

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
        let schema = search_engine::init_schema();
        let index = search_engine::init_index(&schema);
        let reader = search_engine::init_reader(&index);
        let search_state = SearchEngine{index, reader, schema};
        App::new()
        .data(search_state)
        .service(search)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
