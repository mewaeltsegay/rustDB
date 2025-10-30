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
    let client = RustDBClient::new("127.0.0.1", 8000);

    // Test connection
    println!("Testing connection...");
    match client.ping() {
        Ok(response) => println!("Server response: {}", response),
        Err(e) => println!("Error connecting to server: {}", e),
    }

    // Create a table
    println!("\nCreating table...");
    let create_query = "CREATE TABLE Users (id INT PRIMARY KEY, name STRING)";
    match client.execute(create_query) {
        Ok(response) => println!("Create table response: {:?}", response),
        Err(e) => println!("Error creating table: {}", e),
    }

    // Insert data
    println!("\nInserting data...");
    let insert_query = "INSERT INTO Users VALUES (1, 'Alice')";
    match client.execute(insert_query) {
        Ok(response) => println!("Insert response: {:?}", response),
        Err(e) => println!("Error inserting data: {}", e),
    }

    // List tables
    println!("\nListing tables...");
    match client.list_tables() {
        Ok(tables) => println!("Tables in database: {:?}", tables),
        Err(e) => println!("Error listing tables: {}", e),
    }

    // Query data
    println!("\nQuerying data...");
    let select_query = "SELECT * FROM Users";
    match client.execute(select_query) {
        Ok(response) => println!("Query response: {:?}", response),
        Err(e) => println!("Error querying data: {}", e),
    }

    Ok(())
}