use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use tracing;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShardHealth {
    Healthy,
    Degraded,
    Down,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardHealthStatus {
    pub shard_id: String,
    pub node_id: String,
    pub timestamp: u64,
    pub status: String, // "healthy", "degraded", "down"
}

/// Tracks the health status of all shards in the cluster
pub struct ShardHealthMonitor {
    shard_status: Arc<Mutex<HashMap<String, ShardHealth>>>,
    last_heartbeat: Arc<Mutex<HashMap<String, Instant>>>,
    heartbeat_timeout: Duration,
}

impl ShardHealthMonitor {
    pub fn new(heartbeat_timeout_secs: u64) -> Self {
        Self {
            shard_status: Arc::new(Mutex::new(HashMap::new())),
            last_heartbeat: Arc::new(Mutex::new(HashMap::new())),
            heartbeat_timeout: Duration::from_secs(heartbeat_timeout_secs),
        }
    }

    /// Register a new shard as healthy
    pub fn register_shard(&self, shard_id: &str) {
        let mut status = self.shard_status.lock().unwrap();
        status.insert(shard_id.to_string(), ShardHealth::Healthy);

        let mut heartbeats = self.last_heartbeat.lock().unwrap();
        heartbeats.insert(shard_id.to_string(), Instant::now());

        tracing::info!(shard_id, "Registered shard with monitor");
    }

    /// Record a heartbeat for a shard (mark as healthy)
    pub fn record_heartbeat(&self, shard_id: &str) {
        let mut heartbeats = self.last_heartbeat.lock().unwrap();
        heartbeats.insert(shard_id.to_string(), Instant::now());

        let mut status = self.shard_status.lock().unwrap();
        if status.get(shard_id) != Some(&ShardHealth::Healthy) {
            tracing::info!(shard_id, "Shard recovered to healthy");
            status.insert(shard_id.to_string(), ShardHealth::Healthy);
        }
    }

    /// Mark a shard as down (e.g., when ZK node is deleted)
    pub fn mark_shard_down(&self, shard_id: &str) {
        let mut status = self.shard_status.lock().unwrap();
        status.insert(shard_id.to_string(), ShardHealth::Down);
        tracing::warn!(shard_id, "Shard marked as DOWN");
    }

    /// Mark a shard as degraded
    pub fn mark_shard_degraded(&self, shard_id: &str) {
        let mut status = self.shard_status.lock().unwrap();
        if status.get(shard_id) != Some(&ShardHealth::Down) {
            status.insert(shard_id.to_string(), ShardHealth::Degraded);
            tracing::warn!(shard_id, "Shard marked as DEGRADED");
        }
    }

    /// Get current health of a shard
    pub fn get_shard_health(&self, shard_id: &str) -> ShardHealth {
        let status = self.shard_status.lock().unwrap();
        status.get(shard_id).copied().unwrap_or(ShardHealth::Down)
    }

    /// Get all healthy shards
    pub fn get_healthy_shards(&self) -> Vec<String> {
        let status = self.shard_status.lock().unwrap();
        status
            .iter()
            .filter(|(_, health)| *health == &ShardHealth::Healthy)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Check for stale heartbeats and mark those shards as degraded/down
    pub fn check_heartbeat_timeouts(&self) {
        let heartbeats = self.last_heartbeat.lock().unwrap();
        let now = Instant::now();

        let mut status = self.shard_status.lock().unwrap();
        for (shard_id, last_beat) in heartbeats.iter() {
            let elapsed = now.duration_since(*last_beat);
            if elapsed > self.heartbeat_timeout {
                if status.get(shard_id) != Some(&ShardHealth::Down) {
                    tracing::warn!(
                        shard_id,
                        elapsed_secs = elapsed.as_secs(),
                        "Shard heartbeat timed out, marking as DOWN"
                    );
                    status.insert(shard_id.clone(), ShardHealth::Down);
                }
            }
        }
    }

    /// Get a summary of all shard health statuses
    pub fn get_health_summary(&self) -> HashMap<String, String> {
        let status = self.shard_status.lock().unwrap();
        status
            .iter()
            .map(|(id, health)| {
                let health_str = match health {
                    ShardHealth::Healthy => "healthy",
                    ShardHealth::Degraded => "degraded",
                    ShardHealth::Down => "down",
                };
                (id.clone(), health_str.to_string())
            })
            .collect()
    }

    /// Clear all health data (for reset/testing)
    pub fn clear(&self) {
        let mut status = self.shard_status.lock().unwrap();
        let mut heartbeats = self.last_heartbeat.lock().unwrap();
        status.clear();
        heartbeats.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_health() {
        let monitor = ShardHealthMonitor::new(5);
        monitor.register_shard("shard_0");
        assert_eq!(monitor.get_shard_health("shard_0"), ShardHealth::Healthy);
    }

    #[test]
    fn test_mark_down() {
        let monitor = ShardHealthMonitor::new(5);
        monitor.register_shard("shard_0");
        monitor.mark_shard_down("shard_0");
        assert_eq!(monitor.get_shard_health("shard_0"), ShardHealth::Down);
    }

    #[test]
    fn test_heartbeat_recovery() {
        let monitor = ShardHealthMonitor::new(5);
        monitor.register_shard("shard_0");
        monitor.mark_shard_down("shard_0");
        assert_eq!(monitor.get_shard_health("shard_0"), ShardHealth::Down);
        
        monitor.record_heartbeat("shard_0");
        assert_eq!(monitor.get_shard_health("shard_0"), ShardHealth::Healthy);
    }
}
