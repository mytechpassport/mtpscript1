use crate::errors::MtpError;
use crate::runtime::value::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Isolated runtime environment for request execution
pub struct IsolatedRuntime {
    pub id: String,
    pub globals: HashMap<String, Value>,
    pub heap: HashMap<String, Value>,
    pub stack: Vec<Value>,
    pub gas_remaining: u64,
    pub memory_used: usize,
    pub start_time: std::time::Instant,
}

impl IsolatedRuntime {
    pub fn new(id: String, gas_limit: u64, memory_limit_mb: usize) -> Self {
        IsolatedRuntime {
            id,
            globals: HashMap::new(),
            heap: HashMap::new(),
            stack: Vec::new(),
            gas_remaining: gas_limit,
            memory_used: 0,
            start_time: std::time::Instant::now(),
        }
    }

    /// Execute code in isolation
    pub fn execute(&mut self, code: &str) -> Result<Value, MtpError> {
        // Check execution timeout
        if self.start_time.elapsed() > std::time::Duration::from_secs(30) {
            return Err(MtpError::RuntimeError("Execution timeout".into()));
        }

        // Simulate code execution with isolation checks
        self.validate_no_global_leakage()?;
        
        // Placeholder execution
        Ok(Value::String(format!("isolated_result_{}", self.id)))
    }

    /// Allocate memory in isolated heap
    pub fn allocate(&mut self, key: String, value: Value) -> Result<(), MtpError> {
        let size_estimate = self.estimate_size(&value);
        
        if self.memory_used + size_estimate > 100 * 1024 * 1024 { // 100MB limit
            return Err(MtpError::RuntimeError("Memory limit exceeded in isolated runtime".into()));
        }

        self.heap.insert(key, value);
        self.memory_used += size_estimate;
        
        Ok(())
    }

    /// Access isolated heap
    pub fn get_heap(&self, key: &str) -> Option<&Value> {
        self.heap.get(key)
    }

    /// Check that no global state is leaked between requests
    fn validate_no_global_leakage(&self) -> Result<(), MtpError> {
        // In a real implementation, this would check that the runtime
        // hasn't modified any shared global state
        
        // Check that execution hasn't exceeded time limits
        if self.start_time.elapsed() > std::time::Duration::from_secs(30) {
            return Err(MtpError::RuntimeError("Execution timeout in isolated runtime".into()));
        }

        // Check gas remaining
        if self.gas_remaining == 0 {
            return Err(MtpError::RuntimeError("Gas exhausted in isolated runtime".into()));
        }

        Ok(())
    }

    /// Estimate memory usage of a value
    fn estimate_size(&self, value: &Value) -> usize {
        match value {
            Value::Number(_) => 8,
            Value::Boolean(_) => 1,
            Value::String(s) => s.len(),
            Value::Array(arr) => arr.len() * std::mem::size_of::<Value>(),
            Value::Object(obj) => obj.len() * 64, // Rough estimate
            Value::Function(_) => 1024, // Function objects are larger
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
            _ => {} // Other types don't need zeroing
        }
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
    pub fn create_runtime(&self, id: String, gas_limit: u64) -> Result<Arc<Mutex<IsolatedRuntime>>, MtpError> {
        let mut runtimes = self.runtimes.lock().unwrap();
        
        if runtimes.len() >= self.max_concurrent_runtimes {
            return Err(MtpError::RuntimeError("Maximum concurrent runtimes exceeded".into()));
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
            rt1.allocate("key1".to_string(), Value::String("secret1".to_string())).unwrap();
            assert_eq!(rt1.get_heap("key1").unwrap().to_string(), "secret1");
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
    fn test_memory_limits() {
        let manager = RuntimeManager::new(10);
        let runtime = manager.create_runtime("test".to_string(), 1000).unwrap();
        
        {
            let mut rt = runtime.lock().unwrap();
            
            // Should succeed
            rt.allocate("small".to_string(), Value::String("x".to_string())).unwrap();
            
            // Should fail - too much memory
            let large_string = "x".repeat(200 * 1024 * 1024); // 200MB
            assert!(rt.allocate("large".to_string(), Value::String(large_string)).is_err());
        }
        
        manager.cleanup_runtime("test").unwrap();
    }
}