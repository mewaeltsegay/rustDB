use lab::client::RustDBClient;
use std::error::Error;
use tracing;
use tracing_subscriber;
use std::thread;
use std::time::Duration;

fn wait_for_server(client: &RustDBClient, max_retries: u32) -> bool {
    for i in 0..max_retries {
        match client.execute("LIST TABLES") {
            Ok(_) => {
                tracing::info!("Connected to server after {} retries", i);
                return true;
            },
            Err(_) => {
                if i < max_retries - 1 {
                    tracing::info!("Server not ready, waiting...");
                    thread::sleep(Duration::from_secs(1));
                }
            }
        }
    }
    false
}

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("debug")
        .with_thread_ids(true)
        .with_target(false)
        .with_file(true)
        .with_line_number(true)
        .init();

    tracing::info!("Starting shard demonstration");
    let client = RustDBClient::new("127.0.0.1", 8000);

    // Wait for server to be ready
    if !wait_for_server(&client, 10) {
        tracing::error!("Could not connect to server after maximum retries");
        return Ok(());
    }

    // Create a table that will be sharded
    let create_table = "CREATE TABLE ShardTest (
        id INT PRIMARY KEY,
        value STRING,
        timestamp INT
    )";
    
    match client.execute(create_table) {
        Ok(_) => {
            tracing::info!("Created ShardTest table");
            // Give server a moment to propagate to all shards
            thread::sleep(Duration::from_millis(100));
        },
        Err(e) => {
            tracing::error!(error = %e, "Failed to create table");
            return Ok(());
        }
    }

    // Insert data with different keys to demonstrate sharding
    for i in 0..20 {
        let insert = format!(
            "INSERT INTO ShardTest VALUES ({}, 'value_{}', {})",
            i, i, chrono::Utc::now().timestamp()
        );
        
        match client.execute(&insert) {
            Ok(_) => tracing::info!(key = i, "Inserted record"),
            Err(e) => {
                tracing::error!(error = %e, key = i, "Failed to insert");
                continue;
            }
        }
        thread::sleep(Duration::from_millis(50)); // Space out requests
    }

    thread::sleep(Duration::from_millis(100)); // Let inserts settle

    // Query specific records to show shard routing
    let test_keys = [0, 5, 10, 15];
    for key in test_keys {
        let query = format!("SELECT * FROM ShardTest WHERE id = {}", key);
        match client.execute(&query) {
            Ok(response) => tracing::info!(
                key,
                response = ?response,
                "Retrieved record from shard"
            ),
            Err(e) => tracing::error!(error = %e, key, "Failed to query"),
        }
        thread::sleep(Duration::from_millis(50)); // Space out requests
    }

    tracing::info!("Shard demonstration complete");
    Ok(())
}