use std::collections::HashMap;
use crate::models::{AttributeValue,Column};
use crate::db::DBPool;
use crate::db::PersistenceError;
use mysql::prelude::Queryable;
use mysql::from_value_opt;
pub fn mysqlPoolBuilder(db_user:String,db_password:String,db_host:String,db_port:u16,db_name:String) -> mysql::Pool{
    let builder = mysql::OptsBuilder::new()
    .ip_or_hostname(Some(db_host))
    .tcp_port(db_port)
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

pub fn fetch_all_tables_mysql(dbpool: &DBPool) -> Result<Vec<String>, PersistenceError> {
    if let Some(mysql_pool) = get_mysql_pool(dbpool) {
        let mut conn = match mysql_pool.get_conn() {
            Ok(conn) => conn,
            Err(err) => return Err(PersistenceError::MysqlError(err)),
        };
        let query = "SELECT table_name FROM information_schema.tables WHERE table_schema = DATABASE()";
        let tables: Vec<String> = conn.query_map(query, |table_name| table_name)?;
        Ok(tables)
    } else {
        Err(PersistenceError::Unknown)
    }
}

pub fn fetch_all_columns_mysql(dbpool: &DBPool, table_name: &str) -> Result<Vec<Column>, PersistenceError>{
    if let Some(mysql_pool) = get_mysql_pool(dbpool) {
        let mut conn = match mysql_pool.get_conn() {
            Ok(conn) => conn,
            Err(err) => return Err(PersistenceError::MysqlError(err)),
        };
        let query = format!(
            "SELECT column_name, data_type FROM information_schema.columns WHERE table_schema = DATABASE() AND table_name = '{}'",
            table_name
        );
        let columns: Vec<Column> = conn.query_map(query, |(column_name, datatype)| Column {
            name: column_name,
            datatype,
        })?;
        Ok(columns)
    } else {
        Err(PersistenceError::Unknown)
    }  
}