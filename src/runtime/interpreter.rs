use crate::errors::MtpError;
use crate::runtime::value::Value;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Taint level for tracking potentially sensitive data
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaintLevel {
    None,
    Low,
    Medium,
    High,
    Critical,
}

/// Source of taint
#[derive(Debug, Clone)]
pub struct TaintSource {
    pub id: String,
    pub description: String,
}

/// Taint entry for tracking
#[derive(Debug, Clone)]
pub struct TaintEntry {
    pub level: TaintLevel,
    pub source: TaintSource,
}

/// Dynamic taint tracker
#[derive(Debug, Clone)]
pub struct DynamicTaintTracker {
    tainted: HashMap<String, TaintEntry>,
    call_stack: Vec<String>,
}

impl DynamicTaintTracker {
    pub fn new() -> Self {
        DynamicTaintTracker {
            tainted: HashMap::new(),
            call_stack: Vec::new(),
        }
    }

    pub fn mark_tainted(&mut self, key: &str, level: TaintLevel, source: TaintSource) {
        self.tainted.insert(key.to_string(), TaintEntry { level, source });
    }

    pub fn is_tainted(&self, key: &str) -> bool {
        self.tainted.contains_key(key)
    }

    pub fn get_taint_level(&self, key: &str) -> TaintLevel {
        self.tainted.get(key)
            .map(|e| e.level)
            .unwrap_or(TaintLevel::None)
    }

    pub fn check_dangerous_operation(&self, operation: &str, data_key: &str) -> Result<(), MtpError> {
        if let Some(entry) = self.tainted.get(data_key) {
            if entry.level >= TaintLevel::High {
                return Err(MtpError::RuntimeError {
                    error: "TaintViolation".to_string(),
                    message: format!(
                        "Cannot perform '{}' on high-taint data '{}' from source: {}",
                        operation, data_key, entry.source.description
                    ),
                });
            }
        }
        Ok(())
    }

    pub fn propagate(&mut self, result_key: &str, input_keys: &[&str]) {
        let max_taint = input_keys.iter()
            .filter_map(|k| self.tainted.get(*k))
            .max_by_key(|e| e.level);

        if let Some(entry) = max_taint {
            self.tainted.insert(result_key.to_string(), entry.clone());
        }
    }

    pub fn get_call_stack(&self) -> &[String] {
        &self.call_stack
    }

    pub fn push_call(&mut self, func_name: &str) {
        self.call_stack.push(func_name.to_string());
    }

    pub fn pop_call(&mut self) {
        self.call_stack.pop();
    }
}

impl Default for DynamicTaintTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Interpreter configuration
#[derive(Debug, Clone)]
pub struct InterpreterConfig {
    pub max_execution_time: Duration,
    pub max_memory_mb: usize,
    pub gas_limit: u64,
}

impl Default for InterpreterConfig {
    fn default() -> Self {
        InterpreterConfig {
            max_execution_time: Duration::from_secs(30),
            max_memory_mb: 100,
            gas_limit: 10_000_000,
        }
    }
}

/// Builtin function type
pub type BuiltinFn = fn(Vec<Value>) -> Result<Value, String>;

/// Interpreter state
pub struct Interpreter {
    pub config: InterpreterConfig,
    pub start_time: Instant,
    pub gas_used: u64,
    pub memory_used: usize,
    pub globals: HashMap<String, Value>,
    pub global_scope: HashMap<String, Value>,
    pub builtins: HashMap<String, BuiltinFn>,
    pub taint_tracker: DynamicTaintTracker,
}

impl Interpreter {
    pub fn new(config: InterpreterConfig) -> Self {
        let mut interp = Interpreter {
            config,
            start_time: Instant::now(),
            gas_used: 0,
            memory_used: 0,
            globals: HashMap::new(),
            global_scope: HashMap::new(),
            builtins: HashMap::new(),
            taint_tracker: DynamicTaintTracker::new(),
        };
        interp.inject_builtin_objects();
        interp
    }

