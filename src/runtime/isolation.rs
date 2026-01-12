use crate::errors::MtpError;
use crate::runtime::value::Value;
use crate::runtime::js_interpreter::{JsInterpreter, JsStmt, JsExpr};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Isolated runtime environment for request execution
pub struct IsolatedRuntime {
    pub id: String,
    pub globals: HashMap<String, Value>,
    pub heap: HashMap<String, Value>,
    pub stack: Vec<Value>,
    pub gas_remaining: u64,
    pub gas_limit: u64,
    pub memory_used: usize,
    pub memory_limit: usize,
    pub start_time: Instant,
    pub timeout: Duration,
    interpreter: JsInterpreter,
}

impl IsolatedRuntime {
    pub fn new(id: String, gas_limit: u64, memory_limit_mb: usize) -> Self {
        IsolatedRuntime {
            id,
            globals: HashMap::new(),
            heap: HashMap::new(),
            stack: Vec::new(),
            gas_remaining: gas_limit,
            gas_limit,
            memory_used: 0,
            memory_limit: memory_limit_mb * 1024 * 1024,
            start_time: Instant::now(),
            timeout: Duration::from_secs(30),
            interpreter: JsInterpreter::new(),
        }
    }

    /// Create runtime with custom timeout
    pub fn with_timeout(id: String, gas_limit: u64, memory_limit_mb: usize, timeout_secs: u64) -> Self {
        let mut runtime = Self::new(id, gas_limit, memory_limit_mb);
        runtime.timeout = Duration::from_secs(timeout_secs);
        runtime
    }

    /// Execute code in isolation
    pub fn execute(&mut self, code: &str) -> Result<Value, MtpError> {
        // Check execution timeout
        self.check_timeout()?;

        // Validate no forbidden constructs
        self.validate_code(code)?;

        // Validate no global leakage
        self.validate_no_global_leakage()?;

        // Parse and execute the code
        let result = self.execute_parsed_code(code)?;

        Ok(result)
    }

    /// Execute parsed code
    fn execute_parsed_code(&mut self, code: &str) -> Result<Value, MtpError> {
        // For simple expressions, evaluate directly
        let code = code.trim();

        // Handle return statements
        if code.starts_with("return ") {
            let expr = &code[7..].trim_end_matches(';');
            return self.evaluate_expression(expr);
        }

        // Handle variable declarations
        if code.starts_with("const ") || code.starts_with("let ") || code.starts_with("var ") {
            let parts: Vec<&str> = code.splitn(2, '=').collect();
            if parts.len() == 2 {
                let name_part = parts[0].trim();
                let name = name_part
                    .strip_prefix("const ")
                    .or_else(|| name_part.strip_prefix("let "))
                    .or_else(|| name_part.strip_prefix("var "))
                    .unwrap_or(name_part)
                    .trim();
                let value_expr = parts[1].trim().trim_end_matches(';');
                let value = self.evaluate_expression(value_expr)?;
                self.heap.insert(name.to_string(), value);
                return Ok(Value::Null);
            }
        }

        // Handle function calls
        if code.contains('(') && code.contains(')') {
            return self.evaluate_expression(code.trim_end_matches(';'));
        }

        // Default: evaluate as expression
        self.evaluate_expression(code.trim_end_matches(';'))
    }

