use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Mutex;
use rusqlite::{Connection, params};

use crate::errors::runtime::RuntimeError;
use crate::runtime::interpreter::{Interpreter, JsExpr, StoredFunction};
use crate::runtime::value::{FunctionValue, Value};

lazy_static::lazy_static! {
    static ref DB_STORE: Mutex<HashMap<String, Value>> = Mutex::new(HashMap::new());
    static ref SQLITE_CONNECTION: Mutex<Option<Connection>> = Mutex::new(None);
}

/// Initialize SQLite database connection
pub fn init_sqlite(path: &str) -> Result<(), RuntimeError> {
    let conn = Connection::open(path)
        .map_err(|e| RuntimeError::ValueError(format!("Failed to open database: {}", e)))?;

    // Create default tables for testing
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            email TEXT
        )",
        [],
    ).map_err(|e| RuntimeError::ValueError(format!("Failed to create table: {}", e)))?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS test (
            id INTEGER PRIMARY KEY,
            value TEXT
        )",
        [],
    ).map_err(|e| RuntimeError::ValueError(format!("Failed to create table: {}", e)))?;

    let mut db = SQLITE_CONNECTION.lock().unwrap();
    *db = Some(conn);
    Ok(())
}

/// Initialize in-memory SQLite database
pub fn init_sqlite_memory() -> Result<(), RuntimeError> {
    let conn = Connection::open_in_memory()
        .map_err(|e| RuntimeError::ValueError(format!("Failed to open in-memory database: {}", e)))?;

    // Create default tables
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            email TEXT
        )",
        [],
    ).map_err(|e| RuntimeError::ValueError(format!("Failed to create table: {}", e)))?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS test (
            id INTEGER PRIMARY KEY,
            value TEXT
        )",
        [],
    ).map_err(|e| RuntimeError::ValueError(format!("Failed to create table: {}", e)))?;

    // Insert some test data
    conn.execute(
        "INSERT OR REPLACE INTO users (id, name, email) VALUES (42, 'Alice', 'alice@example.com')",
        [],
    ).map_err(|e| RuntimeError::ValueError(format!("Failed to insert test data: {}", e)))?;

    let mut db = SQLITE_CONNECTION.lock().unwrap();
    *db = Some(conn);
    Ok(())
}

pub type EffectFunction = Box<dyn Fn(&[Value]) -> Result<Value, RuntimeError>>;

pub struct EffectRegistry {
    pub effects: HashMap<String, EffectFunction>,
}

