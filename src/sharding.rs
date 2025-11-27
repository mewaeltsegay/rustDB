use std::sync::{Arc, Mutex};
use std::collections::{hash_map::DefaultHasher, HashMap};
use std::time::Duration;
use std::hash::{Hash, Hasher};
use crate::database::Database;
use crate::shard_monitor::{ShardHealthMonitor, ShardHealth};
use serde::{Serialize, Deserialize};
use reqwest::blocking::Client;
use serde_json::Value;
use url::Url;

/// Configuration for a shard node
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShardNodeConfig {
    pub id: String,
    pub url: String,
    pub range_start: u64,
    pub range_end: u64,
    pub is_primary: bool,
    pub primary_shard_id: Option<String>, // If this is a replica, which primary it replicates
}

/// Represents a single shard in the system
pub struct ShardNode {
    pub config: ShardNodeConfig,
    pub db: Arc<Mutex<Database>>,
    pub health_monitor: Arc<ShardHealthMonitor>,
}

impl ShardNode {
    pub fn new(config: ShardNodeConfig, health_monitor: Arc<ShardHealthMonitor>) -> Self {
        tracing::debug!(
            shard_id = config.id,
            "Creating new shard node instance"
        );
        health_monitor.register_shard(&config.id);
        Self {
            config,
            db: Arc::new(Mutex::new(Database::new())),
            health_monitor,
        }
    }

    pub fn contains_key(&self, key: u64) -> bool {
        let contains = key >= self.config.range_start && key < self.config.range_end;
        tracing::trace!(
            shard_id = self.config.id,
            key,
            range_start = self.config.range_start,
            range_end = self.config.range_end,
            contains,
            "Checking key in shard range"
        );
        contains
    }

    pub fn is_healthy(&self) -> bool {
        self.health_monitor.get_shard_health(&self.config.id) == ShardHealth::Healthy
    }

    pub fn record_access(&self) {
        self.health_monitor.record_heartbeat(&self.config.id);
    }
}

/// Manages the sharding configuration and routing
pub struct ShardManager {
    shards: Vec<Arc<ShardNode>>,
    replicas: Vec<Arc<ShardNode>>,
    health_monitor: Arc<ShardHealthMonitor>,
    /// Optional list of remote shard URLs (when coordinator talks to shard servers)
    remote_shard_urls: Vec<String>,
}

impl ShardManager {
    pub fn new(num_shards: usize) -> Self {
        let health_monitor = Arc::new(ShardHealthMonitor::new(30)); // 30-second heartbeat timeout
        let mut shards = Vec::with_capacity(num_shards);
        
        // Check if this is a single-shard container (SHARD_ID env var is set)
        let shard_id_offset = std::env::var("SHARD_ID")
            .ok()
            .and_then(|id| id.parse::<usize>().ok())
            .unwrap_or(0);
        
        let range_size = u64::MAX / num_shards as u64;
        
        tracing::info!(num_shards, "Initializing shard manager");
        
        for i in 0..num_shards {
            let shard_idx = shard_id_offset + i;
            let range_start = shard_idx as u64 * range_size;
            let range_end = if shard_idx == num_shards - 1 {
                u64::MAX
            } else {
                (shard_idx + 1) as u64 * range_size
            };

            let config = ShardNodeConfig {
                id: format!("shard_{}", shard_idx),
                url: format!("http://localhost:{}", 8000 + shard_idx),
                range_start,
                range_end,
                is_primary: true,
                primary_shard_id: None,
            };

            tracing::info!(
                shard_id = config.id,
                range_start = config.range_start,
                range_end = config.range_end,
                "Initializing shard"
            );
            shards.push(Arc::new(ShardNode::new(config, Arc::clone(&health_monitor))));
        }

        tracing::info!("Shard manager initialization complete");
        Self {
            shards,
            replicas: Vec::new(),
            health_monitor,
            remote_shard_urls: Vec::new(),
        }
    }

    /// Start a background thread that periodically checks for heartbeat timeouts
    /// and marks shards as DOWN when they've exceeded the configured timeout.
    pub fn start_heartbeat_checker(&self) {
        let hm = Arc::clone(&self.health_monitor);
        // Spawn a background thread; this is a lightweight periodic check used
        // by the coordinator to keep shard health state up-to-date.
        std::thread::spawn(move || loop {
            std::thread::sleep(Duration::from_secs(5));
            hm.check_heartbeat_timeouts();
        });
    }

    /// Keep local shards healthy by periodically recording heartbeats for them.
    /// This is used when shards are running locally (not in remote-poller mode).
    pub fn start_local_heartbeat_recorder(&self) {
        let hm = Arc::clone(&self.health_monitor);
        let shards: Vec<String> = self.shards.iter().map(|s| s.config.id.clone()).collect();
        std::thread::spawn(move || loop {
            std::thread::sleep(Duration::from_secs(10));
            for shard_id in &shards {
                hm.record_heartbeat(shard_id);
            }
        });
    }

