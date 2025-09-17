// table.rs
/// Trait defining the interface for a table.
/// Provides methods to add, update, delete, and select rows.
pub trait TableInterface {
    /// Adds a new row to the table with the given values.
    fn add_row(&mut self, values: Vec<String>);
    /// Updates all rows matching the predicate with new values.
    fn update_rows<F>(&mut self, set_values: Vec<String>, predicate: F)
    where F: Fn(&Vec<String>) -> bool;
    /// Deletes all rows matching the predicate.
    fn delete_rows<F>(&mut self, predicate: F)
    where F: Fn(&Vec<String>) -> bool;
    /// Selects and returns all rows matching the predicate.
    fn select_rows<F>(&self, predicate: F) -> Vec<Vec<String>>
    where F: Fn(&Vec<String>) -> bool;
}

use crate::row::{Row, RowInterface};

/// Struct representing a table in the database.
/// Stores the table's name, columns, and rows.
pub struct Table {
    name: String,      // Name of the table
    pub columns: Vec<String>, // Column names
    rows: Vec<Box<dyn RowInterface>>, // Rows in the table
}

impl Table {
    /// Creates a new Table with the given name and columns.
    pub fn new(name: String, columns: Vec<String>) -> Self {
        Table {
            name,
            columns,
            rows: Vec::new(),
        }
    }
}

/// Implements the TableInterface trait for the Table struct.
impl TableInterface for Table {
    /// Adds a new row to the table with the given values.
    fn add_row(&mut self, values: Vec<String>) {
        let row = Box::new(Row::new(values));
        self.rows.push(row);
    }

    /// Updates all rows matching the predicate with new values.
    fn update_rows<F>(&mut self, set_values: Vec<String>, predicate: F)
    where F: Fn(&Vec<String>) -> bool {
        for row in &mut self.rows {
            if predicate(row.get_values()) {
                row.set_values(set_values.clone());
            }
        }
    }

    /// Deletes all rows matching the predicate.
    fn delete_rows<F>(&mut self, predicate: F)
    where F: Fn(&Vec<String>) -> bool {
        self.rows.retain(|row| !predicate(row.get_values()));
    }

    /// Selects and returns all rows matching the predicate.
    fn select_rows<F>(&self, predicate: F) -> Vec<Vec<String>>
    where F: Fn(&Vec<String>) -> bool {
        self.rows
            .iter()
            .filter(|row| predicate(row.get_values()))
            .map(|row| row.get_values().clone())
            .collect()
    }
}
