use crate::models;
use models::{Dimension, Filter, Metric, Table};
use std::collections::HashMap;

pub fn initiate_tables() -> Vec<Table> {
    let mut tables = vec![
        Table::new("users"),
        Table::new("products"),
        Table::new("reviews"),
        Table::new("orders"),
    ];
    //users and orders
    tables[3].add_relationship("users", "user_id", "id");
    tables[0].add_relationship("orders", "id", "user_id");

    //products and orders
    tables[3].add_relationship("products", "product_id", "id");
    tables[1].add_relationship("orders", "id", "product_id");

    //products and reviews
    tables[2].add_relationship("products", "product_id", "id");
    tables[1].add_relationship("reviews", "id", "product_id");
    tables
}

pub fn GetQuery(query: &models::RESTInputModel) -> String {
    let tables: Vec<Table> = initiate_tables();
    let metric_fields: Vec<String> = query
        .Metrics
        .iter()
        .map(|metric| metric.Field.clone())
        .collect();
    let dimesion_fields: Vec<String> = query
        .Dimensions
        .iter()
        .map(|metric| metric.Field.clone())
        .collect();
    let metrics_sql = metrics_to_sql(&query.Metrics);
    let dimensions_sql = dimensions_to_sql(&query.Dimensions);
    let mut filter_fields = Vec::new();
    let filters = if let Some(filters) = &query.Filters {
        filter_fields = filters
            .iter()
            .map(|filter| filter.Dimension.Field.clone())
            .collect();
        filters_to_sql(filters)
    } else {
        String::new()
    };
    //this field captures all the column fields that are requested by the user
    let mut all_fields: Vec<String> = Vec::new();
    all_fields.extend(metric_fields);
    all_fields.extend(dimesion_fields);
    all_fields.extend(filter_fields);
    let select_table_sql = all_fields.join(", ");
    //get all the table names requested by the user & process them
    let required_table_names = extract_table_columns(all_fields);
    let table_sql = handle_required_table(tables, required_table_names);

    //generate final mysql query
    let mut query = String::new();
    if !filters.is_empty() {
        query = format!(
            "select {}, {} from {} where {} group by {} ;",
            metrics_sql, dimensions_sql, table_sql, filters, dimensions_sql
        );
    } else {
        query = format!(
            "select {}, {} from {} group by {} ;",
            metrics_sql, dimensions_sql, table_sql, dimensions_sql
        );
    }
    query
}

//this function takes required tables and registered tables and provide the joined table
pub fn handle_required_table(
    registered_table: Vec<Table>,
    required_table_names: Vec<String>,
) -> String {
    //from the tables vector filter the required tables to join

    //check if required tables are registered or not
    let missing_tables: Vec<String> = required_table_names
        .iter()
        .filter(|name| !registered_table.iter().any(|table| table.name == **name))
        .cloned()
        .collect();

    if !missing_tables.is_empty() {
        // Throw an error indicating missing table names
        panic!("Tables are missing: {:?}", missing_tables);
    }

    let table_needed: Vec<Table> = registered_table
        .iter()
        .filter(|table| required_table_names.contains(&table.name))
        .cloned()
        .collect();
    for table in &table_needed {
        table.print_tables();
        println!();
    }
    // "hello".to_string()

    if let Some(query) = generate_join_query(&table_needed) {
        query
        // Execute the query using the MySQL client of your choice
    } else {
        format!("Unable to generate join query due to missing relationship.")
    }
}

//this functions takes column names and provides the String Vector containing Unique required table names
pub fn extract_table_columns(columns: Vec<String>) -> Vec<String> {
    let mut table_list = Vec::new();
    for column in columns {
        let table_name: String = column.as_str().split('.').next().unwrap_or("").to_string();
        if !table_list.contains(&table_name) {
            table_list.push(table_name.clone());
        }
    }
    table_list
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
                    eprintln!(
                        "Unknown aggregation function for column '{}'",
                        dimension.Field
                    );
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
        if valid_operators.contains(&filter.FilterOperator.as_str()) {
            let filter_sql = format!(
                "{} {} {}",
                dimensions_to_sql(&vec![filter.Dimension.clone()]),
                filter.FilterOperator.to_uppercase(),
                filter.FilterValue
            );
            sql_filters.push(filter_sql)
        } else {
            eprintln!(
                "Unknown filter operator for column '{}'",
                filter.Dimension.Field
            );
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

fn find_relationship<'a>(
    child_table: &str,
    relationships: &'a [HashMap<String, (String, String)>],
) -> Option<&'a HashMap<String, (String, String)>> {
    relationships
        .iter()
        .find(|relationship| relationship.contains_key(child_table))
}

fn generate_join_query(tables: &[Table]) -> Option<String> {
    if tables.is_empty() {
        return None;
    }

    let mut query = format!("{}", tables[0].name);

    for i in 1..tables.len() {
        let table = &tables[i];
        let mut join_found = false;
        // Check for relationships with all previously processed tables
        for j in (0..i).rev() {
            let prev_table = &tables[j];

            if let Some(relationship) = find_relationship(&table.name, &prev_table.relationships) {
                let join_condition = relationship
                    .iter()
                    .map(|(_, (parent_col, child_col))| {
                        format!(
                            "{}.{} = {}.{}",
                            prev_table.name, parent_col, table.name, child_col
                        )
                    })
                    .collect::<Vec<String>>()
                    .join(" AND ");

                query += &format!(" JOIN {} ON {}", table.name, join_condition);
                join_found = true;
                break;
            }
        }

        if !join_found {
            println!("{} No Relationship Found", i);
            // If no relationship exists, return None or handle accordingly.
            return None;
        }
    }

    Some(query)
}
