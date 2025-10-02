// query.rs

use crate::schema::{ColumnSchema, ColumnType};

/// Parses a simple query string (e.g., 'id == 1') into a predicate closure.
/// Supports ==, !=, >, <, >=, <= for a single column.
/// Uses the provided column schemas to interpret types when comparing.
pub fn query_to_predicate(
    columns: &[ColumnSchema],
    query: &str,
) -> Box<dyn Fn(&Vec<String>) -> bool> {
    let query = query.trim();
    
    // Handle empty query or "true" as always matching
    if query.is_empty() || query == "true" {
        return Box::new(|_| true);
    }
    
    let ops = ["==", "!=", ">=", "<=", ">", "<"];
    let mut op_found: Option<(&str, usize)> = None;
    for op in &ops {
        if let Some(idx) = query.find(op) {
            op_found = Some((op, idx));
            break;
        }
    }
    if let Some((op, idx)) = op_found {
        let col = query[..idx].trim();
        let raw_val = query[idx + op.len()..]
            .trim()
            .trim_matches('"')
            .trim_matches('\'');
        let col_idx = columns.iter().position(|c| c.name == col);
        if let Some(i) = col_idx {
            let col_schema = columns[i].clone();
            match op {
                "==" => match col_schema.col_type {
                    ColumnType::Int => {
                        if let Ok(n) = raw_val.parse::<i64>() {
                            Box::new(move |row: &Vec<String>| {
                                row.get(i)
                                    .and_then(|v| v.parse::<i64>().ok())
                                    .map_or(false, |v| v == n)
                            })
                        } else {
                            Box::new(|_| false)
                        }
                    }
                    ColumnType::Float => {
                        if let Ok(n) = raw_val.parse::<f64>() {
                            Box::new(move |row: &Vec<String>| {
                                row.get(i)
                                    .and_then(|v| v.parse::<f64>().ok())
                                    .map_or(false, |v| v == n)
                            })
                        } else {
                            Box::new(|_| false)
                        }
                    }
                    ColumnType::String => {
                        let val = raw_val.to_string();
                        Box::new(move |row: &Vec<String>| row.get(i).map_or(false, |v| v == &val))
                    }
                },
                "!=" => match col_schema.col_type {
                    ColumnType::Int => {
                        if let Ok(n) = raw_val.parse::<i64>() {
                            Box::new(move |row: &Vec<String>| {
                                row.get(i)
                                    .and_then(|v| v.parse::<i64>().ok())
                                    .map_or(false, |v| v != n)
                            })
                        } else {
                            Box::new(|_| false)
                        }
                    }
                    ColumnType::Float => {
                        if let Ok(n) = raw_val.parse::<f64>() {
                            Box::new(move |row: &Vec<String>| {
                                row.get(i)
                                    .and_then(|v| v.parse::<f64>().ok())
                                    .map_or(false, |v| v != n)
                            })
                        } else {
                            Box::new(|_| false)
                        }
                    }
                    ColumnType::String => {
                        let val = raw_val.to_string();
                        Box::new(move |row: &Vec<String>| row.get(i).map_or(false, |v| v != &val))
                    }
                },
                ">" | "<" | ">=" | "<=" => {
                    // Numeric comparisons: parse both sides as f64
                    if let Ok(n) = raw_val.parse::<f64>() {
                        match op {
                            ">" => Box::new(move |row: &Vec<String>| {
                                row.get(i)
                                    .and_then(|v| v.parse::<f64>().ok())
                                    .map_or(false, |v| v > n)
                            }),
                            "<" => Box::new(move |row: &Vec<String>| {
                                row.get(i)
                                    .and_then(|v| v.parse::<f64>().ok())
                                    .map_or(false, |v| v < n)
                            }),
                            ">=" => Box::new(move |row: &Vec<String>| {
                                row.get(i)
                                    .and_then(|v| v.parse::<f64>().ok())
                                    .map_or(false, |v| v >= n)
                            }),
                            "<=" => Box::new(move |row: &Vec<String>| {
                                row.get(i)
                                    .and_then(|v| v.parse::<f64>().ok())
                                    .map_or(false, |v| v <= n)
                            }),
                            _ => Box::new(|_| false),
                        }
                    } else {
                        Box::new(|_| false)
                    }
                }
                _ => Box::new(|_| false),
            }
        } else {
            // Column not found
            Box::new(|_| false)
        }
    } else {
        // No operator found
        Box::new(|_| false)
    }
}

// tests moved to tests/integration_tests.rs