impl EffectRegistry {
    pub fn new() -> Self {
        Self {
            effects: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: &str, func: EffectFunction) {
        self.effects.insert(name.to_string(), func);
    }

    pub fn get(&self, name: &str) -> Option<&EffectFunction> {
        self.effects.get(name)
    }
}

/// Execute a SQL query using SQLite
fn execute_sql_read(sql: &str, params_value: &Value) -> Result<Value, String> {
    let db_guard = SQLITE_CONNECTION.lock().unwrap();
    let conn = db_guard.as_ref()
        .ok_or_else(|| "Database not initialized. Call init_sqlite() first.".to_string())?;

    // Extract parameters from Value
    let param_vec = extract_params(params_value)?;

    // Prepare and execute the query
    let mut stmt = conn.prepare(sql)
        .map_err(|e| format!("SQL prepare error: {}", e))?;

    // Get column count and names
    let column_count = stmt.column_count();
    let column_names: Vec<String> = (0..column_count)
        .map(|i| stmt.column_name(i).unwrap_or("").to_string())
        .collect();

    // Execute with parameters and collect results
    let rows_result: Result<Vec<Value>, _> = stmt.query_map(
        rusqlite::params_from_iter(param_vec.iter()),
        |row| {
            let mut row_obj = HashMap::new();
            for (i, name) in column_names.iter().enumerate() {
                let value = match row.get_ref(i) {
                    Ok(rusqlite::types::ValueRef::Null) => Value::Null,
                    Ok(rusqlite::types::ValueRef::Integer(n)) => Value::Number(n),
                    Ok(rusqlite::types::ValueRef::Real(f)) => Value::String(f.to_string()),
                    Ok(rusqlite::types::ValueRef::Text(s)) => {
                        Value::String(String::from_utf8_lossy(s).to_string())
                    }
                    Ok(rusqlite::types::ValueRef::Blob(b)) => {
                        Value::String(hex::encode(b))
                    }
                    Err(_) => Value::Null,
                };
                row_obj.insert(name.clone(), value);
            }
            Ok(Value::Object(row_obj))
        },
    ).and_then(|rows| rows.collect());

    match rows_result {
        Ok(rows) => Ok(Value::Array(rows)),
        Err(e) => Err(format!("SQL query error: {}", e)),
    }
}

/// Execute a SQL write operation using SQLite
fn execute_sql_write(sql: &str, params_value: &Value) -> Result<Value, String> {
    let db_guard = SQLITE_CONNECTION.lock().unwrap();
    let conn = db_guard.as_ref()
        .ok_or_else(|| "Database not initialized. Call init_sqlite() first.".to_string())?;

    // Extract parameters
    let param_vec = extract_params(params_value)?;

    // Execute the statement
    let affected_rows = conn.execute(
        sql,
        rusqlite::params_from_iter(param_vec.iter()),
    ).map_err(|e| format!("SQL execute error: {}", e))?;

    // Get last insert rowid
    let last_id = conn.last_insert_rowid();

    Ok(Value::Object(HashMap::from([
        ("affectedRows".to_string(), Value::Number(affected_rows as i64)),
        ("insertId".to_string(), Value::Number(last_id)),
    ])))
}

/// Extract parameters from Value to rusqlite values
fn extract_params(params_value: &Value) -> Result<Vec<rusqlite::types::Value>, String> {
    match params_value {
        Value::Array(arr) => {
            arr.iter()
                .map(|v| value_to_sqlite(v))
                .collect()
        }
        Value::Object(obj) => {
            // For objects, just use values in iteration order (not ideal, but workable)
            obj.values()
                .map(|v| value_to_sqlite(v))
                .collect()
        }
        Value::Null => Ok(vec![]),
        _ => Err("Parameters must be an array or object".to_string()),
    }
}

/// Convert Value to rusqlite Value
fn value_to_sqlite(v: &Value) -> Result<rusqlite::types::Value, String> {
    match v {
        Value::Null => Ok(rusqlite::types::Value::Null),
        Value::Number(n) => Ok(rusqlite::types::Value::Integer(*n)),
        Value::Boolean(b) => Ok(rusqlite::types::Value::Integer(if *b { 1 } else { 0 })),
        Value::String(s) => Ok(rusqlite::types::Value::Text(s.clone())),
        _ => Err("Unsupported parameter type for SQL".to_string()),
    }
}

pub fn inject_effects(interp: &mut Interpreter, _seed: &[u8; 32]) -> Result<(), RuntimeError> {
    eprintln!("DEBUG: Injecting effects");

    // Initialize in-memory SQLite if not already done
    {
        let db = SQLITE_CONNECTION.lock().unwrap();
        if db.is_none() {
            drop(db);
            init_sqlite_memory()?;
        }
    }

    // DbRead function - executes SQL SELECT queries
    let db_read_func = Value::Function(FunctionValue {
        name: Some("DbRead".to_string()),
        params: vec!["sql".to_string(), "params".to_string()],
        closure: HashMap::new(),
    });
    interp.global_scope.insert("DbRead".to_string(), db_read_func);

    let db_read_body = JsExpr::Call(
        Box::new(JsExpr::Ident("db_read_impl".to_string())),
        vec![
            JsExpr::Ident("sql".to_string()),
            JsExpr::Ident("params".to_string()),
        ],
    );
    interp.function_bodies.insert(
        "DbRead".to_string(),
        StoredFunction {
            params: vec!["sql".to_string(), "params".to_string()],
            body: Box::new(db_read_body),
        },
    );

    interp.builtins.insert("db_read_impl".to_string(), |args| {
        if args.len() != 2 {
            return Err("db_read_impl expects 2 arguments".to_string());
        }
        let sql = match &args[0] {
            Value::String(s) => s,
            _ => return Err("SQL must be a string".to_string()),
        };
        execute_sql_read(sql, &args[1])
    });

    // DbWrite function - executes SQL INSERT/UPDATE/DELETE
    let db_write_func = Value::Function(FunctionValue {
        name: Some("DbWrite".to_string()),
        params: vec!["sql".to_string(), "params".to_string()],
        closure: HashMap::new(),
    });
    interp.global_scope.insert("DbWrite".to_string(), db_write_func);

    let db_write_body = JsExpr::Call(
        Box::new(JsExpr::Ident("db_write_impl".to_string())),
        vec![
            JsExpr::Ident("sql".to_string()),
            JsExpr::Ident("params".to_string()),
        ],
    );
    interp.function_bodies.insert(
        "DbWrite".to_string(),
        StoredFunction {
            params: vec!["sql".to_string(), "params".to_string()],
            body: Box::new(db_write_body),
        },
    );

    interp.builtins.insert("db_write_impl".to_string(), |args| {
        if args.len() != 2 {
            return Err("db_write_impl expects 2 arguments".to_string());
        }
        let sql = match &args[0] {
            Value::String(s) => s,
            _ => return Err("SQL must be a string".to_string()),
        };
        execute_sql_write(sql, &args[1])
    });

    // HttpOut function - makes HTTP requests
    let http_func = Value::Function(FunctionValue {
        name: Some("HttpOut".to_string()),
        params: vec!["method".to_string(), "url".to_string(), "body".to_string()],
        closure: HashMap::new(),
    });
    interp.global_scope.insert("HttpOut".to_string(), http_func);

    let http_body = JsExpr::Call(
        Box::new(JsExpr::Ident("http_impl".to_string())),
        vec![
            JsExpr::Ident("method".to_string()),
            JsExpr::Ident("url".to_string()),
            JsExpr::Ident("body".to_string()),
        ],
    );
    interp.function_bodies.insert(
        "HttpOut".to_string(),
        StoredFunction {
            params: vec!["method".to_string(), "url".to_string(), "body".to_string()],
            body: Box::new(http_body),
        },
    );

    interp.builtins.insert("http_impl".to_string(), |args| {
        if args.len() < 2 {
            return Err("http_impl expects at least 2 arguments".to_string());
        }
        let method = match &args[0] {
            Value::String(s) => s.clone(),
            _ => return Err("Method must be a string".to_string()),
        };
        let url = match &args[1] {
            Value::String(s) => s.clone(),
            _ => return Err("URL must be a string".to_string()),
        };

        // Use reqwest for real HTTP requests
        let client = reqwest::blocking::Client::new();
        let response = match method.to_uppercase().as_str() {
            "GET" => client.get(&url).send(),
            "POST" => {
                let body = if args.len() > 2 {
                    match &args[2] {
                        Value::String(s) => s.clone(),
                        _ => String::new(),
                    }
                } else {
                    String::new()
                };
                client.post(&url).body(body).send()
            }
            "PUT" => {
                let body = if args.len() > 2 {
                    match &args[2] {
                        Value::String(s) => s.clone(),
                        _ => String::new(),
                    }
                } else {
                    String::new()
                };
                client.put(&url).body(body).send()
            }
            "DELETE" => client.delete(&url).send(),
            _ => return Err(format!("Unsupported HTTP method: {}", method)),
        };

        match response {
            Ok(resp) => {
                let status = resp.status().as_u16() as i64;
                let body = resp.text().unwrap_or_default();

                // Try to parse as JSON
                let body_value = if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                    json_value_to_value(&json)
                } else {
                    Value::String(body)
                };

                Ok(Value::Object(HashMap::from([
                    ("status".to_string(), Value::Number(status)),
                    ("body".to_string(), body_value),
                ])))
            }
            Err(e) => Ok(Value::Object(HashMap::from([
                ("error".to_string(), Value::String(e.to_string())),
                ("status".to_string(), Value::Number(0)),
            ]))),
        }
    });

