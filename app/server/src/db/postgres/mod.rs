use crate::db::DBPool;
use crate::models::{AttributeValue,Column,DataResponse};
use crate::db_utils::PersistenceError;

use std::collections::HashMap;
use uuid::Uuid;

use deadpool_postgres::{Config, Manager, ManagerConfig, Pool, RecyclingMethod, Runtime};
use tokio_postgres::{NoTls,Row};
use chrono::NaiveDateTime;



pub fn postgres_pool_builder(db_user:&str,db_password:&str,db_host:&str,db_port:&u16,db_name:&str) -> Pool{
    let mut cfg = Config::new();
    cfg.dbname = Some(db_name.to_string());
    cfg.user = Some(db_user.to_string());
    cfg.host = Some(db_host.to_string());
    cfg.password = Some(db_password.to_string());
    cfg.port = Some(*db_port);
    cfg.manager = Some(ManagerConfig { recycling_method: RecyclingMethod::Fast });
    let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();
    pool
}

pub fn get_postgres_pool(dbpool: &DBPool) -> Option<&Pool> {
    match dbpool {
        DBPool::postgres(postgres_pool) => Some(postgres_pool),
        _ => None,
    }
}

pub async fn fetch_all_tables_postgres(
    dbpool: &DBPool,
) -> Result<Vec<String>, PersistenceError> {
    if let Some(postgres_pool) = get_postgres_pool(dbpool) {

        let query = "SELECT table_name
        FROM information_schema.tables
        WHERE table_schema = current_schema()
          AND table_name NOT LIKE 'sys_%'
          AND table_name NOT LIKE 'pg_%'
          AND table_name NOT IN ('geography_columns', 'geometry_columns');
        ";
        let mut client = postgres_pool.get().await.unwrap();
        let stmt = client.prepare(query).await.map_err(|err| PersistenceError::Unknown)?;

        let rows = client
            .query(&stmt, &[])
            .await
            .map_err(|err| PersistenceError::Unknown)?;

        let tables: Vec<String> = rows
            .iter()
            .map(|row| row.get::<_, String>("table_name"))
            .collect();
        println!("{}",tables.join(","));
        Ok(tables)
    }
    else{
        Err(PersistenceError::Unknown)
    }
}

pub async fn fetch_all_columns_postgres(
    dbpool: &DBPool, table_name: &str
) -> Result<Vec<Column>, PersistenceError> {
    if let Some(postgres_pool) = get_postgres_pool(dbpool) {
        let query = format!(
            "SELECT column_name, data_type  FROM information_schema.columns WHERE table_schema = current_schema() and table_name = '{}'",
            table_name
        );
        let mut client = postgres_pool.get().await.unwrap();
        let stmt = client.prepare(&query).await.map_err(|err| PersistenceError::Unknown)?;

        let rows = client
            .query(&stmt, &[])
            .await
            .map_err(|err| PersistenceError::Unknown)?;

        let columns: Vec<Column> = rows
            .iter()
            .map(|row| 
                Column {
                    name: row.get::<_, String>("column_name"),
                    datatype: row.get::<_, String>("data_type"),
                }
            )
            .collect();

        Ok(columns)
    }
    else{
        Err(PersistenceError::Unknown)
    }
}

pub async fn run_query_postgres(
    column_headers: &Vec<String>,
    query: &String,
    pool: DBPool,
) -> Result<DataResponse, PersistenceError>{
    log::info!("Executing PostGres Query");

    if let Some(postgres_pool) = get_postgres_pool(&pool) {

        let client = postgres_pool.get().await.unwrap();
        let stmt = client.prepare_cached(query).await.unwrap();
        let rows = client.query(&stmt, &[]).await.unwrap();
        // let column_head: Vec<String> = vec!["id".to_string(), "title".to_string()];

        let hash_maps: Vec<HashMap<String, AttributeValue>> = rows
        .iter()
        .map(|row| postgres_row_to_hash_map(&column_headers, row))
        .collect();
        let response_data = hash_maps;
        return Ok(DataResponse { data: response_data });
    }
    else{
        Err(PersistenceError::Unknown)
    }
}

pub fn postgres_row_to_hash_map(
    column_names: &Vec<String>, // Updated parameter type
    row: &Row,
) -> HashMap<String, AttributeValue> {
    let mut hash_map: HashMap<String, AttributeValue> = HashMap::new();
    let mut col_type = String::new();
    for column in row.columns().into_iter(){
        println!("{}",column.name().to_string());
        // println!("{}",);
        col_type = column.type_().to_string();
        println!("{}",col_type);
        match col_type.as_str(){
            "uuid" => {
                hash_map.insert(
                    column.name().to_string(),
                    AttributeValue::String(row.get::<_,Uuid>(column.name()).to_string()),
                );
            },
            "text" => {
                hash_map.insert(
                    column.name().to_string(),
                    AttributeValue::String(row.get::<_,String>(column.name()).to_string()),
                );
            },
            "timestamp" => {
                let timestamp_value: NaiveDateTime = row.get::<_, NaiveDateTime>(column.name());
                let formatted_timestamp: String = timestamp_value.to_string();
            
                hash_map.insert(
                    column.name().to_string(),
                    AttributeValue::String(formatted_timestamp),
                );
            },
            "int2" | "int4" => {
                let value: i32 = row.get::<_, i32>(column.name());
                hash_map.insert(
                    column.name().to_string(),
                    AttributeValue::Float(value as f32),
                );
            },
            "int8" => {
                let value: i64 = row.get::<_, i64>(column.name());
                hash_map.insert(
                    column.name().to_string(),
                    AttributeValue::Float(value as f32),
                );
            },
            "numeric" | "float4" | "float8" => {
                hash_map.insert(
                    column.name().to_string(),
                    AttributeValue::Float(row.get::<_, f32>(column.name())),
                );
            },
            _ => {let _ = String::new();}
        };

    }
    hash_map
}