use crate::errors::MtpError;
use crate::runtime::value::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum JsExpr {
    Literal(Value),
    Ident(String),
    BinaryOp {
        op: String,
        left: Box<JsExpr>,
        right: Box<JsExpr>,
    },
    UnaryOp {
        op: String,
        expr: Box<JsExpr>,
    },
    Call {
        func: Box<JsExpr>,
        args: Vec<JsExpr>,
    },
    Function {
        name: Option<String>,
        params: Vec<String>,
        body: Vec<JsStmt>,
    },
    Return(Box<JsExpr>),
    If {
        cond: Box<JsExpr>,
        then_branch: Vec<JsStmt>,
        else_branch: Option<Vec<JsStmt>>,
    },
    VarDecl {
        name: String,
        init: Option<Box<JsExpr>>,
    },
    Assign {
        target: Box<JsExpr>,
        value: Box<JsExpr>,
    },
    Object(HashMap<String, JsExpr>),
    Array(Vec<JsExpr>),
    Member {
        object: Box<JsExpr>,
        property: Box<JsExpr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum JsStmt {
    Expr(JsExpr),
    Return(Option<JsExpr>),
    If {
        cond: JsExpr,
        then_branch: Vec<JsStmt>,
        else_branch: Option<Vec<JsStmt>>,
    },
    VarDecl {
        name: String,
        init: Option<JsExpr>,
    },
    Function {
        name: String,
        params: Vec<String>,
        body: Vec<JsStmt>,
    },
    Block(Vec<JsStmt>),
}

pub struct JsInterpreter {
    globals: HashMap<String, Value>,
    locals: Vec<HashMap<String, Value>>,
}

impl JsInterpreter {
    pub fn new() -> Self {
        let mut globals = HashMap::new();

        // Add built-in functions
        globals.insert(
            "console.log".to_string(),
            Value::Function(crate::runtime::value::FunctionValue {
                name: "console.log".to_string(),
                params: vec!["message".to_string()],
                body: vec![], // Placeholder
            }),
        );

        JsInterpreter {
            globals,
            locals: vec![HashMap::new()],
        }
    }

    pub fn execute(&mut self, stmts: &[JsStmt]) -> Result<Value, MtpError> {
        let mut result = Value::Null;

        for stmt in stmts {
            result = self.execute_stmt(stmt)?;
        }

        Ok(result)
    }

    fn execute_stmt(&mut self, stmt: &JsStmt) -> Result<Value, MtpError> {
        match stmt {
            JsStmt::Expr(expr) => self.evaluate(expr),
            JsStmt::Return(Some(expr)) => {
                let value = self.evaluate(expr)?;
                Err(MtpError::RuntimeError(format!("Return: {:?}", value))) // Placeholder for return handling
            }
            JsStmt::Return(None) => Ok(Value::Null),
            JsStmt::If {
                cond,
                then_branch,
                else_branch,
            } => {
                let cond_val = self.evaluate(cond)?;
                if self.is_truthy(&cond_val) {
                    self.execute(then_branch)
                } else if let Some(else_branch) = else_branch {
                    self.execute(else_branch)
                } else {
                    Ok(Value::Null)
                }
            }
            JsStmt::VarDecl { name, init } => {
                let value = if let Some(expr) = init {
                    self.evaluate(expr)?
                } else {
                    Value::Null
                };
                self.locals.last_mut().unwrap().insert(name.clone(), value);
                Ok(Value::Null)
            }
            JsStmt::Function { name, params, body } => {
                let func = Value::Function(crate::runtime::value::FunctionValue {
                    name: name.clone(),
                    params: params.clone(),
                    body: vec![], // In real impl, serialize body
                });
                self.globals.insert(name.clone(), func);
                Ok(Value::Null)
            }
            JsStmt::Block(stmts) => {
                self.locals.push(HashMap::new());
                let result = self.execute(stmts);
                self.locals.pop();
                result
            }
        }
    }

    fn evaluate(&mut self, expr: &JsExpr) -> Result<Value, MtpError> {
        match expr {
            JsExpr::Literal(val) => Ok(val.clone()),
            JsExpr::Ident(name) => {
                // Look up in locals first, then globals
                for scope in self.locals.iter().rev() {
                    if let Some(val) = scope.get(name) {
                        return Ok(val.clone());
                    }
                }
                if let Some(val) = self.globals.get(name) {
                    Ok(val.clone())
                } else {
                    Err(MtpError::RuntimeError(format!(
                        "Undefined variable: {}",
                        name
                    )))
                }
            }
            JsExpr::BinaryOp { op, left, right } => {
                let left_val = self.evaluate(left)?;
                let right_val = self.evaluate(right)?;
                self.eval_binary_op(op, &left_val, &right_val)
            }
            JsExpr::UnaryOp { op, expr } => {
                let val = self.evaluate(expr)?;
                self.eval_unary_op(op, &val)
            }
            JsExpr::Call { func, args } => {
                let func_val = self.evaluate(func)?;
                let mut arg_vals = Vec::new();
                for arg in args {
                    arg_vals.push(self.evaluate(arg)?);
                }
                self.call_function(&func_val, &arg_vals)
            }
            JsExpr::Function {
                name: _,
                params: _,
                body: _,
            } => {
                // Return function value
                Ok(Value::Function(crate::runtime::value::FunctionValue {
                    name: "anonymous".to_string(),
                    params: vec![],
                    body: vec![],
                }))
            }
            JsExpr::Return(expr) => self.evaluate(expr),
            JsExpr::If {
                cond,
                then_branch,
                else_branch,
            } => {
                let cond_val = self.evaluate(cond)?;
                if self.is_truthy(&cond_val) {
                    let mut result = Value::Null;
                    for stmt in then_branch {
                        result = self.execute_stmt(stmt)?;
                    }
                    Ok(result)
                } else if let Some(else_branch) = else_branch {
                    let mut result = Value::Null;
                    for stmt in else_branch {
                        result = self.execute_stmt(stmt)?;
                    }
                    Ok(result)
                } else {
                    Ok(Value::Null)
                }
            }
            JsExpr::VarDecl { name, init } => {
                let value = if let Some(expr) = init {
                    self.evaluate(expr)?
                } else {
                    Value::Null
                };
                self.locals.last_mut().unwrap().insert(name.clone(), value);
                Ok(value)
            }
            JsExpr::Assign { target, value } => {
                let val = self.evaluate(value)?;
                match &**target {
                    JsExpr::Ident(name) => {
                        // Assign to variable
                        for scope in self.locals.iter_mut().rev() {
                            if scope.contains_key(name) {
                                scope.insert(name.clone(), val.clone());
                                return Ok(val);
                            }
                        }
                        if self.globals.contains_key(name) {
                            self.globals.insert(name.clone(), val.clone());
                            return Ok(val);
                        }
                        Err(MtpError::RuntimeError(format!(
                            "Undefined variable: {}",
                            name
                        )))
                    }
                    _ => Err(MtpError::RuntimeError("Invalid assignment target".into())),
                }
            }
            JsExpr::Object(fields) => {
                let mut obj = HashMap::new();
                for (key, expr) in fields {
                    obj.insert(key.clone(), self.evaluate(expr)?);
                }
                Ok(Value::Object(obj))
            }
            JsExpr::Array(elements) => {
                let mut arr = Vec::new();
                for elem in elements {
                    arr.push(self.evaluate(elem)?);
                }
                Ok(Value::Array(arr))
            }
            JsExpr::Member { object, property } => {
                let obj_val = self.evaluate(object)?;
                let prop_val = self.evaluate(property)?;

                match (&obj_val, &prop_val) {
                    (Value::Object(obj), Value::String(prop)) => {
                        if let Some(val) = obj.get(prop) {
                            Ok(val.clone())
                        } else {
                            Err(MtpError::RuntimeError(format!(
                                "Property '{}' not found",
                                prop
                            )))
                        }
                    }
                    (Value::Array(arr), Value::Number(idx)) => {
                        let idx = *idx as usize;
                        if idx < arr.len() {
                            Ok(arr[idx].clone())
                        } else {
                            Err(MtpError::RuntimeError("Array index out of bounds".into()))
                        }
                    }
                    _ => Err(MtpError::RuntimeError("Invalid member access".into())),
                }
            }
        }
    }

    fn eval_binary_op(&self, op: &str, left: &Value, right: &Value) -> Result<Value, MtpError> {
        match op {
            "+" => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
                (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
                _ => Err(MtpError::RuntimeError("Invalid operands for +".into())),
            },
            "-" => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)),
                _ => Err(MtpError::RuntimeError("Invalid operands for -".into())),
            },
            "*" => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),
                _ => Err(MtpError::RuntimeError("Invalid operands for *".into())),
            },
            "/" => match (left, right) {
                (Value::Number(a), Value::Number(b)) => {
                    if *b != 0 {
                        Ok(Value::Number(a / b))
                    } else {
                        Err(MtpError::RuntimeError("Division by zero".into()))
                    }
                }
                _ => Err(MtpError::RuntimeError("Invalid operands for /".into())),
            },
            "==" => Ok(Value::Boolean(left == right)),
            "!=" => Ok(Value::Boolean(left != right)),
            "<" => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a < b)),
                _ => Err(MtpError::RuntimeError("Invalid operands for <".into())),
            },
            ">" => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a > b)),
                _ => Err(MtpError::RuntimeError("Invalid operands for >".into())),
            },
            "<=" => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a <= b)),
                _ => Err(MtpError::RuntimeError("Invalid operands for <=".into())),
            },
            ">=" => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a >= b)),
                _ => Err(MtpError::RuntimeError("Invalid operands for >=".into())),
            },
            "&&" => {
                let left_bool = self.is_truthy(left);
                let right_bool = self.is_truthy(right);
                Ok(Value::Boolean(left_bool && right_bool))
            }
            "||" => {
                let left_bool = self.is_truthy(left);
                let right_bool = self.is_truthy(right);
                Ok(Value::Boolean(left_bool || right_bool))
            }
            _ => Err(MtpError::RuntimeError(format!(
                "Unknown binary operator: {}",
                op
            ))),
        }
    }

    fn eval_unary_op(&self, op: &str, val: &Value) -> Result<Value, MtpError> {
        match op {
            "-" => match val {
                Value::Number(n) => Ok(Value::Number(-n)),
                _ => Err(MtpError::RuntimeError("Invalid operand for -".into())),
            },
            "!" => Ok(Value::Boolean(!self.is_truthy(val))),
            _ => Err(MtpError::RuntimeError(format!(
                "Unknown unary operator: {}",
                op
            ))),
        }
    }

    fn call_function(&mut self, func: &Value, args: &[Value]) -> Result<Value, MtpError> {
        match func {
            Value::Function(func_val) => {
                // Create new scope
                let mut new_scope = HashMap::new();
                for (i, param) in func_val.params.iter().enumerate() {
                    if i < args.len() {
                        new_scope.insert(param.clone(), args[i].clone());
                    }
                }
                self.locals.push(new_scope);

                // Execute function body (placeholder)
                let result = Value::String("function_result".to_string());

                self.locals.pop();
                Ok(result)
            }
            _ => Err(MtpError::RuntimeError("Cannot call non-function".into())),
        }
    }

    fn is_truthy(&self, val: &Value) -> bool {
        match val {
            Value::Boolean(b) => *b,
            Value::Null => false,
            Value::Number(n) => *n != 0,
            Value::String(s) => !s.is_empty(),
            Value::Array(arr) => !arr.is_empty(),
            Value::Object(obj) => !obj.is_empty(),
            Value::Function(_) => true,
        }
    }
}
