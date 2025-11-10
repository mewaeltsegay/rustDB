use std::sync::{Arc, Mutex};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use crate::database::Database;
use serde::{Serialize, Deserialize};

/// Configuration for a shard node
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShardNodeConfig {
    pub id: String,
    pub url: String,
    pub range_start: u64,
    pub range_end: u64,
}

/// Represents a single shard in the system
pub struct ShardNode {
    pub config: ShardNodeConfig,
    pub db: Arc<Mutex<Database>>,
}

impl ShardNode {
    pub fn new(config: ShardNodeConfig) -> Self {
        tracing::debug!(
            shard_id = config.id,
            "Creating new shard node instance"
        );
        Self {
            config,
            db: Arc::new(Mutex::new(Database::new())),
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
}

/// Manages the sharding configuration and routing
pub struct ShardManager {
    shards: Vec<Arc<ShardNode>>,
}

impl ShardManager {
    pub fn new(num_shards: usize) -> Self {
        let mut shards = Vec::with_capacity(num_shards);
        let range_size = u64::MAX / num_shards as u64;
        
        tracing::info!(num_shards, "Initializing shard manager");
        
        for i in 0..num_shards {
            let range_start = i as u64 * range_size;
            let range_end = if i == num_shards - 1 {
                u64::MAX
            } else {
                (i + 1) as u64 * range_size
            };

            let config = ShardNodeConfig {
                id: format!("shard_{}", i),
                url: format!("http://localhost:{}", 8000 + i), // Placeholder URLs
                range_start,
                range_end,
            };

            tracing::info!(
                shard_id = config.id,
                range_start = config.range_start,
                range_end = config.range_end,
                "Initializing shard"
            );
            shards.push(Arc::new(ShardNode::new(config)));
        }

        tracing::info!("Shard manager initialization complete");
        Self { shards }
    }

    pub fn get_shard_for_key<K: Hash>(&self, key: &K) -> Arc<ShardNode> {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash = hasher.finish();

        tracing::debug!(
            hash,
            "Finding shard for key"
        );

        // Find the shard that contains this hash
        let shard = self.shards
            .iter()
            .find(|shard| shard.contains_key(hash))
            .expect("No shard found for key")
            .clone();

        tracing::info!(
            hash,
            shard_id = shard.config.id,
            "Key mapped to shard"
        );

        shard
    }

    pub fn get_all_shards(&self) -> Vec<Arc<ShardNode>> {
        self.shards.clone()
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

    /// Get a reference to the shard that should handle this key
    pub fn get_shard<K: Hash>(&self, key: &K) -> Arc<ShardNode> {
        self.shard_manager.get_shard_for_key(key)
    }

    /// Get all shards for operations that need to touch all shards
    pub fn get_all_shards(&self) -> Vec<Arc<ShardNode>> {
        self.shard_manager.get_all_shards()
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