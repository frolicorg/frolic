use crate::db::DBPool;
use crate::models::{AttributeValue,Column,DataResponse};
use crate::db_utils::PersistenceError;
use clickhouse_readonly::{ClickhouseResult, Pool, PoolConfigBuilder};


use futures_util::StreamExt;
use std::collections::HashMap;
use std::borrow::Cow;



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

pub async fn fetch_all_tables_clickhouse(
    dbpool: &DBPool,
) -> Result<Vec<String>, PersistenceError> {
    let query = "SELECT name
    FROM system.tables
    WHERE database = currentDatabase();
    ";
    let sample_query: &String = &String::from(query);
    let column_headers: Vec<String> = vec![String::from("name")];
    let table_data_response = run_query_clickhouse(&column_headers,sample_query,dbpool.clone()).await?;
    let table_names: Vec<String> = table_data_response.data
    .iter()
    .filter_map(|hash_map| hash_map.get("name"))
    .map(|attr_value| match attr_value {
        AttributeValue::String(s) => s.clone(),
        _ => String::new(), // Handle other cases if needed
    })
    .collect();
    println!("{}",table_names.join(","));
    Ok(table_names)

}
pub async fn fetch_all_columns_clickhouse(
    dbpool: &DBPool, table_name: &str
) -> Result<Vec<Column>, PersistenceError> {
    let query = format!(
        "SELECT name AS column_name, type AS data_type
        FROM system.columns
        WHERE database = currentDatabase() AND table = '{}';",
        table_name
    );
    let sample_query: &String = &String::from(query);
    let column_headers: Vec<String> = vec![String::from("column_name"),String::from("column_type")];
    let column_data_response = run_query_clickhouse(&column_headers,sample_query,dbpool.clone()).await?;
    let columns: Vec<Column> = column_data_response
        .data
        .iter()
        .map(|entry| {
            let name = match &entry.get("column_name") {
                Some(AttributeValue::String(s)) => s.clone(),
                _ => String::new(), // Handle the case where the attribute is not a String
            };

            let datatype = match &entry.get("column_type") {
                Some(AttributeValue::String(s)) => s.clone(),
                _ => String::new(), // Handle the case where the attribute is not a String
            };

            Column { name, datatype }
        })
        .collect();
    println!("{:?}", columns);    
    Ok(columns)
}

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
