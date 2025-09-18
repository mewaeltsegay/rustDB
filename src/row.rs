use serde::{Serialize, Deserialize};
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
#[derive(Serialize, Deserialize, Clone)]
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
