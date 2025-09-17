// query.rs

/// Parses a simple query string (e.g., 'id == 1') into a predicate closure.
/// Supports ==, !=, >, <, >=, <= for a single column.
/// Assumes all values are strings except for >, <, >=, <= which are parsed as numbers.
pub fn query_to_predicate(columns: &[String], query: &str) -> Box<dyn Fn(&Vec<String>) -> bool> {
    let query = query.trim();
    let ops = ["==", "!=", ">=", "<=", ">", "<"];
    let mut op_found = None;
    for op in &ops {
        if let Some(idx) = query.find(op) {
            op_found = Some((op, idx));
            break;
        }
    }
    if let Some((op, idx)) = op_found {
        let col = query[..idx].trim();
        let val = query[idx + op.len()..].trim().trim_matches('"');
        let col_idx = columns.iter().position(|c| c == col);
        if let Some(i) = col_idx {
            match *op {
                "==" => {
                    let val = val.to_string();
                    Box::new(move |row: &Vec<String>| row.get(i).map_or(false, |v| v == &val))
                }
                "!=" => {
                    let val = val.to_string();
                    Box::new(move |row: &Vec<String>| row.get(i).map_or(false, |v| v != &val))
                }
                ">" => {
                    let val = val.parse::<f64>().unwrap_or(f64::NAN);
                    Box::new(move |row: &Vec<String>| row.get(i).and_then(|v| v.parse::<f64>().ok()).map_or(false, |v| v > val))
                }
                "<" => {
                    let val = val.parse::<f64>().unwrap_or(f64::NAN);
                    Box::new(move |row: &Vec<String>| row.get(i).and_then(|v| v.parse::<f64>().ok()).map_or(false, |v| v < val))
                }
                ">=" => {
                    let val = val.parse::<f64>().unwrap_or(f64::NAN);
                    Box::new(move |row: &Vec<String>| row.get(i).and_then(|v| v.parse::<f64>().ok()).map_or(false, |v| v >= val))
                }
                "<=" => {
                    let val = val.parse::<f64>().unwrap_or(f64::NAN);
                    Box::new(move |row: &Vec<String>| row.get(i).and_then(|v| v.parse::<f64>().ok()).map_or(false, |v| v <= val))
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
