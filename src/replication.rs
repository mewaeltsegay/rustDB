use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

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
            self.events.lock().unwrap().push(event);
            self.propagate_to_replicas();
        }
    }

    pub fn propagate_to_replicas(&self) {
        if !self.config.is_primary {
            return;
        }

        let events = self.events.lock().unwrap().clone();
        let client = reqwest::blocking::Client::new();

        for replica in &self.config.replicas {
            let _ = client
                .post(&format!("{}/replicate", replica))
                .json(&events)
                .send();
        }
    }

    pub fn apply_events(&self, events: Vec<ReplicationEvent>) -> Result<(), Box<dyn std::error::Error>> {
        if self.config.is_primary {
            return Err("Cannot apply replication events to primary server".into());
        }

        // Get current event count
        let current_count = self.events.lock().unwrap().len();

        // Only apply new events
        let new_events: Vec<_> = events.into_iter().skip(current_count).collect();
        
        if !new_events.is_empty() {
            let mut db = self.db.lock().unwrap();
            let mut events_lock = self.events.lock().unwrap();
            
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

                // Get events from primary
                if let Ok(response) = client.get(&format!("{}/events", primary_url)).send() {
                    if let Ok(new_events) = response.json::<Vec<ReplicationEvent>>() {
                        let mut db_lock = db.lock().unwrap();
                        let mut events_lock = events.lock().unwrap();
                        let current_count = events_lock.len();
                        
                        // Apply new events to the database
                        for event in new_events.iter().skip(current_count) {
                            crate::sql::execute_sql(&mut db_lock, &event.query);
                        }
                        
                        events_lock.extend(new_events.into_iter().skip(current_count));
                    }
                }
            }
        });
    }

    pub fn get_events(&self) -> Vec<ReplicationEvent> {
        self.events.lock().unwrap().clone()
    }

    pub fn is_primary(&self) -> bool {
        self.config.is_primary
    }
}