    /// Inject built-in objects and functions
    pub fn inject_builtin_objects(&mut self) {
        // Math functions
        self.builtins.insert("Math.abs".to_string(), |args| {
            if args.len() != 1 {
                return Err("Math.abs requires exactly 1 argument".to_string());
            }
            match &args[0] {
                Value::Number(n) => Ok(Value::Number(n.abs())),
                _ => Err("Math.abs requires a number".to_string()),
            }
        });

        self.builtins.insert("Math.floor".to_string(), |args| {
            if args.len() != 1 {
                return Err("Math.floor requires exactly 1 argument".to_string());
            }
            match &args[0] {
                Value::Number(n) => Ok(Value::Number(*n)),
                _ => Err("Math.floor requires a number".to_string()),
            }
        });

        self.builtins.insert("Math.ceil".to_string(), |args| {
            if args.len() != 1 {
                return Err("Math.ceil requires exactly 1 argument".to_string());
            }
            match &args[0] {
                Value::Number(n) => Ok(Value::Number(*n)),
                _ => Err("Math.ceil requires a number".to_string()),
            }
        });

        self.builtins.insert("Math.max".to_string(), |args| {
            if args.len() < 2 {
                return Err("Math.max requires at least 2 arguments".to_string());
            }
            let mut max = i64::MIN;
            for arg in args {
                match arg {
                    Value::Number(n) => {
                        if n > max {
                            max = n;
                        }
                    }
                    _ => return Err("Math.max requires numbers".to_string()),
                }
            }
            Ok(Value::Number(max))
        });

        self.builtins.insert("Math.min".to_string(), |args| {
            if args.len() < 2 {
                return Err("Math.min requires at least 2 arguments".to_string());
            }
            let mut min = i64::MAX;
            for arg in args {
                match arg {
                    Value::Number(n) => {
                        if n < min {
                            min = n;
                        }
                    }
                    _ => return Err("Math.min requires numbers".to_string()),
                }
            }
            Ok(Value::Number(min))
        });

        // String functions
        self.builtins.insert("String.length".to_string(), |args| {
            if args.len() != 1 {
                return Err("String.length requires exactly 1 argument".to_string());
            }
            match &args[0] {
                Value::String(s) => Ok(Value::Number(s.len() as i64)),
                _ => Err("String.length requires a string".to_string()),
            }
        });

        self.builtins.insert("String.concat".to_string(), |args| {
            let mut result = String::new();
            for arg in args {
                match arg {
                    Value::String(s) => result.push_str(&s),
                    Value::Number(n) => result.push_str(&n.to_string()),
                    Value::Boolean(b) => result.push_str(&b.to_string()),
                    Value::Null => result.push_str("null"),
                    _ => return Err("String.concat: cannot convert value to string".to_string()),
                }
            }
            Ok(Value::String(result))
        });

        // Array functions
        self.builtins.insert("Array.length".to_string(), |args| {
            if args.len() != 1 {
                return Err("Array.length requires exactly 1 argument".to_string());
            }
            match &args[0] {
                Value::Array(arr) => Ok(Value::Number(arr.len() as i64)),
                _ => Err("Array.length requires an array".to_string()),
            }
        });

        self.builtins.insert("Array.push".to_string(), |args| {
            if args.len() < 2 {
                return Err("Array.push requires at least 2 arguments".to_string());
            }
            match &args[0] {
                Value::Array(arr) => {
                    let mut new_arr = arr.clone();
                    for item in &args[1..] {
                        new_arr.push(item.clone());
                    }
                    Ok(Value::Array(new_arr))
                }
                _ => Err("Array.push requires an array".to_string()),
            }
        });

        // JSON functions
        self.builtins.insert("Json.parse".to_string(), |args| {
            if args.len() != 1 {
                return Err("Json.parse requires exactly 1 argument".to_string());
            }
            match &args[0] {
                Value::String(s) => parse_json_value(s),
                _ => Err("Json.parse requires a string".to_string()),
            }
        });

        self.builtins.insert("Json.stringify".to_string(), |args| {
            if args.len() != 1 {
                return Err("Json.stringify requires exactly 1 argument".to_string());
            }
            Ok(Value::String(stringify_value(&args[0])))
        });

        // Console functions (for debugging)
        self.builtins.insert("console.log".to_string(), |args| {
            for arg in &args {
                print!("{} ", stringify_value(arg));
            }
            println!();
            Ok(Value::Null)
        });

        // Store console.log in globals for call_global_function
        self.globals.insert("console.log".to_string(), Value::Null);
    }

    /// Check if execution timeout has been exceeded
    pub fn check_timeout(&self) -> Result<(), MtpError> {
        if self.start_time.elapsed() > self.config.max_execution_time {
            return Err(MtpError::RuntimeError {
                error: "RuntimeError".to_string(),
                message: "Execution timeout exceeded".into(),
            });
        }
        Ok(())
    }