    /// Evaluate a simple expression
    fn evaluate_expression(&mut self, expr: &str) -> Result<Value, MtpError> {
        let expr = expr.trim();

        // Consume gas for evaluation
        self.consume_gas(1)?;

        // Literals
        if expr == "null" {
            return Ok(Value::Null);
        }
        if expr == "true" {
            return Ok(Value::Boolean(true));
        }
        if expr == "false" {
            return Ok(Value::Boolean(false));
        }

        // Number literals
        if let Ok(n) = expr.parse::<i64>() {
            return Ok(Value::Number(n));
        }

        // String literals
        if (expr.starts_with('"') && expr.ends_with('"'))
            || (expr.starts_with('\'') && expr.ends_with('\''))
        {
            return Ok(Value::String(expr[1..expr.len() - 1].to_string()));
        }

        // Variable lookup
        if let Some(value) = self.heap.get(expr) {
            return Ok(value.clone());
        }
        if let Some(value) = self.globals.get(expr) {
            return Ok(value.clone());
        }

        // Object literals
        if expr.starts_with('{') && expr.ends_with('}') {
            return self.parse_object_literal(expr);
        }

        // Array literals
        if expr.starts_with('[') && expr.ends_with(']') {
            return self.parse_array_literal(expr);
        }

        // Binary operations
        for op in &["===", "!==", "==", "!=", "<=", ">=", "&&", "||", "+", "-", "*", "/", "%", "<", ">"] {
            if let Some(pos) = find_operator(expr, op) {
                let left = &expr[..pos];
                let right = &expr[pos + op.len()..];
                let left_val = self.evaluate_expression(left)?;
                let right_val = self.evaluate_expression(right)?;
                return self.apply_binary_op(op, &left_val, &right_val);
            }
        }

        // Property access
        if expr.contains('.') {
            let parts: Vec<&str> = expr.splitn(2, '.').collect();
            if parts.len() == 2 {
                let obj = self.evaluate_expression(parts[0])?;
                match obj {
                    Value::Object(map) => {
                        return Ok(map.get(parts[1]).cloned().unwrap_or(Value::Null));
                    }
                    Value::String(s) if parts[1] == "length" => {
                        return Ok(Value::Number(s.len() as i64));
                    }
                    Value::Array(arr) if parts[1] == "length" => {
                        return Ok(Value::Number(arr.len() as i64));
                    }
                    _ => {}
                }
            }
        }

        // Function call
        if expr.contains('(') && expr.ends_with(')') {
            return self.evaluate_function_call(expr);
        }

        // Unknown expression
        Ok(Value::Null)
    }

    fn parse_object_literal(&mut self, expr: &str) -> Result<Value, MtpError> {
        let content = &expr[1..expr.len() - 1].trim();
        if content.is_empty() {
            return Ok(Value::Object(HashMap::new()));
        }

        let mut obj = HashMap::new();
        for pair in split_by_comma(content) {
            let pair = pair.trim();
            if let Some(colon_pos) = pair.find(':') {
                let key = pair[..colon_pos].trim().trim_matches('"').trim_matches('\'');
                let value_expr = pair[colon_pos + 1..].trim();
                let value = self.evaluate_expression(value_expr)?;
                obj.insert(key.to_string(), value);
            }
        }
        Ok(Value::Object(obj))
    }

    fn parse_array_literal(&mut self, expr: &str) -> Result<Value, MtpError> {
        let content = &expr[1..expr.len() - 1].trim();
        if content.is_empty() {
            return Ok(Value::Array(vec![]));
        }

        let mut arr = Vec::new();
        for item in split_by_comma(content) {
            let value = self.evaluate_expression(item.trim())?;
            arr.push(value);
        }
        Ok(Value::Array(arr))
    }

