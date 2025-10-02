use lab::*;

#[test]
fn integration_predicate_and_row_and_table_and_db() {
    // Query predicate
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

    let pred_price = query_to_predicate(&cols, "price > 1.5");
    let row1 = vec!["1".to_string(), "2.5".to_string(), "Pen".to_string()];
    let row2 = vec!["2".to_string(), "1.0".to_string(), "Pencil".to_string()];
    assert!(pred_price(&row1));
    assert!(!pred_price(&row2));

    let pred_name = query_to_predicate(&cols, "name == 'Pen'");
    assert!(pred_name(&row1));
    assert!(!pred_name(&row2));

    // Row get/set
    let schema = Schema {
        columns: vec![
            ColumnSchema {
                name: "id".to_string(),
                col_type: ColumnType::Int,
            },
            ColumnSchema {
                name: "name".to_string(),
                col_type: ColumnType::String,
            },
        ],
    };
    let mut r = Row::new(vec!["1".to_string(), "Alice".to_string()]);
    assert_eq!(r.get_by_name("id", &schema), Some(&"1".to_string()));
    assert!(r.set_by_name("name", "Bob".to_string(), &schema));
    assert_eq!(r.get_by_name("name", &schema), Some(&"Bob".to_string()));

    // Table CRUD
    let mut t = Table::new(
        "Test".to_string(),
        schema.clone(),
        Some("id".to_string()),
        vec![],
    );
    t.add_row(vec!["1".to_string(), "Alice".to_string()]);
    t.add_row(vec!["2".to_string(), "Bob".to_string()]);
    // duplicate pk should be rejected
    t.add_row(vec!["1".to_string(), "Carol".to_string()]);
    assert_eq!(t.rows.len(), 2);
    t.update_rows(vec!["".to_string(), "Bobby".to_string()], |r| {
        r.get(0).map(|v| v == "2").unwrap_or(false)
    });
    assert_eq!(t.rows[1].get_values()[1], "Bobby");
    t.delete_rows(|r| r.get(0).map(|v| v == "1").unwrap_or(false));
    assert_eq!(t.rows.len(), 1);

    // Database level
    let mut db = Database::new();
    let cols_db = vec![
        ColumnSchema {
            name: "id".to_string(),
            col_type: ColumnType::Int,
        },
        ColumnSchema {
            name: "name".to_string(),
            col_type: ColumnType::String,
        },
    ];
    db.create_table_with_constraints("People", cols_db, Some("id".to_string()), vec![]);
    db.insert("People", vec!["1".to_string(), "Alice".to_string()]);
    db.insert("People", vec!["2".to_string(), "Bob".to_string()]);
    assert!(db.tables.get("People").map(|t| t.rows.len()).unwrap_or(0) == 2);
    db.update("People", vec!["".to_string(), "Bobby".to_string()], |r| {
        r.get(0).map(|v| v == "2").unwrap_or(false)
    });
    assert_eq!(
        db.tables.get("People").unwrap().rows[1].get_values()[1],
        "Bobby"
    );
    db.delete("People", |r| r.get(0).map(|v| v == "1").unwrap_or(false));
    assert_eq!(db.tables.get("People").unwrap().rows.len(), 1);

    // Use a temp file for save/load
    let tmp = tempfile::NamedTempFile::new().expect("tempfile");
    let fname = tmp.path().to_str().unwrap().to_string();
    let _ = db.save_to_file(&fname);
    let loaded = Database::load_from_file(&fname).unwrap();
    assert!(loaded.tables.get("People").is_some());
    drop(tmp);
}

#[test]
fn parse_create_and_predicate_tests() {
    // create table via execute_sql and inspect schema
    let mut db = Database::new();
    execute_sql(
        &mut db,
        "CREATE TABLE Users(id INT PRIMARY KEY, name STRING, score FLOAT, email UNIQUE)",
    );
    let table_ref = db.tables.get("Users").expect("Users table created");
    assert_eq!(table_ref.schema.columns.len(), 4);
    assert_eq!(table_ref.schema.columns[0].name, "id");
    assert_eq!(table_ref.schema.columns[0].col_type, ColumnType::Int);
    assert_eq!(table_ref.schema.columns[1].name, "name");
    assert_eq!(table_ref.schema.columns[1].col_type, ColumnType::String);
    assert_eq!(table_ref.schema.columns[2].name, "score");
    assert_eq!(table_ref.schema.columns[2].col_type, ColumnType::Float);

    // predicate
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
    let pred_price = query_to_predicate(&cols, "price > 1.5");
    let row1 = vec!["1".to_string(), "2.5".to_string(), "Pen".to_string()];
    let row2 = vec!["2".to_string(), "1.0".to_string(), "Pencil".to_string()];
    assert!(pred_price(&row1));
    assert!(!pred_price(&row2));
}

#[test]
fn create_table_type_synonyms_and_list_tables() {
    let mut db = Database::new();
    // Use different type tokens to ensure parser maps them correctly
    execute_sql(
        &mut db,
        "CREATE TABLE TypesTest(a INTEGER, b REAL, c TEXT);",
    );
    let t = db.tables.get("TypesTest").expect("TypesTest created");
    assert_eq!(t.schema.columns.len(), 3);
    assert_eq!(t.schema.columns[0].col_type, ColumnType::Int);
    assert_eq!(t.schema.columns[1].col_type, ColumnType::Float);
    assert_eq!(t.schema.columns[2].col_type, ColumnType::String);

    // LIST TABLES should not panic and should include our tables
    execute_sql(&mut db, "LIST TABLES");
    assert!(db.tables.contains_key("TypesTest"));
}

#[test]
fn insert_and_update_type_mismatch_handling() {
    let mut db = Database::new();
    // create table with INT and FLOAT columns
    db.create_table_with_constraints(
        "Nums",
        vec![
            ColumnSchema {
                name: "id".to_string(),
                col_type: ColumnType::Int,
            },
            ColumnSchema {
                name: "val".to_string(),
                col_type: ColumnType::Float,
            },
        ],
        Some("id".to_string()),
        vec![],
    );

    // inserting wrong type into id should be rejected
    db.insert("Nums", vec!["notanint".to_string(), "1.23".to_string()]);
    assert_eq!(db.tables.get("Nums").unwrap().rows.len(), 0);

    // insert a correct row
    db.insert("Nums", vec!["1".to_string(), "2.5".to_string()]);
    assert_eq!(db.tables.get("Nums").unwrap().rows.len(), 1);

    // attempt an update that provides an invalid float for 'val' should be rejected
    db.update("Nums", vec!["".to_string(), "notafloat".to_string()], |r| {
        r.get(0).map(|v| v == "1").unwrap_or(false)
    });
    // value should remain unchanged
    let val = db.tables.get("Nums").unwrap().rows[0].get_values()[1].clone();
    assert_eq!(val, "2.5");
}

#[test]
fn predicate_comparison_edge_cases() {
    let cols = vec![ColumnSchema {
        name: "n".to_string(),
        col_type: ColumnType::Float,
    }];
    let p_ge = query_to_predicate(&cols, "n >= 2.5");
    assert!(p_ge(&vec!["2.5".to_string()]));
    assert!(p_ge(&vec!["3.0".to_string()]));
    assert!(!p_ge(&vec!["2.499".to_string()]));

    let p_le = query_to_predicate(&cols, "n <= 1.0");
    assert!(p_le(&vec!["1.0".to_string()]));
    assert!(p_le(&vec!["0.5".to_string()]));
    assert!(!p_le(&vec!["1.0001".to_string()]));
}