    /// Start polling remote shards (if configured). For each remote shard URL
    /// we issue a JSON-RPC request to the shard's `shard_health` method and
    /// update the health monitor based on the response. This method spawns a
    /// background thread and returns immediately.
    pub fn start_remote_poller(&self, urls: Vec<String>) {
        if urls.is_empty() {
            return;
        }
        let hm = Arc::clone(&self.health_monitor);
        std::thread::spawn(move || {
            let client = Client::new();
            loop {
                for url in &urls {
                    // build jsonrpc request
                    let body = serde_json::json!({
                        "jsonrpc": "2.0",
                        "method": "shard_health",
                        "params": [],
                        "id": 1
                    });

                    let resp = client.post(url).json(&body).send();
                    match resp {
                        Ok(r) => {
                            if let Ok(json) = r.json::<Value>() {
                                if let Some(result) = json.get("result") {
                                    // Expect result.shards to be a map of shard_id -> status
                                    if let Some(shards_map) = result.get("shards") {
                                        if let Some(obj) = shards_map.as_object() {
                                            for (shard_id, status_val) in obj.iter() {
                                                if status_val == "healthy" || status_val == &Value::String("healthy".to_string()) {
                                                    hm.record_heartbeat(shard_id);
                                                } else {
                                                    hm.mark_shard_degraded(shard_id);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Err(_e) => {
                            // network/error -> mark the related shard as down. We don't
                            // have the shard id here, so attempt to derive a name from URL
                            // (best-effort) and mark that id down.
                            if let Ok(parsed) = url.parse::<Url>() {
                                if let Some(host) = parsed.host_str() {
                                    let id = format!("{}", host);
                                    hm.mark_shard_down(&id);
                                }
                            }
                        }
                    }
                }
                std::thread::sleep(Duration::from_secs(5));
            }
        });
    }

    /// Register a replica shard for failover
    pub fn register_replica(&mut self, shard_id: &str, replica_node: Arc<ShardNode>) {
        tracing::info!(replica_id = shard_id, "Registering replica shard");
        self.replicas.push(replica_node);
    }

    pub fn get_shard_for_key<K: Hash>(&self, key: &K) -> Arc<ShardNode> {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash = hasher.finish();

        tracing::debug!(
            hash,
            "Finding shard for key"
        );

        // Try to find a healthy shard that contains this hash
        if let Some(shard) = self.shards
            .iter()
            .find(|shard| shard.contains_key(hash) && shard.is_healthy())
        {
            shard.record_access();
            tracing::info!(
                hash,
                shard_id = shard.config.id,
                "Key mapped to healthy shard"
            );
            return shard.clone();
        }

        // Primary is unhealthy; try replicas
        if let Some(replica) = self.replicas
            .iter()
            .find(|replica| replica.contains_key(hash) && replica.is_healthy())
        {
            replica.record_access();
            tracing::warn!(
                hash,
                replica_id = replica.config.id,
                "Primary unhealthy; falling back to replica"
            );
            return replica.clone();
        }

        // All are unhealthy; fall back to any shard that contains the key (best effort)
        if let Some(shard) = self.shards
            .iter()
            .find(|shard| shard.contains_key(hash))
        {
            tracing::warn!(
                hash,
                shard_id = shard.config.id,
                "All shards unhealthy; using best-effort shard"
            );
            return shard.clone();
        }

        // Fallback to primary shard 0 if none found (should not happen)
        tracing::error!("No shard found for key; using fallback");
        self.shards[0].clone()
    }

    pub fn get_all_shards(&self) -> Vec<Arc<ShardNode>> {
        self.shards.clone()
    }

    pub fn get_shard_health_status(&self) -> HashMap<String, String> {
        self.health_monitor.get_health_summary()
    }

    pub fn get_healthy_shards(&self) -> Vec<String> {
        self.health_monitor.get_healthy_shards()
    }
}

/// Main database interface that handles sharding
pub struct ShardedDatabase {
    shard_manager: ShardManager,
}

impl ShardedDatabase {
    pub fn new(num_shards: usize) -> Self {
        Self {
            shard_manager: ShardManager::new(num_shards),
        }
    }

    /// Start the shard manager's background heartbeat checker.
    /// This simply forwards to the internal ShardManager so the coordinator
    /// process can ensure stale shards are marked DOWN in a timely manner.
    pub fn start_heartbeat_checker(&self) {
        self.shard_manager.start_heartbeat_checker();
    }

    /// Start polling remote shards. This forwards to the internal ShardManager's poller.
    pub fn start_remote_poller(&self, urls: Vec<String>) {
        self.shard_manager.start_remote_poller(urls);
    }

    /// Start recording heartbeats for local shards (keep them alive).
    /// Used when shards are running locally, not in remote mode.
    pub fn start_local_heartbeat_recorder(&self) {
        self.shard_manager.start_local_heartbeat_recorder();
    }

    /// Get a reference to the shard that should handle this key
    pub fn get_shard<K: Hash>(&self, key: &K) -> Arc<ShardNode> {
        self.shard_manager.get_shard_for_key(key)
    }

    /// Get all shards for operations that need to touch all shards
    pub fn get_all_shards(&self) -> Vec<Arc<ShardNode>> {
        self.shard_manager.get_all_shards()
    }

    /// Get health status of all shards
    pub fn get_shard_status(&self) -> HashMap<String, String> {
        self.shard_manager.get_shard_health_status()
    }

    /// Get list of healthy shard IDs
    pub fn get_healthy_shards(&self) -> Vec<String> {
        self.shard_manager.get_healthy_shards()
    }

    /// Mark a shard as down (called when detection indicates failure)
    pub fn mark_shard_down(&self, shard_id: &str) {
        // We'd need to expose this through ShardManager in a real implementation
        tracing::warn!(shard_id, "Shard marked as down from external signal");
    }
}

// ZooKeeper integration preparation
#[derive(Serialize, Deserialize)]
pub struct ZkShardConfig {
    pub shards: Vec<ShardNodeConfig>,
    pub version: u64,
}

// TODO: Implement ZooKeeper integration
// This will include:
// 1. Watching for shard configuration changes
// 2. Leader election for shard rebalancing
// 3. Shard migration coordination
// 4. Health monitoring