//a pool builder is needed - output will be a connection pool
use deadpool_postgres::{Pool};
mod postgres;
mod mysql_db;
mod clickhouse_db;

use crate::models::{AttributeValue, Column, DataRequest, DataResponse, Table};
use mysql::prelude::Queryable;
use memcache::Client;
use crate::db_utils::PersistenceError;
use std::collections::HashMap;
use log::error;
use std::fmt;

// use clickhouse::{Client as ClickhouseClient};
use postgres::{postgres_pool_builder,fetch_all_tables_postgres,fetch_all_columns_postgres,run_query_postgres};
use mysql_db::{mysql_pool_builder,fetch_all_tables_mysql,fetch_all_columns_mysql,run_query_mysql};
use clickhouse_db::{clickhouse_test};
// use clickhouse_db::{clickhouse_pool_builder};

// use tokio::runtime;

// pub struct ClickhouseClientWrapper(ClickhouseClient);

// impl ClickhouseClientWrapper {
//     // Method to unwrap and get the inner clickhouse::Client
//     pub fn unwrap(self) -> ClickhouseClient {
//         self.0
//     }
// }
// impl fmt::Debug for ClickhouseClientWrapper {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         // Format the wrapped ClickhouseClient instance here
//         write!(f, "ClickhouseClientWrapper {{ /* details here */ }}")
//     }
// }

// impl Clone for ClickhouseClientWrapper {
//     fn clone(&self) -> Self {
//         ClickhouseClientWrapper(self.0.clone())
//     }
// }
#[derive(Debug, Clone)]
pub enum DBPool {
    mysql(mysql::Pool),
    postgres(deadpool_postgres::Pool),
    // clickhouse(ClickhouseClientWrapper),
    None
}


impl DBPool{
    pub fn new()->DBPool{
        DBPool::None
    }
}

pub fn pool_builder(db_type:&str,db_user:&str,db_password:&str,db_host:&str,db_port:&u16,db_name:&str) -> Result<DBPool, String>{
    match db_type {
        "postgres" => {
            let pool = postgres_pool_builder(db_user, db_password, db_host, db_port, db_name);
            let db_pool = DBPool::postgres(pool);
            Ok(db_pool)
        },
        "mysql" => {
            let pool = mysql_pool_builder(db_user, db_password, db_host, db_port, db_name);
            let db_pool = DBPool::mysql(pool);
            Ok(db_pool)
        },
        // "clickhouse" => {
        //     let pool = clickhouse_pool_builder(db_user, db_password, db_host, db_port, db_name);
        //     let db_pool = DBPool::clickhouse(pool);
        //     Ok(db_pool)
        // }
        _ => Err("Unsupported database type".to_string())
    }
}


pub async fn run_query(
    column_headers: &Vec<String>,
    query: &String,
    pool: DBPool,
    db_type: &str,
) -> Result<DataResponse, PersistenceError> {
    if let Err(err) = clickhouse_test().await {
        eprintln!("Error: {}", err);
    } else {
        println!("Connection successful");
    }
    match db_type {
        "mysql" => {
            log::info!("Executing MySQL Query");
            let response_data = run_query_mysql(column_headers, query, pool).await;
            return response_data;
        },
        "postgres" => {
            let response_data = run_query_postgres(column_headers, query, pool).await;
            return response_data;
        },
        _ => {
            error!("Unsupported database type: {}", db_type);
            let response_data = Vec::<HashMap<String, AttributeValue>>::new();
            Ok(DataResponse { data: response_data })
        }
    }
    // Err(PersistenceError::Unknown)
}


pub async fn fetch_all_tables(pool: &DBPool, db_type: &str) -> Result<Vec<String>, PersistenceError> {
    match db_type {
        "mysql" => {
            log::info!("Executing MySQL Query");
            fetch_all_tables_mysql(pool)
        }
        "postgres" => {
            log::info!("Fetching Postgres Tables");
            fetch_all_tables_postgres(pool).await
        }
        _ => Err(PersistenceError::Unknown),
    }
}

pub async fn fetch_columns_for_table(pool: &DBPool, table_name: &str,db_type:&str) -> Result<Vec<Column>, PersistenceError> {
    match db_type {
        "mysql" => {
            log::info!("Executing MySQL Query");
            fetch_all_columns_mysql(pool,table_name)
        }
        "postgres" => {
            log::info!("Fetching Postgres Tables");
            fetch_all_columns_postgres(pool,table_name).await
        }
        _ => Err(PersistenceError::Unknown),
    }
}

//execute query function is required - output will be a Result<DataResponse, PersistenceError>

