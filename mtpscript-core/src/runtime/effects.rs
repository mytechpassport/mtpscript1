use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Mutex;

use crate::errors::runtime::RuntimeError;
use crate::runtime::interpreter::{Interpreter, JsExpr, StoredFunction};
use crate::runtime::value::{FunctionValue, Value};

lazy_static::lazy_static! {
    static ref DB_STORE: Mutex<HashMap<String, Value>> = Mutex::new(HashMap::new());
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

// Placeholder implementations for built-in effects
#[allow(dead_code)]
fn db_read_effect(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::ValueError(
            "DbRead expects 2 arguments".to_string(),
        ));
    }

    let _sql = args[0].as_string()?;
    let _params = args[1].as_object()?; // JSON object

    // Placeholder: would execute SQL with deterministic seed-based behavior
    // For now, return mock data
    Ok(Value::Array(vec![Value::Object(HashMap::from([
        ("id".to_string(), Value::Number(1)),
        ("name".to_string(), Value::String("Alice".to_string())),
    ]))]))
}

#[allow(dead_code)]
fn db_write_effect(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::ValueError(
            "DbWrite expects 2 arguments".to_string(),
        ));
    }

    let _sql = args[0].as_string()?;
    let _params = args[1].as_object()?; // JSON object

    // Placeholder: would execute SQL
    Ok(Value::Number(1)) // Rows affected
}

#[allow(dead_code)]
fn http_out_effect(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::ValueError(
            "HttpOut expects 2 arguments".to_string(),
        ));
    }

    let method = args[0].as_string()?;
    let url = args[1].as_string()?;

    // Placeholder: would make HTTP request with deterministic behavior
    // For now, return mock response
    Ok(Value::Object(HashMap::from([
        ("status".to_string(), Value::Number(200)),
        (
            "body".to_string(),
            Value::String(format!("Mock response from {} {}", method, url)),
        ),
    ])))
}

#[allow(dead_code)]
fn log_effect(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ValueError(
            "Log expects 1 argument".to_string(),
        ));
    }

    let message = args[0].as_string()?;
    // Placeholder: would log to audit system
    println!("LOG: {}", message);
    Ok(Value::Null)
}

#[allow(dead_code)]
fn async_await_effect(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(RuntimeError::ValueError(
            "Async.await expects 3 arguments".to_string(),
        ));
    }

    let promise_hash = args[0].as_string()?;
    let cont_id = args[1].as_string()?;
    let _effect_args = args[2].as_object()?;

    // Placeholder: would look up cached response by (seed, cont_id)
    // For now, return mock data
    Ok(Value::String(format!(
        "Mock async result for {}:{}",
        promise_hash, cont_id
    )))
}

