use std::collections::HashMap;
#[allow(dead_code)]
impl Table {
    /// Get a value from a row by column name.
    pub fn get_value(&self, row_idx: usize, col_name: &str) -> Option<&String> {
        self.schema
            .columns
            .iter()
            .position(|c| c.name == col_name)
            .and_then(|col_idx| self.rows.get(row_idx)?.get_values().get(col_idx))
    }

    /// Set a value in a row by column name.
    pub fn set_value(&mut self, row_idx: usize, col_name: &str, value: String) -> bool {
        if let Some(col_idx) = self.schema.columns.iter().position(|c| c.name == col_name) {
            if let Some(row) = self.rows.get_mut(row_idx) {
                if col_idx < row.get_values().len() {
                    row.set_by_name(col_name, value, &self.schema);
                    return true;
                }
            }
        }
        false
    }

    /// Selects and returns all rows matching the predicate as Vec<HashMap<String, String>>
    pub fn select_rows_named<F>(&self, predicate: F) -> Vec<HashMap<String, String>>
    where
        F: Fn(&Row) -> bool,
    {
        self.rows
            .iter()
            .filter(|row| predicate(row))
            .map(|row| {
                self.schema
                    .columns
                    .iter()
                    .enumerate()
                    .map(|(i, col)| {
                        (
                            col.name.clone(),
                            row.get_values().get(i).cloned().unwrap_or_default(),
                        )
                    })
                    .collect::<HashMap<String, String>>()
            })
            .collect()
    }
}
use crate::schema::{ColumnType, Schema};
use serde::{Deserialize, Serialize};
// table.rs
/// Trait defining the interface for a table.
/// Provides methods to add, update, delete, and select rows.
pub trait TableInterface {
    /// Adds a new row to the table with the given values.
    fn add_row(&mut self, values: Vec<String>);
    /// Updates all rows matching the predicate with new values.
    fn update_rows<F>(&mut self, set_values: Vec<String>, predicate: F)
    where
        F: Fn(&Vec<String>) -> bool;
    /// Deletes all rows matching the predicate.
    fn delete_rows<F>(&mut self, predicate: F)
    where
        F: Fn(&Vec<String>) -> bool;
    /// Selects and returns all rows matching the predicate.
    fn select_rows<F>(&self, predicate: F) -> Vec<Vec<String>>
    where
        F: Fn(&Vec<String>) -> bool;
}

use crate::row::{Row, RowInterface};

/// Struct representing a table in the database.
/// Stores the table's name, columns, and rows.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Table {
    name: String,                    // Name of the table
    pub schema: Schema,              // Table schema
    pub rows: Vec<Row>,              // Rows in the table (for serialization, use Row directly)
    pub primary_key: Option<String>, // Single-column primary key
    pub unique_columns: Vec<String>, // Unique columns
}

