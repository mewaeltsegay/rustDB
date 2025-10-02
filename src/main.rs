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

fn main() {
    let mut db = init_demo_database();
    println!("Welcome to RustDB. Type 'exit' or 'quit' to leave.");

    println!("Welcome to RustDB. Type 'exit' or 'quit' to leave.");

    loop {
        // parse input from user
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