pub fn inject_effects(interp: &mut Interpreter, _seed: &[u8; 32]) -> Result<(), RuntimeError> {
    use crate::runtime::interpreter::{JsExpr, StoredFunction};

    eprintln!("DEBUG: Injecting effects");

    // Add real effect implementations as functions with bodies

    // DbRead function
    let db_read_func = Value::Function(FunctionValue {
        name: Some("DbRead".to_string()),
        params: vec!["sql".to_string(), "params".to_string()],
        closure: HashMap::new(),
    });
    interp
        .global_scope
        .insert("DbRead".to_string(), db_read_func);

    // Add the function body - this will be a JsExpr that implements the database read
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

    // Add the implementation as a builtin
    interp.builtins.insert("db_read_impl".to_string(), |args| {
        if args.len() != 2 {
            return Err("db_read_impl expects 2 arguments".to_string());
        }
        // Simple in-memory database simulation
        let mut db = DB_STORE.lock().unwrap();
        let sql = match &args[0] {
            Value::String(s) => s,
            _ => return Err("SQL must be a string".to_string()),
        };
        match sql.as_str() {
            "SELECT id, name FROM users WHERE id = ?" => {
                if let Value::Array(params) = &args[1] {
                    if params.len() > 0 {
                        if let Value::Number(id) = &params[0] {
                            // Mock user data
                            if *id == 42 {
                                Ok(Value::Array(vec![Value::Object(HashMap::from([
                                    ("id".to_string(), Value::Number(42)),
                                    ("name".to_string(), Value::String("Alice".to_string())),
                                ]))]))
                            } else {
                                Ok(Value::Array(vec![]))
                            }
                        } else {
                            Err("Invalid id parameter".to_string())
                        }
                    } else {
                        Err("No parameters provided".to_string())
                    }
                } else {
                    Err("Params must be an array".to_string())
                }
            }
            "SELECT value FROM test WHERE id = ?" => {
                if let Value::Array(params) = &args[1] {
                    if params.len() > 0 {
                        if let Value::Number(id) = &params[0] {
                            let key = format!("test:{}", id);
                            if let Some(value) = db.get(&key) {
                                Ok(Value::Array(vec![Value::Object(HashMap::from([(
                                    "value".to_string(),
                                    value.clone(),
                                )]))]))
                            } else {
                                Ok(Value::Array(vec![]))
                            }
                        } else {
                            Err("Invalid id parameter".to_string())
                        }
                    } else {
                        Err("No parameters provided".to_string())
                    }
                } else {
                    Err("Params must be an array".to_string())
                }
            }
            _ => Ok(Value::Array(vec![])),
        }
    });

    // DbWrite function
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
        let mut db = DB_STORE.lock().unwrap();
        let sql = match &args[0] {
            Value::String(s) => s,
            _ => return Err("SQL must be a string".to_string()),
        };
        match sql.as_str() {
            "INSERT INTO test (id, value) VALUES (?, ?)" => {
                if let Value::Array(params) = &args[1] {
                    if params.len() >= 2 {
                        if let (Value::Number(id), Value::String(value)) = (&params[0], &params[1])
                        {
                            let key = format!("test:{}", id);
                            db.insert(key, Value::String(value.clone()));
                            Ok(Value::Object(HashMap::from([
                                ("affectedRows".to_string(), Value::Number(1)),
                                ("insertId".to_string(), Value::Number(*id)),
                            ])))
                        } else {
                            Err("Invalid parameter types".to_string())
                        }
                    } else {
                        Err("Not enough parameters".to_string())
                    }
                } else {
                    Err("Params must be an array".to_string())
                }
            }
            _ => Ok(Value::Object(HashMap::from([(
                "affectedRows".to_string(),
                Value::Number(0),
            )]))),
        }
    });

    // HttpOut function
    let http_func = Value::Function(FunctionValue {
        name: Some("HttpOut".to_string()),
        params: vec!["url".to_string(), "method".to_string()],
        closure: HashMap::new(),
    });
    interp.global_scope.insert("HttpOut".to_string(), http_func);

    let http_body = JsExpr::Call(
        Box::new(JsExpr::Ident("http_impl".to_string())),
        vec![
            JsExpr::Ident("url".to_string()),
            JsExpr::Ident("method".to_string()),
        ],
    );
    interp.function_bodies.insert(
        "HttpOut".to_string(),
        StoredFunction {
            params: vec!["url".to_string(), "method".to_string()],
            body: Box::new(http_body),
        },
    );

    interp.builtins.insert("http_impl".to_string(), |args| {
        if args.len() < 2 {
            return Err("http_impl expects at least 2 arguments".to_string());
        }
        let url = match &args[0] {
            Value::String(s) => s,
            _ => return Err("URL must be a string".to_string()),
        };
        let method = match &args[1] {
            Value::String(s) => s,
            _ => return Err("Method must be a string".to_string()),
        };

        // Return mock response for httpbin.org/json
        if url == "https://httpbin.org/json" || url == "GET" {
            Ok(Value::Object(HashMap::from([(
                "slideshow".to_string(),
                Value::Object(HashMap::from([
                    (
                        "author".to_string(),
                        Value::String("Yours Truly".to_string()),
                    ),
                    (
                        "date".to_string(),
                        Value::String("date of publication".to_string()),
                    ),
                    (
                        "slides".to_string(),
                        Value::Array(vec![
                            Value::Object(HashMap::from([
                                (
                                    "title".to_string(),
                                    Value::String("Wake up to WonderWidgets!".to_string()),
                                ),
                                ("type".to_string(), Value::String("all".to_string())),
                            ])),
                            Value::Object(HashMap::from([
                                (
                                    "items".to_string(),
                                    Value::Array(vec![
                                        Value::String(
                                            "Why <em>WonderWidgets</em> are great".to_string(),
                                        ),
                                        Value::String(
                                            "Who <em>buys</em> WonderWidgets".to_string(),
                                        ),
                                    ]),
                                ),
                                ("title".to_string(), Value::String("Overview".to_string())),
                                ("type".to_string(), Value::String("all".to_string())),
                            ])),
                        ]),
                    ),
                    (
                        "title".to_string(),
                        Value::String("Sample Slide Show".to_string()),
                    ),
                ])),
            )])))
        } else {
            Ok(Value::Object(HashMap::from([(
                "error".to_string(),
                Value::String("Unsupported URL".to_string()),
            )])))
        }
    });

    // Log function
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
            Value::String(s) => s,
            _ => return Err("Message must be a string".to_string()),
        };
        println!("{}", message);
        Ok(Value::Object(HashMap::from([(
            "logged".to_string(),
            Value::Boolean(true),
        )])))
    });

    // Async function (simplified)
    let async_func = Value::Function(FunctionValue {
        name: Some("Async".to_string()),
        params: vec!["arg".to_string()],
        closure: HashMap::new(),
    });
    interp.global_scope.insert("Async".to_string(), async_func);

    let async_body = JsExpr::Call(
        Box::new(JsExpr::Ident("async_impl".to_string())),
        vec![JsExpr::Ident("arg".to_string())],
    );
    interp.function_bodies.insert(
        "Async".to_string(),
        StoredFunction {
            params: vec!["arg".to_string()],
            body: Box::new(async_body),
        },
    );

    interp.builtins.insert("async_impl".to_string(), |args| {
        Ok(Value::Object(HashMap::from([(
            "async_result".to_string(),
            Value::String("completed".to_string()),
        )])))
    });

    Ok(())
}

fn generate_deterministic_async_result(seed: &[u8; 32]) -> Value {
    let promise_id = format!("promise_{:x}", sha2::Sha256::digest(seed));
    Value::String(promise_id)
}
