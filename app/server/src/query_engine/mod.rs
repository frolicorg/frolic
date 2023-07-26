use crate::models;

pub fn GetQuery(query: &models::RESTInputModel) -> String {
    let result = "Hello from my_function!".to_string();
    query.Metrics[0].Field.to_string()
}
