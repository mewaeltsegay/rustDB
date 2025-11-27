/// Extract a shard key from a SQL query string (first integer found)
fn extract_shard_key(query: &str) -> Option<u64> {
    for token in query.split(|c: char| !c.is_ascii_alphanumeric()) {
        if let Ok(num) = token.parse::<u64>() {
            return Some(num);
        }
    }
    None
}

use jsonrpc_core::{Result, IoHandler};
use jsonrpc_derive::rpc;
use jsonrpc_http_server::ServerBuilder;
use crate::database::Database;
use crate::replication::{ReplicationConfig, ReplicationManager};
use crate::row::RowInterface;
use crate::sharding::ShardedDatabase;
use std::sync::Arc;
use std::sync::Mutex;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use hex;

// Define response types for better error handling
#[derive(Serialize, Deserialize)]
pub struct QueryResponse {
    pub success: bool,
    pub message: String,
    pub rows: Option<Vec<Vec<String>>>,
}

#[derive(Serialize, Deserialize)]
pub struct ShardHealthResponse {
    pub shards: std::collections::HashMap<String, String>,
    pub healthy_count: usize,
    pub total_count: usize,
}

#[rpc]
pub trait Rpc {
    #[rpc(name = "execute")]
    fn execute(&self, query: String) -> Result<QueryResponse>;

    #[rpc(name = "ping")]
    fn ping(&self) -> Result<String>;
    
    #[rpc(name = "list_tables")]
    fn list_tables(&self) -> Result<Vec<String>>;

    #[rpc(name = "replication_get_events")]
    fn replication_get_events(&self) -> Result<Vec<crate::replication::ReplicationEvent>>;

    #[rpc(name = "replication_checksum")]
    fn replication_checksum(&self) -> Result<String>;

    #[rpc(name = "replication_apply_events")]
    fn replication_apply_events(&self, events: Vec<crate::replication::ReplicationEvent>) -> Result<bool>;

    #[rpc(name = "replication_register_replica")]
    fn replication_register_replica(&self, url: String) -> Result<bool>;

    #[rpc(name = "shard_health")]
    fn shard_health(&self) -> Result<ShardHealthResponse>;

    #[rpc(name = "shard_status")]
    fn shard_status(&self) -> Result<String>;
}

pub struct RpcServer {
    db: Arc<Mutex<Database>>,
    sharded_db: Option<Arc<ShardedDatabase>>,
    replication_manager: Arc<Mutex<ReplicationManager>>,
}

impl RpcServer {
    pub fn new(config: Option<ReplicationConfig>, num_shards: Option<usize>) -> Self {
        // If sharding is enabled, create sharded database
        let sharded_db = if let Some(n) = num_shards {
            // If SHARD_URLS env is set, prefer remote-shard mode (coordinator polling shards)
            if let Ok(urls_str) = std::env::var("SHARD_URLS") {
                let urls: Vec<String> = urls_str
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                tracing::info!("Coordinator running in remote-shard mode with {} shard URLs", urls.len());
                let sd = Arc::new(ShardedDatabase::new(n));
                sd.start_heartbeat_checker();
                sd.start_remote_poller(urls);
                Some(sd)
            } else {
                // local sharded mode (default) - keep local shards alive with periodic heartbeats
                tracing::info!("Coordinator running in local-shard mode with {} shards", n);
                let sd = Arc::new(ShardedDatabase::new(n));
                sd.start_heartbeat_checker();
                sd.start_local_heartbeat_recorder();
                Some(sd)
            }
        } else {
            None
        };
        let db = Arc::new(Mutex::new(Database::new()));
        
        let replication_manager = Arc::new(Mutex::new(ReplicationManager::new(
            config.unwrap_or_else(|| ReplicationConfig::new_primary()),
            Arc::clone(&db),
        )));

        // If this node is configured as a replica, start its sync and display tasks.
        {
            let repl_guard = replication_manager.lock().unwrap_or_else(|p| p.into_inner());
            if !repl_guard.is_primary() {
                // start background sync with primary
                repl_guard.start_sync_task();
                // start periodic display of local DB for debugging/visibility
                repl_guard.start_display_task();
            }
        }

        RpcServer {
            db,
            sharded_db,
            replication_manager,
        }
    }

