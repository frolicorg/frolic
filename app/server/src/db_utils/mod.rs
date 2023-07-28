use actix_web::http::StatusCode;
use derive_more::{Display, Error, From};
use mysql::from_value_opt;
use mysql::prelude::Queryable;

use crate::models;
use models::{RESTInputModel, ResponseData, ResponseItem};

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

pub fn execute_query(query: &String, pool: &mysql::Pool) -> Result<ResponseData, PersistenceError> {
    let mut conn = pool.get_conn()?;

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
