use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct RESTInputModel {
    pub Metrics: Vec<Metric>,
    pub Dimensions: Vec<Dimension>,
    pub Filters: Option<Vec<Filter>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Metric {
    pub Field: String,
    pub AggregateOperator: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
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

#[derive(Debug, Deserialize, Serialize)]
pub struct RESTResponseModel {
    pub data: Vec<String>,
}
