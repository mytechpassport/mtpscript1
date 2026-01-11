use std::collections::HashMap;

use crate::errors::runtime::RuntimeError;
use crate::gas::counter::GasCounter;
use crate::runtime::value::{FunctionValue, Value};

// Simple JS AST for subset interpreter
#[derive(Debug, Clone)]
pub enum JsExpr {
    Literal(Value),
    Ident(String),
    Array(Vec<JsExpr>),
    Object(Vec<(String, JsExpr)>),
    BinOp(String, Box<JsExpr>, Box<JsExpr>),
    UnaryOp(String, Box<JsExpr>),
    Call(Box<JsExpr>, Vec<JsExpr>),
    Member(Box<JsExpr>, String),
    Index(Box<JsExpr>, Box<JsExpr>),
    If(Box<JsExpr>, Box<JsExpr>, Option<Box<JsExpr>>),
    Assign(String, Box<JsExpr>),
    Block(Vec<JsExpr>),
    Return(Option<Box<JsExpr>>),
    Function(String, Vec<String>, Box<JsExpr>),
}

#[derive(Debug)]
pub struct Interpreter {
    pub global_scope: HashMap<String, Value>,
    pub gas_counter: GasCounter,
    pub heap: Vec<Value>, // Simple heap for objects/arrays
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            global_scope: HashMap::new(),
            gas_counter: GasCounter::new(10_000_000), // Default gas limit
            heap: Vec::new(),
        }
    }

    pub fn set_gas_limit(&mut self, limit: u64) {
        self.gas_counter = GasCounter::new(limit);
    }

    pub fn gas_used(&self) -> u64 {
        self.gas_counter.used()
    }

    pub fn call_global_function(
        &mut self,
        name: &str,
        args: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        let func_val = self
            .global_scope
            .get(name)
            .ok_or_else(|| RuntimeError::ValueError(format!("Function {} not found", name)))?
            .clone();

        self.call_function(&func_val, args)
    }

    pub fn eval(&mut self, expr: &JsExpr) -> Result<Value, RuntimeError> {
        self.eval_expr(expr, &mut HashMap::new())
    }

    /// Execute a string of JS code (simplified - assumes it's a single expression)
    pub fn execute(&mut self, code: &str) -> Result<String, RuntimeError> {
        // This is a simplified implementation
        // In reality, we'd need a proper JS parser
        // For now, just return a placeholder
        Ok(format!("Executed: {}", code))
    }

    fn eval_expr(
        &mut self,
        expr: &JsExpr,
        local_scope: &mut HashMap<String, Value>,
    ) -> Result<Value, RuntimeError> {
        self.gas_counter.consume(1)?; // Base cost for evaluation

        match expr {
            JsExpr::Literal(val) => {
                self.gas_counter.consume(1)?;
                Ok(val.clone())
            }
            JsExpr::Ident(name) => {
                self.gas_counter.consume(1)?;
                if let Some(val) = local_scope.get(name) {
                    Ok(val.clone())
                } else if let Some(val) = self.global_scope.get(name) {
                    Ok(val.clone())
                } else {
                    Err(RuntimeError::ValueError(format!(
                        "Undefined variable: {}",
                        name
                    )))
                }
            }
            JsExpr::Array(elements) => {
                self.gas_counter.consume(5)?;
                let mut vals = Vec::new();
                for elem in elements {
                    vals.push(self.eval_expr(elem, local_scope)?);
                }
                Ok(Value::Array(vals))
            }
            JsExpr::Object(fields) => {
                self.gas_counter.consume(10)?;
                let mut obj = HashMap::new();
                for (key, val_expr) in fields {
                    let val = self.eval_expr(val_expr, local_scope)?;
                    obj.insert(key.clone(), val);
                }
                Ok(Value::Object(obj))
            }
            JsExpr::BinOp(op, left, right) => {
                self.gas_counter.consume(2)?;
                let left_val = self.eval_expr(left, local_scope)?;
                let right_val = self.eval_expr(right, local_scope)?;
                self.eval_binop(op, &left_val, &right_val)
            }
            JsExpr::UnaryOp(op, expr) => {
                self.gas_counter.consume(1)?;
                let val = self.eval_expr(expr, local_scope)?;
                self.eval_unaryop(op, &val)
            }
            JsExpr::Call(func_expr, args) => {
                self.gas_counter.consume(5)?;
                let func_val = self.eval_expr(func_expr, local_scope)?;
                let mut arg_vals = Vec::new();
                for arg in args {
                    arg_vals.push(self.eval_expr(arg, local_scope)?);
                }
                self.call_function(&func_val, arg_vals)
            }
            JsExpr::Member(obj_expr, prop) => {
                self.gas_counter.consume(1)?;
                let obj_val = self.eval_expr(obj_expr, local_scope)?;
                match obj_val {
                    Value::Object(ref obj) => Ok(obj.get(prop).cloned().unwrap_or(Value::Null)),
                    _ => Err(RuntimeError::TypeError(
                        "Cannot access property of non-object".to_string(),
                    )),
                }
            }
            JsExpr::Index(arr_expr, idx_expr) => {
                self.gas_counter.consume(1)?;
                let arr_val = self.eval_expr(arr_expr, local_scope)?;
                let idx_val = self.eval_expr(idx_expr, local_scope)?;
                let idx = idx_val.as_number()? as usize;
                match arr_val {
                    Value::Array(ref arr) => Ok(arr.get(idx).cloned().unwrap_or(Value::Null)),
                    Value::String(ref s) => Ok(s
                        .chars()
                        .nth(idx)
                        .map(|c| Value::String(c.to_string()))
                        .unwrap_or(Value::Null)),
                    _ => Err(RuntimeError::TypeError(
                        "Cannot index into non-array/string".to_string(),
                    )),
                }
            }
            JsExpr::If(cond, then_branch, else_branch) => {
                self.gas_counter.consume(1)?;
                let cond_val = self.eval_expr(cond, local_scope)?;
                if cond_val.as_boolean()? {
                    self.eval_expr(then_branch, local_scope)
                } else if let Some(else_expr) = else_branch {
                    self.eval_expr(else_expr, local_scope)
                } else {
                    Ok(Value::Null)
                }
            }
            JsExpr::Assign(name, expr) => {
                self.gas_counter.consume(1)?;
                let val = self.eval_expr(expr, local_scope)?;
                local_scope.insert(name.clone(), val.clone());
                Ok(val)
            }
            JsExpr::Block(stmts) => {
                let mut result = Value::Null;
                for stmt in stmts {
                    result = self.eval_expr(stmt, local_scope)?;
                }
                Ok(result)
            }
            JsExpr::Return(expr) => {
                if let Some(expr) = expr {
                    self.eval_expr(expr, local_scope)
                } else {
                    Ok(Value::Null)
                }
            }
            JsExpr::Function(name, params, _body) => {
                self.gas_counter.consume(5)?;
                let func = Value::Function(FunctionValue {
                    name: Some(name.clone()),
                    params: params.clone(),
                    closure: local_scope.clone(),
                });
                self.global_scope.insert(name.clone(), func.clone());
                Ok(func)
            }
        }
    }

    fn eval_binop(&self, op: &str, left: &Value, right: &Value) -> Result<Value, RuntimeError> {
        match op {
            "+" => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
                (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
                _ => Err(RuntimeError::TypeError(
                    "Invalid operands for +".to_string(),
                )),
            },
            "-" => {
                let a = left.as_number()?;
                let b = right.as_number()?;
                Ok(Value::Number(a - b))
            }
            "*" => {
                let a = left.as_number()?;
                let b = right.as_number()?;
                Ok(Value::Number(a * b))
            }
            "/" => {
                let a = left.as_number()?;
                let b = right.as_number()?;
                if b == 0 {
                    Err(RuntimeError::ValueError("Division by zero".to_string()))
                } else {
                    Ok(Value::Number(a / b))
                }
            }
            "%" => {
                let a = left.as_number()?;
                let b = right.as_number()?;
                Ok(Value::Number(a % b))
            }
            "==" => Ok(Value::Boolean(left == right)),
            "!=" => Ok(Value::Boolean(left != right)),
            "<" => {
                let a = left.as_number()?;
                let b = right.as_number()?;
                Ok(Value::Boolean(a < b))
            }
            ">" => {
                let a = left.as_number()?;
                let b = right.as_number()?;
                Ok(Value::Boolean(a > b))
            }
            "<=" => {
                let a = left.as_number()?;
                let b = right.as_number()?;
                Ok(Value::Boolean(a <= b))
            }
            ">=" => {
                let a = left.as_number()?;
                let b = right.as_number()?;
                Ok(Value::Boolean(a >= b))
            }
            "&&" => {
                let a = left.as_boolean()?;
                let b = right.as_boolean()?;
                Ok(Value::Boolean(a && b))
            }
            "||" => {
                let a = left.as_boolean()?;
                let b = right.as_boolean()?;
                Ok(Value::Boolean(a || b))
            }
            _ => Err(RuntimeError::ValueError(format!(
                "Unknown binary operator: {}",
                op
            ))),
        }
    }

    fn eval_unaryop(&self, op: &str, val: &Value) -> Result<Value, RuntimeError> {
        match op {
            "!" => Ok(Value::Boolean(!val.as_boolean()?)),
            "-" => Ok(Value::Number(-val.as_number()?)),
            _ => Err(RuntimeError::ValueError(format!(
                "Unknown unary operator: {}",
                op
            ))),
        }
    }

    pub fn call_function(
        &mut self,
        func_val: &Value,
        args: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        match func_val {
            Value::Function(func) => {
                if args.len() != func.params.len() {
                    return Err(RuntimeError::ValueError(format!(
                        "Expected {} arguments, got {}",
                        func.params.len(),
                        args.len()
                    )));
                }

                let mut local_scope = func.closure.clone();
                for (param, arg) in func.params.iter().zip(args) {
                    local_scope.insert(param.clone(), arg);
                }

                // Placeholder: functions don't have bodies yet, just return null
                Ok(Value::Null)
            }
            _ => Err(RuntimeError::TypeError(
                "Cannot call non-function".to_string(),
            )),
        }
    }
}
