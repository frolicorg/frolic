use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DataRequest {
    pub metrics: Option<Vec<Metric>>,
    pub dimensions: Option<Vec<Dimension>>,
    pub filters: Option<Vec<Filter>>,
    pub orderby: Option<OrderBy>,
    pub limit: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Metric {
    pub field: String,
    pub aggregate_operator: Option<String>,
    pub name: Option<String>,
    pub distinct : Option<bool>,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OrderBy {
    pub field: Vec<String>,
    pub order: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Dimension {
    pub field: String,
    pub transformation: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Filter {
    pub dimension: Dimension,
    pub filter_operator: String,
    pub filter_value: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DataResponse {
    pub data: Vec<HashMap<String, String>>,
}

// #[derive(Debug, Deserialize, Serialize, Clone)]
// pub enum DataType {
//     string,
//     varchar,
//     int,
//     bigint,
//     float,
//     datetime, // Add more data types as needed
// }

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Column {
    pub name: String,
    pub datatype: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Table {
    pub name: String,
    pub columns: Vec<Column>,
    pub relationships: Vec<HashMap<String, (String, String)>>, // Child table -> (Parent column, Child column)
}

pub struct AppState {
    pub app_name: String,
    pub tables: Vec<Table>,
    pub is_caching: bool,
    pub caching_expiry: u32,
}

impl Table {
    pub fn new(name: &str) -> Self {
        Table {
            name: name.to_string(),
            columns: Vec::new(),
            relationships: Vec::new(),
        }
    }

    pub fn add_relationship(&mut self, child: &str, parent_column: &str, child_column: &str) {
        let mut new_relationship = HashMap::new();
        new_relationship.insert(
            child.to_string(),
            (parent_column.to_string(), child_column.to_string()),
        );
        self.relationships.push(new_relationship);
    }

    pub fn print_tables(&self) {
        log::info!("Table: {}", self.name);
        for relationship in &self.relationships {
            for (child_table, (parent_column, child_column)) in relationship {
                log::info!(
                    "  -> Child Table: {}, Parent Column: {}, Child Column: {}",
                    child_table,
                    parent_column,
                    child_column
                );
            }
        }
    }
}
