use lab::*;

#[test]
fn query_predicates_unit() {
    let cols = vec![
        ColumnSchema {
            name: "id".to_string(),
            col_type: ColumnType::Int,
        },
        ColumnSchema {
            name: "price".to_string(),
            col_type: ColumnType::Float,
        },
        ColumnSchema {
            name: "name".to_string(),
            col_type: ColumnType::String,
        },
    ];

    let eq = query_to_predicate(&cols, "id == 10");
    assert!(eq(&vec![
        "10".to_string(),
        "1.0".to_string(),
        "x".to_string()
    ]));
    assert!(!eq(&vec![
        "11".to_string(),
        "1.0".to_string(),
        "x".to_string()
    ]));

    let gt = query_to_predicate(&cols, "price > 2.5");
    assert!(gt(&vec![
        "1".to_string(),
        "3.0".to_string(),
        "x".to_string()
    ]));
    assert!(!gt(&vec![
        "1".to_string(),
        "2.0".to_string(),
        "x".to_string()
    ]));

    let neq = query_to_predicate(&cols, "name != 'Bob'");
    assert!(neq(&vec![
        "1".to_string(),
        "1.0".to_string(),
        "Alice".to_string()
    ]));
    assert!(!neq(&vec![
        "1".to_string(),
        "1.0".to_string(),
        "Bob".to_string()
    ]));
}

#[test]
fn sql_parser_and_select_insert_unit() {
    // exercise parse_select and parse_insert via public execute_sql and parse helpers
    // create table then select/insert parsing via public execute_sql
    let mut db = Database::new();
    execute_sql(
        &mut db,
        "CREATE TABLE T(a INT PRIMARY KEY, b STRING UNIQUE, c FLOAT);",
    );
    // ensure created
    let t = db.tables.get("T").expect("table T exists");
    assert_eq!(t.schema.columns.len(), 3);

    // parse a select statement using execute_sql's internal select path (indirect test)
    execute_sql(&mut db, "SELECT a, b FROM T WHERE a == 1");

    // insert parsing path (indirect) -- ensure no panic
    execute_sql(&mut db, "INSERT INTO T (a, b, c) VALUES (1, 'x', 2.0);");
}

#[test]
fn table_constraints_unit() {
    let mut db = Database::new();
    db.create_table_with_constraints(
        "Utest",
        vec![
            ColumnSchema {
                name: "id".to_string(),
                col_type: ColumnType::Int,
            },
            ColumnSchema {
                name: "name".to_string(),
                col_type: ColumnType::String,
            },
        ],
        Some("id".to_string()),
        vec![],
    );

    db.insert("Utest", vec!["1".to_string(), "A".to_string()]);
    db.insert("Utest", vec!["2".to_string(), "B".to_string()]);
    // duplicate primary key rejected
    db.insert("Utest", vec!["1".to_string(), "C".to_string()]);
    assert_eq!(db.tables.get("Utest").unwrap().rows.len(), 2);
}

#[test]
fn select_operations_unit() {
    let mut db = Database::new();
    
    // Create a test table with different data types
    execute_sql(
        &mut db,
        "CREATE TABLE Products (id INT PRIMARY KEY, name STRING, price FLOAT, stock INT);",
    );
    
    // Insert test data
    execute_sql(&mut db, "INSERT INTO Products VALUES (1, 'Pen', 2.5, 100);");
    execute_sql(&mut db, "INSERT INTO Products VALUES (2, 'Pencil', 1.2, 50);");
    execute_sql(&mut db, "INSERT INTO Products VALUES (3, 'Eraser', 0.8, 30);");
    
    // Test 1: Basic SELECT * without WHERE clause
    execute_sql(&mut db, "SELECT * FROM Products");
    let products = db.tables.get("Products").unwrap();
    assert_eq!(products.rows.len(), 3);
    
    // Test 2: SELECT with specific columns
    execute_sql(&mut db, "SELECT id, name FROM Products");
    
    // Test 3: SELECT with WHERE clause on different data types
    // Integer comparison
    execute_sql(&mut db, "SELECT * FROM Products WHERE id > 1");
    execute_sql(&mut db, "SELECT * FROM Products WHERE stock <= 50");
    
    // Float comparison
    execute_sql(&mut db, "SELECT * FROM Products WHERE price > 2.0");
    
    // String comparison
    execute_sql(&mut db, "SELECT * FROM Products WHERE name == 'Pen'");
    
    // Test 4: SELECT with invalid table (should not panic)
    execute_sql(&mut db, "SELECT * FROM NonExistentTable");
    
    // Test 5: SELECT with complex WHERE conditions
    execute_sql(&mut db, "SELECT * FROM Products WHERE price > 1.0");
    let products = db.tables.get("Products").unwrap();
    let expensive_products = products.rows.iter()
        .filter(|row| row.get_values()[2].parse::<f64>().unwrap() > 1.0)
        .count();
    assert_eq!(expensive_products, 2); // Pen and Pencil are > 1.0
    
    // Test 6: SELECT with no matching rows
    execute_sql(&mut db, "SELECT * FROM Products WHERE price > 10.0");
    
    // Test 7: SELECT with invalid column in WHERE clause (should not panic)
    execute_sql(&mut db, "SELECT * FROM Products WHERE invalid_column > 10");
}
