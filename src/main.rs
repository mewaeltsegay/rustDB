mod query;
mod sql;
mod database;
mod table;
mod row;

use database::{Database, DatabaseInterface};
use sql::execute_sql;
fn main() {
    // Create a new database
    let mut my_database = Database::new();

    // --- SQL CREATE TABLE with PRIMARY KEY and UNIQUE ---
    println!("\n-- SQL CREATE TABLE with PRIMARY KEY and UNIQUE --");
    execute_sql(&mut my_database, "CREATE TABLE Users (id PRIMARY KEY, name, email UNIQUE, age)");

    // Insert valid rows
    println!("\n-- SQL INSERT valid rows --");
    execute_sql(&mut my_database, "INSERT INTO Users (id, name, email, age) VALUES (1, 'Alice', 'alice@example.com', 30)");
    execute_sql(&mut my_database, "INSERT INTO Users (id, name, email, age) VALUES (2, 'Bob', 'bob@example.com', 25)");

    // Attempt to insert duplicate primary key
    println!("\n-- SQL INSERT duplicate PRIMARY KEY (should fail) --");
    execute_sql(&mut my_database, "INSERT INTO Users (id, name, email, age) VALUES (1, 'Carol', 'carol@example.com', 22)");

    // Attempt to insert duplicate unique column
    println!("\n-- SQL INSERT duplicate UNIQUE (should fail) --");
    execute_sql(&mut my_database, "INSERT INTO Users (id, name, email, age) VALUES (3, 'Dave', 'alice@example.com', 28)");

    // Show table after attempted violations
    println!("\n-- SQL SELECT after constraint tests --");
    execute_sql(&mut my_database, "SELECT * FROM Users WHERE id >= 0");

    // Attempt to update to duplicate primary key
    println!("\n-- SQL UPDATE to duplicate PRIMARY KEY (should fail) --");
    execute_sql(&mut my_database, "UPDATE Users SET id = 2 WHERE id == 1");

    // Attempt to update to duplicate unique column
    println!("\n-- SQL UPDATE to duplicate UNIQUE (should fail) --");
    execute_sql(&mut my_database, "UPDATE Users SET email = 'bob@example.com' WHERE id == 1");

    // Show table after attempted update violations
    println!("\n-- SQL SELECT after update constraint tests --");
    execute_sql(&mut my_database, "SELECT * FROM Users WHERE id >= 0");
}
