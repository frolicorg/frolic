use std::collections::HashMap;
use crate::models::{AttributeValue,Column,DataResponse};
use crate::db::DBPool;
use crate::db::PersistenceError;
use mysql::prelude::Queryable;
use mysql::from_value_opt;

pub static TABLE_QUERY: &str = "SELECT table_name FROM information_schema.tables WHERE table_schema = DATABASE()";

pub fn column_query(table_name: &str)-> String {
    format!(
        "SELECT column_name, data_type FROM information_schema.columns WHERE table_schema = DATABASE() AND table_name = '{}'",
        table_name)
}

pub fn mysql_pool_builder(db_user:&str,db_password:&str,db_host:&str,db_port:&u16,db_name:&str) -> mysql::Pool{
    let builder = mysql::OptsBuilder::new()
    .ip_or_hostname(Some(db_host))
    .tcp_port(*db_port)
    .db_name(Some(db_name))
    .user(Some(db_user))
    .pass(Some(db_password));

    let pool = mysql::Pool::new(builder).unwrap();
    pool
}
pub fn get_mysql_pool(dbpool: &DBPool) -> Option<&mysql::Pool> {
    match dbpool {
        DBPool::mysql(mysql_pool) => Some(mysql_pool),
        _ => None,
    }
}

pub fn sql_row_to_hash_map(
    column_headers: &Vec<String>,
    row: &mysql::Row,
) -> HashMap<String, AttributeValue> {
    let mut hash_map: HashMap<String, AttributeValue> = HashMap::new();

    for (index, column) in row.columns_ref().iter().enumerate() {
        if let Some(Ok(value)) = row.get_opt(index) {
            if let Some(key) = column_headers.get(index) {
                let value_as_float = from_value_opt::<f32>(value);

                match value_as_float {
                    Ok(float_value) => {
                        hash_map.insert(
                            key.to_string(),
                            AttributeValue::Float(round_float_decimals(&float_value)),
                        );
                    }
                    Err(error) => {
                        hash_map.insert(
                            key.to_string(),
                            AttributeValue::String(
                                from_value_opt::<String>(error.0)
                                    .unwrap_or_else(|_| "NULL".to_string()),
                            ),
                        );
                    }
                }
            }
        } else {
        }
    }

    hash_map
}

fn round_float_decimals(value: &f32) -> f32 {
    (value * 100.0).round() / 100.0
}

pub async fn run_query_mysql(
    column_headers: &Vec<String>,
    query: &String,
    pool: DBPool,
) -> Result<DataResponse, PersistenceError>{
    if let Some(mysql_pool) = get_mysql_pool(&pool) {
        let mut conn = match mysql_pool.get_conn() {
            Ok(conn) => conn,
            Err(err) => return Err(PersistenceError::MysqlError(err)),
        };

        let response_data = match conn.query_map(query, |row: mysql::Row| {
            sql_row_to_hash_map(column_headers, &row)
        }) {
            Ok(response_data) => response_data,
            Err(err) => return Err(PersistenceError::MysqlError(err)),
        };

        return Ok(DataResponse { data: response_data });
    }
    else{
        Err(PersistenceError::Unknown)
    }
}