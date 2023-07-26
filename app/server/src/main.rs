use actix_web::{get, post, web, Result, App, HttpResponse, HttpServer, Responder};
// use serde::Deserialize;

mod models;
use models::RESTInputModel;

/// extract `Info` using serde
async fn rest_api(info: web::Json<RESTInputModel>) -> Result<String> {
    Ok(format!("Welcome {}!", info.Metrics[0].Field))
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
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
            .route("/hello", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}