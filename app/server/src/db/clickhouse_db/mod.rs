use crate::db::DBPool;
use crate::models::{AttributeValue,Column,DataResponse};
use crate::db_utils::PersistenceError;
use clickhouse_readonly::{ClickhouseResult, Pool, PoolConfigBuilder};


use futures_util::StreamExt;
use std::collections::HashMap;
use std::borrow::Cow;

pub static TABLE_QUERY: &str = "SELECT name
FROM system.tables
WHERE database = currentDatabase();
";

pub fn column_query(table_name: &str)-> String {
    format!(
        "SELECT name AS column_name, type AS data_type
        FROM system.columns
        WHERE database = currentDatabase() AND table = '{}';",
        table_name)

}

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
    log::info!("Executing Clickhouse Query");
    println!("{}",query);
    let mut hash_map: HashMap<String, AttributeValue> = HashMap::new();
    let mut hash_maps: Vec<HashMap<String, AttributeValue>> = Vec::new();
    if let Some(clickhouse_pool) = get_clickhouse_pool(&pool) {
        let mut handle = clickhouse_pool.get_handle().await?;
        let mut stream = handle.query(query).stream();
    
        while let Some(row) = stream.next().await {
            
            let row = row?;
            for index in 0..row.len(){
                if let Some(key) = column_headers.get(index) {
                    let col_name = row.name(index)?;
                    let col_type = row.sql_type(index)?;
                    match col_type.to_string(){
                        Cow::Borrowed("String") => {
                            let val: String = row.get(index)?;
                            hash_map.insert(
                                key.to_string(),
                                AttributeValue::String(val)
                            );
                        },
                        
                        Cow::Borrowed("Int8") | Cow::Borrowed("Int16") | Cow::Borrowed("Int32") | Cow::Borrowed("Int64") => {
                            let val: i64 = row.get(index)?;
                            hash_map.insert(
                                key.to_string(),
                                AttributeValue::Float(val as f32),
                            );
                        },
                        _ => {let _ = String::new();}
                        
                    }

                }    
                
            }
            hash_maps.push(hash_map.clone());
        }
    
        return Ok(DataResponse { data: hash_maps });
    }
    else{
        Err(PersistenceError::Unknown)
    }
}