    /// Consume gas
    pub fn consume_gas(&mut self, amount: u64) -> Result<(), MtpError> {
        self.gas_used = self.gas_used.saturating_add(amount);
        if self.gas_used > self.config.gas_limit {
            return Err(MtpError::RuntimeError {
                error: "RuntimeError".to_string(),
                message: format!(
                    "Gas limit exceeded: {} > {}",
                    self.gas_used, self.config.gas_limit
                ),
            });
        }
        Ok(())
    }

    /// Allocate memory
    pub fn allocate_memory(&mut self, size: usize) -> Result<(), MtpError> {
        self.memory_used = self.memory_used.saturating_add(size);
        let max_bytes = self.config.max_memory_mb * 1024 * 1024;
        if self.memory_used > max_bytes {
            return Err(MtpError::RuntimeError {
                error: "RuntimeError".to_string(),
                message: format!(
                    "Memory limit exceeded: {} > {} MB",
                    self.memory_used / (1024 * 1024),
                    self.config.max_memory_mb
                ),
            });
        }
        Ok(())
    }

    /// Get gas used
    pub fn gas_used(&self) -> u64 {
        self.gas_used
    }

    /// Evaluate array access
    fn evaluate_array_access(&mut self, arr_name: &str, index_str: &str, variables: &HashMap<String, Value>) -> Result<Value, MtpError> {
        // Get array
        let arr = if let Some(Value::Array(a)) = variables.get(arr_name) {
            a.clone()
        } else if let Some(Value::Array(a)) = self.global_scope.get(arr_name) {
            a.clone()
        } else {
            return Ok(Value::Null);
        };

        // Get index
        let index = if let Ok(i) = index_str.parse::<usize>() {
            i
        } else if let Some(Value::Number(i)) = variables.get(index_str) {
            *i as usize
        } else if let Some(Value::Number(i)) = self.global_scope.get(index_str) {
            *i as usize
        } else {
            return Ok(Value::Null);
        };

        // Check bounds
        if index >= arr.len() {
            return Err(MtpError::RuntimeError {
                error: "ArrayBoundsError".to_string(),
                message: format!("Array index {} out of bounds (length {})", index, arr.len()),
            });
        }

        Ok(arr[index].clone())
    }

    /// Evaluate an expression string
    fn evaluate_expression(&mut self, expr: &str, variables: &HashMap<String, Value>) -> Result<Value, MtpError> {
        let expr = expr.trim();

        // Handle literals
        if expr.starts_with('"') && expr.ends_with('"') {
            return Ok(Value::String(expr[1..expr.len()-1].to_string()));
        }
        if expr.starts_with('{') && expr.ends_with('}') {
            return self.parse_object(expr, variables);
        }
        if let Ok(num) = expr.parse::<i64>() {
            return Ok(Value::Number(num));
        }
        if expr == "true" {
            return Ok(Value::Boolean(true));
        }
        if expr == "false" {
            return Ok(Value::Boolean(false));
        }
        if expr == "null" {
            return Ok(Value::Null);
        }

        // Handle variables
        if let Some(value) = variables.get(expr) {
            return Ok(value.clone());
        }
        if let Some(value) = self.global_scope.get(expr) {
            return Ok(value.clone());
        }

        // Handle array access like arr[1]
        if expr.contains('[') && expr.ends_with(']') {
            if let Some(bracket_pos) = expr.find('[') {
                let arr_name = &expr[..bracket_pos];
                let index_str = &expr[bracket_pos+1..expr.len()-1];
                return self.evaluate_array_access(arr_name, index_str, variables);
            }
        }

        // Handle function calls
        if expr.contains('(') && expr.ends_with(')') {
            return self.evaluate_function_call(expr, variables);
        }

        // Handle arrays
        if expr.starts_with('[') && expr.ends_with(']') {
            return self.parse_array(expr, variables);
        }

        // Default to string
        Ok(Value::String(expr.to_string()))
    }

