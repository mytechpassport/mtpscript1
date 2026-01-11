use std::collections::HashMap;

use crate::errors::runtime::RuntimeError;
use crate::gas::counter::GasCounter;
use crate::runtime::value::{FunctionValue, Value};

// Simple JS AST for subset interpreter
#[derive(Debug, Clone)]
pub enum JsExpr {
    // Expressions
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

    // Statements
    Assign(String, Box<JsExpr>),
    Block(Vec<JsExpr>),
    Return(Option<Box<JsExpr>>),

    // Legacy - function as expression (anonymous)
    Function(String, Vec<String>, Box<JsExpr>),

    // New statement types for JS subset parsing
    /// A program: sequence of statements to execute
    Program(Vec<JsExpr>),

    /// A function declaration: function name(params) { body }
    FunctionDecl {
        name: String,
        params: Vec<String>,
        body: Box<JsExpr>,
    },

    /// Const declaration: const name = value;
    Const {
        name: String,
        value: Box<JsExpr>,
    },

    /// Expression statement (an expression followed by semicolon)
    ExprStmt(Box<JsExpr>),
}

/// Stored function body - decoupled from Value to avoid circular dependencies
#[derive(Debug, Clone)]
pub struct StoredFunction {
    pub params: Vec<String>,
    pub body: Box<JsExpr>,
}

#[derive(Debug)]
pub struct Interpreter {
    pub global_scope: HashMap<String, Value>,
    pub gas_counter: GasCounter,
    pub heap: Vec<Value>, // Simple heap for objects/arrays
    pub builtins: HashMap<String, fn(Value) -> Result<Value, String>>,
    /// Storage for function bodies - keyed by function name
    pub function_bodies: HashMap<String, StoredFunction>,
    /// Execution timeout in milliseconds
    pub timeout_ms: u64,
    /// Start time for timeout checking
    pub start_time: std::time::Instant,
}

impl Interpreter {
    pub fn new() -> Self {
        let builtins = crate::effects::builtins::get_builtin_functions();
        let mut interpreter = Self {
            global_scope: HashMap::new(),
            gas_counter: GasCounter::new(crate::runtime::get_gas_limit()),
            heap: Vec::new(),
            builtins,
            function_bodies: HashMap::new(),
            timeout_ms: 30_000, // 30 seconds default
            start_time: std::time::Instant::now(),
        };

        // Inject built-in objects (JSON, Decimal, etc.)
        interpreter.inject_builtin_objects();

        interpreter
    }

    /// Inject built-in objects like JSON into global scope
    fn inject_builtin_objects(&mut self) {
        // Create JSON object with methods
        // Note: We use special handling for these in eval_call
        let mut json_obj = HashMap::new();
        json_obj.insert(
            "parse".to_string(),
            Value::Function(FunctionValue {
                name: Some("JSON.parse".to_string()),
                params: vec!["s".to_string()],
                closure: HashMap::new(),
            }),
        );
        json_obj.insert(
            "stringify".to_string(),
            Value::Function(FunctionValue {
                name: Some("JSON.stringify".to_string()),
                params: vec!["v".to_string()],
                closure: HashMap::new(),
            }),
        );
        json_obj.insert(
            "stringifyCanonical".to_string(),
            Value::Function(FunctionValue {
                name: Some("JSON.stringifyCanonical".to_string()),
                params: vec!["v".to_string()],
                closure: HashMap::new(),
            }),
        );
        self.global_scope
            .insert("JSON".to_string(), Value::Object(json_obj));

        // Create Decimal object with methods
        let mut decimal_obj = HashMap::new();
        decimal_obj.insert(
            "fromString".to_string(),
            Value::Function(FunctionValue {
                name: Some("Decimal.fromString".to_string()),
                params: vec!["s".to_string()],
                closure: HashMap::new(),
            }),
        );
        decimal_obj.insert(
            "toString".to_string(),
            Value::Function(FunctionValue {
                name: Some("Decimal.toString".to_string()),
                params: vec!["d".to_string()],
                closure: HashMap::new(),
            }),
        );
        self.global_scope
            .insert("Decimal".to_string(), Value::Object(decimal_obj));
    }

    pub fn set_gas_limit(&mut self, limit: u64) {
        self.gas_counter = GasCounter::new(limit);
    }

    pub fn set_timeout(&mut self, timeout_ms: u64) {
        self.timeout_ms = timeout_ms;
        self.start_time = std::time::Instant::now();
    }

    pub fn gas_used(&self) -> u64 {
        self.gas_counter.used()
    }

