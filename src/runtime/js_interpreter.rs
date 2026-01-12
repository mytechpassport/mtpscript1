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

/// Return signal for proper return handling
#[derive(Debug)]
pub struct ReturnValue(pub Value);

pub struct JsInterpreter {
    globals: HashMap<String, Value>,
    locals: Vec<HashMap<String, Value>>,
    functions: HashMap<String, StoredFunction>,
}

#[derive(Clone)]
pub struct StoredFunction {
    pub params: Vec<String>,
    pub body: Vec<JsStmt>,
}

impl JsInterpreter {
    pub fn new() -> Self {
        let mut globals = HashMap::new();

        // Add console object with log method
        let mut console_obj = HashMap::new();
        console_obj.insert(
            "log".to_string(),
            Value::Function(crate::runtime::value::FunctionValue {
                name: "log".to_string(),
                params: vec!["message".to_string()],
                body: vec![],
            }),
        );
        globals.insert("console".to_string(), Value::Object(console_obj));

        JsInterpreter {
            globals,
            locals: vec![HashMap::new()],
            functions: HashMap::new(),
        }
    }

    pub fn execute(&mut self, stmts: &[JsStmt]) -> Result<Value, MtpError> {
        let mut result = Value::Null;

        for stmt in stmts {
            match self.execute_stmt(stmt) {
                Ok(val) => result = val,
                Err(MtpError::ReturnSignal(val)) => return Ok(val),
                Err(e) => return Err(e),
            }
        }

        Ok(result)
    }

