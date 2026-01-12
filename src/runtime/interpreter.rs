use crate::errors::MtpError;
use crate::runtime::value::Value;
use crate::taint::{DynamicTaintTracker, TaintLevel, TaintSource};
use std::collections::HashMap;
use std::time::{Duration, Instant};

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

/// JavaScript subset interpreter
pub struct Interpreter {
    config: InterpreterConfig,
    start_time: Instant,
    gas_used: u64,
    memory_used: usize,
    globals: HashMap<String, Value>,
    taint_tracker: DynamicTaintTracker,
}

impl Interpreter {
    pub fn new(config: InterpreterConfig) -> Self {
        let mut globals = HashMap::new();

        // Add built-in functions
        globals.insert(
            "console.log".to_string(),
            Value::String("builtin".to_string()),
        );

        Interpreter {
            config,
            start_time: Instant::now(),
            gas_used: 0,
            memory_used: 0,
            globals,
            taint_tracker: DynamicTaintTracker::new(),
        }
    }

    /// Execute JavaScript code
    pub fn execute(&mut self, js_code: &str) -> Result<Value, MtpError> {
        self.check_timeout()?;

        // Very basic JS interpretation
        let mut lines = js_code.lines();
        let mut result = Value::Null;

        for line in lines {
            let line = line.trim();
            if line.is_empty() || line.starts_with("//") {
                continue;
            }
            if line.contains("return") {
                // Extract return value
                if let Some(return_part) = line.strip_prefix("return ") {
                    if let Some(semicolon) = return_part.strip_suffix(";") {
                        result = Value::String(semicolon.to_string());
                    } else {
                        result = Value::String(return_part.to_string());
                    }
                }
                break;
            }
            // Handle assignments
            if line.contains(" = ") {
                // For now, just acknowledge the assignment
                continue;
            }
            // Handle function calls
            if line.contains("(") && line.contains(")") {
                // For now, just acknowledge the call
                continue;
            }
        }

        Ok(result)
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

    /// Get memory used in MB
    pub fn memory_used_mb(&self) -> f64 {
        self.memory_used as f64 / (1024.0 * 1024.0)
    }

    /// Get execution time
    pub fn execution_time(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Set gas limit
    pub fn set_gas_limit(&mut self, limit: u64) {
        self.config.gas_limit = limit;
    }

    /// Get gas used
    pub fn gas_used(&self) -> u64 {
        self.gas_used
    }

    /// Execute code and return JSON result
    pub fn execute_to_json(&mut self, code: &str) -> Result<String, MtpError> {
        let result = self.execute(code)?;
        Ok(format!("{:?}", result)) // Placeholder JSON serialization
    }

    /// Call a global function
    pub fn call_global_function(
        &mut self,
        name: &str,
        args: Vec<Value>,
    ) -> Result<Value, MtpError> {
        match self.globals.get(name) {
            Some(func) => {
                // Placeholder function call implementation
                match name {
                    "console.log" => {
                        println!("{:?}", args);
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

    /// Inject built-in objects
    pub fn inject_builtin_objects(&mut self) {
        // Already done in constructor
    }

    /// Get gas used
    pub fn gas_used(&self) -> u64 {
        self.gas_used
    }

    /// Execute code and return JSON result
    pub fn execute_to_json(&mut self, code: &str) -> Result<String, MtpError> {
        let result = self.execute(code)?;
        Ok(format!("{:?}", result)) // Placeholder JSON serialization
    }

    /// Call a global function
    pub fn call_global_function(
        &mut self,
        name: &str,
        args: Vec<Value>,
    ) -> Result<Value, MtpError> {
        match self.globals.get(name) {
            Some(func) => {
                // Placeholder function call implementation
                match name {
                    "console.log" => {
                        println!("{:?}", args);
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

    /// Inject built-in objects
    pub fn inject_builtin_objects(&mut self) {
        // Already done in constructor
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
        self.taint_tracker
            .check_dangerous_operation(operation, data_key)
    }

    /// Propagate taint through operations
    pub fn propagate_taint(&mut self, result_key: &str, input_keys: &[&str]) {
        self.taint_tracker.propagate(result_key, input_keys);
    }

    /// Get taint report
    pub fn get_taint_report(&self) -> String {
        // Simple report for now
        format!(
            "Taint tracking active. Call stack depth: {}",
            self.taint_tracker.get_call_stack().len()
        )
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
}