impl Table {
    /// Creates a new Table with the given name, columns, primary key, and unique columns.
    pub fn new(
        name: String,
        schema: Schema,
        primary_key: Option<String>,
        unique_columns: Vec<String>,
    ) -> Self {
        Table {
            name,
            schema,
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
        // Type checking
        for (i, val) in values.iter().enumerate() {
            if let Some(col) = self.schema.columns.get(i) {
                let valid = match col.col_type {
                    ColumnType::Int => val.parse::<i64>().is_ok(),
                    ColumnType::Float => val.parse::<f64>().is_ok(),
                    ColumnType::String => true,
                };
                if !valid {
                    println!(
                        "Type error: value '{}' does not match column '{}' type {:?}",
                        val, col.name, col.col_type
                    );
                    return;
                }
            }
        }
        // Check primary key constraint
        if let Some(pk_col) = &self.primary_key {
            if let Some(pk_idx) = self.schema.columns.iter().position(|c| &c.name == pk_col) {
                let pk_val = values.get(pk_idx);
                if pk_val.is_none() {
                    println!(
                        "Primary key column '{}' missing in inserted values!",
                        pk_col
                    );
                    return;
                }
                let pk_val = pk_val.unwrap();
                for row in &self.rows {
                    if let Some(existing_val) = row.get_values().get(pk_idx) {
                        if existing_val == pk_val {
                            println!(
                                "Primary key constraint violation: '{}' must be unique!",
                                pk_col
                            );
                            return;
                        }
                    }
                }
            }
        }
        // Check unique constraints
        for uniq_col in &self.unique_columns {
            if let Some(uniq_idx) = self.schema.columns.iter().position(|c| &c.name == uniq_col) {
                let uniq_val = values.get(uniq_idx);
                if uniq_val.is_none() {
                    continue;
                }
                let uniq_val = uniq_val.unwrap();
                for row in &self.rows {
                    if let Some(existing_val) = row.get_values().get(uniq_idx) {
                        if existing_val == uniq_val {
                            println!(
                                "Unique constraint violation: '{}' must be unique!",
                                uniq_col
                            );
                            return;
                        }
                    }
                }
            }
        }
        let row = Row::new(values);
        self.rows.push(row);
    }

    /// Updates all rows matching the predicate with new values, enforcing primary key and unique constraints.
    fn update_rows<F>(&mut self, set_values: Vec<String>, predicate: F)
    where
        F: Fn(&Vec<String>) -> bool,
    {
        // Type checking for non-empty update values
        for (i, val) in set_values.iter().enumerate() {
            if !val.is_empty() {
                // Only check non-empty values
                if let Some(col) = self.schema.columns.get(i) {
                    let valid = match col.col_type {
                        ColumnType::Int => val.parse::<i64>().is_ok(),
                        ColumnType::Float => val.parse::<f64>().is_ok(),
                        ColumnType::String => true,
                    };
                    if !valid {
                        println!(
                            "Type error: value '{}' does not match column '{}' type {:?}",
                            val, col.name, col.col_type
                        );
                        return;
                    }
                }
            }
        }

        // Collect rows to update and create simulated state
        let to_update: Vec<usize> = self
            .rows
            .iter()
            .enumerate()
            .filter_map(|(i, row)| {
                if predicate(row.get_values()) {
                    Some(i)
                } else {
                    None
                }
            })
            .collect();

        let mut simulated = self.rows.clone();

        // Create updated rows by merging existing values with set_values
        for &row_idx in &to_update {
            let mut new_values = simulated[row_idx].get_values().clone();
            for (i, val) in set_values.iter().enumerate() {
                if i < new_values.len() && !val.is_empty() {
                    new_values[i] = val.clone();
                }
            }
            simulated[row_idx].set_values(new_values);
        }

        // Check constraints on simulated state
        let pk_idx = self
            .primary_key
            .as_ref()
            .and_then(|pk| self.schema.columns.iter().position(|c| &c.name == pk));

        // Check primary key constraint
        if let Some(idx) = pk_idx {
            let mut seen = std::collections::HashSet::new();
            for row in &simulated {
                if let Some(val) = row.get_values().get(idx) {
                    if !seen.insert(val) {
                        println!(
                            "Primary key constraint violation on update: '{}' must be unique!",
                            self.schema.columns[idx].name
                        );
                        return;
                    }
                }
            }
        }

        // Check unique constraints
        for uniq_col in &self.unique_columns {
            if let Some(uniq_idx) = self.schema.columns.iter().position(|c| &c.name == uniq_col) {
                let mut seen = std::collections::HashSet::new();
                for row in &simulated {
                    if let Some(val) = row.get_values().get(uniq_idx) {
                        if !seen.insert(val) {
                            println!(
                                "Unique constraint violation on update: '{}' must be unique!",
                                uniq_col
                            );
                            return;
                        }
                    }
                }
            }
        }

        // All checks passed, apply updates
        for &row_idx in &to_update {
            let mut new_values = self.rows[row_idx].get_values().clone();
            for (i, val) in set_values.iter().enumerate() {
                if i < new_values.len() && !val.is_empty() {
                    new_values[i] = val.clone();
                }
            }
            self.rows[row_idx].set_values(new_values);
        }
    }

    /// Deletes all rows matching the predicate.
    fn delete_rows<F>(&mut self, predicate: F)
    where
        F: Fn(&Vec<String>) -> bool,
    {
        self.rows.retain(|row| !predicate(row.get_values()));
    }

    /// Selects and returns all rows matching the predicate.
    fn select_rows<F>(&self, predicate: F) -> Vec<Vec<String>>
    where
        F: Fn(&Vec<String>) -> bool,
    {
        self.rows
            .iter()
            .filter(|row| predicate(row.get_values()))
            .map(|row| row.get_values().clone())
            .collect()
    }
}

// tests moved to integration tests in tests/