    fn execute_stmt(&mut self, stmt: &JsStmt) -> Result<Value, MtpError> {
        match stmt {
            JsStmt::Expr(expr) => self.evaluate(expr),
            JsStmt::Return(Some(expr)) => {
                let value = self.evaluate(expr)?;
                Err(MtpError::ReturnSignal(value))
            }
            JsStmt::Return(None) => Err(MtpError::ReturnSignal(Value::Null)),
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
                // Store function for later execution
                self.functions.insert(
                    name.clone(),
                    StoredFunction {
                        params: params.clone(),
                        body: body.clone(),
                    },
                );

                let func = Value::Function(crate::runtime::value::FunctionValue {
                    name: name.clone(),
                    params: params.clone(),
                    body: vec![],
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
                // Evaluate function expression
                let func_val = self.evaluate(func)?;
                let mut arg_vals = Vec::new();
                for arg in args {
                    arg_vals.push(self.evaluate(arg)?);
                }
                self.call_function(&func_val, &arg_vals, func)
            }
            JsExpr::Function {
                name,
                params,
                body,
            } => {
                // Return function value (anonymous or named)
                let func_name = name.clone().unwrap_or_else(|| "anonymous".to_string());
                Ok(Value::Function(crate::runtime::value::FunctionValue {
                    name: func_name,
                    params: params.clone(),
                    body: vec![],
                }))
            }
            JsExpr::Return(expr) => {
                let value = self.evaluate(expr)?;
                Err(MtpError::ReturnSignal(value))
            }
            JsExpr::If {
                cond,
                then_branch,
                else_branch,
            } => {
                let cond_val = self.evaluate(cond)?;
                if self.is_truthy(&cond_val) {
                    let mut result = Value::Null;
                    for stmt in then_branch {
                        match self.execute_stmt(stmt) {
                            Ok(val) => result = val,
                            Err(e) => return Err(e),
                        }
                    }
                    Ok(result)
                } else if let Some(else_branch) = else_branch {
                    let mut result = Value::Null;
                    for stmt in else_branch {
                        match self.execute_stmt(stmt) {
                            Ok(val) => result = val,
                            Err(e) => return Err(e),
                        }
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
                self.locals.last_mut().unwrap().insert(name.clone(), value.clone());
                Ok(value)
            }
            JsExpr::Assign { target, value } => {
                let val = self.evaluate(value)?;
                match &**target {
                    JsExpr::Ident(name) => {
                        // Assign to variable in nearest scope
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
                    JsExpr::Member { object, property } => {
                        let obj_val = self.evaluate(object)?;
                        let prop_val = self.evaluate(property)?;
                        // Object mutation would go here (not supported in pure MTPScript)
                        Err(MtpError::RuntimeError("Object mutation not supported".into()))
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
                            Ok(Value::Null)
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
                    (Value::String(s), Value::String(prop)) if prop == "length" => {
                        Ok(Value::Number(s.len() as i64))
                    }
                    (Value::Array(arr), Value::String(prop)) if prop == "length" => {
                        Ok(Value::Number(arr.len() as i64))
                    }
                    _ => Err(MtpError::RuntimeError("Invalid member access".into())),
                }
            }
        }
    }

    fn eval_binary_op(&self, op: &str, left: &Value, right: &Value) -> Result<Value, MtpError> {
        match op {
            "+" => match (left, right) {
                (Value::Number(a), Value::Number(b)) => {
                    // Checked addition for overflow
                    a.checked_add(*b)
                        .map(Value::Number)
                        .ok_or_else(|| MtpError::RuntimeError("Integer overflow".into()))
                }
                (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
                (Value::String(a), b) => Ok(Value::String(format!("{}{}", a, b))),
                (a, Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
                _ => Err(MtpError::RuntimeError("Invalid operands for +".into())),
            },
            "-" => match (left, right) {
                (Value::Number(a), Value::Number(b)) => {
                    a.checked_sub(*b)
                        .map(Value::Number)
                        .ok_or_else(|| MtpError::RuntimeError("Integer underflow".into()))
                }
                _ => Err(MtpError::RuntimeError("Invalid operands for -".into())),
            },
            "*" => match (left, right) {
                (Value::Number(a), Value::Number(b)) => {
                    a.checked_mul(*b)
                        .map(Value::Number)
                        .ok_or_else(|| MtpError::RuntimeError("Integer overflow".into()))
                }
                _ => Err(MtpError::RuntimeError("Invalid operands for *".into())),
            },
            "/" => match (left, right) {
                (Value::Number(a), Value::Number(b)) => {
                    if *b == 0 {
                        Err(MtpError::RuntimeError("Division by zero".into()))
                    } else {
                        Ok(Value::Number(a / b))
                    }
                }
                _ => Err(MtpError::RuntimeError("Invalid operands for /".into())),
            },
            "%" => match (left, right) {
                (Value::Number(a), Value::Number(b)) => {
                    if *b == 0 {
                        Err(MtpError::RuntimeError("Division by zero".into()))
                    } else {
                        Ok(Value::Number(a % b))
                    }
                }
                _ => Err(MtpError::RuntimeError("Invalid operands for %".into())),
            },
            "==" => Ok(Value::Boolean(self.values_equal(left, right))),
            "===" => Ok(Value::Boolean(self.values_strict_equal(left, right))),
            "!=" => Ok(Value::Boolean(!self.values_equal(left, right))),
            "!==" => Ok(Value::Boolean(!self.values_strict_equal(left, right))),
            "<" => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a < b)),
                (Value::String(a), Value::String(b)) => Ok(Value::Boolean(a < b)),
                _ => Err(MtpError::RuntimeError("Invalid operands for <".into())),
            },
            ">" => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a > b)),
                (Value::String(a), Value::String(b)) => Ok(Value::Boolean(a > b)),
                _ => Err(MtpError::RuntimeError("Invalid operands for >".into())),
            },
            "<=" => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a <= b)),
                (Value::String(a), Value::String(b)) => Ok(Value::Boolean(a <= b)),
                _ => Err(MtpError::RuntimeError("Invalid operands for <=".into())),
            },
            ">=" => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a >= b)),
                (Value::String(a), Value::String(b)) => Ok(Value::Boolean(a >= b)),
                _ => Err(MtpError::RuntimeError("Invalid operands for >=".into())),
            },
            "&&" => {
                if !self.is_truthy(left) {
                    Ok(left.clone())
                } else {
                    Ok(right.clone())
                }
            }
            "||" => {
                if self.is_truthy(left) {
                    Ok(left.clone())
                } else {
                    Ok(right.clone())
                }
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
                _ => Err(MtpError::RuntimeError("Invalid operand for unary -".into())),
            },
            "+" => match val {
                Value::Number(n) => Ok(Value::Number(*n)),
                _ => Err(MtpError::RuntimeError("Invalid operand for unary +".into())),
            },
            "!" => Ok(Value::Boolean(!self.is_truthy(val))),
            "typeof" => {
                let type_str = match val {
                    Value::Number(_) => "number",
                    Value::String(_) => "string",
                    Value::Boolean(_) => "boolean",
                    Value::Null => "object", // JS quirk
                    Value::Array(_) => "object",
                    Value::Object(_) => "object",
                    Value::Function(_) => "function",
                };
                Ok(Value::String(type_str.to_string()))
            }
            _ => Err(MtpError::RuntimeError(format!(
                "Unknown unary operator: {}",
                op
            ))),
        }
    }

    fn call_function(&mut self, func: &Value, args: &[Value], func_expr: &JsExpr) -> Result<Value, MtpError> {
        match func {
            Value::Function(func_val) => {
                // Check if it's console.log
                if func_val.name == "log" {
                    for arg in args {
                        println!("{}", arg);
                    }
                    return Ok(Value::Null);
                }

                // Look up stored function body
                let stored = self.functions.get(&func_val.name).cloned();

                if let Some(stored_func) = stored {
                    // Create new scope with parameters bound
                    let mut new_scope = HashMap::new();
                    for (i, param) in stored_func.params.iter().enumerate() {
                        let value = if i < args.len() {
                            args[i].clone()
                        } else {
                            Value::Null
                        };
                        new_scope.insert(param.clone(), value);
                    }

                    self.locals.push(new_scope);

                    // Execute function body
                    let result = match self.execute(&stored_func.body) {
                        Ok(val) => Ok(val),
                        Err(MtpError::ReturnSignal(val)) => Ok(val),
                        Err(e) => Err(e),
                    };

                    self.locals.pop();
                    result
                } else {
                    // Function not found in stored functions
                    Err(MtpError::RuntimeError(format!(
                        "Function '{}' not defined",
                        func_val.name
                    )))
                }
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
            Value::Array(arr) => true, // Arrays are always truthy in JS
            Value::Object(_) => true,
            Value::Function(_) => true,
        }
    }

    fn values_equal(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Null, Value::Null) => true,
            // Type coercion
            (Value::Number(n), Value::String(s)) | (Value::String(s), Value::Number(n)) => {
                if let Ok(parsed) = s.parse::<i64>() {
                    *n == parsed
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn values_strict_equal(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Null, Value::Null) => true,
            (Value::Array(a), Value::Array(b)) => {
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| self.values_strict_equal(x, y))
            }
            _ => false,
        }
    }
}

impl Default for JsInterpreter {
    fn default() -> Self {
        Self::new()
    }
}