    /// Parse an object literal
    fn parse_object(&mut self, obj_str: &str, variables: &HashMap<String, Value>) -> Result<Value, MtpError> {
        let mut obj = HashMap::new();
        let content = &obj_str[1..obj_str.len()-1];

        if content.trim().is_empty() {
            return Ok(Value::Object(obj));
        }

        // Parse key-value pairs (simple implementation)
        let mut depth = 0;
        let mut current = String::new();
        let mut pairs = Vec::new();

        for c in content.chars() {
            match c {
                '{' | '[' => {
                    depth += 1;
                    current.push(c);
                }
                '}' | ']' => {
                    depth -= 1;
                    current.push(c);
                }
                ',' if depth == 0 => {
                    pairs.push(current.trim().to_string());
                    current.clear();
                }
                _ => current.push(c),
            }
        }
        if !current.trim().is_empty() {
            pairs.push(current.trim().to_string());
        }

        for pair in pairs {
            if let Some(colon_pos) = pair.find(':') {
                let key = pair[..colon_pos].trim().trim_matches('"');
                let value_expr = pair[colon_pos+1..].trim();
                let value = self.evaluate_expression(value_expr, variables)?;
                obj.insert(key.to_string(), value);
            }
        }

        Ok(Value::Object(obj))
    }

    /// Parse an array literal
    fn parse_array(&mut self, arr_str: &str, variables: &HashMap<String, Value>) -> Result<Value, MtpError> {
        let mut arr = Vec::new();
        let content = &arr_str[1..arr_str.len()-1];

        if content.trim().is_empty() {
            return Ok(Value::Array(arr));
        }

        // Parse elements (handle nested structures)
        let mut depth = 0;
        let mut current = String::new();
        let mut elements = Vec::new();

        for c in content.chars() {
            match c {
                '{' | '[' => {
                    depth += 1;
                    current.push(c);
                }
                '}' | ']' => {
                    depth -= 1;
                    current.push(c);
                }
                ',' if depth == 0 => {
                    elements.push(current.trim().to_string());
                    current.clear();
                }
                _ => current.push(c),
            }
        }
        if !current.trim().is_empty() {
            elements.push(current.trim().to_string());
        }

        for elem in elements {
            let value = self.evaluate_expression(&elem, variables)?;
            arr.push(value);
        }

        Ok(Value::Array(arr))
    }

    /// Evaluate a function call
    fn evaluate_function_call(&mut self, call: &str, variables: &HashMap<String, Value>) -> Result<Value, MtpError> {
        if let Some(open_paren) = call.find('(') {
            let func_name = &call[..open_paren];
            let args_str = &call[open_paren+1..call.len()-1];

            // Parse arguments
            let args: Vec<Value> = if args_str.trim().is_empty() {
                vec![]
            } else {
                let mut depth = 0;
                let mut current = String::new();
                let mut arg_strs = Vec::new();

                for c in args_str.chars() {
                    match c {
                        '{' | '[' | '(' => {
                            depth += 1;
                            current.push(c);
                        }
                        '}' | ']' | ')' => {
                            depth -= 1;
                            current.push(c);
                        }
                        ',' if depth == 0 => {
                            arg_strs.push(current.trim().to_string());
                            current.clear();
                        }
                        _ => current.push(c),
                    }
                }
                if !current.trim().is_empty() {
                    arg_strs.push(current.trim().to_string());
                }

                let mut args = Vec::new();
                for arg_str in arg_strs {
                    let value = self.evaluate_expression(&arg_str, variables)?;
                    args.push(value);
                }
                args
            };

            // Handle array_get specially
            if func_name == "array_get" {
                if args.len() != 2 {
                    return Ok(Value::Null);
                }
                if let (Value::Array(arr), Value::Number(idx)) = (&args[0], &args[1]) {
                    let index = *idx as usize;
                    if index >= arr.len() {
                        return Err(MtpError::RuntimeError {
                            error: "ArrayBoundsError".to_string(),
                            message: format!("Array index {} out of bounds", index),
                        });
                    }
                    return Ok(arr[index].clone());
                }
                return Ok(Value::Null);
            }

            // Try builtin functions
            if let Some(builtin) = self.builtins.get(func_name).copied() {
                return builtin(args).map_err(|e| MtpError::RuntimeError {
                    error: "BuiltinError".to_string(),
                    message: e,
                });
            }

            Ok(Value::Null)
        } else {
            Ok(Value::Null)
        }
    }

