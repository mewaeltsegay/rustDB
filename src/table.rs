// table.rs
/// Trait defining the interface for a table.
/// Provides methods to add, update, delete, and select rows.
pub trait TableInterface {
    /// Adds a new row to the table with the given values.
    fn add_row(&mut self, values: Vec<String>);
    /// Updates the row at the specified index with new values.
    fn update_row(&mut self, row_index: usize, set_values: Vec<String>);
    /// Deletes the row at the specified index.
    fn delete_row(&mut self, row_index: usize);
    /// Selects and returns the values of the row at the specified index, if it exists.
    fn select_row(&self, row_index: usize) -> Option<Vec<String>>;
}

use crate::row::{Row, RowInterface};

/// Struct representing a table in the database.
/// Stores the table's name, columns, and rows.
pub struct Table {
    name: String,      // Name of the table
    columns: Vec<String>, // Column names
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

    /// Updates the row at the specified index with new values.
    fn update_row(&mut self, row_index: usize, set_values: Vec<String>) {
        if row_index < self.rows.len() {
            self.rows[row_index].set_values(set_values);
        }
    }

    /// Deletes the row at the specified index.
    fn delete_row(&mut self, row_index: usize) {
        if row_index < self.rows.len() {
            self.rows.remove(row_index);
        }
    }

    /// Selects and returns the values of the row at the specified index, if it exists.
    fn select_row(&self, row_index: usize) -> Option<Vec<String>> {
        if row_index < self.rows.len() {
            Some(self.rows[row_index].get_values().clone())
        } else {
            None
        }
    }
}
