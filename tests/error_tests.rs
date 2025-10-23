use lab::*;

#[test]
fn sql_parser_edge_cases() {
    let mut db = Database::new();
    
    // Invalid SQL operations should be ignored
    execute_sql(&mut db, "INVALID SQL");
    assert_eq!(db.tables.len(), 0, "Invalid SQL should not create tables");
    
    // CREATE without proper syntax should be ignored
    execute_sql(&mut db, "CREATE TABLE");
    assert_eq!(db.tables.len(), 0, "CREATE TABLE without name/columns should not create a table");
    
    // Valid table for testing malformed queries
    execute_sql(&mut db, "CREATE TABLE Users(id INT PRIMARY KEY, name STRING);");
    assert!(db.tables.contains_key("Users"), "Valid CREATE TABLE should work");
    
    // Malformed SELECT - these should not panic
    execute_sql(&mut db, "SELECT"); // Missing everything
    execute_sql(&mut db, "SELECT FROM"); // Missing columns and table
    execute_sql(&mut db, "SELECT * FROM"); // Missing table
    execute_sql(&mut db, "SELECT * FROM NonexistentTable"); // Invalid table
    
    // Malformed INSERT - these should not add rows
    execute_sql(&mut db, "INSERT INTO;"); // Missing everything
    execute_sql(&mut db, "INSERT INTO Users;"); // Missing VALUES
    execute_sql(&mut db, "INSERT INTO Users VALUES;"); // Missing values
    execute_sql(&mut db, "INSERT INTO NonexistentTable VALUES (1);"); // Invalid table
    assert_eq!(db.tables.get("Users").unwrap().rows.len(), 0);
    
    // Type mismatches in INSERT
    execute_sql(&mut db, "INSERT INTO Users VALUES (notanint, 'name');"); // Bad int
    assert_eq!(db.tables.get("Users").unwrap().rows.len(), 0);
    
    // Valid insert for update/delete tests
    execute_sql(&mut db, "INSERT INTO Users VALUES (1, 'Alice');");
    assert_eq!(db.tables.get("Users").unwrap().rows.len(), 1);
    
    // Malformed UPDATE - these should not modify rows
    execute_sql(&mut db, "UPDATE;"); // Missing everything
    execute_sql(&mut db, "UPDATE Users;"); // Missing SET
    execute_sql(&mut db, "UPDATE Users SET;"); // Missing assignments
    execute_sql(&mut db, "UPDATE NonexistentTable SET id = 1;"); // Invalid table
    
    // Type mismatches in UPDATE
    execute_sql(&mut db, "UPDATE Users SET id = notanint WHERE id == 1;"); // Bad int
    let first_row = &db.tables.get("Users").unwrap().rows[0];
    assert_eq!(first_row.get_values()[0], "1"); // Value unchanged
}

#[test]
fn constraint_violations() {
    let mut db = Database::new();
    
    // Test UNIQUE constraint on non-PK column
    execute_sql(&mut db, "CREATE TABLE Emails(id INT PRIMARY KEY, email STRING UNIQUE);");
    
    // First insert succeeds
    execute_sql(&mut db, "INSERT INTO Emails VALUES (1, 'a@b.com');");
    let rows = db.tables.get("Emails").unwrap().rows.len();
    assert_eq!(rows, 1, "First insert should succeed");
    
    // Duplicate email fails
    execute_sql(&mut db, "INSERT INTO Emails (id, email) VALUES (2, 'a@b.com');");
    assert_eq!(db.tables.get("Emails").unwrap().rows.len(), 1, "Duplicate email should be rejected");
    
    // Different email succeeds
    execute_sql(&mut db, "INSERT INTO Emails VALUES (2, 'c@d.com');");
    assert_eq!(db.tables.get("Emails").unwrap().rows.len(), 2);
    
    // UPDATE violating UNIQUE constraint
    execute_sql(&mut db, "UPDATE Emails SET email = 'a@b.com' WHERE id == 2;");
    // Should remain unchanged
    assert_eq!(
        db.tables.get("Emails").unwrap().rows[1].get_values()[1],
        "c@d.com"
    );
    
    // Test multiple UNIQUE constraints
    execute_sql(
        &mut db,
        "CREATE TABLE Users(id INT PRIMARY KEY, email STRING UNIQUE, username STRING UNIQUE);"
    );
    
    // First insert succeeds
    execute_sql(&mut db, "INSERT INTO Users VALUES (1, 'a@b.com', 'alice')");
    assert_eq!(db.tables.get("Users").unwrap().rows.len(), 1);
    
    // Duplicate email fails
    execute_sql(&mut db, "INSERT INTO Users VALUES (2, 'a@b.com', 'bob')");
    assert_eq!(db.tables.get("Users").unwrap().rows.len(), 1);
    
    // Duplicate username fails
    execute_sql(&mut db, "INSERT INTO Users VALUES (2, 'c@d.com', 'alice')");
    assert_eq!(db.tables.get("Users").unwrap().rows.len(), 1);
    
    // Unique values succeed
    execute_sql(&mut db, "INSERT INTO Users VALUES (2, 'c@d.com', 'bob')");
    assert_eq!(db.tables.get("Users").unwrap().rows.len(), 2);
}

#[test]
fn save_load_edge_cases() {
    let mut db = Database::new();
    execute_sql(&mut db, "CREATE TABLE Test(id INT PRIMARY KEY, name STRING);");
    
    // Save to invalid path should fail gracefully
    assert!(db.save_to_file("/invalid/path/db.json").is_err());
    
    // Load from nonexistent file should fail gracefully
    assert!(Database::load_from_file("/nonexistent/db.json").is_err());
    
    // Test large dataset persistence
    for i in 0..1000 {
        db.insert(
            "Test",
            vec![i.to_string(), format!("user{}", i)],
        );
    }
    assert_eq!(db.tables.get("Test").unwrap().rows.len(), 1000);
    
    // Save and load large dataset
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let path = tmp.path().to_str().unwrap().to_string();
    
    // Save should succeed
    assert!(db.save_to_file(&path).is_ok());
    
    // Load should restore all data
    let loaded = Database::load_from_file(&path).unwrap();
    assert_eq!(
        loaded.tables.get("Test").unwrap().rows.len(),
        1000
    );
    
    // Verify some random records
    let loaded_table = loaded.tables.get("Test").unwrap();
    for i in [0, 42, 999] {
        let row = &loaded_table.rows[i];
        assert_eq!(row.get_values()[0], i.to_string());
        assert_eq!(row.get_values()[1], format!("user{}", i));
    }
    
    // Clean up
    drop(tmp);
}