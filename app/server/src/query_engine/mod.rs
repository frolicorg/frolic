use crate::models;
use models::Metric;

pub fn GetQuery(query: &models::RESTInputModel) -> String {
    let result = "Hello from my_function!".to_string();
    // query.Metrics[0].Field.to_string()
    metrics_to_sql_columns(&query.Metrics)
}

// Function to convert metrics to SQL columns string.
pub fn metrics_to_sql_columns(metrics: &Vec<Metric>) -> String {
    let mut sql_columns = Vec::new();
    let valid_aggregations = ["sum", "avg", "count", "max", "min"]; 
    for metric in metrics {
        if valid_aggregations.contains(&metric.AggregateOperator.as_str()) {
            let column_sql = format!("{}({})", metric.AggregateOperator.to_uppercase(), metric.Field);
            sql_columns.push(column_sql);
        } else {
            eprintln!("Unknown aggregation function for column '{}'", metric.Field);
        }
    }
    sql_columns.join(", ")
}