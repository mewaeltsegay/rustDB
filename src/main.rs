use std::io::Write;

mod database;
mod query;
mod row;
mod schema;
mod sql;
mod table;

// use std::io::Stdin;

use database::Database;
use sql::execute_sql;

fn init_demo_database() -> Database {
    let mut db = Database::new();
    
    // Create Products table and insert sample data
    execute_sql(
        &mut db,
        "CREATE TABLE Products (id INT PRIMARY KEY, name STRING, price FLOAT, stock INT)",
    );
    execute_sql(
        &mut db,
        "INSERT INTO Products (id, name, price, stock) VALUES (1, 'Pen', 2.5, 100)",
    );
    execute_sql(
        &mut db,
        "INSERT INTO Products (id, name, price, stock) VALUES (2, 'Pencil', 1.2, 50)",
    );
    execute_sql(
        &mut db,
        "INSERT INTO Products (id, name, price, stock) VALUES (3, 'Eraser', 0.8, 30)",
    );
    
    db
}

mod server;
mod client;
mod replication;

use crate::replication::ReplicationConfig;

fn run_cli_mode() {
    let mut db = init_demo_database();
    println!("Welcome to RustDB CLI mode. Type 'exit' or 'quit' to leave.");

    loop {
        let mut input = String::new();
        print!("rustdb> ");
        std::io::stdout().flush().unwrap();
        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let input = input.trim();
        if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
            println!("Exiting RustDB. Goodbye!");
            break;
        }
        if !input.is_empty() {
            execute_sql(&mut db, input);
        }
    }
}

fn main() {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "--server" => {
                let mut arg_iter = args.iter().skip(2);
                let mut port = 3030;
                let mut is_replica = false;
                let mut primary_url = None;
                let mut replicas_arg: Option<String> = None;

                while let Some(arg) = arg_iter.next() {
                    match arg.as_str() {
                        "--port" => {
                            if let Some(p) = arg_iter.next() {
                                port = p.parse().unwrap_or(3030);
                            }
                        }
                        "--replica" => {
                            is_replica = true;
                        }
                        "--primary-url" => {
                            if let Some(url) = arg_iter.next() {
                                primary_url = Some(url.to_string());
                            }
                        }
                        "--replicas" => {
                            if let Some(list) = arg_iter.next() {
                                replicas_arg = Some(list.to_string());
                            }
                        }
                        _ => {}
                    }
                }

                let config = if is_replica {
                    if let Some(primary) = primary_url {
                        println!("Starting RustDB in replica mode...");
                        Some(ReplicationConfig::new_replica(primary))
                    } else {
                        eprintln!("Error: --primary-url is required for replica servers");
                        std::process::exit(1);
                    }
                } else {
                    println!("Starting RustDB in primary mode...");
                    let mut cfg = ReplicationConfig::new_primary();
                    if let Some(list) = replicas_arg {
                        // parse comma-separated list of replica URLs
                        for r in list.split(',') {
                            let url = r.trim().to_string();
                            if !url.is_empty() {
                                cfg.replicas.insert(url);
                            }
                        }
                    }
                    Some(cfg)
                };

                let server = server::start_server(port, config);
                println!("RustDB RPC Server running on http://127.0.0.1:{}", port);
                if is_replica {
                    println!("Syncing with primary server...");
                }
                server.wait();
            }
            "--client" => {
                println!("Starting RustDB in client mode...");
                match client::run_client_example() {
                    Ok(_) => println!("Client operations completed successfully"),
                    Err(e) => eprintln!("Client error: {}", e),
                }
            }
            _ => {
                println!("Unknown option: {}", args[1]);
                println!("Usage:");
                println!("  cargo run                                                    # Run in CLI mode");
                println!("  cargo run -- --server [--port <port>]                       # Run in primary server mode");
                println!("  cargo run -- --server --replica --primary-url <url> [--port <port>] # Run in replica mode");
                println!("  cargo run -- --client                                       # Run in client mode");
            }
        }
    } else {
        run_cli_mode();
    }
}
