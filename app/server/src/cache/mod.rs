use crate::models;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};
use sha2::{Digest, Sha256};
use memcache::Client;
use models::{ResponseData};
// Function to serialize & deserialize the data output of run_query to JSON

pub fn serialize_data<T: Serialize>(data: &ResponseData) -> Result<String, String> {
    match to_string(data) {
        Ok(serialized) => Ok(serialized),
        Err(err) => Err(format!("Serialization error: {}", err)),
    }
}

pub fn deserialize_data<T: Deserialize<'static>>(serialized: &str) -> Result<ResponseData, String> {
    match from_str::<ResponseData>(serialized) {
        Ok(deserialized) => Ok(deserialized),
        Err(err) => Err(format!("Deserialization error: {}", err)),
    }
}

pub fn sanitize_query(query: &str) -> String {
    query.chars().filter(|c| c.is_alphanumeric()).collect()
}

pub fn hash_query_to_unique_id(query: &str) -> String {
    // Create a SHA-256 hash of the sanitized query
    let sanitized_query: String = query.chars().filter(|c| c.is_alphanumeric()).collect();
    let mut hasher = Sha256::new();
    hasher.update(sanitized_query.as_bytes());
    let hash_result = hasher.finalize();

    // Convert the hash bytes to a hexadecimal string and truncate it if necessary
    let hex_hash = hex::encode(hash_result);
    let max_length = 249; // Maximum supported key length
    if hex_hash.len() <= max_length {
        hex_hash
    } else {
        hex_hash[..max_length].to_string()
    }
}

// Function to clean the cache if the limit is reached
//{TODO}
pub fn clean_cache_if_needed(cache_client: &Client) {
    // Get the list of all keys currently in the cache
    let stats = cache_client.stats().unwrap();
    // let keys: Vec<String> = stats.iter().map(|(_, key)| key.to_string()).collect();
    let keys: Vec<String> = stats
        .iter()
        .flat_map(|(_, stats)| stats.keys().cloned())
        .collect();
    // let keys: &Vec<String> = stats
    //     .into_iter()
    //     .filter_map(|(key, )| key.parse::<String>().ok())
    //     .collect();
    println!("{}", keys.join(","))
    // let curr_items_values: Vec<u64> = stats
    //     .iter()
    //     .filter_map(|(_, stat)| stat.get("curr_items").and_then(|value| value.parse().ok()))
    //     .collect();
    // println!("{:?}",curr_items_values);
    // If the number of keys exceeds the maximum cache size, remove the oldest keys
    // if keys.len() > MAX_CACHE_SIZE {
    //     // Sort the keys based on their insertion order (the ones inserted first will come first)
    //     let mut sorted_keys = VecDeque::from(&keys);
    //     sorted_keys.make_contiguous().sort();

    //     // Determine the number of keys to remove from the cache
    //     let num_keys_to_remove = keys.len() - MAX_CACHE_SIZE;

    //     // Remove the oldest keys from the cache
    //     for key in sorted_keys.into_iter().take(num_keys_to_remove) {
    //         cache_client.delete(&key).ok();
    //     }
    // }
    // else {
    //     println!("Number of Keys : {}",keys.len());
    // }
}