    pub fn is_primary(&self) -> bool {
        self.replication_manager
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .is_primary()
    }
}

impl Rpc for RpcServer {
    fn execute(&self, query: String) -> Result<QueryResponse> {
        // Only primary can execute write queries
        let repl = self.replication_manager.lock().unwrap_or_else(|p| p.into_inner());
        if !repl.is_primary() {
            return Ok(QueryResponse {
                success: false,
                message: "This is a replica server. Write operations are only allowed on the primary server.".to_string(),
                rows: None,
            });
        }

        // Some operations (like CREATE TABLE) need to happen on all shards
        // We'll capture SELECT results (if any) from the DB we executed against
        let mut captured_rows: Option<Vec<Vec<String>>> = None;

        // small helper to collect rows for a SELECT from a Database reference
        let collect_select_rows = |db: &crate::database::Database, query: &str| -> Option<Vec<Vec<String>>> {
            let q = query.trim_end_matches(';');
            let upper = q.to_uppercase();
            if !upper.starts_with("SELECT") || !upper.contains("FROM") {
                return None;
            }
            // parse columns between SELECT and FROM
            let mut columns: Vec<String> = vec![];
            let mut table = String::new();
            let mut where_clause = String::new();
            if let Some(select_idx) = upper.find("SELECT ") {
                if let Some(from_idx) = upper.find(" FROM ") {
                    if from_idx > select_idx + 7 {
                        let cols = &q[select_idx + 7..from_idx];
                        columns = cols.split(',').map(|s| s.trim().to_string()).collect();
                    }
                    let after_from = &q[from_idx + 6..];
                    if !after_from.is_empty() {
                        if let Some(where_idx) = after_from.to_uppercase().find(" WHERE ") {
                            table = after_from[..where_idx].trim().to_string();
                            where_clause = after_from[where_idx + 7..].trim().to_string();
                            if where_clause.is_empty() {
                                where_clause = "true".to_string();
                            }
                        } else {
                            table = after_from.trim().to_string();
                            where_clause = "".to_string();
                        }
                    }
                }
            }

            if table.is_empty() || !db.tables.contains_key(&table) {
                return Some(Vec::new());
            }

            let selected_columns = if columns == vec!["*".to_string()] || columns.is_empty() {
                db.get_table_columns(&table)
            } else {
                columns
            };

            let table_ref = db.tables.get(&table).unwrap();
            let table_schema_cols = table_ref.schema.columns.clone();
            let pred = crate::query::query_to_predicate(&table_schema_cols, &where_clause);

            let mut rows: Vec<Vec<String>> = Vec::new();
            for row in &table_ref.rows {
                if pred(row.get_values()) {
                    let mut out_row: Vec<String> = Vec::new();
                    for col in &selected_columns {
                        let val = row.get_by_name(col, &table_ref.schema).cloned().unwrap_or_default();
                        out_row.push(val);
                    }
                    rows.push(out_row);
                }
            }
            Some(rows)
        };

        if let Some(ref sharded_db) = self.sharded_db {
            if query.to_uppercase().starts_with("CREATE TABLE") {
                // Broadcast CREATE TABLE to all shards
                for shard in sharded_db.get_all_shards() {
                    let mut db = shard.db.lock().unwrap_or_else(|p| p.into_inner());
                    crate::sql::execute_sql(&mut db, &query);
                }
            } else {
                // Route other queries based on shard key
                let key = extract_shard_key(&query).unwrap_or(0);
                let shard = sharded_db.get_shard(&key);
                let mut db = shard.db.lock().unwrap_or_else(|p| p.into_inner());
                crate::sql::execute_sql(&mut db, &query);
                // capture SELECT results from this shard DB if applicable
                if let Some(rows) = collect_select_rows(&db, &query) {
                    captured_rows = Some(rows);
                }
            }
        } else {
            let mut db = self.db.lock().unwrap_or_else(|p| p.into_inner());
            crate::sql::execute_sql(&mut db, &query);
            // capture SELECT results from the main DB if applicable
            if let Some(rows) = collect_select_rows(&db, &query) {
                captured_rows = Some(rows);
            }
        }

        repl.record_event(query.clone());
        Ok(QueryResponse {
            success: true,
            message: "Query executed successfully".to_string(),
            rows: captured_rows,
        })
    }

