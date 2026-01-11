use std::collections::HashMap;

/// Runtime value representation
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(i64),
    Boolean(bool),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
    Function(FunctionValue),
    Null,
}

#[derive(Debug, Clone)]
pub struct FunctionValue {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<u8>, // Serialized bytecode or AST
}

impl Value {
    /// Get string representation for debugging
    pub fn to_string(&self) -> String {
        match self {
            Value::Number(n) => n.to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::String(s) => format!("\"{}\"", s),
            Value::Array(arr) => format!("[{} items]", arr.len()),
            Value::Object(obj) => format!("{{{}}} entries", obj.len()),
            Value::Function(f) => format!("<function {}>", f.name),
            Value::Null => "null".to_string(),
        }
    }

    /// Check if value is truthy
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::Null => false,
            Value::Number(n) => *n != 0,
            Value::String(s) => !s.is_empty(),
            Value::Array(arr) => !arr.is_empty(),
            Value::Object(obj) => !obj.is_empty(),
            Value::Function(_) => true,
        }
    }

    /// Add two values
    pub fn add(&self, other: &Value) -> Result<Value, &'static str> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a
                .checked_add(*b)
                .map(Value::Number)
                .ok_or("Integer overflow in addition"),
            (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
            _ => Err("Cannot add these types"),
        }
    }

    /// Compare two values
    pub fn equals(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Null, Value::Null) => true,
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Object(a), Value::Object(b)) => a == b,
            _ => false,
        }
    }
}