    /// Execute JavaScript code
    pub fn execute(&mut self, js_code: &str) -> Result<Value, MtpError> {
        self.check_timeout()?;
        self.consume_gas(100)?; // Base cost per execution

        let mut variables: HashMap<String, Value> = HashMap::new();
        let mut result = Value::Null;

        for line in js_code.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("//") {
                continue;
            }

            self.consume_gas(10)?; // Cost per line

            // Handle return statements
            if line.starts_with("return ") {
                let return_expr = line.strip_prefix("return ")
                    .unwrap()
                    .trim()
                    .strip_suffix(';')
                    .unwrap_or(line.strip_prefix("return ").unwrap().trim());
                result = self.evaluate_expression(return_expr, &variables)?;
                break;
            }

            // Handle variable declarations: let/const/var name = value;
            if line.starts_with("let ") || line.starts_with("const ") || line.starts_with("var ") {
                let without_keyword = if line.starts_with("let ") {
                    &line[4..]
                } else if line.starts_with("const ") {
                    &line[6..]
                } else {
                    &line[4..]
                };

                if let Some(eq_pos) = without_keyword.find('=') {
                    let var_name = without_keyword[..eq_pos].trim();
                    let value_expr = without_keyword[eq_pos+1..]
                        .trim()
                        .strip_suffix(';')
                        .unwrap_or(without_keyword[eq_pos+1..].trim());
                    let value = self.evaluate_expression(value_expr, &variables)?;
                    variables.insert(var_name.to_string(), value);
                }
                continue;
            }

            // Handle assignments without keyword
            if line.contains(" = ") && !line.starts_with("if ") {
                let parts: Vec<&str> = line.splitn(2, " = ").collect();
                if parts.len() == 2 {
                    let var_name = parts[0].trim();
                    let expr = parts[1].trim().strip_suffix(';').unwrap_or(parts[1].trim());
                    let value = self.evaluate_expression(expr, &variables)?;
                    variables.insert(var_name.to_string(), value);
                }
                continue;
            }

            // Handle function calls without assignment
            if line.contains('(') && line.contains(')') && !line.contains(" = ") {
                let call = line.strip_suffix(';').unwrap_or(line);
                let _ = self.evaluate_function_call(call, &variables)?;
            }
        }

        Ok(result)
    }

    /// Execute code and return JSON result
    pub fn execute_to_json(&mut self, code: &str) -> Result<String, MtpError> {
        let result = self.execute(code)?;
        Ok(stringify_value(&result))
    }

    /// Call a global function
    pub fn call_global_function(
        &mut self,
        name: &str,
        args: Vec<Value>,
    ) -> Result<Value, MtpError> {
        // First check builtins
        if let Some(builtin) = self.builtins.get(name).copied() {
            return builtin(args).map_err(|e| MtpError::RuntimeError {
                error: "RuntimeError".to_string(),
                message: e,
            });
        }

        // Check globals
        match self.globals.get(name) {
            Some(_) => {
                match name {
                    "console.log" => {
                        for arg in &args {
                            print!("{} ", stringify_value(arg));
                        }
                        println!();
                        Ok(Value::Null)
                    }
                    _ => Err(MtpError::RuntimeError {
                        error: "RuntimeError".to_string(),
                        message: format!("Unknown function: {}", name),
                    }),
                }
            }
            None => Err(MtpError::RuntimeError {
                error: "RuntimeError".to_string(),
                message: format!("Undefined function: {}", name),
            }),
        }
    }

    /// Mark external input as tainted
    pub fn mark_input_tainted(&mut self, key: &str, level: TaintLevel, source_desc: &str) {
        let source = TaintSource {
            id: format!("input_{}", key),
            description: source_desc.to_string(),
        };
        self.taint_tracker.mark_tainted(key, level, source);
    }

    /// Check if data is tainted before dangerous operations
    pub fn check_taint_before_operation(
        &self,
        operation: &str,
        data_key: &str,
    ) -> Result<(), MtpError> {
        self.taint_tracker.check_dangerous_operation(operation, data_key)
    }

    /// Propagate taint through operations
    pub fn propagate_taint(&mut self, result_key: &str, input_keys: &[&str]) {
        self.taint_tracker.propagate(result_key, input_keys);
    }

    /// Get taint report
    pub fn get_taint_report(&self) -> String {
        format!(
            "Taint tracking active. Call stack depth: {}",
            self.taint_tracker.get_call_stack().len()
        )
    }
}

