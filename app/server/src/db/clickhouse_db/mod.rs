// use crate::db::DBPool;
// use crate::models::{AttributeValue,Column,DataResponse};
// use crate::db_utils::PersistenceError;
use clickhouse_readonly::{ClickhouseResult, Pool, PoolConfigBuilder};

use futures_util::StreamExt;
// use std::collections::HashMap;
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

pub async fn clickhouse_test() -> ClickhouseResult<()> {


    let config = PoolConfigBuilder::new(
        "clickhouse://localhost:9000/".parse().unwrap(),
        "test".to_string(),
        "default".to_string(),
        "".to_string(),
        false,
    ).build();
    
    let pool = Pool::new(config);
    let mut handle = pool.get_handle().await?;

    let mut stream = handle.query("SELECT Comments FROM orders").stream();

    while let Some(row) = stream.next().await {
        let row = row?;
        let comment: String = row.get("Comments")?;
        // let ticker: String = row.get("asset_symbol")?;
        // let rate: String = row.get("deposit")?;
        eprintln!("Found {comment}");
    }

    Ok(())
}

// pub fn clickhouse_pool_builder(db_user:&str,db_password:&str,db_host:&str,db_port:&u16,db_name:&str) -> ClickhouseClientWrapper{
//     let database_url = format!(
//         "clickhouse://{}:{}",
//         db_host, db_port
//     );
//     let client = Client::default()
//     .with_url("http://localhost:8123")
//     .with_user(db_user)
//     .with_password(db_password)
//     .with_database(db_name);
//     ClickhouseClientWrapper(client)
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
