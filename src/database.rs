use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::collections::HashMap;

use crate::row::RowInterface;
use crate::schema::{ColumnSchema, Schema};
use crate::table::{Table, TableInterface};

pub trait DatabaseInterface {
    fn create_table_with_constraints(
        &mut self,
        table_name: &str,
        columns: Vec<ColumnSchema>,
        primary_key: Option<String>,
        unique_columns: Vec<String>,
    );
    fn create_table(&mut self, table_name: &str, columns: Vec<ColumnSchema>);
    fn list_tables(&self, tables: &Vec<String>);
    fn insert(&mut self, table_name: &str, values: Vec<String>);
    fn update<F>(&mut self, table_name: &str, set_values: Vec<String>, predicate: F)
    where
        F: Fn(&Vec<String>) -> bool;
    fn delete<F>(&mut self, table_name: &str, predicate: F)
    where
        F: Fn(&Vec<String>) -> bool;
    fn select<F>(&self, table_name: &str, columns: Vec<String>, predicate: F) -> Vec<Vec<String>>
    where
        F: Fn(&Vec<String>) -> bool;
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Database {
    pub tables: HashMap<String, Table>,
}

impl Database {
    pub fn new() -> Self {
        Database {
            tables: HashMap::new(),
        }
    }

    pub fn get_table_columns(&self, table_name: &str) -> Vec<String> {
        if let Some(table) = self.tables.get(table_name) {
            table
                .schema
                .columns
                .iter()
                .map(|c| c.name.clone())
                .collect()
        } else {
            vec![]
        }
    }

    pub fn save_to_file(&self, path: &str) -> std::io::Result<()> {
        let path = std::path::Path::new(path);
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Parent directory does not exist",
                ));
            }
        }

        let json = serde_json::to_string_pretty(self).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
        })?;

        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    pub fn load_from_file(path: &str) -> std::io::Result<Self> {
        let path = std::path::Path::new(path);
        if !path.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "File does not exist",
            ));
        }

        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        serde_json::from_str(&contents).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
        })
    }
}

impl DatabaseInterface for Database {
    fn create_table_with_constraints(
        &mut self,
        table_name: &str,
        columns: Vec<ColumnSchema>,
        primary_key: Option<String>,
        unique_columns: Vec<String>,
    ) {
        let schema = Schema { columns };
        let table = Table::new(table_name.to_string(), schema, primary_key, unique_columns);
        self.tables.insert(table_name.to_string(), table);
        println!("Created table: {}", table_name);
    }

    fn create_table(&mut self, table_name: &str, columns: Vec<ColumnSchema>) {
        let schema = Schema { columns };
        let table = Table::new(table_name.to_string(), schema, None, vec![]);
        self.tables.insert(table_name.to_string(), table);
        println!("Created table: {}", table_name);
    }

    fn list_tables(&self, tables: &Vec<String>) {
        println!("Tables in the database:");
        println!("{:-<20}-", "");

        for table_name in tables {
            println!("{:<20} |", table_name);
            println!("{:-<20}-", "");
        }
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
    where
        F: Fn(&Vec<String>) -> bool,
    {
        if let Some(table) = self.tables.get_mut(table_name) {
            table.update_rows(set_values, predicate);
            println!("Updated matching rows in table: {}", table_name);
        } else {
            println!("Table not found: {}", table_name);
        }
    }

    fn delete<F>(&mut self, table_name: &str, predicate: F)
    where
        F: Fn(&Vec<String>) -> bool,
    {
        if let Some(table) = self.tables.get_mut(table_name) {
            table.delete_rows(predicate);
            println!("Deleted matching rows in table: {}", table_name);
        } else {
            println!("Table not found: {}", table_name);
        }
    }

    fn select<F>(&self, table_name: &str, columns: Vec<String>, _predicate: F) -> Vec<Vec<String>>
    where
        F: Fn(&Vec<String>) -> bool,
    {
        if let Some(table) = self.tables.get(table_name) {
            println!("Selecting from table: {}", table_name);
            let col_names: Vec<_> = if columns == vec!["*".to_string()] {
                table
                    .schema
                    .columns
                    .iter()
                    .map(|c| c.name.clone())
                    .collect()
            } else {
                columns.clone()
            };

            let col_widths: Vec<_> = col_names
                .iter()
                .map(|name| {
                    let max_val = table
                        .rows
                        .iter()
                        .map(|row| {
                            row.get_by_name(name, &table.schema)
                                .map(|v| v.len())
                                .unwrap_or(0)
                        })
                        .max()
                        .unwrap_or(0);
                    std::cmp::max(name.len(), max_val)
                })
                .collect();

            for (h, w) in col_names.iter().zip(&col_widths) {
                print!("{:<width$} ", h, width = w);
            }
            println!();
            for w in &col_widths {
                print!("{:-<width$}-", "", width = *w);
            }
            println!();

            let mut results: Vec<Vec<String>> = Vec::new();
            for row in &table.rows {
                if _predicate(row.get_values()) {
                    let mut out_row: Vec<String> = Vec::new();
                    for (i, col) in col_names.iter().enumerate() {
                        let val = row
                            .get_by_name(col, &table.schema)
                            .map(|s| s.clone())
                            .unwrap_or_default();
                        out_row.push(val.clone());
                        print!("{:<width$} ", val, width = col_widths[i]);
                    }
                    println!();
                    results.push(out_row);
                }
            }
            results
        } else {
            println!("Table not found: {}", table_name);
            Vec::new()
        }
    }
}
