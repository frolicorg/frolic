use crate::models;
use models::{Metric,Dimension,Filter};

pub fn GetQuery(query: &models::RESTInputModel) -> String {
    let result = "Hello from my_function!".to_string();
    // query.Metrics[0].Field.to_string()
    let metrics = metrics_to_sql(&query.Metrics);
    let dimensions = dimensions_to_sql(&query.Dimensions);
    let filters = if let Some(filters) = &query.Filters {
        filters_to_sql(filters)
    } else {
        String::new()
        // Define a default behavior or an empty filter list if there's no data.
        // For example, you can return an empty vector:
        // Vec::new()
    };
    // let filters = filters_to_sql(&query.Filters);
    // let filters = filters_to_sql(&query.Filters);
    format!("select {}, {} from table where {} group by {}", metrics, dimensions,filters,dimensions) 
}

// Function to convert metrics to SQL columns string.
pub fn metrics_to_sql(metrics: &Vec<Metric>) -> String {
    let mut sql_columns = Vec::new();
    let valid_aggregations = ["sum", "avg", "count", "max", "min"]; 

    for metric in metrics {
        match &metric.AggregateOperator {
            Some(operator) => {
                let uppercase_aggregate = operator.to_uppercase();
                let aggregate_str = operator.as_str();
                println!("{}", aggregate_str);
    
                if valid_aggregations.contains(&aggregate_str) {
                    let column_sql = format!("{}({})", uppercase_aggregate, metric.Field);
                    sql_columns.push(column_sql);
                } else {
                    eprintln!("Unknown aggregation function for column '{}'", metric.Field);
                }
            }
            None => {
                let column_sql = format!("({})", metric.Field);
                sql_columns.push(column_sql);
            }
        }
    }
    sql_columns.join(", ")
}

pub fn dimensions_to_sql(dimensions: &Vec<Dimension>) -> String {
    let mut sql_columns = Vec::new();
    let valid_transformations = ["year", "month"]; 
    for dimension in dimensions {
        match &dimension.Transformations {
            Some(operator) => {
                let uppercase_transformation = operator.to_uppercase();
                let transformation_str = operator.as_str();
                if valid_transformations.contains(&transformation_str) {
                    let column_sql = format!("{}({})", uppercase_transformation, dimension.Field);
                    sql_columns.push(column_sql);
                } else {
                    eprintln!("Unknown aggregation function for column '{}'", dimension.Field);
                }
            }
            None => {
                let column_sql = format!("({})", dimension.Field);
                sql_columns.push(column_sql);
            }
        }

    }
    sql_columns.join(", ")
}

pub fn filters_to_sql(filters: &Vec<Filter>) -> String {
    let mut sql_filters = Vec::new();
    let valid_operators = [">", "<", "="]; 

    for filter in filters {

        if valid_operators.contains(&filter.FilterOperator.as_str()){
            let filter_sql = format!(
                "{} {} {}", 
                dimensions_to_sql(&vec![filter.Dimension.clone()]), 
                filter.FilterOperator.to_uppercase(),
                filter.FilterValue
            );
            sql_filters.push(filter_sql)
        }
        else{
            eprintln!("Unknown filter operator for column '{}'", filter.Dimension.Field);
        }

        // match &filter.FilterOperator {
        //     Some(operator) => {
        //         let uppercase_aggregate = operator.to_uppercase();
        //         let aggregate_str = operator.as_str();
        //         println!("{}", aggregate_str);
    
        //         if valid_aggregations.contains(&aggregate_str) {
        //             let column_sql = format!("{}({})", uppercase_aggregate, metric.Field);
        //             sql_columns.push(column_sql);
        //         } else {
        //             eprintln!("Unknown aggregation function for column '{}'", metric.Field);
        //         }
        //     }
        //     None => {
        //         let column_sql = format!("({})", metric.Field);
        //         sql_columns.push(column_sql);
        //     }
        // }
    }
    sql_filters.join(", ")
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
