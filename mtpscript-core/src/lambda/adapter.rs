use crate::errors::MtpError;
use crate::runtime::Interpreter;
use serde::{Deserialize, Serialize};
use std::env;

/// Lambda adapter for host effects
pub struct LambdaAdapter {
    gas_limit: u64,
}

impl LambdaAdapter {
    /// Create a new Lambda adapter
    pub fn new() -> Result<Self, MtpError> {
        let gas_limit = env::var("MTP_GAS_LIMIT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10_000_000);

        // Validate gas limit range
        if gas_limit < 1 || gas_limit > 2_000_000_000 {
            return Err(MtpError::GasLimitOutOfRange {
                error: "GasLimitOutOfRange".to_string(),
                provided: gas_limit,
                min: 1,
                max: 2_000_000_000,
            });
        }

        Ok(Self { gas_limit })
    }

    /// Inject Lambda-specific effects into interpreter
    pub fn inject_lambda_effects(
        &self,
        interpreter: &mut Interpreter,
        seed: &[u8; 32],
    ) -> Result<(), MtpError> {
        // Inject gas limit into interpreter
        interpreter.set_gas_limit(self.gas_limit);

        // Log gas limit to audit
        eprintln!("AUDIT: gasLimit={}", self.gas_limit);

        // Inject Lambda-specific effects
        self.inject_environment_effect(interpreter, seed)?;
        self.inject_logging_effect(interpreter, seed)?;
        self.inject_time_effect(interpreter, seed)?;

        Ok(())
    }

    /// Inject environment variable access effect
    fn inject_environment_effect(
        &self,
        interpreter: &mut Interpreter,
        _seed: &[u8; 32],
    ) -> Result<(), MtpError> {
        use crate::runtime::interpreter::{JsExpr, StoredFunction};
        use crate::runtime::value::{FunctionValue, Value};
        use std::collections::HashMap;

        // Allowed environment variables for Lambda
        let _allowed_vars = vec![
            "AWS_LAMBDA_FUNCTION_NAME",
            "AWS_LAMBDA_FUNCTION_VERSION",
            "AWS_LAMBDA_FUNCTION_MEMORY_SIZE",
            "AWS_REGION",
            "AWS_DEFAULT_REGION",
        ];

        // Register the GetEnv function
        let func = Value::Function(FunctionValue {
            name: Some("GetEnv".to_string()),
            params: vec!["name".to_string()],
            closure: HashMap::new(),
        });
        interpreter.global_scope.insert("GetEnv".to_string(), func);

        // Register the function body that calls the builtin
        let body = JsExpr::Call(
            Box::new(JsExpr::Ident("getenv_impl".to_string())),
            vec![JsExpr::Ident("name".to_string())],
        );
        interpreter.function_bodies.insert(
            "GetEnv".to_string(),
            StoredFunction {
                params: vec!["name".to_string()],
                body: Box::new(body),
            },
        );

        // Register the builtin implementation
        interpreter
            .builtins
            .insert("getenv_impl".to_string(), |args| {
                if args.len() != 1 {
                    return Err("getenv_impl expects 1 argument".to_string());
                }
                let name = match &args[0] {
                    Value::String(s) => s,
                    _ => return Err("Environment variable name must be a string".to_string()),
                };

                // Only allow specific environment variables
                let allowed = [
                    "AWS_LAMBDA_FUNCTION_NAME",
                    "AWS_LAMBDA_FUNCTION_VERSION",
                    "AWS_LAMBDA_FUNCTION_MEMORY_SIZE",
                    "AWS_REGION",
                    "AWS_DEFAULT_REGION",
                ];

                if !allowed.contains(&name.as_str()) {
                    return Ok(Value::Null);
                }

                match env::var(name) {
                    Ok(val) => Ok(Value::String(val)),
                    Err(_) => Ok(Value::Null),
                }
            });

        Ok(())
    }

    /// Inject structured logging effect
    fn inject_logging_effect(
        &self,
        interpreter: &mut Interpreter,
        _seed: &[u8; 32],
    ) -> Result<(), MtpError> {
        use crate::runtime::interpreter::{JsExpr, StoredFunction};
        use crate::runtime::value::{FunctionValue, Value};
        use std::collections::HashMap;

        // Register the LambdaLog function
        let func = Value::Function(FunctionValue {
            name: Some("LambdaLog".to_string()),
            params: vec!["level".to_string(), "message".to_string()],
            closure: HashMap::new(),
        });
        interpreter
            .global_scope
            .insert("LambdaLog".to_string(), func);

        // Register the function body
        let body = JsExpr::Call(
            Box::new(JsExpr::Ident("lambda_log_impl".to_string())),
            vec![
                JsExpr::Ident("level".to_string()),
                JsExpr::Ident("message".to_string()),
            ],
        );
        interpreter.function_bodies.insert(
            "LambdaLog".to_string(),
            StoredFunction {
                params: vec!["level".to_string(), "message".to_string()],
                body: Box::new(body),
            },
        );

        // Register the builtin implementation
        interpreter
            .builtins
            .insert("lambda_log_impl".to_string(), |args| {
                if args.len() != 2 {
                    return Err("lambda_log_impl expects 2 arguments".to_string());
                }
                let level = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => "INFO".to_string(),
                };
                let message = match &args[1] {
                    Value::String(s) => s.clone(),
                    other => format!("{}", other),
                };

                // Log to stderr in structured JSON format for CloudWatch
                let log_entry = serde_json::json!({
                    "level": level,
                    "message": message
                });
                eprintln!("{}", log_entry);

                Ok(Value::Boolean(true))
            });

        Ok(())
    }

