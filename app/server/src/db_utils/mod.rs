use actix_web::http::StatusCode;
use derive_more::{Display, Error, From};
use mysql::from_value_opt;
use mysql::prelude::Queryable;
use serde::{Serialize, Deserialize};

use crate::models;
use models::{RESTInputModel, ResponseData};

// use mysql::prelude::*;
use mysql::{Pool, PooledConn};

#[derive(Debug, Display, Error, From)]
pub enum PersistenceError {
    EmptyBankName,
    EmptyCountry,
    EmptyBranch,
    EmptyLocation,
    EmptyTellerName,
    EmptyCustomerName,
    MysqlError(mysql::Error),
    Unknown,
}

impl actix_web::ResponseError for PersistenceError {
    fn status_code(&self) -> StatusCode {
        match self {
            PersistenceError::EmptyBankName
            | PersistenceError::EmptyCountry
            | PersistenceError::EmptyBranch
            | PersistenceError::EmptyLocation
            | PersistenceError::EmptyTellerName
            | PersistenceError::EmptyCustomerName => StatusCode::BAD_REQUEST,

            PersistenceError::MysqlError(_) | PersistenceError::Unknown => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
}

pub fn execute_query(
    query: &String,
    sql_connection_pool: &mysql::Pool,
) -> Result<ResponseData, PersistenceError> {
    let mut conn = sql_connection_pool.get_conn()?;

    Ok(ResponseData {
        data: run_query(&query, &mut conn)?,
    })
}

fn run_query(
    query: &String,
    conn: &mut mysql::PooledConn,
) -> mysql::error::Result<Vec<Vec<String>>> {
    conn.query_map(query, |row: mysql::Row| {
        let test = sql_row_to_string_list(row);
        test
    })
}

fn sql_row_to_string_list(row: mysql::Row) -> Vec<String> {
    let mut string_list = Vec::new();

    for (index, column) in row.columns_ref().iter().enumerate() {
        if let Some(Ok(value)) = row.get_opt(index) {
            let value_as_string = from_value_opt::<String>(value);
            string_list.push(value_as_string.unwrap_or_else(|_| "NULL".to_string()));
        } else {
            string_list.push("NULL".to_string());
        }
    }

    string_list
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub datatype: String,
}

// Struct to represent table information
#[derive(Debug, Serialize, Deserialize)]
pub struct TableInfo {
    pub name: String,
    pub columns: Vec<ColumnInfo>,
}

// Function to fetch all table names in the database
pub fn fetch_all_tables(pool: &Pool) -> Result<Vec<String>, mysql::Error> {
    let mut conn = pool.get_conn()?;
    let query = "SELECT table_name FROM information_schema.tables WHERE table_schema = DATABASE()";
    let tables: Vec<String> = conn.query_map(query, |table_name| table_name)?;
    Ok(tables)
}

// Function to fetch columns and their data types for a given table
pub fn fetch_columns_for_table(pool: &Pool, table_name: &str) -> Result<Vec<ColumnInfo>, mysql::Error> {
    let mut conn = pool.get_conn()?;
    let query = format!(
        "SELECT column_name, data_type FROM information_schema.columns WHERE table_schema = DATABASE() AND table_name = '{}'",
        table_name
    );
    let columns: Vec<ColumnInfo> = conn.query_map(query, |(column_name, datatype)| ColumnInfo {
        name: column_name,
        datatype,
    })?;
    Ok(columns)
}

pub fn create_table_schema(pool: &Pool) -> () {
    match fetch_all_tables(&pool) {
        Ok(tables) => {
            let mut table_info_list: Vec<TableInfo> = Vec::new();
            for table_name in &tables {
                match fetch_columns_for_table(&pool, table_name) {
                    Ok(columns) => {
                        let table_info = TableInfo {
                            name: table_name.clone(),
                            columns,
                        };
                        table_info_list.push(table_info);
                    }
                    Err(err) => eprintln!("Error fetching columns for table {}: {:?}", table_name, err),
                }
            }

            // Convert the table_info_list to a JSON string
            let json_string = serde_json::to_string_pretty(&table_info_list)
                .expect("Error converting to JSON");

            // Write the JSON string to a file
            std::fs::write("data/table_schema_db.json", json_string)
                .expect("Error writing to file");
        }
        Err(err) => eprintln!("Error fetching tables: {:?}", err),
    }
}