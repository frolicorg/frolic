use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct RESTInputModel {
    pub Metrics: Vec<Metric>,
    pub Dimensions: Vec<Dimension>,
    pub Filters: Option<Vec<Filter>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Table {
    pub name: String,
    pub relationships: Vec<HashMap<String, (String, String)>>, // Child table -> (Parent column, Child column)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Metric {
    pub Field: String,
    pub AggregateOperator: Option<String>,
}

#[derive(Debug, Deserialize, Serialize,Clone)]
pub struct Dimension {
    pub Field: String,
    pub Transformations: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Filter {
    pub Dimension: Dimension,
    pub FilterOperator: String,
    pub FilterValue: String,
}

impl Table {
    pub fn new(name: &str) -> Self {
        Table {
            name: name.to_string(),
            relationships: Vec::new(),
        }
    }

    pub fn add_relationship(
        &mut self,
        child: &str,
        parent_column: &str,
        child_column: &str,
    ) {
        let mut new_relationship = HashMap::new();
        new_relationship.insert(
            child.to_string(),
            (
                parent_column.to_string(),
                child_column.to_string(),
            ),
        );
        self.relationships.push(new_relationship);
    }
    pub fn print_tables(&self) {
        println!("Table: {}", self.name);
        for relationship in &self.relationships {
            for (child_table, (parent_column, child_column)) in relationship {
                println!(
                    "  -> Child Table: {}, Parent Column: {}, Child Column: {}",
                    child_table, parent_column, child_column
                );
            }
        }
    }
}

