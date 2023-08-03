use crate::models;
use models::{Dimension, Filter, Metric, Table};
use std::collections::HashMap;

pub fn get_query(query: &models::RESTInputModel, tables: &Vec<Table>) -> String {
    let tables_json = serde_json::to_string_pretty(&tables).unwrap();
    // log::info!("{}", tables_json);

    //get the tables setup
    let tables: Vec<Table> = tables.iter().cloned().collect();

    //fetch the columns requested by user
    let mut filter_fields = Vec::new();
    let mut metric_fields = Vec::new();
    let mut dimension_fields = Vec::new();
    let mut all_fields: Vec<String> = Vec::new();
    let mut metrics_sql = String::new();
    let mut dimensions_sql = String::new();
    let mut dimensions_group_sql = String::new();

    if let Some(metrics) = &query.metrics {
        metrics_sql = metrics_to_sql(metrics);
        // println!("{}",metrics_sql);
        metric_fields = metrics
            .iter()
            .map(|metric| metric.field.clone())
            .collect();
        all_fields.extend(metric_fields);
    };
    if let Some(dimensions) = &query.dimensions {
        dimensions_sql = dimensions_to_sql(dimensions,false);
        dimensions_group_sql = "group by ".to_string() + &dimensions_to_sql(dimensions,true);
        dimension_fields = dimensions
            .iter()
            .map(|dimension| dimension.field.clone())
            .collect();
        all_fields.extend(dimension_fields);
    };
    
    if let Some(filters) = &query.filters {
        filter_fields = filters
            .iter()
            .map(|filter| filter.dimension.field.clone())
            .collect();
        all_fields.extend(filter_fields);
    };

    //check if the columns are present in the tables or not, if present create a hashmap as well to get the datatype
    let mut field_datatype_map: HashMap<&String, &str> = HashMap::new();
    for field in &all_fields{
        match find_column_datatype(&tables, field) {
            Some(datatype) => {
                println!("Column datatype: {}", datatype);
                field_datatype_map.insert(field, datatype);
            }
            None => return format!("Column : {} not found or invald input format",field)
        }
    }
    
    let filters_sql = if let Some(filters) = &query.filters {
        filters_to_sql(filters,&field_datatype_map)
    } else {
        String::new()
    };
    //get all the table names requested by the user & process them
    let required_table_names = extract_table_columns(all_fields);
    let table_sql = handle_required_table(tables, required_table_names);

    //generate final mysql query
    let mut comma = String::new();
    if dimensions_sql != "" && metrics_sql != ""{
        comma = ",".to_string();
    }
    format!(
        "select {} {} {} from {} {} {} ;",
        dimensions_sql,comma, metrics_sql, table_sql, filters_sql, dimensions_group_sql
    )
}

//this function takes required tables and registered tables and provide the joined table
pub fn handle_required_table(
    registered_table: Vec<Table>,
    required_table_names: Vec<String>,) -> String {
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
        match &metric.aggregate_operator {
            Some(operator) => {
                let uppercase_aggregate = operator.to_uppercase();
                let aggregate_str = operator.as_str();
                println!("{}", aggregate_str);

                if valid_aggregations.contains(&aggregate_str) {
                    let column_sql = match &metric.name{
                        Some(nm)=>{
                            format!("{}({}) as {}", uppercase_aggregate,&metric.field, nm)
                        }
                        None =>{
                            format!("{}({})", uppercase_aggregate, &metric.field)
                        }
                    };

                    sql_columns.push(column_sql);
                } else {
                    eprintln!("Unknown aggregation function for column '{}'", metric.field);
                }
            }
            None => {
                let column_sql = format!("({})", metric.field);
                sql_columns.push(column_sql);
            }
        }
    }
    sql_columns.join(", ")
}

pub fn dimensions_to_sql(dimensions: &Vec<Dimension>,group:bool) -> String {
    let mut sql_columns = Vec::new();
    let valid_transformations = ["year", "month"];
    for dimension in dimensions {
        match &dimension.transformations {
            Some(operator) => {
                let uppercase_transformation = operator.to_uppercase();
                let transformation_str = operator.as_str();
                if valid_transformations.contains(&transformation_str) {

                    let column_sql = match &dimension.name{
                        Some(nm)=>{
                            if group{
                                format!("{}({})", uppercase_transformation,&dimension.field)
                            }
                            else{
                                format!("{}({}) as {}", uppercase_transformation,&dimension.field, nm)
                            }
                        }
                        None =>{
                            format!("{}({})", uppercase_transformation, &dimension.field)
                        }
                    };
                    sql_columns.push(column_sql);
                } else {
                    eprintln!(
                        "Unknown aggregation function for column '{}'",
                        dimension.field
                    );
                }
            }
            None => {
                let column_sql = format!("({})", dimension.field);
                sql_columns.push(column_sql);
            }
        }
    }
    sql_columns.join(", ")
}

pub fn filters_to_sql(filters: &Vec<Filter>,field_datatype_map: &HashMap<&String, &str>) -> String {
    let mut sql_filters = Vec::new();
    let valid_operators = [">", "<", "="];
    for filter in filters {
        let mut datatype_field = String::new();
        match field_datatype_map.get(&filter.dimension.field.to_string()) {
            Some(datatype) => datatype_field = datatype.to_string(),
            None => return format!("Field '{}' not found in the map", &filter.dimension.field),
        }
        // Choose the format string based on the datatype
        // let format_string = match datatype_field.as_str() {
        //     "varchar" => "{} {} \"{}\"",
        //     "int" | "bigint" => "{} {} {}",
        //     _ => panic!("Unsupported datatype"), // Add appropriate handling for other datatypes if needed
        // };
        if valid_operators.contains(&filter.filter_operator.as_str()) {


            let filter_sql = match datatype_field.as_str(){
                "varchar" | "datetime" => format!(
                    "{} {} \"{}\"",
                    dimensions_to_sql(&vec![filter.dimension.clone()],true),
                    filter.filter_operator.to_uppercase(),
                    filter.filter_value
                ),
                "int" | "bigint" | "float" => format!(
                    "{} {} {}",
                    dimensions_to_sql(&vec![filter.dimension.clone()],true),
                    filter.filter_operator.to_uppercase(),
                    filter.filter_value
                ),
                _ => panic!("Unsupported datatype"),
            };

            sql_filters.push(filter_sql)
        } else {
            eprintln!(
                "Unknown filter operator for column '{}'",
                filter.dimension.field
            );
        }
    }
    format!("where {}",sql_filters.join(" and "))
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
                // break;
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


pub fn find_column_datatype<'a>(tables: &'a [Table], column_name: &'a str) -> Option<&'a str> {
    // Split the input string into table name and column name
    let parts: Vec<&str> = column_name.split('.').collect();
    if parts.len() != 2 {
        return None; // The input string should be in the format 'table.column_name'
    }

    let table_name = parts[0];
    let column_name = parts[1];

    // Find the table with the given name
    let table = tables.iter().find(|t| t.name == table_name)?;

    // Find the column with the given name in the table's columns
    let column = table.columns.iter().find(|c| c.name == column_name)?;

    // Return the datatype of the found column
    Some(&column.datatype)
}
