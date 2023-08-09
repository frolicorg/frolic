use std::collections::HashMap;
use crate::models::{AttributeValue};
use crate::db::DBPool;
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
                        hash_map.insert(key.to_string(), AttributeValue::Float(float_value));
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