/// Parse a JSON string into a Value
fn parse_json_value(s: &str) -> Result<Value, String> {
    let s = s.trim();

    if s == "null" {
        return Ok(Value::Null);
    }
    if s == "true" {
        return Ok(Value::Boolean(true));
    }
    if s == "false" {
        return Ok(Value::Boolean(false));
    }

    if let Ok(n) = s.parse::<i64>() {
        return Ok(Value::Number(n));
    }

    if s.starts_with('"') && s.ends_with('"') {
        return Ok(Value::String(s[1..s.len()-1].to_string()));
    }

    if s.starts_with('[') && s.ends_with(']') {
        let content = &s[1..s.len()-1];
        if content.trim().is_empty() {
            return Ok(Value::Array(vec![]));
        }

        let mut arr = Vec::new();
        let mut depth = 0;
        let mut current = String::new();

        for c in content.chars() {
            match c {
                '{' | '[' => {
                    depth += 1;
                    current.push(c);
                }
                '}' | ']' => {
                    depth -= 1;
                    current.push(c);
                }
                ',' if depth == 0 => {
                    arr.push(parse_json_value(current.trim())?);
                    current.clear();
                }
                _ => current.push(c),
            }
        }
        if !current.trim().is_empty() {
            arr.push(parse_json_value(current.trim())?);
        }
        return Ok(Value::Array(arr));
    }

    if s.starts_with('{') && s.ends_with('}') {
        let content = &s[1..s.len()-1];
        if content.trim().is_empty() {
            return Ok(Value::Object(HashMap::new()));
        }

        let mut obj = HashMap::new();
        let mut depth = 0;
        let mut current = String::new();

        for c in content.chars() {
            match c {
                '{' | '[' => {
                    depth += 1;
                    current.push(c);
                }
                '}' | ']' => {
                    depth -= 1;
                    current.push(c);
                }
                ',' if depth == 0 => {
                    parse_json_pair(&current, &mut obj)?;
                    current.clear();
                }
                _ => current.push(c),
            }
        }
        if !current.trim().is_empty() {
            parse_json_pair(&current, &mut obj)?;
        }
        return Ok(Value::Object(obj));
    }

    Err(format!("Invalid JSON: {}", s))
}

fn parse_json_pair(pair: &str, obj: &mut HashMap<String, Value>) -> Result<(), String> {
    if let Some(colon_pos) = pair.find(':') {
        let key = pair[..colon_pos].trim().trim_matches('"');
        let value = parse_json_value(pair[colon_pos+1..].trim())?;
        obj.insert(key.to_string(), value);
    }
    Ok(())
}

/// Stringify a Value to JSON
fn stringify_value(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
        Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(stringify_value).collect();
            format!("[{}]", items.join(","))
        }
        Value::Object(obj) => {
            let pairs: Vec<String> = obj.iter()
                .map(|(k, v)| format!("\"{}\":{}", k, stringify_value(v)))
                .collect();
            format!("{{{}}}", pairs.join(","))
        }
        Value::Function(f) => format!("\"<function {}>\"", f.name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gas_limit() {
        let config = InterpreterConfig {
            gas_limit: 1000,
            ..Default::default()
        };
        let mut interp = Interpreter::new(config);

        assert!(interp.consume_gas(500).is_ok());
        assert!(interp.consume_gas(600).is_err());
    }

    #[test]
    fn test_memory_limit() {
        let config = InterpreterConfig {
            max_memory_mb: 1,
            ..Default::default()
        };
        let mut interp = Interpreter::new(config);

        assert!(interp.allocate_memory(500 * 1024).is_ok()); // 500KB
        assert!(interp.allocate_memory(1024 * 1024).is_err()); // Would exceed 1MB
    }

    #[test]
    fn test_timeout() {
        let config = InterpreterConfig {
            max_execution_time: Duration::from_millis(1),
            ..Default::default()
        };
        let interp = Interpreter::new(config);

        std::thread::sleep(Duration::from_millis(10));
        assert!(interp.check_timeout().is_err());
    }

    #[test]
    fn test_taint_tracking() {
        let mut tracker = DynamicTaintTracker::new();
        let source = TaintSource {
            id: "user_input".to_string(),
            description: "User provided data".to_string(),
        };

        tracker.mark_tainted("password", TaintLevel::Critical, source);
        assert!(tracker.is_tainted("password"));
        assert_eq!(tracker.get_taint_level("password"), TaintLevel::Critical);
        assert!(tracker.check_dangerous_operation("eval", "password").is_err());
    }

    #[test]
    fn test_execute_simple() {
        let mut interp = Interpreter::new(InterpreterConfig::default());
        let result = interp.execute("return 42;").unwrap();
        assert_eq!(result, Value::Number(42));
    }

    #[test]
    fn test_execute_with_variables() {
        let mut interp = Interpreter::new(InterpreterConfig::default());
        let result = interp.execute("let x = 10;\nreturn x;").unwrap();
        assert_eq!(result, Value::Number(10));
    }
}
