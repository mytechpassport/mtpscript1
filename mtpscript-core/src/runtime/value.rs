use std::collections::HashMap;
use std::fmt;

use crate::errors::runtime::RuntimeError;
use crate::types::decimal::Decimal;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(i64),
    Boolean(bool),
    String(String),
    Decimal(Decimal),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
    Function(FunctionValue),
    Null,
}

#[derive(Debug, Clone)]
pub struct FunctionValue {
    pub name: Option<String>,
    pub params: Vec<String>,
    pub closure: HashMap<String, Value>,
}

impl PartialEq for FunctionValue {
    fn eq(&self, _other: &Self) -> bool {
        false // Functions are not comparable
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::String(s) => write!(f, "{}", s),
            Value::Decimal(d) => write!(f, "{}", d),
            Value::Array(arr) => {
                write!(f, "[")?;
                for (i, val) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", val)?;
                }
                write!(f, "]")
            }
            Value::Object(obj) => {
                write!(f, "{{")?;
                for (i, (key, val)) in obj.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "\"{}\": {}", key, val)?;
                }
                write!(f, "}}")
            }
            Value::Function(func) => write!(f, "function({})", func.params.join(", ")),
            Value::Null => write!(f, "null"),
        }
    }
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Number(_) => "number",
            Value::Boolean(_) => "boolean",
            Value::String(_) => "string",
            Value::Decimal(_) => "decimal",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
            Value::Function(_) => "function",
            Value::Null => "null",
        }
    }

    pub fn as_number(&self) -> Result<i64, RuntimeError> {
        match self {
            Value::Number(n) => Ok(*n),
            _ => Err(RuntimeError::TypeError(format!(
                "Expected number, got {}",
                self.type_name()
            ))),
        }
    }

    pub fn as_boolean(&self) -> Result<bool, RuntimeError> {
        match self {
            Value::Boolean(b) => Ok(*b),
            _ => Err(RuntimeError::TypeError(format!(
                "Expected boolean, got {}",
                self.type_name()
            ))),
        }
    }

    pub fn as_string(&self) -> Result<&str, RuntimeError> {
        match self {
            Value::String(s) => Ok(s),
            _ => Err(RuntimeError::TypeError(format!(
                "Expected string, got {}",
                self.type_name()
            ))),
        }
    }

    pub fn as_array(&self) -> Result<&[Value], RuntimeError> {
        match self {
            Value::Array(a) => Ok(a),
            _ => Err(RuntimeError::TypeError(format!(
                "Expected array, got {}",
                self.type_name()
            ))),
        }
    }

    pub fn as_object(&self) -> Result<&HashMap<String, Value>, RuntimeError> {
        match self {
            Value::Object(o) => Ok(o),
            _ => Err(RuntimeError::TypeError(format!(
                "Expected object, got {}",
                self.type_name()
            ))),
        }
    }

    pub fn as_function(&self) -> Result<&FunctionValue, RuntimeError> {
        match self {
            Value::Function(f) => Ok(f),
            _ => Err(RuntimeError::TypeError(format!(
                "Expected function, got {}",
                self.type_name()
            ))),
        }
    }

    pub fn to_json_string(&self) -> Result<String, RuntimeError> {
        // Simple JSON serialization - in real impl, use canonical JSON
        match self {
            Value::Number(n) => Ok(n.to_string()),
            Value::Boolean(b) => Ok(b.to_string()),
            Value::String(s) => Ok(format!("\"{}\"", s.replace("\"", "\\\""))),
            Value::Decimal(d) => Ok(d.to_string()),
            Value::Array(a) => {
                let items: Result<Vec<String>, _> = a.iter().map(|v| v.to_json_string()).collect();
                Ok(format!("[{}]", items?.join(",")))
            }
            Value::Object(o) => {
                let mut pairs = Vec::new();
                for (k, v) in o {
                    pairs.push(format!("\"{}\":{}", k, v.to_json_string()?));
                }
                Ok(format!("{{{}}}", pairs.join(",")))
            }
            Value::Function(_) => Err(RuntimeError::TypeError(
                "Cannot serialize function".to_string(),
            )),
            Value::Null => Ok("null".to_string()),
        }
    }
}
