use rusqlite::Connection;
use std::collections::HashMap;
use std::sync::Mutex;

use crate::errors::runtime::RuntimeError;
use crate::runtime::interpreter::{Interpreter, JsExpr, StoredFunction};
use crate::runtime::value::{FunctionValue, Value};

lazy_static::lazy_static! {
    static ref DB_STORE: Mutex<HashMap<String, Value>> = Mutex::new(HashMap::new());
    static ref SQLITE_CONNECTION: Mutex<Option<Connection>> = Mutex::new(None);
    /// Async operation cache - keyed by (seed_hex, cont_id) for deterministic replay per §7-a
    static ref ASYNC_CACHE: Mutex<HashMap<String, Value>> = Mutex::new(HashMap::new());
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
    )
    .map_err(|e| RuntimeError::ValueError(format!("Failed to create table: {}", e)))?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS test (
            id INTEGER PRIMARY KEY,
            value TEXT
        )",
        [],
    )
    .map_err(|e| RuntimeError::ValueError(format!("Failed to create table: {}", e)))?;

    let mut db = SQLITE_CONNECTION.lock().unwrap_or_else(|e| e.into_inner());
    *db = Some(conn);
    Ok(())
}

/// Initialize in-memory SQLite database
pub fn init_sqlite_memory() -> Result<(), RuntimeError> {
    let conn = Connection::open_in_memory().map_err(|e| {
        RuntimeError::ValueError(format!("Failed to open in-memory database: {}", e))
    })?;

    // Create default tables
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            email TEXT
        )",
        [],
    )
    .map_err(|e| RuntimeError::ValueError(format!("Failed to create table: {}", e)))?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS test (
            id INTEGER PRIMARY KEY,
            value TEXT
        )",
        [],
    )
    .map_err(|e| RuntimeError::ValueError(format!("Failed to create table: {}", e)))?;

    // Insert some test data
    conn.execute(
        "INSERT OR REPLACE INTO users (id, name, email) VALUES (42, 'Alice', 'alice@example.com')",
        [],
    )
    .map_err(|e| RuntimeError::ValueError(format!("Failed to insert test data: {}", e)))?;

    let mut db = SQLITE_CONNECTION.lock().unwrap_or_else(|e| e.into_inner());
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
    let db_guard = SQLITE_CONNECTION.lock().unwrap_or_else(|e| e.into_inner());
    let conn = db_guard
        .as_ref()
        .ok_or_else(|| "Database not initialized. Call init_sqlite() first.".to_string())?;

    // Extract parameters from Value
    let param_vec = extract_params(params_value)?;

    // Prepare and execute the query
    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| format!("SQL prepare error: {}", e))?;

    // Get column count and names
    let column_count = stmt.column_count();
    let column_names: Vec<String> = (0..column_count)
        .map(|i| stmt.column_name(i).unwrap_or("").to_string())
        .collect();

    // Execute with parameters and collect results
    let rows_result: Result<Vec<Value>, _> = stmt
        .query_map(rusqlite::params_from_iter(param_vec.iter()), |row| {
            let mut row_obj = HashMap::new();
            for (i, name) in column_names.iter().enumerate() {
                let value = match row.get_ref(i) {
                    Ok(rusqlite::types::ValueRef::Null) => Value::Null,
                    Ok(rusqlite::types::ValueRef::Integer(n)) => Value::Number(n),
                    Ok(rusqlite::types::ValueRef::Real(f)) => Value::String(f.to_string()),
                    Ok(rusqlite::types::ValueRef::Text(s)) => {
                        Value::String(String::from_utf8_lossy(s).to_string())
                    }
                    Ok(rusqlite::types::ValueRef::Blob(b)) => Value::String(hex::encode(b)),
                    Err(_) => Value::Null,
                };
                row_obj.insert(name.clone(), value);
            }
            Ok(Value::Object(row_obj))
        })
        .and_then(|rows| rows.collect());

    match rows_result {
        Ok(rows) => Ok(Value::Array(rows)),
        Err(e) => Err(format!("SQL query error: {}", e)),
    }
}

/// Execute a SQL write operation using SQLite
fn execute_sql_write(sql: &str, params_value: &Value) -> Result<Value, String> {
    let db_guard = SQLITE_CONNECTION.lock().unwrap_or_else(|e| e.into_inner());
    let conn = db_guard
        .as_ref()
        .ok_or_else(|| "Database not initialized. Call init_sqlite() first.".to_string())?;

    // Extract parameters
    let param_vec = extract_params(params_value)?;

    // Execute the statement
    let affected_rows = conn
        .execute(sql, rusqlite::params_from_iter(param_vec.iter()))
        .map_err(|e| format!("SQL execute error: {}", e))?;

    // Get last insert rowid
    let last_id = conn.last_insert_rowid();

    Ok(Value::Object(HashMap::from([
        (
            "affectedRows".to_string(),
            Value::Number(affected_rows as i64),
        ),
        ("insertId".to_string(), Value::Number(last_id)),
    ])))
}

