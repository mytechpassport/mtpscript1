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
fn db_read_effect(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::ValueError(
            "DbRead expects 2 arguments".to_string(),
        ));
    }

    let sql = args[0].as_string()?;
    let _params = args[1].as_object()?; // JSON object

    // Placeholder: would execute SQL with deterministic seed-based behavior
    // For now, return mock data
    Ok(Value::Array(vec![Value::Object(HashMap::from([
        ("id".to_string(), Value::Number(1)),
        ("name".to_string(), Value::String("Alice".to_string())),
    ]))]))
}

fn db_write_effect(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::ValueError(
            "DbWrite expects 2 arguments".to_string(),
        ));
    }

    let sql = args[0].as_string()?;
    let _params = args[1].as_object()?; // JSON object

    // Placeholder: would execute SQL
    Ok(Value::Number(1)) // Rows affected
}

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
    // Inject effect functions directly into global scope
    // In real implementation, these would be proper function objects that call the effects

    // For now, inject as mock functions
    let db_read_func = Value::Function(FunctionValue {
        name: Some("DbRead".to_string()),
        params: vec!["sql".to_string(), "params".to_string()],
        closure: HashMap::new(),
    });
    interp
        .global_scope
        .insert("DbRead".to_string(), db_read_func);

    let db_write_func = Value::Function(FunctionValue {
        name: Some("DbWrite".to_string()),
        params: vec!["sql".to_string(), "params".to_string()],
        closure: HashMap::new(),
    });
    interp
        .global_scope
        .insert("DbWrite".to_string(), db_write_func);

    let http_func = Value::Function(FunctionValue {
        name: Some("HttpOut".to_string()),
        params: vec!["method".to_string(), "url".to_string()],
        closure: HashMap::new(),
    });
    interp.global_scope.insert("HttpOut".to_string(), http_func);

    let log_func = Value::Function(FunctionValue {
        name: Some("Log".to_string()),
        params: vec!["message".to_string()],
        closure: HashMap::new(),
    });
    interp.global_scope.insert("Log".to_string(), log_func);

    let async_func = Value::Function(FunctionValue {
        name: Some("Async".to_string()),
        params: vec![
            "promiseHash".to_string(),
            "contId".to_string(),
            "args".to_string(),
        ],
        closure: HashMap::new(),
    });
    interp.global_scope.insert("Async".to_string(), async_func);

    Ok(())
}
