use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryResponse {
    pub success: bool,
    pub message: String,
    pub rows: Option<Vec<Vec<String>>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: serde_json::Value,
    id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    result: Option<serde_json::Value>,
    error: Option<JsonRpcError>,
    id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

pub struct RustDBClient {
    client: reqwest::blocking::Client,
    endpoint: String,
}

impl RustDBClient {
    pub fn new(host: &str, port: u16) -> Self {
        RustDBClient {
            client: reqwest::blocking::Client::new(),
            endpoint: format!("http://{}:{}", host, port),
        }
    }

    fn send_request(&self, method: &str, params: serde_json::Value) -> std::result::Result<serde_json::Value, Box<dyn Error>> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id: 1,
        };

        let response: JsonRpcResponse = self.client
            .post(&self.endpoint)
            .json(&request)
            .send()?
            .json()?;

        match (response.result, response.error) {
            (Some(result), _) => Ok(result),
            (None, Some(error)) => Err(error.message.into()),
            _ => Err("Invalid response from server".into()),
        }
    }

    pub fn execute(&self, query: &str) -> std::result::Result<QueryResponse, Box<dyn Error>> {
        let params = serde_json::json!([query]);
        let result = self.send_request("execute", params)?;
        Ok(serde_json::from_value(result)?)
    }

    pub fn ping(&self) -> std::result::Result<String, Box<dyn Error>> {
        let params = serde_json::json!([]);
        let result = self.send_request("ping", params)?;
        Ok(result.as_str()
            .ok_or("Invalid response type")?
            .to_string())
    }

    pub fn list_tables(&self) -> std::result::Result<Vec<String>, Box<dyn Error>> {
        let params = serde_json::json!([]);
        let result = self.send_request("list_tables", params)?;

        Ok(serde_json::from_value(result)?)
    }
}

// Example usage in a binary
pub fn run_client_example() -> std::result::Result<(), Box<dyn Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("debug")
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    let client = RustDBClient::new("127.0.0.1", 8000);

    // Test connection
    tracing::info!("Testing connection...");
    match client.ping() {
        Ok(response) => tracing::info!(response, "Server is responsive"),
        Err(e) => tracing::error!(error = %e, "Connection failed"),
    }

    // Create tables to demonstrate sharding
    tracing::info!("Creating tables for sharding demonstration...");
    let create_queries = [
        "CREATE TABLE Users (id INT PRIMARY KEY, name STRING, department STRING)",
        "CREATE TABLE Orders (order_id INT PRIMARY KEY, user_id INT, amount FLOAT)",
    ];

    for query in create_queries {
        match client.execute(query) {
            Ok(_) => tracing::info!(query, "Table created successfully"),
            Err(e) => tracing::error!(error = %e, query, "Failed to create table"),
        }
    }

    // Insert data across different shards
    tracing::info!("Inserting data across shards...");
    let user_data = [
        (1, "Alice", "Engineering"),
        (2, "Bob", "Sales"),
        (3, "Charlie", "Marketing"),
        (4, "Diana", "Engineering"),
    ];

    for (id, name, dept) in user_data {
        let query = format!(
            "INSERT INTO Users (id, name, department) VALUES ({}, '{}', '{}')",
            id, name, dept
        );
        match client.execute(&query) {
            Ok(_) => tracing::info!(user_id = id, name, "User inserted successfully"),
            Err(e) => tracing::error!(error = %e, user_id = id, "Failed to insert user"),
        }
    }

    // Insert orders that will be sharded by order_id
    let orders = [
        (1001, 1, 100.0),
        (1002, 2, 200.0),
        (1003, 3, 150.0),
        (1004, 4, 300.0),
    ];

    for (order_id, user_id, amount) in orders {
        let query = format!(
            "INSERT INTO Orders (order_id, user_id, amount) VALUES ({}, {}, {})",
            order_id, user_id, amount
        );
        match client.execute(&query) {
            Ok(_) => tracing::info!(order_id, user_id, amount, "Order inserted successfully"),
            Err(e) => tracing::error!(error = %e, order_id, "Failed to insert order"),
        }
    }

    // Query data from different shards
    tracing::info!("Querying data from shards...");
    let queries = [
        "SELECT * FROM Users WHERE id = 1",
        "SELECT * FROM Users WHERE id = 3",
        "SELECT * FROM Orders WHERE order_id = 1001",
        "SELECT * FROM Orders WHERE order_id = 1003",
    ];

    for query in queries {
        match client.execute(query) {
            Ok(response) => tracing::info!(query, response = ?response, "Query executed successfully"),
            Err(e) => tracing::error!(error = %e, query, "Query failed"),
        }
    }

    // List all tables to verify
    tracing::info!("Listing all tables...");
    match client.list_tables() {
        Ok(tables) => tracing::info!(tables = ?tables, "Retrieved table list"),
        Err(e) => tracing::error!(error = %e, "Failed to list tables"),
    }

    Ok(())
}