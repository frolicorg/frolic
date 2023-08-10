use actix_cors::Cors;
use actix_web::{
    dev::ServiceRequest, get, post, web, App, Error, HttpResponse, HttpServer, Responder, Result,
};
use env_logger;
use log;
use models::{AppState, DataRequest, Table};
mod config;
mod db_utils;
mod cache;
mod models;
mod query_engine;
mod db;
use actix_web::middleware::Logger;
use actix_web_httpauth::extractors::bearer::{BearerAuth, Config};
use actix_web_httpauth::extractors::AuthenticationError;
mod middlewares;
use actix_web_httpauth::middleware::HttpAuthentication;
use memcache::Client;
use std::fs::File;
use std::io::Read;
use std::pin::Pin;
use db::{pool_builder};
use db_utils::{execute_query,fetch_schema};

#[post("/api")]
async fn rest_api(
    json_query: web::Json<DataRequest>,
    // sql_connection_pool: web::Data<mysql::Pool>,
    db_shared_data: web::Data<db::DBPool>,
    memcache_connection_client: web::Data<Option<Client>>,
    app_state: web::Data<AppState>,
) -> actix_web::Result<impl Responder> {
    let sql_query = query_engine::get_query(&json_query, &app_state.tables);
    let response_data = web::block(move || {
        execute_query(
            &json_query,
            &sql_query,
            &db_shared_data,
            &app_state.app_config,
            &memcache_connection_client,
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

fn read_tables_from_file(file_path: &str) -> Result<Vec<Table>, Box<dyn std::error::Error>> {
    // Read the contents of the file
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    // Deserialize the JSON data into a vector of Table structs
    let tables: Vec<Table> = serde_json::from_str(&contents)?;

    Ok(tables)
}

async fn validator(req: ServiceRequest, credentials: BearerAuth) -> Result<ServiceRequest, Error> {
    log::info!("Middleware");

    let config = req
        .app_data::<Config>()
        .map(|data| Pin::new(data).get_ref().clone())
        .unwrap_or_else(Default::default);

    match middlewares::validate_token(credentials.token()) {
        Ok(res) => {
            if res == true {
                Ok(req)
            } else {
                Err(AuthenticationError::from(config).into())
            }
        }
        Err(_) => Err(AuthenticationError::from(config).into()),
    }
}

// #[actix_web::main]
#[tokio::main]
async fn main() -> std::io::Result<()> {
    // initialize environment
    dotenv::dotenv().ok();

    // initialize logger
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("setting up app configurations");

    // read the configurations
    let config =
        config::read_config_file("config.toml").expect("Error reading the configuration file.");

    // setup database connection
    let db_config = config.database.clone();
    let db_type = db_config.db_type;
    let db_user = db_config.db_user;
    let db_password = db_config.db_password;
    let db_host = db_config.db_host;
    let db_name = db_config.db_name;
    let db_port = db_config.db_port;

    log::info!("initializing database connection");
    //setup the pool;
    let mut db_pool = db::DBPool::new();
    match pool_builder(&db_type, &db_user, &db_password, &db_host, &db_port, &db_name) {
        Ok(db_pool_local) => {
            // Use the database pool
            db_pool = db_pool_local;
        }
        Err(error) => {
            eprintln!("Error: {}", error);
        }
    };
    let db_shared_data = web::Data::new(db_pool.clone());
    // let result = execute_query(postgres_pool.clone()).await;
    //setup cache server client
    let cache_server =
        "memcache://".to_string() + &config.caching.cache_host.to_string() + ":11211";
    log::info!("{}", cache_server);

    let cache_client = match memcache::Client::connect(cache_server) {
        Ok(client) => Some(client),
        Err(_) => {
            log::error!("Error: Failed to connect to memcache server.");
            None
        }
    };

    let memcache_connection_client = web::Data::new(cache_client);

    log::info!("Importing table schema");
    if config.schema.fetch_schema == true {
        fetch_schema(
            db_pool.clone(),
            config.schema.relationship_file.clone(),
            config.schema.schema_file.clone(),
            db_type.clone(),
        )
        .await;
    }

    let tables = match read_tables_from_file(&config.schema.schema_file) {
        Ok(tables) => tables,
        Err(err) => {
            log::error!("{}", err);
            vec![]
        }
    };

    log::info!("Configuring authentication");
    log::info!("{}", config.authentication.authority);
    // let auth = HttpAuthentication::bearer(validator);

    HttpServer::new(move || {
        let cors = Cors::default().allow_any_origin().send_wildcard();

        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .wrap(Cors::permissive())
            // .wrap(auth)
            // .app_data(sql_shared_data.clone())
            .app_data(db_shared_data.clone())
            .app_data(memcache_connection_client.clone())
            .app_data(web::Data::new(AppState {
                app_name: String::from("Actix Web"),
                tables: tables.clone(),
                is_caching: config.caching.cache_enabled.clone(),
                caching_expiry: config.caching.cache_expiry.clone(),
                app_config: config.clone(),
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
