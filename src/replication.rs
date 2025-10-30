use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use crate::row::RowInterface;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationConfig {
    pub is_primary: bool,
    pub primary_url: Option<String>,
    pub replicas: HashSet<String>,
    pub sync_interval: Duration,
}

impl Default for ReplicationConfig {
    fn default() -> Self {
        Self {
            is_primary: true,
            primary_url: None,
            replicas: HashSet::new(),
            sync_interval: Duration::from_secs(5),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationEvent {
    pub timestamp: u64,
    pub query: String,
}

pub struct ReplicationManager {
    config: ReplicationConfig,
    events: Arc<Mutex<Vec<ReplicationEvent>>>,
    db: Arc<Mutex<crate::database::Database>>,
}

impl ReplicationConfig {
    pub fn new_primary() -> Self {
        Self {
            is_primary: true,
            primary_url: None,
            replicas: HashSet::new(),
            sync_interval: Duration::from_secs(5),
        }
    }

    pub fn new_replica(primary_url: String) -> Self {
        Self {
            is_primary: false,
            primary_url: Some(primary_url),
            replicas: HashSet::new(),
            sync_interval: Duration::from_secs(5),
        }
    }
}

impl ReplicationManager {
    pub fn new(config: ReplicationConfig, db: Arc<Mutex<crate::database::Database>>) -> Self {
        Self {
            config,
            events: Arc::new(Mutex::new(Vec::new())),
            db,
        }
    }

    pub fn record_event(&self, query: String) {
        if self.config.is_primary {
            let event = ReplicationEvent {
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                query,
            };

            // Push event into local store, recovering if mutex was poisoned
            {
                let mut events_lock = self.events.lock().unwrap_or_else(|p| p.into_inner());
                events_lock.push(event.clone());
            }

            // Spawn a background thread to propagate this event to replicas so we don't
            // create/drop blocking runtimes from within the HTTP worker thread.
            let replicas: Vec<String> = self.config.replicas.iter().cloned().collect();
            std::thread::spawn(move || {
                if replicas.is_empty() {
                    return;
                }
                let client = reqwest::blocking::Client::new();
                let events_payload = vec![event];
                for replica in &replicas {
                    let _ = client
                        .post(&format!("{}/replicate", replica))
                        .json(&events_payload)
                        .send();
                }
            });
        }
    }

    pub fn propagate_to_replicas(&self) {
        // keep the original behavior for callers that want a full propagate,
        // but ensure we don't panic on poisoned locks. This function is
        // safe to call from a background thread.
        if !self.config.is_primary {
            return;
        }

        let events = self.events.lock().unwrap_or_else(|p| p.into_inner()).clone();
        let client = reqwest::blocking::Client::new();

        // Send events as a JSON-RPC call to each replica so we reuse the
        // same RPC transport instead of raw HTTP endpoints.
        let rpc_req = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "replication_apply_events",
            "params": [events],
            "id": 1
        });

        for replica in &self.config.replicas {
            let _ = client.post(replica).json(&rpc_req).send();
        }
    }

    pub fn apply_events(&self, events: Vec<ReplicationEvent>) -> Result<(), Box<dyn std::error::Error>> {
        if self.config.is_primary {
            return Err("Cannot apply replication events to primary server".into());
        }

        // Get current event count (recover if mutex was poisoned)
        let current_count = self.events.lock().unwrap_or_else(|p| p.into_inner()).len();

        // Only apply new events
        let new_events: Vec<_> = events.into_iter().skip(current_count).collect();
        
        if !new_events.is_empty() {
            let mut db = self.db.lock().unwrap_or_else(|p| p.into_inner());
            let mut events_lock = self.events.lock().unwrap_or_else(|p| p.into_inner());
            
            for event in new_events {
                // Apply the query to the database
                crate::sql::execute_sql(&mut db, &event.query);
                events_lock.push(event);
            }
        }

        Ok(())
    }

    pub fn start_sync_task(&self) {
        if self.config.is_primary {
            return;
        }

        let primary_url = match &self.config.primary_url {
            Some(url) => url.clone(),
            None => return,
        };

        let events = self.events.clone();
        let interval = self.config.sync_interval;
        let db = self.db.clone();

        std::thread::spawn(move || {
            let client = reqwest::blocking::Client::new();
            loop {
                std::thread::sleep(interval);

                // Call primary via JSON-RPC to get events
                let rpc_req = serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": "replication_get_events",
                    "params": [],
                    "id": 1
                });

                if let Ok(response) = client.post(&primary_url).json(&rpc_req).send() {
                    if let Ok(rpc_res_val) = response.json::<serde_json::Value>() {
                        if let Some(result) = rpc_res_val.get("result") {
                            if let Ok(new_events) = serde_json::from_value::<Vec<ReplicationEvent>>(result.clone()) {
                                let mut db_lock = db.lock().unwrap_or_else(|p| p.into_inner());
                                let mut events_lock = events.lock().unwrap_or_else(|p| p.into_inner());
                                let current_count = events_lock.len();

                                // Apply new events to the database
                                for event in new_events.iter().skip(current_count) {
                                    crate::sql::execute_sql(&mut db_lock, &event.query);
                                }

                                events_lock.extend(new_events.into_iter().skip(current_count));
                            }
                        }
                    }
                }
            }
        });
    }

    /// Start a background thread that periodically prints the current tables and rows
    /// on replica nodes. This helps visually verify that replicas have the same content
    /// as the primary in container logs.
    pub fn start_display_task(&self) {
        if self.config.is_primary {
            return;
        }

        let db = self.db.clone();
        let interval = self.config.sync_interval;

        std::thread::spawn(move || {
            loop {
                std::thread::sleep(interval);

                let db_lock = db.lock().unwrap_or_else(|p| p.into_inner());
                println!("[replica] Current database snapshot:");
                for (tname, table) in &db_lock.tables {
                    println!("[replica] Table: {}", tname);
                    // print schema header
                    let headers: Vec<_> = table.schema.columns.iter().map(|c| c.name.clone()).collect();
                    println!("[replica] Columns: {:?}", headers);
                    // print rows
                    for (i, row) in table.rows.iter().enumerate() {
                        let vals = row.get_values().clone();
                        println!("[replica]   row[{}]: {:?}", i, vals);
                    }
                }
            }
        });
    }

    pub fn get_events(&self) -> Vec<ReplicationEvent> {
        self.events.lock().unwrap_or_else(|p| p.into_inner()).clone()
    }

    pub fn is_primary(&self) -> bool {
        self.config.is_primary
    }

    /// Add a replica URL to the primary configuration so future events are propagated.
    pub fn add_replica(&mut self, url: String) {
        self.config.replicas.insert(url);
    }
}