    fn ping(&self) -> Result<String> {
        Ok("pong".to_string())
    }
    
    fn list_tables(&self) -> Result<Vec<String>> {
        let db = self.db.lock().unwrap_or_else(|p| p.into_inner());
        Ok(db.tables.keys().cloned().collect())
    }

    fn replication_get_events(&self) -> Result<Vec<crate::replication::ReplicationEvent>> {
        let repl = self.replication_manager.lock().unwrap_or_else(|p| p.into_inner());
        Ok(repl.get_events())
    }

    fn replication_checksum(&self) -> Result<String> {
        // Build a deterministic string representation of the DB and SHA256 it
        let db = self.db.lock().unwrap_or_else(|p| p.into_inner());
        // Collect table names sorted for deterministic ordering
        let mut table_names: Vec<_> = db.tables.keys().cloned().collect();
        table_names.sort();

        let mut s = String::new();
        for tname in table_names {
            if let Some(table) = db.tables.get(&tname) {
                s.push_str(&format!("TABLE:{};", tname));
                // schema
                for col in &table.schema.columns {
                    s.push_str(&format!("COL:{}:{:?};", col.name, col.col_type));
                }
                // rows in insertion order
                for row in &table.rows {
                    for val in row.get_values() {
                        s.push_str(&format!("VAL:{};", val));
                    }
                }
            }
        }

        let mut hasher = Sha256::new();
        hasher.update(s.as_bytes());
        let digest = hasher.finalize();
        Ok(hex::encode(digest))
    }

    fn replication_apply_events(&self, events: Vec<crate::replication::ReplicationEvent>) -> Result<bool> {
        let repl = self.replication_manager.lock().unwrap_or_else(|p| p.into_inner());
        match repl.apply_events(events) {
            Ok(_) => Ok(true),
            Err(_e) => Err(jsonrpc_core::Error::internal_error()),
        }
    }

    fn replication_register_replica(&self, url: String) -> Result<bool> {
        // Only primary should accept registrations
        let mut repl = self.replication_manager.lock().unwrap_or_else(|p| p.into_inner());
        if !repl.is_primary() {
            return Ok(false);
        }
        repl.add_replica(url);
        Ok(true)
    }

    fn shard_health(&self) -> Result<ShardHealthResponse> {
        if let Some(ref sharded_db) = self.sharded_db {
            let status = sharded_db.get_shard_status();
            let healthy_count = status.values().filter(|v| v == &&"healthy".to_string()).count();
            let total_count = status.len();
            Ok(ShardHealthResponse {
                shards: status,
                healthy_count,
                total_count,
            })
        } else {
            Ok(ShardHealthResponse {
                shards: std::collections::HashMap::new(),
                healthy_count: 0,
                total_count: 0,
            })
        }
    }

    fn shard_status(&self) -> Result<String> {
        if let Some(ref sharded_db) = self.sharded_db {
            let healthy = sharded_db.get_healthy_shards();
            let status = sharded_db.get_shard_status();
            let msg = format!(
                "Total shards: {}, Healthy: {}, Unhealthy: {}. Status: {:?}",
                status.len(),
                healthy.len(),
                status.len() - healthy.len(),
                status
            );
            Ok(msg)
        } else {
            Ok("No sharding enabled on this node".to_string())
        }
    }
}

pub fn start_server(port: u16, config: Option<ReplicationConfig>, num_shards: Option<usize>) -> jsonrpc_http_server::Server {
    let rpc = RpcServer::new(config, num_shards);
    let mut io = IoHandler::new();
    io.extend_with(rpc.to_delegate());

    let server = ServerBuilder::new(io)
        .threads(3)
        .cors(jsonrpc_http_server::DomainsValidation::AllowOnly(vec![
            "http://localhost:3000".into(),
            "http://127.0.0.1:3000".into(),
        ]))
        // Bind to 0.0.0.0 so the server is reachable from outside the container
        .start_http(&format!("0.0.0.0:{}", port).parse().unwrap())
        .expect("Unable to start RPC server");

    println!("RPC Server running on http://0.0.0.0:{}", port);
    server
}
