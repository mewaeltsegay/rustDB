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

    // Select from the table (print all rows)
    my_database.select("Users", vec!["id".to_string(), "name".to_string(), "age".to_string()], "");

    // Update a row (set "age" to "31" for the first row)
    my_database.update("Users", vec!["1".to_string(), "Alice".to_string(), "31".to_string()], "");

    // Delete a row (delete the first row)
    my_database.delete("Users", "");

    // Select again to see changes
    my_database.select("Users", vec!["id".to_string(), "name".to_string(), "age".to_string()], "");
}
