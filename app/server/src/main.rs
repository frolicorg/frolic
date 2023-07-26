use actix_web::{get, post, web, Result, App, HttpResponse, HttpServer, Responder};
// use serde::Deserialize;

mod models;
use models::RESTInputModel;

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

// #[derive(Deserialize)]
// struct Info {
//     username: String,
// }

/// extract `Info` using serde
async fn index(info: web::Json<RESTInputModel>) -> Result<String> {
    Ok(format!("Welcome {}!", info.Metrics[0].Field))
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
            .route("/test", web::post().to(index))
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}