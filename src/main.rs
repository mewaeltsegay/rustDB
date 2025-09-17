mod query;
mod sql;
use query::query_to_predicate;
use sql::execute_sql;
// main.rs
mod database;
mod table;
mod row;

use database::{Database, DatabaseInterface};

fn main() {
    // Create a new database
    let mut my_database = Database::new();

    // Create a table with columns: "id", "name", "age"
    let columns = vec!["id".to_string(), "name".to_string(), "age".to_string()];
    my_database.create_table("Users", columns);

    // Insert some rows into the "Users" table
    my_database.insert("Users", vec!["1".to_string(), "Alice".to_string(), "30".to_string()]);
    my_database.insert("Users", vec!["2".to_string(), "Bob".to_string(), "25".to_string()]);


    // --- SQL-like query examples ---
    println!("\n-- SQL-like SELECT --");
    execute_sql(&mut my_database, "SELECT * FROM Users WHERE age > 25");

    println!("\n-- SQL-like INSERT --");
    execute_sql(&mut my_database, "INSERT INTO Users (id, name, age) VALUES (3, 'Carol', 22)");

    println!("\n-- SQL-like UPDATE --");
    execute_sql(&mut my_database, "UPDATE Users SET age = 40 WHERE id == 2");

    println!("\n-- SQL-like DELETE --");
    execute_sql(&mut my_database, "DELETE FROM Users WHERE name == 'Alice'");

    println!("\n-- SQL-like SELECT (after changes) --");
    execute_sql(&mut my_database, "SELECT * FROM Users WHERE id >= 0");
}
