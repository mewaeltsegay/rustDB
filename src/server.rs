use jsonrpc_core::{Result, IoHandler};
use jsonrpc_derive::rpc;
use jsonrpc_http_server::ServerBuilder;
use crate::database::Database;
use crate::replication::{ReplicationConfig, ReplicationManager};
use std::sync::Arc;
use std::sync::Mutex;
use serde::{Deserialize, Serialize};

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

        RpcServer {
            db,
            replication_manager,
        }
    }

    pub fn is_primary(&self) -> bool {
        self.replication_manager.lock().unwrap().is_primary()
    }
}

impl Rpc for RpcServer {
    fn execute(&self, query: String) -> Result<QueryResponse> {
        // Only primary can execute write queries
        let repl = self.replication_manager.lock().unwrap();
        if !repl.is_primary() {
            return Ok(QueryResponse {
                success: false,
                message: "This is a replica server. Write operations are only allowed on the primary server.".to_string(),
                rows: None,
            });
        }

        let mut db = self.db.lock().unwrap();
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
        let db = self.db.lock().unwrap();
        Ok(db.tables.keys().cloned().collect())
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
        .start_http(&format!("127.0.0.1:{}", port).parse().unwrap())
        .expect("Unable to start RPC server");

    println!("RPC Server running on http://127.0.0.1:{}", port);
    server
}
