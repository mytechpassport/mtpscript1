use sha2::{Digest, Sha256};
use std::collections::HashMap;

use crate::errors::runtime::RuntimeError;
use crate::runtime::interpreter::Interpreter;
use crate::runtime::value::{FunctionValue, Value};

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

pub fn inject_effects(interp: &mut Interpreter, seed: &[u8; 32]) -> Result<(), RuntimeError> {
    // Use seed for deterministic effect implementations and caching
    // Cache key is (seed, effect_name)
    let cache_key_prefix = format!("{:x}", sha2::Sha256::digest(seed));

    // For now, inject deterministic mock functions based on seed
    // In real implementation, these would call actual effect handlers with seed-based determinism

    // DbRead: return deterministic data based on seed
    let db_read_data = generate_deterministic_db_data(seed);
    let db_read_func = Value::Function(FunctionValue {
        name: Some("DbRead".to_string()),
        params: vec!["sql".to_string(), "params".to_string()],
        closure: HashMap::from([("data".to_string(), db_read_data)]),
    });
    interp
        .global_scope
        .insert("DbRead".to_string(), db_read_func);

    let db_write_result = generate_deterministic_db_write_result(seed);
    let db_write_func = Value::Function(FunctionValue {
        name: Some("DbWrite".to_string()),
        params: vec!["sql".to_string(), "params".to_string()],
        closure: HashMap::from([("result".to_string(), db_write_result)]),
    });
    interp
        .global_scope
        .insert("DbWrite".to_string(), db_write_func);

    let http_result = generate_deterministic_http_result(seed);
    let http_func = Value::Function(FunctionValue {
        name: Some("HttpOut".to_string()),
        params: vec!["method".to_string(), "url".to_string()],
        closure: HashMap::from([("response".to_string(), http_result)]),
    });
    interp.global_scope.insert("HttpOut".to_string(), http_func);

    let log_func = Value::Function(FunctionValue {
        name: Some("Log".to_string()),
        params: vec!["message".to_string()],
        closure: HashMap::new(), // Logging doesn't need determinism
    });
    interp.global_scope.insert("Log".to_string(), log_func);

    let async_result = generate_deterministic_async_result(seed);
    let async_func = Value::Function(FunctionValue {
        name: Some("Async".to_string()),
        params: vec![
            "promiseHash".to_string(),
            "contId".to_string(),
            "args".to_string(),
        ],
        closure: HashMap::from([("async_data".to_string(), async_result)]),
    });
    interp.global_scope.insert("Async".to_string(), async_func);

    Ok(())
}

fn generate_deterministic_db_data(seed: &[u8; 32]) -> Value {
    // Use seed to generate deterministic mock data
    let id =
        ((seed[0] as u32) << 24 | (seed[1] as u32) << 16 | (seed[2] as u32) << 8 | seed[3] as u32)
            as i64;
    let name_len = (seed[4] % 10) + 5; // 5-14 chars
    let name = (0..name_len)
        .map(|i| (b'A' + (seed[5 + i as usize] % 26)) as char)
        .collect::<String>();
    Value::Array(vec![Value::Object(HashMap::from([
        ("id".to_string(), Value::Number(id)),
        ("name".to_string(), Value::String(name)),
    ]))])
}

fn generate_deterministic_db_write_result(seed: &[u8; 32]) -> Value {
    let affected_rows = seed[0] as i64 % 100 + 1; // 1-100
    Value::Object(HashMap::from([
        ("affectedRows".to_string(), Value::Number(affected_rows)),
        ("insertId".to_string(), Value::Number(seed[1] as i64)),
    ]))
}

fn generate_deterministic_http_result(seed: &[u8; 32]) -> Value {
    let status = if seed[0] % 2 == 0 { 200 } else { 404 };
    let body_len = (seed[1] % 50) + 10; // 10-59 chars
    let body = (0..body_len)
        .map(|i| (b'a' + (seed[2 + i as usize] % 26)) as char)
        .collect::<String>();
    Value::Object(HashMap::from([
        ("status".to_string(), Value::Number(status)),
        ("body".to_string(), Value::String(body)),
    ]))
}

fn generate_deterministic_async_result(seed: &[u8; 32]) -> Value {
    let promise_id = format!("promise_{:x}", sha2::Sha256::digest(seed));
    Value::String(promise_id)
}
