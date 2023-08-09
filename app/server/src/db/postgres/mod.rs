use deadpool_postgres::{Config, Manager, ManagerConfig, Pool, RecyclingMethod, Runtime};
use tokio_postgres::{NoTls,Row};
use tokio_postgres::types::FromSql; 
use uuid::Uuid;
use crate::db::DBPool;
use std::collections::HashMap;
use crate::models;
use models::{AttributeValue};
use mysql::from_value_opt;
use chrono::NaiveDateTime;


pub fn postgresPoolBuilder(db_user:String,db_password:String,db_host:String,db_port:u16,db_name:String) -> Pool{
    let mut cfg = Config::new();
    cfg.dbname = Some(db_name);
    cfg.user = Some(db_user);
    cfg.host = Some(db_host);
    cfg.password = Some(db_password);
    cfg.port = Some(db_port);
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

pub fn postgres_row_to_hash_map(
    column_names: &Vec<String>, // Updated parameter type
    row: &Row,
) -> HashMap<String, AttributeValue> {
    let mut hash_map: HashMap<String, AttributeValue> = HashMap::new();
    let mut col_type = String::new();
    for Column in row.columns().into_iter(){
        println!("{}",Column.name().to_string());
        // println!("{}",);
        col_type = Column.type_().to_string();
        println!("{}",col_type);
        match col_type.as_str(){
            "uuid" => {
                hash_map.insert(
                    Column.name().to_string(),
                    AttributeValue::String(row.get::<_,Uuid>(Column.name()).to_string()),
                );
            },
            "text" => {
                hash_map.insert(
                    Column.name().to_string(),
                    AttributeValue::String(row.get::<_,String>(Column.name()).to_string()),
                );
            },
            "timestamp" => {
                let timestamp_value: NaiveDateTime = row.get::<_, NaiveDateTime>(Column.name());
                let formatted_timestamp: String = timestamp_value.to_string();
            
                hash_map.insert(
                    Column.name().to_string(),
                    AttributeValue::String(formatted_timestamp),
                );
            },
            "int2" | "int4" => {
                let value: i32 = row.get::<_, i32>(Column.name());
                hash_map.insert(
                    Column.name().to_string(),
                    AttributeValue::Float(value as f32),
                );
            },
            "int8" => {
                let value: i64 = row.get::<_, i64>(Column.name());
                hash_map.insert(
                    Column.name().to_string(),
                    AttributeValue::Float(value as f32),
                );
            },
            "numeric" | "float4" | "float8" => {
                hash_map.insert(
                    Column.name().to_string(),
                    AttributeValue::Float(row.get::<_, f32>(Column.name())),
                );
            },
            // "int2" | "int4" | "int8" | "numeric" | "float4" | "float8" => {
            //     hash_map.insert(
            //         Column.name().to_string(),
            //         AttributeValue::Float(row.get::<_,f32>(Column.name()) as f32),
            //     );
            // },
            _ => {
                // Handle other cases if needed
                String::new(); // or any other default value
            }
        };

    }
    hash_map
}