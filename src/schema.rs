use serde::{Deserialize, Serialize};

/// Represents the type of a column in a table schema.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ColumnType {
    Int,
    String,
    Float,
}

/// Represents a column in a schema (name and type).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ColumnSchema {
    pub name: String,
    pub col_type: ColumnType,
}

/// Represents the schema of a table (list of columns).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Schema {
    pub columns: Vec<ColumnSchema>,
}
