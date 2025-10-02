use crate::schema::Schema;
#[allow(dead_code)]
impl Row {
    /// Get a reference to a value by column name using the schema.
    pub fn get_by_name<'a>(&'a self, col_name: &str, schema: &Schema) -> Option<&'a String> {
        schema
            .columns
            .iter()
            .position(|c| c.name == col_name)
            .and_then(|idx| self.values.get(idx))
    }

    /// Set a value by column name using the schema.
    pub fn set_by_name(&mut self, col_name: &str, value: String, schema: &Schema) -> bool {
        if let Some(idx) = schema.columns.iter().position(|c| c.name == col_name) {
            if idx < self.values.len() {
                self.values[idx] = value;
                return true;
            }
        }
        false
    }
}
use serde::{Deserialize, Serialize};
// row.rs

/// Trait defining the interface for a row in a table.
/// Provides methods to get and set the values of the row.
pub trait RowInterface {
    /// Returns a reference to the values stored in the row.
    fn get_values(&self) -> &Vec<String>;
    /// Sets the values of the row.
    fn set_values(&mut self, values: Vec<String>);
}

/// Struct representing a single row in a table.
/// Stores the values for each column as a vector of strings.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Row {
    values: Vec<String>,
}

impl Row {
    /// Creates a new Row with the given values.
    pub fn new(values: Vec<String>) -> Self {
        Row { values }
    }
}

/// Implements the RowInterface trait for the Row struct.
impl RowInterface for Row {
    /// Returns a reference to the values stored in the row.
    fn get_values(&self) -> &Vec<String> {
        &self.values
    }

    /// Sets the values of the row.
    fn set_values(&mut self, values: Vec<String>) {
        self.values = values;
    }
}

// tests moved to integration tests in tests/
