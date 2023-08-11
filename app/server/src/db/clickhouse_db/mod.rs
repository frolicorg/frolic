use crate::db::DBPool;
use crate::models::{AttributeValue,Column,DataResponse};
use crate::db_utils::PersistenceError;
use clickhouse_readonly::{ClickhouseResult, Pool, PoolConfigBuilder};

use futures_util::StreamExt;
use std::collections::HashMap;
// use std::{env, error::Error};
// use futures_util::StreamExt;
// use serde::Deserialize;

// use crate::db::{ClickhouseClientWrapper};
// use clickhouse::Client;
// use clickhouse::Row;


// #[derive(Row, Deserialize)]
// struct MyRow<'a> {
//     no: u32,
//     name: &'a str,
// }



pub fn clickhouse_pool_builder(db_user:&str,db_password:&str,db_host:&str,db_port:&u16,db_name:&str) -> Pool{

    let config = PoolConfigBuilder::new(
        "clickhouse://localhost:9000/".parse().unwrap(),
        "test".to_string(),
        "default".to_string(),
        "".to_string(),
        false,
    ).build();
    
    let pool = Pool::new(config);
    pool
}

pub fn get_clickhouse_pool(dbpool: &DBPool) -> Option<&Pool> {
    match dbpool {
        DBPool::clickhouse(clickhouse_pool) => Some(clickhouse_pool),
        _ => None,
    }
}

pub async fn run_query_clickhouse(
    column_headers: &Vec<String>,
    query: &String,
    pool: DBPool,
) -> Result<DataResponse, PersistenceError>{
    log::info!("Executing PostGres Query");
    
    let mut hash_map: HashMap<String, AttributeValue> = HashMap::new();
    let mut hash_maps: Vec<HashMap<String, AttributeValue>> = Vec::new();
    if let Some(clickhouse_pool) = get_postgres_pool(&pool) {

        let mut handle = clickhouse_pool.get_handle().await?;

        let mut stream = handle.query("SELECT Comments FROM orders").stream();
    
        while let Some(row) = stream.next().await {
            let row = row?;
            // let comment: String = row.get("Comments")?;
            hash_maps.insert("Comments",AttributeValue::String(row.get::<_,String>("Comments")));
            // let ticker: String = row.get("asset_symbol")?;
            // let rate: String = row.get("deposit")?;
            eprintln!("Found {comment}");
        }
    
       
    


        // let client = postgres_pool.get().await.unwrap();
        // let stmt = client.prepare_cached(query).await.unwrap();
        // let rows = client.query(&stmt, &[]).await.unwrap();
        // // let column_head: Vec<String> = vec!["id".to_string(), "title".to_string()];

        // let hash_maps: Vec<HashMap<String, AttributeValue>> = rows
        // .iter()
        // .map(|row| postgres_row_to_hash_map(&column_headers, row))
        // .collect();
        // let response_data = hash_maps;
        return Ok(DataResponse { data: hash_maps });
    }
    else{
        Err(PersistenceError::Unknown)
    }
}
// pub async fn clickhouse_test() -> ClickhouseResult<()> {

// }
// pub fn get_clickhouse_pool(dbpool: &DBPool) -> Option<&ClickhouseClientWrapper> {
//     match dbpool {
//         DBPool::clickhouse(clickhouse_pool) => Some(clickhouse_pool),
//         _ => None,
//     }
// }

// pub async fn run_query_clickhouse(
//     column_headers: &Vec<String>,
//     query: &String,
//     pool: DBPool,
// ) -> Result<DataResponse, PersistenceError>{
//     log::info!("Executing PostGres Query");

//     if let Some(clickhouse_pool) = get_clickhouse_pool(&pool) {

//         let mut cursor = clickhouse_pool.unwrap()
//         .query("SELECT ?fields FROM some WHERE no BETWEEN ? AND ?")
//         .bind(500)
//         .bind(504)
//         .fetch::<MyRow<'_>>()?;
    
//         while let Some(row) = cursor.next().await? { 
//             println!("Printing");
//         }
//         // let stmt = client.prepare_cached(query).await.unwrap();
//         // let rows = client.query(&stmt, &[]).await.unwrap();
//         // let column_head: Vec<String> = vec!["id".to_string(), "title".to_string()];

//         let mut hash_map: HashMap<String, AttributeValue> = HashMap::new();
//         let response_data = hash_map;
//         return Ok(DataResponse { data: response_data });
//     }
//     else{
//         Err(PersistenceError::Unknown)
//     }
// }