/// Extract parameters from Value to rusqlite values
fn extract_params(params_value: &Value) -> Result<Vec<rusqlite::types::Value>, String> {
    match params_value {
        Value::Array(arr) => arr.iter().map(|v| value_to_sqlite(v)).collect(),
        Value::Object(obj) => {
            // For objects, just use values in iteration order (not ideal, but workable)
            obj.values().map(|v| value_to_sqlite(v)).collect()
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

/// Execute an HTTP request synchronously per TECHSPECV5.md §7-a
/// This blocks until the request completes - no JavaScript event loop visible inside VM
fn execute_http_request(method: &str, url: &str, body: &Value) -> Result<Value, String> {
    // Use reqwest in blocking mode for synchronous execution
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let request_body = match body {
        Value::Null => None,
        Value::String(s) => Some(s.clone()),
        other => Some(value_to_json_string(other)),
    };

    let response = match method.to_uppercase().as_str() {
        "GET" => client.get(url).send(),
        "POST" => {
            let mut req = client.post(url);
            if let Some(b) = &request_body {
                req = req.header("Content-Type", "application/json").body(b.clone());
            }
            req.send()
        }
        "PUT" => {
            let mut req = client.put(url);
            if let Some(b) = &request_body {
                req = req.header("Content-Type", "application/json").body(b.clone());
            }
            req.send()
        }
        "DELETE" => client.delete(url).send(),
        "PATCH" => {
            let mut req = client.patch(url);
            if let Some(b) = &request_body {
                req = req.header("Content-Type", "application/json").body(b.clone());
            }
            req.send()
        }
        _ => return Err(format!("Unsupported HTTP method: {}", method)),
    };

    match response {
        Ok(resp) => {
            let status = resp.status().as_u16() as i64;
            let headers: HashMap<String, Value> = resp
                .headers()
                .iter()
                .map(|(k, v)| {
                    (
                        k.to_string(),
                        Value::String(v.to_str().unwrap_or("").to_string()),
                    )
                })
                .collect();

            let body_text = resp.text().unwrap_or_default();

            // Try to parse body as JSON, fall back to string
            let body_value = if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body_text) {
                json_value_to_value(&json)
            } else {
                Value::String(body_text)
            };

            Ok(Value::Object(HashMap::from([
                ("status".to_string(), Value::Number(status)),
                ("headers".to_string(), Value::Object(headers)),
                ("body".to_string(), body_value),
            ])))
        }
        Err(e) => {
            // Return error as a structured response per §16 Error System
            Ok(Value::Object(HashMap::from([
                ("error".to_string(), Value::String(e.to_string())),
                ("status".to_string(), Value::Number(0)),
            ])))
        }
    }
}

/// Convert a Value to a JSON string for HTTP body
fn value_to_json_string(v: &Value) -> String {
    match v {
        Value::Null => "null".to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::Decimal(d) => format!("\"{}\"", d), // Decimals as strings per spec §4-a
        Value::String(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
        Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(value_to_json_string).collect();
            format!("[{}]", items.join(","))
        }
        Value::Object(obj) => {
            let fields: Vec<String> = obj
                .iter()
                .map(|(k, v)| format!("\"{}\":{}", k, value_to_json_string(v)))
                .collect();
            format!("{{{}}}", fields.join(","))
        }
        Value::Function(_) => "null".to_string(), // Functions can't be serialized
    }
}

pub fn inject_effects(interp: &mut Interpreter, _seed: &[u8; 32]) -> Result<(), RuntimeError> {

    // Initialize in-memory SQLite if not already done
    {
        let db = SQLITE_CONNECTION.lock().unwrap_or_else(|e| e.into_inner());
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
    interp
        .global_scope
        .insert("DbRead".to_string(), db_read_func);

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
    interp
        .global_scope
        .insert("DbWrite".to_string(), db_write_func);

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
            return Err("http_impl expects at least 2 arguments (method, url)".to_string());
        }

        let method = match &args[0] {
            Value::String(s) => s.clone(),
            _ => return Err("method must be a string".to_string()),
        };

        let url = match &args[1] {
            Value::String(s) => s.clone(),
            _ => return Err("url must be a string".to_string()),
        };

        let body = if args.len() > 2 {
            args[2].clone()
        } else {
            Value::Null
        };

        // Execute HTTP request synchronously per TECHSPECV5.md §7-a
        execute_http_request(&method, &url, &body)
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

    // Async function - implements deterministic await per §7-a
    let async_func = Value::Function(FunctionValue {
        name: Some("Async".to_string()),
        params: vec![
            "promise_hash".to_string(),
            "cont_id".to_string(),
            "effect_args".to_string(),
        ],
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
            params: vec![
                "promise_hash".to_string(),
                "cont_id".to_string(),
                "effect_args".to_string(),
            ],
            body: Box::new(async_body),
        },
    );

    // Store seed for async operations - clone into the closure
    let seed_hex = hex::encode(_seed);
    interp.global_scope.insert(
        "__async_seed".to_string(),
        Value::String(seed_hex.clone()),
    );

    interp.builtins.insert("async_impl".to_string(), |args| {
        if args.len() < 3 {
            return Err("async_impl expects at least 3 arguments (promise_hash, cont_id, effect_args)".to_string());
        }

        let promise_hash = match &args[0] {
            Value::String(s) => s.clone(),
            _ => return Err("promise_hash must be a string".to_string()),
        };

        let cont_id = match &args[1] {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            _ => return Err("cont_id must be a string or number".to_string()),
        };

        // Build cache key from promise_hash and cont_id per §7-a
        let cache_key = format!("{}:{}", promise_hash, cont_id);

        // Check cache for deterministic replay
        {
            let cache = ASYNC_CACHE.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(cached_result) = cache.get(&cache_key) {
                return Ok(cached_result.clone());
            }
        }

        // Execute the effect based on effect_args
        let effect_args = &args[2];
        let result = execute_async_effect(effect_args)?;

        // Cache the result for deterministic replay
        {
            let mut cache = ASYNC_CACHE.lock().unwrap_or_else(|e| e.into_inner());
            cache.insert(cache_key, result.clone());
        }

        Ok(result)
    });

    Ok(())
}

/// Execute an async effect and return the result
/// Per §7-a, this blocks synchronously - no JS event loop visible inside VM
fn execute_async_effect(effect_args: &Value) -> Result<Value, String> {
    match effect_args {
        // If effect_args is an object with an "effect" field, dispatch based on effect type
        Value::Object(obj) => {
            if let Some(effect_type) = obj.get("effect") {
                match effect_type {
                    Value::String(s) if s == "DbRead" => {
                        // Execute DbRead effect
                        let sql = obj.get("sql").and_then(|v| {
                            if let Value::String(s) = v { Some(s.as_str()) } else { None }
                        }).ok_or("DbRead requires 'sql' field")?;
                        let params = obj.get("params").cloned().unwrap_or(Value::Array(vec![]));
                        execute_sql_read(sql, &params)
                    }
                    Value::String(s) if s == "DbWrite" => {
                        // Execute DbWrite effect
                        let sql = obj.get("sql").and_then(|v| {
                            if let Value::String(s) = v { Some(s.as_str()) } else { None }
                        }).ok_or("DbWrite requires 'sql' field")?;
                        let params = obj.get("params").cloned().unwrap_or(Value::Array(vec![]));
                        execute_sql_write(sql, &params)
                    }
                    Value::String(s) if s == "HttpOut" => {
                        // Execute HttpOut effect using the real HTTP client
                        let method = obj.get("method").and_then(|v| {
                            if let Value::String(s) = v { Some(s.clone()) } else { None }
                        }).unwrap_or_else(|| "GET".to_string());
                        let url = obj.get("url").and_then(|v| {
                            if let Value::String(s) = v { Some(s.clone()) } else { None }
                        }).ok_or("HttpOut requires 'url' field")?;
                        let body = obj.get("body").cloned().unwrap_or(Value::Null);

                        // Execute HTTP request synchronously per TECHSPECV5.md §7-a
                        execute_http_request(&method, &url, &body)
                    }
                    _ => {
                        // Unknown effect type, return the args as-is
                        Ok(effect_args.clone())
                    }
                }
            } else {
                // No effect field, return the args as-is (backwards compatibility)
                Ok(effect_args.clone())
            }
        }
        // For non-object args, return as-is (backwards compatibility)
        _ => Ok(effect_args.clone()),
    }
}

/// Clear the async cache - useful for testing
pub fn clear_async_cache() {
    let mut cache = ASYNC_CACHE.lock().unwrap_or_else(|e| e.into_inner());
    cache.clear();
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
        serde_json::Value::Object(obj) => Value::Object(
            obj.iter()
                .map(|(k, v)| (k.clone(), json_value_to_value(v)))
                .collect(),
        ),
    }
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
        )
        .unwrap();

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
            &Value::Array(vec![
                Value::Number(1),
                Value::String("test_value".to_string()),
            ]),
        )
        .unwrap();

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
        )
        .unwrap();

        match read_result {
            Value::Array(rows) => {
                assert!(!rows.is_empty());
            }
            _ => panic!("Expected array result"),
        }
    }
}
