//a pool builder is needed - output will be a connection pool
use deadpool_postgres::{Pool};
mod postgres;
mod mysql_db;
use crate::db::postgres::get_postgres_pool;
use crate::db_utils::{get_column_headers};
use crate::cache::{
    clean_cache_if_needed, deserialize_data, hash_query_to_unique_id, sanitize_query,
    serialize_data,
};
use postgres::{postgresPoolBuilder,postgres_row_to_hash_map};
use mysql_db::{mysqlPoolBuilder,get_mysql_pool,sql_row_to_hash_map};
use crate::models::{AttributeValue, Column, DataRequest, DataResponse, Table};
use mysql::prelude::Queryable;
use memcache::Client;
use crate::db_utils::PersistenceError;
use std::collections::HashMap;
use actix_web::rt::Runtime;
// use tokio::runtime;


#[derive(Debug, Clone)]
pub enum DBPool {
    mysql(mysql::Pool),
    postgres(deadpool_postgres::Pool),
    None
}

impl DBPool{
    pub fn new()->DBPool{
        DBPool::None
    }
}

pub fn poolBuilder(db_type:String,db_user:String,db_password:String,db_host:String,db_port:u16,db_name:String) -> Result<DBPool, String>{
    if (db_type == "postgres"){
        let pool = postgresPoolBuilder(db_user, db_password, db_host, db_port, db_name);
        let db_pool = DBPool::postgres(pool);
        Ok(db_pool)
    }
    else if db_type == "mysql"{
        let pool = mysqlPoolBuilder(db_user, db_password, db_host, db_port, db_name);
        let db_pool = DBPool::mysql(pool);
        Ok(db_pool)
    }
    else{
        Err("Unsupported database type".to_string())
    }
}

pub fn execute_query(
    json_query: &DataRequest,
    query: &String,
    db_connection_pool: &DBPool,
    db_type: &String,
    cache_client: &Option<Client>,
    is_caching: &bool,
    caching_expiry: &u32,
) -> Result<DataResponse, PersistenceError> {


    // Check if the result is already in the cache
    let cache_key = format!("{}", hash_query_to_unique_id(query));

    log::info!("Caching : {}", is_caching);
    if *is_caching {
        if let Some(client) = cache_client {
            if let Ok(cached_result) = client.get::<String>(&cache_key) {
                if let Some(result) = cached_result {
                    match deserialize_data::<DataResponse>(&result) {
                        Ok(response) => return Ok(response),
                        Err(err) => log::info!("DeSerialization failed: {}", err),
                    }
                }
            }
        }
    }



    let column_headers: Vec<String> = get_column_headers(&json_query);
    // Execute the query
    let rt = Runtime::new().unwrap();
    let response: Result<DataResponse, PersistenceError> = match rt.block_on(run_query(&column_headers, &query, db_connection_pool.clone(), db_type.to_string())) {
        Ok(res) => Ok(res),
        Err(err) => Err(PersistenceError::Unknown),
    };
    // let response = match run_query(&column_headers, &query, db_connection_pool.clone(),db_type.to_string()){
    //     Ok(res)=>Ok(res),
    //     Err(err)=>Err(PersistenceError::MysqlError(err))
    // };

    if *is_caching {
    
        if let Some(client) = cache_client {
            if let Some(data_response) = extract_data_response(&response) {
                // Use the extracted DataResponse
                match serialize_data::<String>(&data_response) {
                    Ok(json) => {
                        client.set(&cache_key, json, caching_expiry.clone()).ok();
                        // Remove the oldest item from the cache if the limit is reached
                        clean_cache_if_needed(client);
                    }
    
                    Err(err) => log::error!("Serialization failed: {}", err),
                }
            } else {
                return Err(PersistenceError::Unknown);
                // Handle the error case
            }    

        }
    }

    if let Some(data_response) = extract_data_response(&response) {
        // Use the extracted DataResponse
        return Ok(data_response.clone());
    } else {
        return Err(PersistenceError::Unknown);
        // Handle the error case
    }
}


fn extract_data_response(response: &Result<DataResponse, PersistenceError>) -> Option<&DataResponse> {
    match response {
        Ok(data_response) => Some(data_response),
        Err(_) => None,
    }
}
pub async fn run_query(
    column_headers: &Vec<String>,
    query: &String,
    pool: DBPool,
    db_type: String,
) -> Result<DataResponse, PersistenceError> {
    if db_type == "mysql" {
        log::info!("Executing MySQL Query");

        if let Some(mysql_pool) = get_mysql_pool(&pool) {
            let mut conn = match mysql_pool.get_conn() {
                Ok(conn) => conn,
                Err(err) => return Err(PersistenceError::MysqlError(err)),
            };

            let response_data = match conn.query_map(query, |row: mysql::Row| {
                sql_row_to_hash_map(column_headers, &row)
            }) {
                Ok(response_data) => response_data,
                Err(err) => return Err(PersistenceError::MysqlError(err)),
            };

            return Ok(DataResponse { data: response_data });
        }
    }
    else if db_type == "postgres" {
        log::info!("Executing PostGres Query");

        if let Some(postgres_pool) = get_postgres_pool(&pool) {
        
            // let mut client = match postgres_pool.get().await.unwrap() {
            //     Ok(client) => client,
            //     Err(err) => return Err(PersistenceError::MysqlError(err)),
            // };

            let mut client = postgres_pool.get().await.unwrap();
            let stmt = client.prepare_cached("select * from your_table_name;").await.unwrap();
            let rows = client.query(&stmt, &[]).await.unwrap();
            // let handle = thread::spawn(move || {
            //     let mut client = postgres_pool_clone.lock().unwrap(); // Lock the mutex if needed
            //     let stmt = client.prepare_cached(&query).unwrap();
            //     let rows = client.query(&stmt, &[]).unwrap();
            // });
            
            // handle.join().unwrap();
            let column_head: Vec<String> = vec!["id".to_string(), "title".to_string()];

            let hash_maps: Vec<HashMap<String, AttributeValue>> = rows
            .iter()
            .map(|row| postgres_row_to_hash_map(&column_head, row))
            .collect();
            // for row in rows {
            //     let event_id: Uuid = row.get("id");
            //     let event_name: String = row.get("title");
            //     // You can access other columns in a similar way
        
            //     println!("Event ID: {}, Event Name: {}", event_id, event_name);
            // }
            
            // format!("Done")
            let response_data = hash_maps;

            // let response_data = match conn.query_map(query, |row: mysql::Row| {
            //     postgres_row_to_hash_map(column_headers, &row)
            // }) {
            //     Ok(response_data) => response_data,
            //     Err(err) => return Err(PersistenceError::MysqlError(err)),
            // };

            return Ok(DataResponse { data: response_data });
        }
    }
    let response_data = Vec::<HashMap<String, AttributeValue>>::new();
    return Ok(DataResponse { data: response_data });
    // Err(PersistenceError::Unknown)
}


//execute query function is required - output will be a Result<DataResponse, PersistenceError>

