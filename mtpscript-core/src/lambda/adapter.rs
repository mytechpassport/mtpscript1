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
        _interpreter: &mut Interpreter,
        _seed: &[u8; 32],
    ) -> Result<(), MtpError> {
        // In a real implementation, this would inject a function that safely accesses env vars
        // For Lambda, we might restrict to specific allowed env vars
        Ok(())
    }

    /// Inject structured logging effect
    fn inject_logging_effect(
        &self,
        _interpreter: &mut Interpreter,
        _seed: &[u8; 32],
    ) -> Result<(), MtpError> {
        // Lambda logs go to CloudWatch via stdout/stderr
        Ok(())
    }

    /// Inject deterministic time effect
    fn inject_time_effect(
        &self,
        _interpreter: &mut Interpreter,
        _seed: &[u8; 32],
    ) -> Result<(), MtpError> {
        // Time should be deterministic based on seed, not wall clock
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