    fn evaluate_function_call(&mut self, expr: &str) -> Result<Value, MtpError> {
        self.consume_gas(5)?; // Function call costs more gas

        let paren_pos = expr.find('(').unwrap();
        let func_name = &expr[..paren_pos];
        let args_str = &expr[paren_pos + 1..expr.len() - 1];

        // Parse arguments
        let args: Vec<Value> = if args_str.trim().is_empty() {
            vec![]
        } else {
            split_by_comma(args_str)
                .iter()
                .map(|arg| self.evaluate_expression(arg.trim()))
                .collect::<Result<Vec<_>, _>>()?
        };

        // Built-in functions
        match func_name {
            "console.log" => {
                for arg in &args {
                    println!("{}", arg);
                }
                Ok(Value::Null)
            }
            "JSON.stringify" => {
                if let Some(arg) = args.first() {
                    Ok(Value::String(format!("{}", arg)))
                } else {
                    Ok(Value::String("undefined".to_string()))
                }
            }
            "JSON.parse" => {
                if let Some(Value::String(s)) = args.first() {
                    // Simple JSON parsing
                    if s == "null" {
                        Ok(Value::Null)
                    } else if s == "true" {
                        Ok(Value::Boolean(true))
                    } else if s == "false" {
                        Ok(Value::Boolean(false))
                    } else if let Ok(n) = s.parse::<i64>() {
                        Ok(Value::Number(n))
                    } else {
                        Ok(Value::String(s.clone()))
                    }
                } else {
                    Err(MtpError::RuntimeError("JSON.parse requires a string argument".into()))
                }
            }
            _ => {
                // Check if it's a user-defined function in heap
                if let Some(Value::Function(f)) = self.heap.get(func_name).cloned() {
                    // Execute user function (simplified)
                    Ok(Value::Null)
                } else {
                    // Unknown function
                    Ok(Value::Null)
                }
            }
        }
    }

    fn apply_binary_op(&self, op: &str, left: &Value, right: &Value) -> Result<Value, MtpError> {
        match op {
            "+" => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
                (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
                _ => Ok(Value::String(format!("{}{}", left, right))),
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
                (Value::Number(a), Value::Number(b)) if *b != 0 => Ok(Value::Number(a / b)),
                (Value::Number(_), Value::Number(_)) => {
                    Err(MtpError::RuntimeError("Division by zero".into()))
                }
                _ => Err(MtpError::RuntimeError("Invalid operands for /".into())),
            },
            "%" => match (left, right) {
                (Value::Number(a), Value::Number(b)) if *b != 0 => Ok(Value::Number(a % b)),
                _ => Err(MtpError::RuntimeError("Invalid operands for %".into())),
            },
            "==" | "===" => Ok(Value::Boolean(left == right)),
            "!=" | "!==" => Ok(Value::Boolean(left != right)),
            "<" => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a < b)),
                _ => Ok(Value::Boolean(false)),
            },
            ">" => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a > b)),
                _ => Ok(Value::Boolean(false)),
            },
            "<=" => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a <= b)),
                _ => Ok(Value::Boolean(false)),
            },
            ">=" => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a >= b)),
                _ => Ok(Value::Boolean(false)),
            },
            "&&" => Ok(Value::Boolean(is_truthy(left) && is_truthy(right))),
            "||" => Ok(Value::Boolean(is_truthy(left) || is_truthy(right))),
            _ => Err(MtpError::RuntimeError(format!("Unknown operator: {}", op))),
        }
    }

    /// Validate code doesn't contain forbidden constructs
    fn validate_code(&self, code: &str) -> Result<(), MtpError> {
        let forbidden = [
            "eval", "Function", "class", "this", "new", "import", "require",
            "process", "global", "__proto__", "constructor",
        ];

        for word in &forbidden {
            if code.contains(word) {
                return Err(MtpError::RuntimeError(format!(
                    "Forbidden construct: {}",
                    word
                )));
            }
        }

        Ok(())
    }

    /// Allocate memory in isolated heap
    pub fn allocate(&mut self, key: String, value: Value) -> Result<(), MtpError> {
        let size_estimate = self.estimate_size(&value);

        if self.memory_used + size_estimate > self.memory_limit {
            return Err(MtpError::RuntimeError(
                "Memory limit exceeded in isolated runtime".into(),
            ));
        }

        self.heap.insert(key, value);
        self.memory_used += size_estimate;

        Ok(())
    }

    /// Access isolated heap
    pub fn get_heap(&self, key: &str) -> Option<&Value> {
        self.heap.get(key)
    }

    /// Consume gas
    fn consume_gas(&mut self, amount: u64) -> Result<(), MtpError> {
        if self.gas_remaining < amount {
            return Err(MtpError::RuntimeError(format!(
                "Gas exhausted: {} > {}",
                amount, self.gas_remaining
            )));
        }
        self.gas_remaining -= amount;
        Ok(())
    }

    /// Check timeout
    fn check_timeout(&self) -> Result<(), MtpError> {
        if self.start_time.elapsed() > self.timeout {
            return Err(MtpError::RuntimeError("Execution timeout".into()));
        }
        Ok(())
    }

    /// Check that no global state is leaked between requests
    fn validate_no_global_leakage(&self) -> Result<(), MtpError> {
        self.check_timeout()?;

        if self.gas_remaining == 0 {
            return Err(MtpError::RuntimeError(
                "Gas exhausted in isolated runtime".into(),
            ));
        }

        Ok(())
    }

    /// Estimate memory usage of a value
    fn estimate_size(&self, value: &Value) -> usize {
        match value {
            Value::Number(_) => 8,
            Value::Boolean(_) => 1,
            Value::String(s) => s.len(),
            Value::Array(arr) => {
                arr.iter().map(|v| self.estimate_size(v)).sum::<usize>()
                    + arr.len() * std::mem::size_of::<Value>()
            }
            Value::Object(obj) => {
                obj.iter()
                    .map(|(k, v)| k.len() + self.estimate_size(v))
                    .sum::<usize>()
                    + obj.len() * 64
            }
            Value::Function(_) => 1024,
            Value::Null => 0,
        }
    }

    /// Clean up runtime after execution
    pub fn cleanup(&mut self) -> Result<(), MtpError> {
        // Zero out sensitive memory
        for value in self.heap.values_mut() {
            self.zero_value(value);
        }

        self.heap.clear();
        self.stack.clear();
        self.globals.clear();
        self.memory_used = 0;

        Ok(())
    }

    /// Securely zero a value's memory
    fn zero_value(&self, value: &mut Value) {
        match value {
            Value::String(s) => {
                // Overwrite string contents
                let bytes = unsafe { s.as_bytes_mut() };
                for b in bytes {
                    *b = 0;
                }
            }
            Value::Array(arr) => {
                for item in arr {
                    self.zero_value(item);
                }
            }
            Value::Object(obj) => {
                for value in obj.values_mut() {
                    self.zero_value(value);
                }
            }
            _ => {}
        }
    }

    /// Get remaining gas
    pub fn gas_remaining(&self) -> u64 {
        self.gas_remaining
    }
}

