use deadpool_postgres::{Config, Manager, ManagerConfig, Pool, RecyclingMethod, Runtime};
use tokio_postgres::{NoTls,Row};
use tokio_postgres::types::FromSql; 
use uuid::Uuid;
use crate::db::DBPool;
use std::collections::HashMap;
use crate::models;
use models::{AttributeValue};
use mysql::from_value_opt;


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
    hash_map
}