    /// Inject deterministic time effect
    fn inject_time_effect(
        &self,
        interpreter: &mut Interpreter,
        seed: &[u8; 32],
    ) -> Result<(), MtpError> {
        use crate::runtime::interpreter::{JsExpr, StoredFunction};
        use crate::runtime::value::{FunctionValue, Value};
        use sha2::{Digest, Sha256};
        use std::collections::HashMap;

        // Compute deterministic timestamp from seed
        let mut hasher = Sha256::new();
        hasher.update(seed);
        hasher.update(b"time_seed");
        let hash = hasher.finalize();

        // Use first 8 bytes as a deterministic "timestamp" offset
        let time_offset = u64::from_le_bytes(hash[0..8].try_into().unwrap());

        // Register the GetTime function
        let func = Value::Function(FunctionValue {
            name: Some("GetTime".to_string()),
            params: vec![],
            closure: HashMap::new(),
        });
        interpreter.global_scope.insert("GetTime".to_string(), func);

        // Store the deterministic time value in global scope for the builtin to access
        interpreter.global_scope.insert(
            "__deterministic_time".to_string(),
            Value::Number(time_offset as i64),
        );

        // Register the function body
        let body = JsExpr::Call(Box::new(JsExpr::Ident("gettime_impl".to_string())), vec![]);
        interpreter.function_bodies.insert(
            "GetTime".to_string(),
            StoredFunction {
                params: vec![],
                body: Box::new(body),
            },
        );

        // Register the builtin implementation
        interpreter
            .builtins
            .insert("gettime_impl".to_string(), |_args| {
                // Return deterministic timestamp based on seed
                // In real implementation, this would be computed from the seed
                // For now, return a fixed value that represents "Lambda invocation time"
                Ok(Value::Number(1704067200000)) // 2024-01-01T00:00:00Z in milliseconds
            });

        Ok(())
    }

    /// Get current gas limit
    pub fn gas_limit(&self) -> u64 {
        self.gas_limit
    }
}

/// Lambda invocation metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LambdaMetadata {
    pub request_id: String,
    pub account_id: String,
    pub function_version: String,
    pub function_name: String,
    pub memory_size: u32,
    pub remaining_time_in_millis: u32,
}

