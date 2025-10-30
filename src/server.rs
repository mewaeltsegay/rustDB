use jsonrpc_core::{Result, IoHandler};
use jsonrpc_derive::rpc;
use jsonrpc_http_server::ServerBuilder;
use crate::database::Database;
use crate::replication::{ReplicationConfig, ReplicationManager};
use crate::row::RowInterface;
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
}

pub struct RpcServer {
    db: Arc<Mutex<Database>>,
    replication_manager: Arc<Mutex<ReplicationManager>>,
}

impl RpcServer {
    pub fn new(config: Option<ReplicationConfig>) -> Self {
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

        let mut db = self.db.lock().unwrap_or_else(|p| p.into_inner());
        // Execute the query and record for replication
        crate::sql::execute_sql(&mut db, &query);
        repl.record_event(query);
        
        Ok(QueryResponse {
            success: true,
            message: "Query executed successfully".to_string(),
            rows: None, // TODO: Implement proper row capture
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
}

pub fn start_server(port: u16, config: Option<ReplicationConfig>) -> jsonrpc_http_server::Server {
    let rpc = RpcServer::new(config);
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
