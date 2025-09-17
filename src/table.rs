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
    pub primary_key: Option<String>, // Single-column primary key
    pub unique_columns: Vec<String>, // Unique columns
}

impl Table {
    /// Creates a new Table with the given name, columns, primary key, and unique columns.
    pub fn new(name: String, columns: Vec<String>, primary_key: Option<String>, unique_columns: Vec<String>) -> Self {
        Table {
            name,
            columns,
            rows: Vec::new(),
            primary_key,
            unique_columns,
        }
    }
}

/// Implements the TableInterface trait for the Table struct.
impl TableInterface for Table {
    /// Adds a new row to the table with the given values, enforcing primary key and unique constraints.
    fn add_row(&mut self, values: Vec<String>) {
        // Check primary key constraint
        if let Some(pk_col) = &self.primary_key {
            if let Some(pk_idx) = self.columns.iter().position(|c| c == pk_col) {
                let pk_val = values.get(pk_idx);
                if pk_val.is_none() {
                    println!("Primary key column '{}' missing in inserted values!", pk_col);
                    return;
                }
                let pk_val = pk_val.unwrap();
                for row in &self.rows {
                    if let Some(existing_val) = row.get_values().get(pk_idx) {
                        if existing_val == pk_val {
                            println!("Primary key constraint violation: '{}' must be unique!", pk_col);
                            return;
                        }
                    }
                }
            }
        }
        // Check unique constraints
        for uniq_col in &self.unique_columns {
            if let Some(uniq_idx) = self.columns.iter().position(|c| c == uniq_col) {
                let uniq_val = values.get(uniq_idx);
                if uniq_val.is_none() { continue; }
                let uniq_val = uniq_val.unwrap();
                for row in &self.rows {
                    if let Some(existing_val) = row.get_values().get(uniq_idx) {
                        if existing_val == uniq_val {
                            println!("Unique constraint violation: '{}' must be unique!", uniq_col);
                            return;
                        }
                    }
                }
            }
        }
        let row = Box::new(Row::new(values));
        self.rows.push(row);
    }

    /// Updates all rows matching the predicate with new values, enforcing primary key and unique constraints.
    fn update_rows<F>(&mut self, set_values: Vec<String>, predicate: F)
    where F: Fn(&Vec<String>) -> bool {
        // Find indices for constraints
        let pk_idx = self.primary_key.as_ref().and_then(|pk| self.columns.iter().position(|c| c == pk));
        let uniq_indices: Vec<_> = self.unique_columns.iter().filter_map(|uc| self.columns.iter().position(|c| c == uc)).collect();
        // Collect indices of rows to update
        let to_update: Vec<usize> = self.rows.iter().enumerate().filter_map(|(i, row)| if predicate(row.get_values()) { Some(i) } else { None }).collect();
        for &row_idx in &to_update {
            // Check primary key constraint
            if let Some(idx) = pk_idx {
                let new_val = set_values.get(idx);
                if let Some(new_val) = new_val {
                    for (other_idx, other) in self.rows.iter().enumerate() {
                        if other_idx == row_idx { continue; }
                        if let Some(existing_val) = other.get_values().get(idx) {
                            if existing_val == new_val {
                                println!("Primary key constraint violation on update: '{}' must be unique!", self.columns[idx]);
                                return;
                            }
                        }
                    }
                }
            }
            // Check unique constraints
            for &uniq_idx in &uniq_indices {
                let new_val = set_values.get(uniq_idx);
                if let Some(new_val) = new_val {
                    for (other_idx, other) in self.rows.iter().enumerate() {
                        if other_idx == row_idx { continue; }
                        if let Some(existing_val) = other.get_values().get(uniq_idx) {
                            if existing_val == new_val {
                                println!("Unique constraint violation on update: '{}' must be unique!", self.columns[uniq_idx]);
                                return;
                            }
                        }
                    }
                }
            }
        }
        // Perform updates after all checks pass
        for &row_idx in &to_update {
            self.rows[row_idx].set_values(set_values.clone());
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