    // Log function - audit logging
    let log_func = Value::Function(FunctionValue {
        name: Some("Log".to_string()),
        params: vec!["message".to_string()],
        closure: HashMap::new(),
    });
    interp.global_scope.insert("Log".to_string(), log_func);

    let log_body = JsExpr::Call(
        Box::new(JsExpr::Ident("log_impl".to_string())),
        vec![JsExpr::Ident("message".to_string())],
    );
    interp.function_bodies.insert(
        "Log".to_string(),
        StoredFunction {
            params: vec!["message".to_string()],
            body: Box::new(log_body),
        },
    );

    interp.builtins.insert("log_impl".to_string(), |args| {
        if args.len() != 1 {
            return Err("log_impl expects 1 argument".to_string());
        }
        let message = match &args[0] {
            Value::String(s) => s.clone(),
            other => format!("{}", other),
        };

        // Output to stderr as JSON for audit logging
        let timestamp = chrono::Utc::now().to_rfc3339();
        let log_entry = serde_json::json!({
            "timestamp": timestamp,
            "level": "INFO",
            "message": message
        });
        eprintln!("{}", log_entry);

        Ok(Value::Object(HashMap::from([
            ("logged".to_string(), Value::Boolean(true)),
            ("timestamp".to_string(), Value::String(timestamp)),
        ])))
    });

