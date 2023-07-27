use actix_web::http::StatusCode;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, Result};
use derive_more::{Display, Error, From};
use env_logger;
use log;
use mysql::prelude::Queryable;
use std::env;
mod models;
use models::{RESTInputModel, ResponseData, ResponseItem};
mod query_engine;

#[post("/api")]
async fn rest_api(json_query: web::Json<RESTInputModel>) -> Result<String> {
    let sql_query = query_engine::GetQuery(&json_query);

    Ok(format!("SQL: \n{}!", sql_query))
}

#[post("/get_query")]
async fn get_query(json_query: web::Json<RESTInputModel>) -> Result<String> {
    let sql_query = query_engine::GetQuery(&json_query);
    Ok(format!("SQL: \n{}!", sql_query))
}

#[derive(Debug, Display, Error, From)]
pub enum PersistenceError {
    EmptyBankName,
    EmptyCountry,
    EmptyBranch,
    EmptyLocation,
    EmptyTellerName,
    EmptyCustomerName,
    MysqlError(mysql::Error),
    Unknown,
}

impl actix_web::ResponseError for PersistenceError {
    fn status_code(&self) -> StatusCode {
        match self {
            PersistenceError::EmptyBankName
            | PersistenceError::EmptyCountry
            | PersistenceError::EmptyBranch
            | PersistenceError::EmptyLocation
            | PersistenceError::EmptyTellerName
            | PersistenceError::EmptyCustomerName => StatusCode::BAD_REQUEST,

            PersistenceError::MysqlError(_) | PersistenceError::Unknown => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
}

#[get("/sample_query")]
pub(crate) async fn sample_query(
    data: web::Data<mysql::Pool>,
) -> actix_web::Result<impl Responder> {
    let bank_response_data = web::block(move || get_bank_data(&data)).await??;
    Ok(web::Json(bank_response_data))
}

pub fn get_bank_data(pool: &mysql::Pool) -> Result<ResponseData, PersistenceError> {
    let mut conn = pool.get_conn()?;
    let mut query = r"
        SELECT category, price FROM products
        "
    .to_string();

    Ok(ResponseData {
        data: run_query(&mut query, &mut conn)?,
    })
}

fn run_query(
    query: &mut String,
    conn: &mut mysql::PooledConn,
) -> mysql::error::Result<Vec<ResponseItem>> {
    conn.query_map(query, |(my_bank_name, my_country)| ResponseItem {
        bank_name: my_bank_name,
        country: my_country,
    })
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

struct AppState {
    app_name: String,
}

fn get_conn_builder(
    db_user: String,
    db_password: String,
    db_host: String,
    db_port: u16,
    db_name: String,
) -> mysql::OptsBuilder {
    mysql::OptsBuilder::new()
        .ip_or_hostname(Some(db_host))
        .tcp_port(db_port)
        .db_name(Some(db_name))
        .user(Some(db_user))
        .pass(Some(db_password))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // initialize environment
    dotenv::dotenv().ok();

    // initialize logger
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("setting up app from environment");

    let db_user = env::var("MYSQL_USER").expect("MYSQL_USER is not set in .env file");
    let db_password = env::var("MYSQL_PASSWORD").expect("MYSQL_PASSWORD is not set in .env file");
    let db_host = env::var("MYSQL_HOST").expect("MYSQL_HOST is not set in .env file");
    let db_port = env::var("MYSQL_PORT").expect("MYSQL_PORT is not set in .env file");
    let db_name = env::var("MYSQL_DBNAME").expect("MYSQL_DBNAME is not set in .env file");
    let db_port = db_port.parse().unwrap();

    let builder = get_conn_builder(db_user, db_password, db_host, db_port, db_name);

    log::info!("initializing database connection");
    let pool = mysql::Pool::new(builder).unwrap();

    let shared_data = web::Data::new(pool);

    HttpServer::new(move || {
        App::new()
            .app_data(shared_data.clone())
            .app_data(web::Data::new(AppState {
                app_name: String::from("Actix Web"),
            }))
            .service(hello)
            .service(echo)
            .service(sample_query)
            .service(get_query)
            .service(rest_api)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
