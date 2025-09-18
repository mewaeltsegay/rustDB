use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::{Write, Read};
// database.rs
pub trait DatabaseInterface {
    fn create_table_with_constraints(&mut self, table_name: &str, columns: Vec<String>, primary_key: Option<String>, unique_columns: Vec<String>);
    fn create_table(&mut self, table_name: &str, columns: Vec<String>);
    fn insert(&mut self, table_name: &str, values: Vec<String>);
    /// Updates all rows matching the predicate with new values.
    fn update<F>(&mut self, table_name: &str, set_values: Vec<String>, predicate: F)
    where F: Fn(&Vec<String>) -> bool;
    /// Deletes all rows matching the predicate.
    fn delete<F>(&mut self, table_name: &str, predicate: F)
    where F: Fn(&Vec<String>) -> bool;
    /// Selects and prints all rows matching the predicate.
    fn select<F>(&self, table_name: &str, columns: Vec<String>, predicate: F)
    where F: Fn(&Vec<String>) -> bool;
}

use std::collections::HashMap;
use crate::table::{Table, TableInterface};


#[derive(Serialize, Deserialize, Clone)]
pub struct Database {
    pub tables: HashMap<String, Table>,
}

impl Database {
    /// Returns the columns of the table with the given name, or an empty vec if not found.
    pub fn get_table_columns(&self, table_name: &str) -> Vec<String> {
        if let Some(table) = self.tables.get(table_name) {
            table.columns.clone()
        } else {
            vec![]
        }
    }
    pub fn new() -> Self {
        Database {
            tables: HashMap::new(),
        }
    }

    /// Save the database to a file as JSON
    pub fn save_to_file(&self, path: &str) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self).unwrap();
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    /// Load the database from a file (JSON)
    pub fn load_from_file(path: &str) -> std::io::Result<Self> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let db: Database = serde_json::from_str(&contents).unwrap();
        Ok(db)
    }
}

impl DatabaseInterface for Database {
    /// Create a table with constraints (primary key and unique columns)
    fn create_table_with_constraints(&mut self, table_name: &str, columns: Vec<String>, primary_key: Option<String>, unique_columns: Vec<String>) {
        let table = Table::new(table_name.to_string(), columns, primary_key, unique_columns);
        self.tables.insert(table_name.to_string(), table);
        println!("Created table: {}", table_name);
    }

    fn create_table(&mut self, table_name: &str, columns: Vec<String>) {
        let table = Table::new(table_name.to_string(), columns, None, vec![]);
        self.tables.insert(table_name.to_string(), table);
        println!("Created table: {}", table_name);
    }

    fn insert(&mut self, table_name: &str, values: Vec<String>) {
        if let Some(table) = self.tables.get_mut(table_name) {
            table.add_row(values);
            println!("Inserted values into table: {}", table_name);
        } else {
            println!("Table not found: {}", table_name);
        }
    }

    fn update<F>(&mut self, table_name: &str, set_values: Vec<String>, predicate: F)
    where F: Fn(&Vec<String>) -> bool {
        if let Some(table) = self.tables.get_mut(table_name) {
            table.update_rows(set_values, predicate);
            println!("Updated matching rows in table: {}", table_name);
        } else {
            println!("Table not found: {}", table_name);
        }
    }

    fn delete<F>(&mut self, table_name: &str, predicate: F)
    where F: Fn(&Vec<String>) -> bool {
        if let Some(table) = self.tables.get_mut(table_name) {
            table.delete_rows(predicate);
            println!("Deleted matching rows in table: {}", table_name);
        } else {
            println!("Table not found: {}", table_name);
        }
    }

    fn select<F>(&self, table_name: &str, columns: Vec<String>, predicate: F)
    where F: Fn(&Vec<String>) -> bool {
        if let Some(table) = self.tables.get(table_name) {
            println!("Selecting from table: {}", table_name);
            let rows = table.select_rows(&predicate);
            // Print header
            let col_indices: Vec<_> = if columns == vec!["*"] {
                (0..table.columns.len()).collect()
            } else {
                columns.iter().filter_map(|c| table.columns.iter().position(|tc| tc == c)).collect()
            };
            let header: Vec<_> = col_indices.iter().map(|&i| &table.columns[i]).collect();
            let col_widths: Vec<_> = col_indices.iter().map(|&i| {
                let max_val = rows.iter().map(|row| row.get(i).map(|v| v.len()).unwrap_or(0)).max().unwrap_or(0);
                std::cmp::max(table.columns[i].len(), max_val)
            }).collect();
            // Print header row
            for (h, w) in header.iter().zip(&col_widths) {
                print!("{:<width$} ", h, width = w);
            }
            println!();
            // Print separator
            for w in &col_widths {
                print!("{:-<width$}-", "", width = *w);
            }
            println!();
            // Print rows
            for row in &rows {
                for (col_idx, w) in col_indices.iter().zip(&col_widths) {
                    let val = row.get(*col_idx).map(|s| s.as_str()).unwrap_or("");
                    print!("{:<width$} ", val, width = w);
                }
                println!();
            }
        } else {
            println!("Table not found: {}", table_name);
        }
    }
}