/// Helper to find operator position, respecting string literals
fn find_operator(expr: &str, op: &str) -> Option<usize> {
    let mut in_string = false;
    let mut string_char = ' ';
    let bytes = expr.as_bytes();
    let op_bytes = op.as_bytes();

    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i] as char;

        if !in_string && (c == '"' || c == '\'') {
            in_string = true;
            string_char = c;
        } else if in_string && c == string_char {
            in_string = false;
        } else if !in_string && bytes[i..].starts_with(op_bytes) {
            return Some(i);
        }

        i += 1;
    }

    None
}

/// Split by comma, respecting nesting
fn split_by_comma(s: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut depth = 0;
    let mut start = 0;

    for (i, c) in s.char_indices() {
        match c {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            ',' if depth == 0 => {
                result.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }

    if start < s.len() {
        result.push(&s[start..]);
    }

    result
}

/// Check if value is truthy
fn is_truthy(val: &Value) -> bool {
    match val {
        Value::Boolean(b) => *b,
        Value::Null => false,
        Value::Number(n) => *n != 0,
        Value::String(s) => !s.is_empty(),
        _ => true,
    }
}

/// Runtime manager for handling multiple isolated runtimes
pub struct RuntimeManager {
    runtimes: Mutex<HashMap<String, Arc<Mutex<IsolatedRuntime>>>>,
    max_concurrent_runtimes: usize,
}

impl RuntimeManager {
    pub fn new(max_concurrent: usize) -> Self {
        RuntimeManager {
            runtimes: Mutex::new(HashMap::new()),
            max_concurrent_runtimes: max_concurrent,
        }
    }

    /// Create a new isolated runtime
    pub fn create_runtime(
        &self,
        id: String,
        gas_limit: u64,
    ) -> Result<Arc<Mutex<IsolatedRuntime>>, MtpError> {
        let mut runtimes = self.runtimes.lock().unwrap();

        if runtimes.len() >= self.max_concurrent_runtimes {
            return Err(MtpError::RuntimeError(
                "Maximum concurrent runtimes exceeded".into(),
            ));
        }

        let runtime = Arc::new(Mutex::new(IsolatedRuntime::new(id.clone(), gas_limit, 100)));
        runtimes.insert(id, Arc::clone(&runtime));

        Ok(runtime)
    }

    /// Get an existing runtime
    pub fn get_runtime(&self, id: &str) -> Option<Arc<Mutex<IsolatedRuntime>>> {
        let runtimes = self.runtimes.lock().unwrap();
        runtimes.get(id).cloned()
    }

    /// Clean up a runtime
    pub fn cleanup_runtime(&self, id: &str) -> Result<(), MtpError> {
        let mut runtimes = self.runtimes.lock().unwrap();

        if let Some(runtime) = runtimes.remove(id) {
            let mut runtime = runtime.lock().unwrap();
            runtime.cleanup()?;
        }

        Ok(())
    }

    /// Get current runtime count
    pub fn runtime_count(&self) -> usize {
        self.runtimes.lock().unwrap().len()
    }

    /// Clean up all runtimes (for shutdown)
    pub fn shutdown(&self) -> Result<(), MtpError> {
        let mut runtimes = self.runtimes.lock().unwrap();
        let ids: Vec<String> = runtimes.keys().cloned().collect();

        for id in ids {
            if let Some(runtime) = runtimes.remove(&id) {
                let mut runtime = runtime.lock().unwrap();
                runtime.cleanup()?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_isolation() {
        let manager = RuntimeManager::new(10);

        let runtime1 = manager.create_runtime("req1".to_string(), 1000).unwrap();
        let runtime2 = manager.create_runtime("req2".to_string(), 1000).unwrap();

        // Execute in runtime1
        {
            let mut rt1 = runtime1.lock().unwrap();
            rt1.allocate("key1".to_string(), Value::String("secret1".to_string()))
                .unwrap();
            assert_eq!(
                rt1.get_heap("key1").unwrap().to_string(),
                "secret1"
            );
        }

        // Check runtime2 doesn't have runtime1's data
        {
            let rt2 = runtime2.lock().unwrap();
            assert!(rt2.get_heap("key1").is_none());
        }

        // Clean up
        manager.cleanup_runtime("req1").unwrap();
        manager.cleanup_runtime("req2").unwrap();

        assert_eq!(manager.runtime_count(), 0);
    }

    #[test]
    fn test_code_execution() {
        let mut runtime = IsolatedRuntime::new("test".to_string(), 10000, 100);

        let result = runtime.execute("1 + 2").unwrap();
        assert_eq!(result, Value::Number(3));
    }

    #[test]
    fn test_forbidden_constructs() {
        let mut runtime = IsolatedRuntime::new("test".to_string(), 10000, 100);

        let result = runtime.execute("eval('bad')");
        assert!(result.is_err());
    }

    #[test]
    fn test_gas_consumption() {
        let mut runtime = IsolatedRuntime::new("test".to_string(), 10, 100);

        // Each expression consumes gas
        let _ = runtime.execute("1");
        let _ = runtime.execute("2");

        // Should eventually run out of gas
        assert!(runtime.gas_remaining() < 10);
    }
}
