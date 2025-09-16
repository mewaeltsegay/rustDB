// database.rs
pub trait DatabaseInterface {
    fn create_table(&mut self, table_name: &str, columns: Vec<String>);
    fn insert(&mut self, table_name: &str, values: Vec<String>);
    fn update(&mut self, table_name: &str, set_values: Vec<String>, condition: &str);
    fn delete(&mut self, table_name: &str, condition: &str);
    fn select(&self, table_name: &str, columns: Vec<String>, condition: &str);
}

use std::collections::HashMap;
use crate::table::{Table, TableInterface};


pub struct Database {
    tables: HashMap<String, Box<dyn TableInterface>>,
}

impl Database {
    pub fn new() -> Self {
        Database {
            tables: HashMap::new(),
        }
    }
}

impl DatabaseInterface for Database {
    fn create_table(&mut self, table_name: &str, columns: Vec<String>) {
        let table = Box::new(Table::new(table_name.to_string(), columns));
        self.tables.insert(table_name.to_string(), table);
        println!("Created table: {}", table_name);
    }

    fn insert(&mut self, table_name: &str, values: Vec<String>) {
        if let Some(table) = self.tables.get_mut(table_name) {
            table.add_row(values);
            println!("Inserted values into table: {}", table_name);
        } else {
            println!("Table not found: {}", table_name);
        }
    }

    fn update(&mut self, table_name: &str, set_values: Vec<String>, condition: &str) {
        let _ = condition;
        if let Some(table) = self.tables.get_mut(table_name) {
            // For simplicity, we update the first row in the table
            table.update_row(0, set_values);
            println!("Updated table: {}", table_name);
        } else {
            println!("Table not found: {}", table_name);
        }
    }

    fn delete(&mut self, table_name: &str, condition: &str) {
        let _ = condition;
        if let Some(table) = self.tables.get_mut(table_name) {
            // For simplicity, we delete the first row in the table
            table.delete_row(0);
            println!("Deleted row in table: {}", table_name);
        } else {
            println!("Table not found: {}", table_name);
        }
    }

    fn select(&self, table_name: &str, columns: Vec<String>, condition: &str) {
        let _ = columns;
        let _ = condition;
        if let Some(table) = self.tables.get(table_name) {
            println!("Selecting from table: {}", table_name);
            // For simplicity, we select only the first row
            if let Some(row) = table.select_row(0) {
                for value in row {
                    print!("{} ", value);
                }
                println!();
            }
        } else {
            println!("Table not found: {}", table_name);
        }
    }
}
