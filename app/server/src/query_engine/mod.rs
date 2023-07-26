use crate::models;
use models::{Metric,Dimension};

pub fn GetQuery(query: &models::RESTInputModel) -> String {
    let result = "Hello from my_function!".to_string();
    // query.Metrics[0].Field.to_string()
    let metrics = metrics_to_sql(&query.Metrics);
    let dimensions = dimensions_to_sql(&query.Dimensions);
    format!("{}{}", metrics, dimensions) 
}

// Function to convert metrics to SQL columns string.
pub fn metrics_to_sql(metrics: &Vec<Metric>) -> String {
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

pub fn dimensions_to_sql(dimensions: &Vec<Dimension>) -> String {
    let mut sql_columns = Vec::new();
    let valid_transformations = ["year", "month"]; 
    for dimension in dimensions {
        if valid_transformations.contains(&dimension.Transformations.as_str()) {
            let column_sql = format!("{}({})", dimension.Transformations.to_uppercase(), dimension.Field);
            sql_columns.push(column_sql);
        } else {
            eprintln!("Unknown aggregation function for column '{}'", dimension.Field);
        }
    }
    sql_columns.join(", ")
}


// Function to generate SQL columns based on dimensions and transformations
// pub fn dimensions_to_sql(dimensions: &[Dimension]) -> String {
//     let mut sql_columns = Vec::new();

//     for dimension in dimensions {
//         let mut sql_column = format!("{}", dimension.Field);

//         for transformation in &dimension.transformations {
//             match transformation {
//                 Transformation::Year => {
//                     sql_column = format!("YEAR({}) AS {}_", sql_column, dimension.Field);
//                 }
//                 Transformation::Month => {
//                     sql_column = format!("MONTH({}) AS {}_", sql_column, dimension.Field);
//                 }
//                 // Add more transformation patterns here
//             }
//         }

//         sql_columns.push(sql_column.trim_end_matches(',').to_string());
//     }

//     sql_columns.join(", ")
// }

// pub fn dimensions_to_sql(dimension: &Dimension) -> String {
//     let mut sql_expr = format!("{} AS {}_", dimension.Field, dimension.Field);

//     if let Some(transformations) = &dimension.transformations {
//         for transformation in transformations {
//             match transformation {
//                 Transformation::Year => {
//                     sql_expr = format!("YEAR({}) AS {}_", sql_expr, dimension.Field);
//                 }
//                 Transformation::Month => {
//                     sql_expr = format!("MONTH({}) AS {}_", sql_expr, dimension.Field);
//                 }
//                 // Add other transformation cases as needed
//             }
//         }
//     }

//     sql_expr
// }
