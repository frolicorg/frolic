use crate::config::AppConfig;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
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
    pub distinct: Option<bool>,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OrderBy {
    pub field: Vec<String>,
    pub order: Order,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Order {
    asc,
    desc,
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

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct DataResponse {
    pub data: Vec<HashMap<String, AttributeValue>>,
}

#[derive(Debug, Clone)]
pub enum AttributeValue {
    NULL,
    String(String),
    Float(f32),
}

impl Serialize for AttributeValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            AttributeValue::NULL => serializer.serialize_unit(),
            AttributeValue::String(ref s) => serializer.serialize_str(s),
            AttributeValue::Float(f) => serializer.serialize_f32(f),
        }
    }
}

impl<'de> Deserialize<'de> for AttributeValue {
    fn deserialize<D>(deserializer: D) -> Result<AttributeValue, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum ValueHelper {
            Null,
            String(String),
            Float(f32),
        }

        let value_helper = ValueHelper::deserialize(deserializer)?;
        match value_helper {
            ValueHelper::Null => Ok(AttributeValue::NULL),
            ValueHelper::String(s) => Ok(AttributeValue::String(s)),
            ValueHelper::Float(f) => Ok(AttributeValue::Float(f)),
        }
    }
}

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
    pub app_config: AppConfig,
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
