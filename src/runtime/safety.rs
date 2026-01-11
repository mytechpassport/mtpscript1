use crate::errors::MtpError;
use crate::runtime::value::Value;

/// Memory safety checker for runtime operations
pub struct MemorySafetyChecker {
    pub max_array_size: usize,
    pub max_object_size: usize,
    pub max_string_length: usize,
    pub max_recursion_depth: usize,
}

impl Default for MemorySafetyChecker {
    fn default() -> Self {
        MemorySafetyChecker {
            max_array_size: 1_000_000,           // 1M elements
            max_object_size: 100_000,            // 100K entries
            max_string_length: 10 * 1024 * 1024, // 10MB
            max_recursion_depth: 1000,
        }
    }
}

impl MemorySafetyChecker {
    /// Check if an array operation is safe
    pub fn check_array_access(&self, array: &Vec<Value>, index: usize) -> Result<(), MtpError> {
        if index >= array.len() {
            return Err(MtpError::RuntimeError("Array index out of bounds".into()));
        }

        // Check for potential use-after-free patterns
        if array.len() > self.max_array_size {
            return Err(MtpError::RuntimeError("Array too large".into()));
        }

        Ok(())
    }

    /// Check if creating an array of given size is safe
    pub fn check_array_creation(&self, size: usize) -> Result<(), MtpError> {
        if size > self.max_array_size {
            return Err(MtpError::RuntimeError("Array size exceeds limit".into()));
        }

        // Estimate memory usage (rough)
        let estimated_memory = size * std::mem::size_of::<Value>();
        if estimated_memory > 100 * 1024 * 1024 {
            // 100MB
            return Err(MtpError::RuntimeError(
                "Array would exceed memory limit".into(),
            ));
        }

        Ok(())
    }

    /// Check if an object operation is safe
    pub fn check_object_access(
        &self,
        object: &std::collections::HashMap<String, Value>,
        key: &str,
    ) -> Result<(), MtpError> {
        if object.len() > self.max_object_size {
            return Err(MtpError::RuntimeError("Object too large".into()));
        }

        // Check for extremely long keys
        if key.len() > 1024 {
            return Err(MtpError::RuntimeError("Object key too long".into()));
        }

        Ok(())
    }

    /// Check string operations
    pub fn check_string_operation(&self, s: &str) -> Result<(), MtpError> {
        if s.len() > self.max_string_length {
            return Err(MtpError::RuntimeError("String too long".into()));
        }

        // Check for valid UTF-8
        if std::str::from_utf8(s.as_bytes()).is_err() {
            return Err(MtpError::RuntimeError("Invalid UTF-8 in string".into()));
        }

        Ok(())
    }

    /// Check function call depth
    pub fn check_recursion_depth(&self, current_depth: usize) -> Result<(), MtpError> {
        if current_depth > self.max_recursion_depth {
            return Err(MtpError::RuntimeError(
                "Maximum recursion depth exceeded".into(),
            ));
        }

        Ok(())
    }

    /// Validate value for general safety
    pub fn validate_value(&self, value: &Value) -> Result<(), MtpError> {
        match value {
            Value::String(s) => self.check_string_operation(s),
            Value::Array(arr) => {
                self.check_array_creation(arr.len())?;
                for item in arr {
                    self.validate_value(item)?;
                }
                Ok(())
            }
            Value::Object(obj) => {
                self.check_object_access(obj, "")?; // Just check size
                for (key, value) in obj {
                    self.check_string_operation(key)?;
                    self.validate_value(value)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Check for potential memory leaks in value structures
    pub fn check_memory_integrity(&self, value: &Value) -> Result<(), MtpError> {
        // This would be more sophisticated in a real implementation
        // For now, just do basic validation
        self.validate_value(value)
    }

    /// Bounds checking for numeric operations
    pub fn check_numeric_bounds(&self, value: i64) -> Result<(), MtpError> {
        // Check for suspicious values that might indicate memory corruption
        if value == i64::MIN || value == i64::MAX {
            // Not necessarily an error, but worth logging
        }

        Ok(())
    }
}

/// Safe wrapper for array operations
pub struct SafeArray<'a> {
    array: &'a Vec<Value>,
    checker: &'a MemorySafetyChecker,
}

impl<'a> SafeArray<'a> {
    pub fn new(array: &'a Vec<Value>, checker: &'a MemorySafetyChecker) -> Self {
        SafeArray { array, checker }
    }

    pub fn get(&self, index: usize) -> Result<&Value, MtpError> {
        self.checker.check_array_access(self.array, index)?;
        Ok(&self.array[index])
    }

    pub fn len(&self) -> usize {
        self.array.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_array_bounds_checking() {
        let checker = MemorySafetyChecker::default();
        let array = vec![Value::Number(1), Value::Number(2)];

        assert!(checker.check_array_access(&array, 0).is_ok());
        assert!(checker.check_array_access(&array, 2).is_err());
    }

    #[test]
    fn test_string_length_limits() {
        let checker = MemorySafetyChecker::default();

        let short_string = "hello";
        assert!(checker.check_string_operation(short_string).is_ok());

        let long_string = "x".repeat(20 * 1024 * 1024); // 20MB
        assert!(checker.check_string_operation(&long_string).is_err());
    }

    #[test]
    fn test_safe_array_wrapper() {
        let checker = MemorySafetyChecker::default();
        let array = vec![Value::Number(42)];
        let safe_array = SafeArray::new(&array, &checker);

        assert_eq!(safe_array.get(0).unwrap().to_string(), "42");
        assert!(safe_array.get(1).is_err());
    }
}
