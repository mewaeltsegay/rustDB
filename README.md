# Simple Rust Database Project

This project is a basic implementation of a simple in-memory database system in Rust. It demonstrates the use of traits, structs, and dynamic dispatch to model database tables and rows, and provides basic CRUD (Create, Read, Update, Delete) operations.

## Features
- **Database**: Manages multiple tables, each identified by a unique name.
- **Table**: Stores rows and column names, and supports adding, updating, deleting, and selecting rows.
- **Row**: Represents a single record in a table, storing values as strings.
- **Traits**: Used to define interfaces for tables and rows, allowing for flexible and extensible design.

## File Structure
- `src/main.rs`: Entry point. Demonstrates usage of the database, including creating tables, inserting, updating, deleting, and selecting rows.
- `src/database.rs`: Contains the `Database` struct and `DatabaseInterface` trait, implementing database-level operations.
- `src/table.rs`: Contains the `Table` struct and `TableInterface` trait, implementing table-level operations.
- `src/row.rs`: Contains the `Row` struct and `RowInterface` trait, implementing row-level operations.

## Example Usage
The main function creates a database, adds a table called `Users` with columns `id`, `name`, and `age`, and performs several operations:

```
let mut my_database = Database::new();
my_database.create_table("Users", vec!["id".to_string(), "name".to_string(), "age".to_string()]);
my_database.insert("Users", vec!["1".to_string(), "Alice".to_string(), "30".to_string()]);
my_database.select("Users", vec!["id".to_string(), "name".to_string(), "age".to_string()], "");
my_database.update("Users", vec!["1".to_string(), "Alice".to_string(), "31".to_string()], "");
my_database.delete("Users", "");
```

## How to Run
1. Ensure you have [Rust](https://www.rust-lang.org/tools/install) installed.
2. Clone or download this repository.
3. Open a terminal in the project directory.
4. Run:
   ```
   cargo run
   ```

## Notes
- This project is for educational purposes and does not persist data to disk.
- All data is stored in memory and lost when the program exits.
- The CRUD operations are simplified (e.g., always operate on the first row for update/delete/select).

## License
MIT License
