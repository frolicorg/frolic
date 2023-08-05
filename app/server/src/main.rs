use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, Result};
use env_logger;
use log;
use models::{AppState, DataRequest, Table};
use std::env;
mod db_utils;
mod config;
use db_utils::{add_table_relationship, create_table_schema};
mod cache;
mod models;
mod query_engine;
use memcache::Client;
use std::fs::File;
use std::io::Read;
use crate::config::AppConfig;

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

    log::info!("setting up app configurations");

    
    // read the configurations
    let config = config::read_config_file("config.ini").expect("Error reading the configuration file.");  
    
    //setup database connection
    let db_user = config.database.db_user;
    let db_password = config.database.db_password;
    let db_host = config.database.db_host;
    let db_name = config.database.db_name;
    let db_port = config.database.db_port;
    let builder = get_conn_builder(db_user, db_password, db_host, db_port, db_name);

    log::info!("initializing database connection");
    let pool = mysql::Pool::new(builder).unwrap();
    let sql_shared_data = web::Data::new(pool.clone());
    
    //setup cache server client
    let cache_server = "memcache://".to_string() + &config.caching.cache_host.to_string() + ":11211";
    println!("{}",cache_server);
    let cache_client = match memcache::Client::connect(cache_server) {
        Ok(client) => Some(client),
        Err(_) => {
            log::error!("Error: Failed to connect to memcache server.");
            None
        }
    };
    let memcache_shared_data = cache_client.clone();
    let memcache_connection_client = web::Data::new(cache_client);

    //fetch the schema from the database

    log::info!("importing table schema");
    if (config.other.fetch_schema == true){
        db_utils::fetch_schema(&pool.clone(),config.other.relationship_file.clone(),config.other.schema_file.clone());
    }
    let tables = match read_tables_from_file(&config.other.schema_file) {
        Ok(tables) => tables,
        Err(err) => {
            log::error!("{}", err);
            vec![]
        }
    };


    HttpServer::new(move || {
        App::new()
            .app_data(sql_shared_data.clone())
            .app_data(memcache_connection_client.clone())
            .app_data(web::Data::new(AppState {
                app_name: String::from("Actix Web"),
                tables: tables.clone(),
                is_caching: config.caching.cache_enabled.clone(),
                caching_expiry: config.caching.cache_expiry.clone(),
            }))
            .service(hello)
            .service(echo)
            .service(get_query)
            .service(rest_api)
            // .service(fetch_schema)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
