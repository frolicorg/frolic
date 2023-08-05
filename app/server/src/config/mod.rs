use ini::Ini;
use std::{path::Path, default};

// Define a struct to hold the configurations
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub caching: CachingConfig,
    pub other: OtherConfig,
}

// Database configurations
pub struct DatabaseConfig {
    pub db_host: String,
    pub db_port: u16,
    pub db_name: String,
    pub db_user: String,
    pub db_password: String,
}

// Caching configurations
pub struct CachingConfig {
    pub cache_enabled: bool,
    pub cache_expiry: u32,
    pub cache_type: String,
    pub cache_host: String,
    pub cache_port: u16,
}

// Other custom configurations
pub struct OtherConfig {
    pub fetch_schema: bool,
    pub relationship_file: String,
    pub schema_file: String,
}

// Function to read the configurations from the INI file
pub fn read_config_file(file_path: &str) -> Result<AppConfig, Box<dyn std::error::Error>> {
    // Load the INI file
    let ini = Ini::load_from_file(Path::new(file_path))?;
    // Read the database configurations
    let database = ini.section(Some("database")).unwrap();
    let db_host = database.get("db_host").unwrap_or("localhost");
    let db_port = database.get("db_port").unwrap_or("5432").parse()?;
    let db_name = database.get("db_name").unwrap_or("my_database");
    let db_user = database.get("db_user").unwrap_or("my_user");
    let db_password = database.get("db_password").unwrap_or("my_password");
    // Read the caching configurations
    let caching = ini.section(Some("caching")).unwrap();
    let cache_enabled = caching.get("cache_enabled").unwrap_or("true").parse()?;
    let cache_expiry = caching.get("cache_expiry").unwrap_or("3600").parse()?;
    let cache_type = caching.get("cache_type").unwrap_or("memcached");
    let cache_host = caching.get("cache_host").unwrap_or("localhost");
    let cache_port = caching.get("cache_port").unwrap_or("11211").parse()?;
    // Read other custom configurations
    let other = ini.section(Some("other")).unwrap();
    let fetch_schema = other.get("fetch_schema").unwrap_or("true").parse()?;
    let relationship_file = other.get("relationship_file").unwrap_or("data/relationships.json");
    let schmea_file = other.get("schema_file").unwrap_or("data/table_schema_db.json");


    // Create and return the AppConfig struct
    let app_config = AppConfig {
        database: DatabaseConfig {
            db_host: db_host.to_string(),
            db_port,
            db_name: db_name.to_string(),
            db_user: db_user.to_string(),
            db_password: db_password.to_string(),
        },
        caching: CachingConfig {
            cache_enabled,
            cache_expiry,
            cache_type: cache_type.to_string(),
            cache_host: cache_host.to_string(),
            cache_port
        },
        other: OtherConfig {
            fetch_schema,
            relationship_file: relationship_file.to_string(),
            schema_file: schmea_file.to_string(),
        },
    };

    Ok(app_config)
}


