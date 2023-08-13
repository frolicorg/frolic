use crate::cache;
use crate::config::AppConfig;
use crate::models;
use crate::db;
use actix_web::http::StatusCode;
use derive_more::{Display, Error, From};
use log;
use memcache::Client;
use models::{DataRequest, DataResponse, Table};
use mysql::prelude::Queryable;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Read;
use db::{fetch_all_tables,fetch_columns_for_table,run_query};
use db::{DBPool};
use actix_web::rt::Runtime;
use clickhouse_readonly::{ClickhouseError};

// use mysql::prelude::*;
use cache::{
    clean_cache_if_needed, deserialize_data, hash_query_to_unique_id, sanitize_query,
    serialize_data,
};
use mysql::Pool;

const MAX_CACHE_SIZE: usize = 50;

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

impl From<ClickhouseError> for PersistenceError {
    fn from(error: ClickhouseError) -> Self {
        // You can decide how to map ClickhouseError variants to PersistenceError variants here
        PersistenceError::Unknown
    }
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Relationship {
    pub parent_table: String,
    pub child_table: String,
    pub parent_column: String,
    pub child_column: String,
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

pub fn get_column_headers(json_query: &DataRequest) -> Vec<String> {
    let mut column_headers: Vec<String> = Vec::new();

    if let Some(ref dimensions) = json_query.dimensions {
        column_headers.extend(dimensions.iter().map(|item| {
            if let Some(ref name) = item.name {
                name.clone()
            } else {
                item.field.clone()
            }
        }));
    }

    if let Some(ref metrics) = json_query.metrics {
        column_headers.extend(metrics.iter().map(|item| {
            if let Some(ref name) = item.name {
                name.clone()
            } else {
                item.field.clone()
            }
        }));
    }

    column_headers
}


pub fn add_table_relationship(input_file_path: &str, output_file_path: &str) -> () {
    // Read the JSON file into a string
    let mut json_str = String::new();
    File::open(&input_file_path)
        .and_then(|mut file| file.read_to_string(&mut json_str))
        .expect("Error reading JSON file");

    // Deserialize the JSON string into relationships vector
    let relationships: Vec<Relationship> =
        serde_json::from_str(&json_str).expect("Error parsing JSON");

    for relationship in &relationships {
        let cloned_relationship = relationship.clone();
        if let Err(err) = update_relationship(&output_file_path, cloned_relationship) {
            log::error!("Error: {:?}", err);
        } else {
            log::info!("Relationship updated/added successfully!");
        }
    }
}

fn update_relationship(
    file_path: &str,
    relationship: Relationship,
) -> Result<(), Box<dyn std::error::Error>> {
    // Read the JSON file

    let json_str = fs::read_to_string(file_path)?;

    // Deserialize the JSON into a vector of Relationship structs
    let mut tables: Vec<Table> = serde_json::from_str(&json_str)?;
    // let mut relationships: Vec<Relationship> = serde_json::from_str(&json_str)?;

    if let Some(table) = tables
        .iter_mut()
        .find(|t| t.name == relationship.parent_table)
    {
        // Create a new relationship entry
        let new_relationship = HashMap::from([(
            relationship.child_table.to_string(),
            (
                relationship.parent_column.to_string(),
                relationship.child_column.to_string(),
            ),
        )]);

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


pub fn execute_query(
    json_query: &DataRequest,
    query: &String,
    db_connection_pool: &DBPool,
    app_config: &AppConfig,
    cache_client: &Option<Client>,
) -> Result<DataResponse, PersistenceError> {
    let db_type = &app_config.database.db_type;
    let is_caching = &app_config.caching.cache_enabled;
    let caching_expiry = &app_config.caching.cache_expiry;
    // Check if the result is already in the cache
    let cache_key = format!("{}", hash_query_to_unique_id(query));

    log::info!("Caching : {}", is_caching);
    if *is_caching {
        if let Some(client) = cache_client {
            if let Ok(cached_result) = client.get::<String>(&cache_key) {
                if let Some(result) = cached_result {
                    match deserialize_data::<DataResponse>(&result) {
                        Ok(response) => return Ok(response),
                        Err(err) => log::info!("DeSerialization failed: {}", err),
                    }
                }
            }
        }
    }
    //if not found then run the query and add it to cache server
    let column_headers: Vec<String> = get_column_headers(&json_query);
    let rt = Runtime::new().unwrap();
    let response: Result<DataResponse, PersistenceError> = match rt.block_on(run_query(&column_headers, &query, db_connection_pool.clone(), db_type)) {
        Ok(res) => Ok(res),
        Err(err) => Err(PersistenceError::Unknown),
    };
    if *is_caching {
    
        if let Some(client) = cache_client {
            if let Some(data_response) = extract_data_response(&response) {
                // Use the extracted DataResponse
                match serialize_data::<String>(&data_response) {
                    Ok(json) => {
                        client.set(&cache_key, json, caching_expiry.clone()).ok();
                        // Remove the oldest item from the cache if the limit is reached
                        clean_cache_if_needed(client);
                    }
    
                    Err(err) => log::error!("Serialization failed: {}", err),
                }
            } else {
                return Err(PersistenceError::Unknown);
                // Handle the error case
            }    

        }
    }

    if let Some(data_response) = extract_data_response(&response) {
        // Use the extracted DataResponse
        return Ok(data_response.clone());
    } else {
        return Err(PersistenceError::Unknown);
        // Handle the error case
    }
}

fn extract_data_response(response: &Result<DataResponse, PersistenceError>) -> Option<&DataResponse> {
    match response {
        Ok(data_response) => Some(data_response),
        Err(_) => None,
    }
}

pub async fn fetch_schema(
    pool: DBPool,
    relationship_file: String,
    schema_file: String,
    db_type:String,
) -> String {
    println!("fetching {} scehma",db_type);
    create_table_schema(&pool, &schema_file,&db_type).await;
    add_table_relationship(&relationship_file, &schema_file);
    format!("Note : Please restart the Application so that the changed reflect")
    // Ok()
}

pub async fn create_table_schema(pool: &DBPool, output_file_path: &str,db_type:&str) -> () {
    match fetch_all_tables(&pool,&db_type).await {
        Ok(tables) => {
            let mut table_info_list: Vec<Table> = Vec::new();
            let mut relationships: Vec<HashMap<String, (String, String)>> = Vec::new();
            for table_name in &tables {
                match fetch_columns_for_table(&pool, table_name,&db_type).await {
                    Ok(columns) => {
                        let table_info = Table {
                            name: table_name.clone(),
                            columns,
                            relationships: relationships.clone(),
                        };
                        table_info_list.push(table_info);
                    }
                    Err(err) => {
                        log::error!("Error fetching columns for table {}: {:?}", table_name, err)
                    }
                }
            }

            // Convert the table_info_list to a JSON string
            let json_string =
                serde_json::to_string_pretty(&table_info_list).expect("Error converting to JSON");

            // Write the JSON string to a file
            std::fs::write(output_file_path, json_string).expect("Error writing to file");
        }
        Err(err) => log::error!("Error fetching tables: {:?}", err),
    }
}