/// Parse Lambda metadata from environment
pub fn get_lambda_metadata() -> Result<LambdaMetadata, MtpError> {
    Ok(LambdaMetadata {
        request_id: env::var("_X_AMZN_TRACE_ID").unwrap_or_else(|_| "unknown".to_string()),
        account_id: env::var("AWS_ACCOUNT_ID").unwrap_or_else(|_| "unknown".to_string()),
        function_version: env::var("AWS_LAMBDA_FUNCTION_VERSION")
            .unwrap_or_else(|_| "$LATEST".to_string()),
        function_name: env::var("AWS_LAMBDA_FUNCTION_NAME")
            .unwrap_or_else(|_| "unknown".to_string()),
        memory_size: env::var("AWS_LAMBDA_FUNCTION_MEMORY_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(128),
        remaining_time_in_millis: 30000, // Would be injected by runtime
    })
}

/// Validate Lambda environment
pub fn validate_lambda_environment() -> Result<(), MtpError> {
    let required_vars = [
        "AWS_LAMBDA_RUNTIME_API",
        "AWS_LAMBDA_FUNCTION_NAME",
        "AWS_LAMBDA_FUNCTION_VERSION",
    ];

    for var in &required_vars {
        if env::var(var).is_err() {
            return Err(MtpError::Runtime {
                error: "Runtime".to_string(),
                message: format!("Required environment variable {} not set", var),
            });
        }
    }

    Ok(())
}

/// Lambda error response
#[derive(Debug, Serialize)]
pub struct LambdaError {
    pub error_message: String,
    pub error_type: String,
    pub stack_trace: Option<Vec<String>>,
}

impl LambdaError {
    pub fn from_mtp_error(error: &MtpError) -> Self {
        Self {
            error_message: error.to_string(),
            error_type: match error {
                MtpError::GasExhausted { .. } => "GasExhausted",
                MtpError::Security { .. } => "SecurityError",
                MtpError::Runtime { .. } => "RuntimeError",
                MtpError::Build { .. } => "BuildError",
                MtpError::Io { .. } => "IoError",
                MtpError::GasLimitOutOfRange { .. } => "GasLimitOutOfRange",
                MtpError::IntegrityError { .. } => "IntegrityError",
                MtpError::RateLimitError { .. } => "RateLimitError",
                MtpError::ValidationError { .. } => "ValidationError",
                MtpError::ModuleError { .. } => "ModuleError",
                MtpError::TypeError { .. } => "TypeError",
                MtpError::ParseError { .. } => "ParseError",
            }
            .to_string(),
            stack_trace: None, // MTPScript doesn't expose stack traces in production
        }
    }
}

/// Send error response to Lambda runtime API
pub fn send_lambda_error(error: &MtpError, request_id: &str) -> Result<(), MtpError> {
    let runtime_api = env::var("AWS_LAMBDA_RUNTIME_API").map_err(|_| MtpError::Runtime {
        error: "Runtime".to_string(),
        message: "AWS_LAMBDA_RUNTIME_API not set".to_string(),
    })?;

    let client = reqwest::blocking::Client::new();
    let url = format!(
        "http://{}/2018-06-01/runtime/invocation/{}/error",
        runtime_api, request_id
    );

    let lambda_error = LambdaError::from_mtp_error(error);
    let body = serde_json::to_string(&lambda_error)?;

    let _ = client
        .post(&url)
        .body(body)
        .send()
        .map_err(|e| MtpError::Runtime {
            error: "Runtime".to_string(),
            message: format!("Failed to send error: {}", e),
        })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gas_limit_validation() {
        // Valid range
        env::set_var("MTP_GAS_LIMIT", "1000000");
        let adapter = LambdaAdapter::new();
        assert!(adapter.is_ok());

        // Too low
        env::set_var("MTP_GAS_LIMIT", "0");
        let adapter = LambdaAdapter::new();
        assert!(adapter.is_err());

        // Too high
        env::set_var("MTP_GAS_LIMIT", "3000000000");
        let adapter = LambdaAdapter::new();
        assert!(adapter.is_err());
    }

    #[test]
    fn test_lambda_error_creation() {
        let error = MtpError::GasExhausted {
            error: "GasExhausted".to_string(),
            gasLimit: 1000,
            gasUsed: 1001,
        };
        let lambda_error = LambdaError::from_mtp_error(&error);

        assert_eq!(lambda_error.error_type, "GasExhausted");
        assert!(lambda_error.error_message.contains("1000"));
        assert!(lambda_error.stack_trace.is_none());
    }

    #[test]
    fn test_metadata_parsing() {
        env::set_var("AWS_LAMBDA_FUNCTION_NAME", "test-function");
        env::set_var("AWS_LAMBDA_FUNCTION_VERSION", "1");
        env::set_var("AWS_LAMBDA_FUNCTION_MEMORY_SIZE", "256");

        let metadata = get_lambda_metadata().unwrap();
        assert_eq!(metadata.function_name, "test-function");
        assert_eq!(metadata.memory_size, 256);
    }
}