    // Async function
    let async_func = Value::Function(FunctionValue {
        name: Some("Async".to_string()),
        params: vec!["promise_hash".to_string(), "cont_id".to_string(), "effect_args".to_string()],
        closure: HashMap::new(),
    });
    interp.global_scope.insert("Async".to_string(), async_func);

    let async_body = JsExpr::Call(
        Box::new(JsExpr::Ident("async_impl".to_string())),
        vec![
            JsExpr::Ident("promise_hash".to_string()),
            JsExpr::Ident("cont_id".to_string()),
            JsExpr::Ident("effect_args".to_string()),
        ],
    );
    interp.function_bodies.insert(
        "Async".to_string(),
        StoredFunction {
            params: vec!["promise_hash".to_string(), "cont_id".to_string(), "effect_args".to_string()],
            body: Box::new(async_body),
        },
    );

    interp.builtins.insert("async_impl".to_string(), |args| {
        // Async operations are cached by (seed, contId)
        // For now, generate deterministic result based on inputs
        let result_hash = if args.len() >= 2 {
            let hash_input = format!("{:?}{:?}", args[0], args[1]);
            let hash = sha2::Sha256::digest(hash_input.as_bytes());
            hex::encode(&hash[..8])
        } else {
            "default".to_string()
        };

        Ok(Value::Object(HashMap::from([
            ("async_result".to_string(), Value::String("completed".to_string())),
            ("result_id".to_string(), Value::String(result_hash)),
        ])))
    });

    Ok(())
}

/// Convert serde_json::Value to our Value type
fn json_value_to_value(json: &serde_json::Value) -> Value {
    match json {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Boolean(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Number(i)
            } else if let Some(f) = n.as_f64() {
                Value::String(f.to_string())
            } else {
                Value::Null
            }
        }
        serde_json::Value::String(s) => Value::String(s.clone()),
        serde_json::Value::Array(arr) => {
            Value::Array(arr.iter().map(json_value_to_value).collect())
        }
        serde_json::Value::Object(obj) => {
            Value::Object(
                obj.iter()
                    .map(|(k, v)| (k.clone(), json_value_to_value(v)))
                    .collect(),
            )
        }
    }
}

fn generate_deterministic_async_result(seed: &[u8; 32]) -> Value {
    let promise_id = format!("promise_{}", hex::encode(&sha2::Sha256::digest(seed)[..8]));
    Value::String(promise_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sqlite_memory() {
        init_sqlite_memory().unwrap();

        // Test read
        let result = execute_sql_read(
            "SELECT id, name FROM users WHERE id = ?",
            &Value::Array(vec![Value::Number(42)]),
        ).unwrap();

        match result {
            Value::Array(rows) => {
                assert!(!rows.is_empty());
                if let Value::Object(row) = &rows[0] {
                    assert_eq!(row.get("name"), Some(&Value::String("Alice".to_string())));
                }
            }
            _ => panic!("Expected array result"),
        }
    }

    #[test]
    fn test_sqlite_write() {
        init_sqlite_memory().unwrap();

        // Test write
        let result = execute_sql_write(
            "INSERT INTO test (id, value) VALUES (?, ?)",
            &Value::Array(vec![Value::Number(1), Value::String("test_value".to_string())]),
        ).unwrap();

        match result {
            Value::Object(obj) => {
                assert_eq!(obj.get("affectedRows"), Some(&Value::Number(1)));
            }
            _ => panic!("Expected object result"),
        }

        // Verify the write
        let read_result = execute_sql_read(
            "SELECT value FROM test WHERE id = ?",
            &Value::Array(vec![Value::Number(1)]),
        ).unwrap();

        match read_result {
            Value::Array(rows) => {
                assert!(!rows.is_empty());
            }
            _ => panic!("Expected array result"),
        }
    }
}
