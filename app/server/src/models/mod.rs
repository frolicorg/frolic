use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct RESTInputModel {
    pub Metrics: Vec<Metric>,
    pub Dimensions: Vec<Dimension>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Metric {
    pub Field: String,
    pub AggregateOperator: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Dimension {
    pub Field: String,
    pub Transformations: Option<String>,
}
