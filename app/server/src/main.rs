use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, Result};
use env_logger;
use log;
use models::{AppState, DataRequest, Table};
use std::env;
mod db_utils;
use db_utils::{add_table_relationship, create_table_schema};
mod cache;
mod models;
mod query_engine;
use memcache::Client;
use std::fs::File;
use std::io::Read;

#[post("/api")]
async fn rest_api(
    json_query: web::Json<DataRequest>,
    sql_connection_pool: web::Data<mysql::Pool>,
    memcache_connection_client: web::Data<Option<Client>>,
    app_state: web::Data<AppState>,
) -> actix_web::Result<impl Responder> {
    let mut is_caching = app_state.is_caching.clone();
    let sql_query = query_engine::get_query(&json_query, &app_state.tables);
    let response_data = web::block(move || {
        db_utils::execute_query(
            &json_query,
            &sql_query,
            &sql_connection_pool,
            &memcache_connection_client,
            &is_caching,
            &app_state.caching_expiry,
        )
    })
    .await??;
    Ok(web::Json(response_data))
}

#[post("/get_query")]
async fn get_query(
    json_query: web::Json<DataRequest>,
    app_state: web::Data<AppState>,
) -> Result<String> {
    let sql_query = query_engine::get_query(&json_query, &app_state.tables);
    Ok(format!("SQL:\n{}!", sql_query))
}

#[get("/fetch_schema")]
async fn fetch_schema(sql_connection_pool: web::Data<mysql::Pool>) -> Result<String> {
    let output_file_path = "data/table_schema_db.json";
    create_table_schema(&sql_connection_pool, output_file_path);

    let input_file_path = "data/relationships.json";
    add_table_relationship(input_file_path, output_file_path);

    Ok(format!(
        "Note : Please restart the Application so that the changed reflect"
    ))
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
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

fn read_tables_from_file(file_path: &str) -> Result<Vec<Table>, Box<dyn std::error::Error>> {
    // Read the contents of the file
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    // Deserialize the JSON data into a vector of Table structs
    let tables: Vec<Table> = serde_json::from_str(&contents)?;

    Ok(tables)
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
    let sql_shared_data = web::Data::new(pool.clone());

    let cache_client = match memcache::Client::connect("memcache://memcache:11211") {
        Ok(client) => Some(client),
        Err(_) => {
            eprintln!("Error: Failed to connect to memcache server.");
            None
        }
    };
    // let memcache_shared_data = cache_client.clone();
    let memcache_connection_client = web::Data::new(cache_client);
    log::info!("importing table schema");

    //import tabled from schema file
    let output_file_path = "data/table_schema_db.json";
    let tables = match read_tables_from_file(&output_file_path) {
        Ok(tables) => tables,
        Err(err) => {
            log::error!("{}", err);
            vec![]
        }
    };

    let is_caching = true;
    let caching_expiry = 3600;

    // let tables_json = serde_json::to_string_pretty(&tables).unwrap();
    // log::info!("{}", tables_json);

    HttpServer::new(move || {
        App::new()
            .app_data(sql_shared_data.clone())
            .app_data(memcache_connection_client.clone())
            .app_data(web::Data::new(AppState {
                app_name: String::from("Actix Web"),
                tables: tables.clone(),
                is_caching: is_caching.clone(),
                caching_expiry: caching_expiry.clone(),
            }))
            .service(hello)
            .service(echo)
            .service(get_query)
            .service(rest_api)
            .service(fetch_schema)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
