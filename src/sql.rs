fn parse_create_table(sql: &str) -> (String, Vec<String>, Option<String>, Vec<String>) {
    // Example: CREATE TABLE Users (id PRIMARY KEY, name, email UNIQUE, age)
    let sql = sql.trim_end_matches(';');
    let upper = sql.to_uppercase();
    let mut table = String::new();
    let mut columns = vec![];
    let mut primary_key = None;
    let mut unique_columns = vec![];
    if let Some(table_idx) = upper.find("TABLE ") {
        let after_table = &sql[table_idx + 6..];
        if let Some(paren_idx) = after_table.find('(') {
            table = after_table[..paren_idx].trim().to_string();
            if let Some(end_paren_idx) = after_table.find(')') {
                let cols_str = &after_table[paren_idx + 1..end_paren_idx];
                for col_def in cols_str.split(',') {
                    let col_def = col_def.trim();
                    let parts: Vec<&str> = col_def.split_whitespace().collect();
                    if !parts.is_empty() {
                        let col_name = parts[0].to_string();
                        columns.push(col_name.clone());
                        if parts.len() > 1 {
                            if parts[1].to_uppercase() == "PRIMARY" && parts.get(2).map(|s| s.to_uppercase()) == Some("KEY".to_string()) {
                                primary_key = Some(col_name.clone());
                            } else if parts[1].to_uppercase() == "UNIQUE" {
                                unique_columns.push(col_name.clone());
                            }
                        }
                    }
                }
            }
        }
    }
    (table, columns, primary_key, unique_columns)
}
// sql.rs
// Minimal SQL-like query parser and dispatcher for CRUD operations

use crate::database::{Database, DatabaseInterface};
use crate::query::query_to_predicate;

/// Dispatches a SQL-like query string to the appropriate database operation.
pub fn execute_sql(db: &mut Database, sql: &str) {
    let sql = sql.trim();
    if sql.to_uppercase().starts_with("CREATE TABLE") {
        // Example: CREATE TABLE Users (id PRIMARY KEY, name, email UNIQUE, age)
        let (table, columns, primary_key, unique_columns) = parse_create_table(sql);
        db.create_table_with_constraints(&table, columns, primary_key, unique_columns);
    } else if sql.to_uppercase().starts_with("SELECT") {
        // Example: SELECT * FROM Users WHERE age > 25
        let (columns, table, where_clause) = parse_select(sql);
        let table_columns = db.get_table_columns(&table);
        let selected_columns = if columns == ["*"] {
            table_columns.clone()
        } else {
            columns
        };
        let pred = query_to_predicate(&table_columns, &where_clause);
        db.select(&table, selected_columns, pred);
    } else if sql.to_uppercase().starts_with("INSERT") {
        // Example: INSERT INTO Users (id, name, age) VALUES (3, 'Carol', 22)
        let (table, values) = parse_insert(sql);
        db.insert(&table, values);
    } else if sql.to_uppercase().starts_with("UPDATE") {
        // Example: UPDATE Users SET age = 40 WHERE id == 2
        let (table, set_values, where_clause) = parse_update(sql);
        let table_columns = db.get_table_columns(&table);
        let pred = query_to_predicate(&table_columns, &where_clause);
        db.update(&table, set_values, pred);
    } else if sql.to_uppercase().starts_with("DELETE") {
        // Example: DELETE FROM Users WHERE id == 2
        let (table, where_clause) = parse_delete(sql);
        let table_columns = db.get_table_columns(&table);
        let pred = query_to_predicate(&table_columns, &where_clause);
        db.delete(&table, pred);
    } else {
        println!("Unsupported SQL operation.");
    }
}

// Helper functions for parsing SQL-like queries (very basic, not robust)
fn parse_select(sql: &str) -> (Vec<String>, String, String) {
    // SELECT col1, col2 FROM table WHERE condition
    let mut columns = vec![];
    let mut table = String::new();
    let mut where_clause = String::new();
    let sql = sql.trim_end_matches(';');
    let upper = sql.to_uppercase();
    if let Some(select_idx) = upper.find("SELECT ") {
        if let Some(from_idx) = upper.find(" FROM ") {
            let cols = &sql[select_idx + 7..from_idx];
            columns = cols.split(',').map(|s| s.trim().to_string()).collect();
            let after_from = &sql[from_idx + 6..];
            if let Some(where_idx) = after_from.to_uppercase().find(" WHERE ") {
                table = after_from[..where_idx].trim().to_string();
                where_clause = after_from[where_idx + 7..].trim().to_string();
            } else {
                table = after_from.trim().to_string();
            }
        }
    }
    (columns, table, where_clause)
}

fn parse_insert(sql: &str) -> (String, Vec<String>) {
    // INSERT INTO table (col1, col2) VALUES (val1, val2)
    let sql = sql.trim_end_matches(';');
    let upper = sql.to_uppercase();
    let mut table = String::new();
    let mut values = vec![];
    if let Some(into_idx) = upper.find("INTO ") {
        let after_into = &sql[into_idx + 5..];
        if let Some(paren_idx) = after_into.find('(') {
            table = after_into[..paren_idx].trim().to_string();
            if let Some(vals_idx) = upper.find("VALUES (") {
                let vals_start = vals_idx + 8;
                let vals_end = sql[vals_start..].find(')').map(|i| vals_start + i).unwrap_or(sql.len());
                let vals_str = &sql[vals_start..vals_end];
                values = vals_str.split(',').map(|s| s.trim().trim_matches('"').trim_matches('\'')).map(|s| s.to_string()).collect();
            }
        }
    }
    (table, values)
}

fn parse_update(sql: &str) -> (String, Vec<String>, String) {
    // UPDATE table SET col1 = val1, col2 = val2 WHERE condition
    let sql = sql.trim_end_matches(';');
    let upper = sql.to_uppercase();
    let mut table = String::new();
    let mut set_values = vec![];
    let mut where_clause = String::new();
    if let Some(update_idx) = upper.find("UPDATE ") {
        let after_update = &sql[update_idx + 7..];
        if let Some(set_idx) = after_update.to_uppercase().find(" SET ") {
            table = after_update[..set_idx].trim().to_string();
            let after_set = &after_update[set_idx + 5..];
            if let Some(where_idx) = after_set.to_uppercase().find(" WHERE ") {
                let set_str = &after_set[..where_idx];
                set_values = set_str.split(',').map(|s| s.split('=').nth(1).unwrap_or("").trim().trim_matches('"').trim_matches('\'')).map(|s| s.to_string()).collect();
                where_clause = after_set[where_idx + 7..].trim().to_string();
            } else {
                set_values = after_set.split(',').map(|s| s.split('=').nth(1).unwrap_or("").trim().trim_matches('"').trim_matches('\'')).map(|s| s.to_string()).collect();
            }
        }
    }
    (table, set_values, where_clause)
}

fn parse_delete(sql: &str) -> (String, String) {
    // DELETE FROM table WHERE condition
    let sql = sql.trim_end_matches(';');
    let upper = sql.to_uppercase();
    let mut table = String::new();
    let mut where_clause = String::new();
    if let Some(from_idx) = upper.find("FROM ") {
        let after_from = &sql[from_idx + 5..];
        if let Some(where_idx) = after_from.to_uppercase().find(" WHERE ") {
            table = after_from[..where_idx].trim().to_string();
            where_clause = after_from[where_idx + 7..].trim().to_string();
        } else {
            table = after_from.trim().to_string();
        }
    }
    (table, where_clause)
}

