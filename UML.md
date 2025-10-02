# UML Diagram for Rust In-Memory Database

Below is a simple UML class diagram representing the main components of the project (module names reflect the `src/` layout):

```mermaid
classDiagram
    class Database {
        +tables: HashMap<String, Table>
        +create_table_with_constraints(name, cols, pk, uniques)
        +insert(name, values)
        +select(name, columns, predicate)
        +update(name, set_values, predicate)
        +delete(name, predicate)
        +save_to_file(path)
        +load_from_file(path)
    }
    class Table {
        +name: String
        +schema: Schema
        +rows: Vec<Row>
        +primary_key: Option<String>
        +unique_columns: Vec<String>
        +add_row(values)
        +update_rows(set_values, predicate)
        +delete_rows(predicate)
        +select_rows(predicate)
    }
    class Row {
        +values: Vec<String>
    }
    class SQLModule {
        +execute_sql(db, sql)
        -parse_create_table(sql)
        -parse_select(sql)
        -parse_insert(sql)
        -parse_update(sql)
        -parse_delete(sql)
    }
    class Query {
        +query_to_predicate(columns, query)
    }

    Database "1" -- "*" Table
    Table "1" -- "*" Row
    Database ..> SQLModule : uses
    SQLModule ..> Query : uses
```

- `Database` manages multiple `Table`s and exposes a public interface in `src/database.rs`.
- `Table` contains schema information (`Schema`, `ColumnSchema`, `ColumnType`) and enforces constraints.
- `Row` stores values as strings (the typed behavior is enforced at Table operations).
- `SQLModule` (`src/sql.rs`) contains the public `execute_sql` dispatcher; its parsing helpers are internal.
- `Query` (`src/query.rs`) builds typed predicates for WHERE clauses.