    pub fn call_global_function(
        &mut self,
        name: &str,
        args: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        if let Some(builtin) = self.builtins.get(name) {
            // Built-in functions take one argument (for now, simple case)
            if args.len() != 1 {
                return Err(RuntimeError::ValueError(format!(
                    "Builtin {} expects 1 argument",
                    name
                )));
            }
            builtin(args[0].clone()).map_err(RuntimeError::ValueError)
        } else {
            let func_val = self
                .global_scope
                .get(name)
                .ok_or_else(|| RuntimeError::ValueError(format!("Function {} not found", name)))?
                .clone();

            self.call_function(&func_val, args)
        }
    }

    pub fn eval(&mut self, expr: &JsExpr) -> Result<Value, RuntimeError> {
        self.eval_expr(expr, &mut HashMap::new())
    }

    /// Execute a string of JS code
    ///
    /// Parses the JS subset code and evaluates it, returning the result
    /// as a JSON string (or the raw value for non-JSON results).
    pub fn execute(&mut self, code: &str) -> Result<Value, RuntimeError> {
        use crate::runtime::js_parser::parse_js_program;

        // Parse the JS code into AST
        let ast = parse_js_program(code)?;

        // Evaluate the program
        self.eval(&ast)
    }

    /// Execute JS code and return result as JSON string
    pub fn execute_to_json(&mut self, code: &str) -> Result<String, RuntimeError> {
        let result = self.execute(code)?;
        result.to_json_string()
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
            JsExpr::Function(name, params, body) => {
                self.gas_counter.consume(5)?;
                // Store function body for later execution
                self.function_bodies.insert(
                    name.clone(),
                    StoredFunction {
                        params: params.clone(),
                        body: body.clone(),
                    },
                );
                let func = Value::Function(FunctionValue {
                    name: Some(name.clone()),
                    params: params.clone(),
                    closure: local_scope.clone(),
                });
                self.global_scope.insert(name.clone(), func.clone());
                Ok(func)
            }

            // New statement types
            JsExpr::Program(statements) => {
                let mut result = Value::Null;
                for stmt in statements {
                    result = self.eval_expr(stmt, local_scope)?;
                }
                Ok(result)
            }

            JsExpr::FunctionDecl { name, params, body } => {
                self.gas_counter.consume(5)?;
                // Store function body for later execution
                self.function_bodies.insert(
                    name.clone(),
                    StoredFunction {
                        params: params.clone(),
                        body: body.clone(),
                    },
                );
                let func = Value::Function(FunctionValue {
                    name: Some(name.clone()),
                    params: params.clone(),
                    closure: local_scope.clone(),
                });
                self.global_scope.insert(name.clone(), func.clone());
                Ok(func)
            }

            JsExpr::Const { name, value } => {
                self.gas_counter.consume(1)?;
                let val = self.eval_expr(value, local_scope)?;
                local_scope.insert(name.clone(), val.clone());
                Ok(val)
            }

            JsExpr::ExprStmt(expr) => self.eval_expr(expr, local_scope),
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
            "==" | "===" => Ok(Value::Boolean(left == right)),
            "!=" | "!==" => Ok(Value::Boolean(left != right)),
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
                // Get function name
                let func_name = func.name.as_ref().ok_or_else(|| {
                    RuntimeError::ValueError("Anonymous functions not supported".to_string())
                })?;

                // Check if this is a builtin function first
                if let Some(builtin) = self.builtins.get(func_name).cloned() {
                    // Builtins take single argument for now
                    if args.len() != 1 {
                        return Err(RuntimeError::ValueError(format!(
                            "Builtin {} expects 1 argument, got {}",
                            func_name,
                            args.len()
                        )));
                    }
                    self.gas_counter.consume(10)?; // Builtin call cost
                    return builtin(args[0].clone()).map_err(RuntimeError::ValueError);
                }

                // Not a builtin - check argument count against params
                if args.len() != func.params.len() {
                    return Err(RuntimeError::ValueError(format!(
                        "Expected {} arguments, got {}",
                        func.params.len(),
                        args.len()
                    )));
                }

                // Set up local scope with closure and arguments
                let mut local_scope = func.closure.clone();
                for (param, arg) in func.params.iter().zip(args) {
                    local_scope.insert(param.clone(), arg);
                }

                // Clone the body to avoid borrow checker issues
                let body = self
                    .function_bodies
                    .get(func_name)
                    .ok_or_else(|| {
                        RuntimeError::ValueError(format!(
                            "Function body not found for: {}",
                            func_name
                        ))
                    })?
                    .body
                    .clone();

                // Consume function call gas cost
                self.gas_counter.consume(5)?;

                // Execute the function body
                self.eval_expr(&body, &mut local_scope)
            }
            _ => Err(RuntimeError::TypeError(
                "Cannot call non-function".to_string(),
            )),
        }
    }
}
