use actix_web::{get, post, web, Result, App, HttpResponse, HttpServer, Responder};

mod models;
use models::RESTInputModel;
mod query_engine;

async fn rest_api(jsonQuery: web::Json<RESTInputModel>) -> Result<String> {
    let sql = query_engine::GetQuery(&jsonQuery);
    Ok(format!("SQL: \n{}!", sql))
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

struct AppState {
    app_name: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .app_data(web::Data::new(AppState {
                app_name: String::from("Actix Web"),
            }))
            .service(hello)
            .service(echo)
            .route("/api", web::post().to(rest_api))
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}