use actix_web::http::StatusCode;
use derive_more::{Display, Error, From};
use mysql::from_value_opt;
use mysql::prelude::Queryable;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Read;
use crate::models;
use models::{RESTInputModel, ResponseData,Table,Column};
use memcache::Client;
use serde_json::{to_string,from_str};
use sha2::{Sha256, Digest};
use hex;
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


#[derive(Debug, Serialize, Deserialize,Clone)]
pub struct Relationship {
    pub parent_table: String,
    pub child_table: String,
    pub parent_column: String,
    pub child_column: String,
}

#[derive(Serialize,Deserialize)]
struct SerializedData {
    data: Vec<Vec<String>>,
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
    cache_client: &Client,
) -> Result<ResponseData, PersistenceError> {
    // Check if the result is already in the cache
    let cache_key = format!("{}", hash_query_to_unique_id(query));
    if let Ok(cached_result) = cache_client.get::<String>(&cache_key) {
        if let Some(result) = cached_result {
            let deserialized_data = deserialize_data(&result).unwrap();
            let result = ResponseData {
                data: deserialized_data,
            };
            return Ok(result);
        }
    }

    let mut conn = sql_connection_pool.get_conn()?;

    // Execute the query
    let result = ResponseData {
        data: run_query(&query, &mut conn)?,
    };
    match serialize_data(&result.data) {
        Ok(json) => {
            println!("{}",&cache_key);
            cache_client.set(&cache_key, json, 0).ok();
        }

        
        Err(err) => println!("Serialization failed: {}", err),
    }

    Ok(result)
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

// #[derive(Debug, Serialize, Deserialize)]
// pub struct ColumnInfo {
//     pub name: String,
//     pub datatype: String,
// }

// Struct to represent table information
// #[derive(Debug, Serialize, Deserialize)]
// pub struct TableInfo {
//     pub name: String,
//     pub columns: Vec<ColumnInfo>,
//     pub relationships: 
// }

// Function to fetch all table names in the database
pub fn fetch_all_tables(pool: &Pool) -> Result<Vec<String>, mysql::Error> {
    let mut conn = pool.get_conn()?;
    let query = "SELECT table_name FROM information_schema.tables WHERE table_schema = DATABASE()";
    let tables: Vec<String> = conn.query_map(query, |table_name| table_name)?;
    Ok(tables)
}

// Function to fetch columns and their data types for a given table
pub fn fetch_columns_for_table(pool: &Pool, table_name: &str) -> Result<Vec<Column>, mysql::Error> {
    let mut conn = pool.get_conn()?;
    let query = format!(
        "SELECT column_name, data_type FROM information_schema.columns WHERE table_schema = DATABASE() AND table_name = '{}'",
        table_name
    );
    let columns: Vec<Column> = conn.query_map(query, |(column_name, datatype)| Column {
        name: column_name,
        datatype,
    })?;
    Ok(columns)
}

pub fn update_relationship(
    file_path: &str,
    relationship: Relationship,
) -> Result<(), Box<dyn std::error::Error>> {
    // Read the JSON file
    
    let json_str = fs::read_to_string(file_path)?;

    // Deserialize the JSON into a vector of Relationship structs
    let mut tables: Vec<Table> = serde_json::from_str(&json_str)?;
    // let mut relationships: Vec<Relationship> = serde_json::from_str(&json_str)?;

    if let Some(table) = tables.iter_mut().find(|t| t.name == relationship.parent_table) {
        // Create a new relationship entry
        let new_relationship = HashMap::from([(relationship.child_table.to_string(), (relationship.parent_column.to_string(), relationship.child_column.to_string()))]);

        // Insert the new relationship into the table's relationships
        table.relationships.push(new_relationship);
    // Find the relationship to update
    }

    // Serialize the modified vector back to JSON
    let updated_json_str = serde_json::to_string_pretty(&tables)?;

    // Write the updated JSON back to the file
    fs::write(file_path, updated_json_str)?;

    Ok(())
}

pub fn add_table_relationship(input_file_path: &str,output_file_path: &str) -> () {

    // Read the JSON file into a string
    let mut json_str = String::new();
    File::open(&input_file_path)
        .and_then(|mut file| file.read_to_string(&mut json_str))
        .expect("Error reading JSON file");

    // Deserialize the JSON string into relationships vector
    let relationships: Vec<Relationship> = serde_json::from_str(&json_str)
        .expect("Error parsing JSON");
    
    for relationship in &relationships {
        let cloned_relationship = relationship.clone();
        if let Err(err) = update_relationship(&output_file_path, cloned_relationship) {
            eprintln!("Error: {:?}", err);
        } else {
            println!("Relationship updated/added successfully!");
        }
    }
}

pub fn create_table_schema(pool: &Pool,output_file_path: &str) -> () {
    match fetch_all_tables(&pool) {
        Ok(tables) => {
            let mut table_info_list: Vec<Table> = Vec::new();
            let mut relationships: Vec<HashMap<String, (String, String)>> = Vec::new();
            for table_name in &tables {
                match fetch_columns_for_table(&pool, table_name) {
                    Ok(columns) => {
                        let table_info = Table {
                            name: table_name.clone(),
                            columns,
                            relationships:relationships.clone(),
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
            std::fs::write(output_file_path, json_string)
                .expect("Error writing to file");
        }
        Err(err) => eprintln!("Error fetching tables: {:?}", err),
    }
}
// Function to serialize the data output of run_query to JSON
fn serialize_data(data: &Vec<Vec<String>>) -> Result<String, String> {
    // Convert the data into a structured format
    let result = SerializedData { data: data.to_vec() };
    match to_string(&result) {
        Ok(serialized) => Ok(serialized),
        Err(err) => Err(format!("Serialization error: {}", err)),
    }
}
fn deserialize_data(serialized: &str) -> Result<Vec<Vec<String>>, String> {
    match from_str::<SerializedData>(serialized) {
        Ok(deserialized) => Ok(deserialized.data),
        Err(err) => Err(format!("Deserialization error: {}", err)),
    }
}

fn sanitize_query(query: &str) -> String {
    query.chars().filter(|c| c.is_alphanumeric()).collect()
}

fn hash_query_to_unique_id(query: &str) -> String {
    // Create a SHA-256 hash of the sanitized query
    let sanitized_query: String = query.chars().filter(|c| c.is_alphanumeric()).collect();
    let mut hasher = Sha256::new();
    hasher.update(sanitized_query.as_bytes());
    let hash_result = hasher.finalize();

    // Convert the hash bytes to a hexadecimal string and truncate it if necessary
    let hex_hash = hex::encode(hash_result);
    let max_length = 249; // Maximum supported key length
    if hex_hash.len() <= max_length {
        hex_hash
    } else {
        hex_hash[..max_length].to_string()
    }
}