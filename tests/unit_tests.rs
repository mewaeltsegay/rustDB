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
