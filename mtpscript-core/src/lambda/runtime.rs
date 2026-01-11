use crate::errors::MtpError;
use crate::runtime::{clone_interpreter, inject_effects, value::Value, Interpreter};
use crate::security::SandboxConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::time::Instant;

/// Lambda runtime context
#[derive(Debug, Clone)]
pub struct LambdaRuntime {
    snapshot_path: String,
    gas_limit: u64,
    sandbox_config: SandboxConfig,
}

/// Lambda invocation payload
#[derive(Debug, Clone, Deserialize)]
pub struct LambdaPayload {
    pub request_id: String,
    pub account_id: String,
    pub function_version: String,
    pub body: serde_json::Value,
    pub headers: HashMap<String, String>,
}

/// Lambda response
#[derive(Debug, Serialize)]
pub struct LambdaResponse {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}

/// AWS Lambda custom runtime implementation
impl LambdaRuntime {
    /// Create a new Lambda runtime
    pub fn new(snapshot_path: String) -> Self {
        let gas_limit = env::var("MTP_GAS_LIMIT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10_000_000);

        Self {
            snapshot_path,
            gas_limit,
            sandbox_config: SandboxConfig::default(),
        }
    }

    /// Main runtime loop
    pub fn run(&self) -> Result<(), MtpError> {
        loop {
            // Get next invocation
            let invocation = self.get_next_invocation()?;

            // Process the invocation
            let response = self.process_invocation(invocation)?;

            // Send response
            self.send_response(&response)?;

            // Optional: send initialization error if needed
            // This would be for cold start failures
        }
    }

    /// Get next invocation from Lambda runtime API
    fn get_next_invocation(&self) -> Result<LambdaPayload, MtpError> {
        // Call Lambda Runtime API
        let runtime_api = env::var("AWS_LAMBDA_RUNTIME_API")
            .map_err(|_| MtpError::Runtime("AWS_LAMBDA_RUNTIME_API not set".to_string()))?;

        let client = reqwest::blocking::Client::new();
        let url = format!("http://{}/2018-06-01/runtime/invocation/next", runtime_api);

        let response = client
            .get(&url)
            .send()
            .map_err(|e| MtpError::Runtime(format!("Failed to get invocation: {}", e)))?;

        if !response.status().is_success() {
            return Err(MtpError::Runtime("Failed to get invocation".to_string()));
        }

        let request_id = response
            .headers()
            .get("lambda-runtime-aws-request-id")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("")
            .to_string();

        let account_id = env::var("AWS_ACCOUNT_ID").unwrap_or_else(|_| "unknown".to_string());

        let function_version =
            env::var("AWS_LAMBDA_FUNCTION_VERSION").unwrap_or_else(|_| "$LATEST".to_string());

        let headers = response
            .headers()
            .iter()
            .filter_map(|(k, v)| v.to_str().ok().map(|v| (k.to_string(), v.to_string())))
            .collect();

        let body: serde_json::Value = response
            .json()
            .map_err(|e| MtpError::Runtime(format!("Failed to parse invocation body: {}", e)))?;

        Ok(LambdaPayload {
            request_id,
            account_id,
            function_version,
            body,
            headers,
        })
    }

    /// Process a single invocation
    fn process_invocation(&self, payload: LambdaPayload) -> Result<LambdaResponse, MtpError> {
        let start_time = Instant::now();

        // Load snapshot
        let snapshot_data = fs::read(&self.snapshot_path)?;

        // Clone interpreter (should be < 2ms worst case)
        let clone_start = Instant::now();
        let mut interpreter = clone_interpreter(&snapshot_data)?;
        let clone_duration = clone_start.elapsed();

        // Log cold start performance
        if clone_duration.as_millis() >= 2 {
            eprintln!("Warning: Clone took {}ms", clone_duration.as_millis());
        }

        // Compute deterministic seed
        let seed = self.compute_seed(&payload)?;

        // Inject effects
        inject_effects(&mut interpreter, &seed)?;

        // Set gas limit
        interpreter.set_gas_limit(self.gas_limit);

        // Execute the API handler
        // In a real implementation, this would route to the correct API based on the payload
        let result = self.execute_api_handler(&mut interpreter, &payload)?;

        // Create response
        let response = LambdaResponse {
            status_code: 200,
            headers: HashMap::from([
                ("content-type".to_string(), "application/json".to_string()),
                ("x-mtp-seed".to_string(), hex::encode(&seed)),
                (
                    "x-mtp-gas-used".to_string(),
                    interpreter.gas_used().to_string(),
                ),
            ]),
            body: result,
        };

        Ok(response)
    }

    /// Compute deterministic seed for this invocation
    fn compute_seed(&self, payload: &LambdaPayload) -> Result<[u8; 32], MtpError> {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(payload.request_id.as_bytes());
        hasher.update(payload.account_id.as_bytes());
        hasher.update(payload.function_version.as_bytes());
        hasher.update(b"mtpscript-v5.1");

        // Add snapshot content hash
        let snapshot_data = fs::read(&self.snapshot_path)?;
        let snapshot_hash = Sha256::new().chain_update(&snapshot_data).finalize();
        hasher.update(&snapshot_hash);

        // Add gas limit
        hasher.update(self.gas_limit.to_string().as_bytes());

        let result = hasher.finalize();
        let mut seed = [0u8; 32];
        seed.copy_from_slice(&result);
        Ok(seed)
    }

    /// Execute the appropriate API handler
    fn execute_api_handler(
        &self,
        interpreter: &mut Interpreter,
        payload: &LambdaPayload,
    ) -> Result<String, MtpError> {
        // In a real implementation, this would parse the MTPScript APIs and route based on method/path
        // For now, assume a single handler

        // Convert payload to JSON for the handler
        let input_json = serde_json::to_string(&payload.body)?;

        // Execute handler (this is a placeholder - would call the actual API)
        let result =
            interpreter.call_global_function("handler", vec![Value::String(input_json)])?;

        // Convert result back to string
        match result {
            Value::String(s) => Ok(s),
            _ => Ok(result.to_string()),
        }
    }

    /// Send response to Lambda runtime API
    fn send_response(&self, response: &LambdaResponse) -> Result<(), MtpError> {
        let runtime_api = env::var("AWS_LAMBDA_RUNTIME_API")
            .map_err(|_| MtpError::Runtime("AWS_LAMBDA_RUNTIME_API not set".to_string()))?;

        let request_id = env::var("_X_AMZN_TRACE_ID").unwrap_or_else(|_| "unknown".to_string());

        let client = reqwest::blocking::Client::new();
        let url = format!(
            "http://{}/2018-06-01/runtime/invocation/{}/response",
            runtime_api, request_id
        );

        let response_body = serde_json::to_string(response)?;

        let lambda_response = client
            .post(&url)
            .body(response_body)
            .send()
            .map_err(|e| MtpError::Runtime(format!("Failed to send response: {}", e)))?;

        if !lambda_response.status().is_success() {
            return Err(MtpError::Runtime("Failed to send response".to_string()));
        }

        Ok(())
    }

    /// Send initialization error (for cold start failures)
    pub fn send_init_error(&self, error: &MtpError) -> Result<(), MtpError> {
        let runtime_api = env::var("AWS_LAMBDA_RUNTIME_API")
            .map_err(|_| MtpError::Runtime("AWS_LAMBDA_RUNTIME_API not set".to_string()))?;

        let client = reqwest::blocking::Client::new();
        let url = format!("http://{}/2018-06-01/runtime/init/error", runtime_api);

        let error_body = serde_json::json!({
            "errorMessage": error.to_string(),
            "errorType": "InitializationError"
        });

        let _ = client
            .post(&url)
            .json(&error_body)
            .send()
            .map_err(|e| MtpError::Runtime(format!("Failed to send init error: {}", e)))?;

        Ok(())
    }
}

/// Optimized cold start: preload snapshot into memory
pub struct PreloadedRuntime {
    snapshot_data: Vec<u8>,
    runtime: LambdaRuntime,
}

impl PreloadedRuntime {
    /// Create preloaded runtime for faster cold starts
    pub fn new(snapshot_path: String) -> Result<Self, MtpError> {
        let snapshot_data = fs::read(&snapshot_path)?;
        let runtime = LambdaRuntime::new(snapshot_path);

        Ok(Self {
            snapshot_data,
            runtime,
        })
    }

    /// Run with preloaded snapshot
    pub fn run(&self) -> Result<(), MtpError> {
        // Use preloaded data instead of reading from disk each time
        let mut runtime = self.runtime.clone();
        // Override snapshot loading to use preloaded data

        loop {
            let invocation = runtime.get_next_invocation()?;
            let response = runtime
                .process_invocation_with_snapshot(invocation.clone(), &self.snapshot_data)?;
            runtime.send_response(&response)?;
        }
    }
}

impl LambdaRuntime {
    /// Process invocation with preloaded snapshot
    fn process_invocation_with_snapshot(
        &self,
        payload: LambdaPayload,
        snapshot_data: &[u8],
    ) -> Result<LambdaResponse, MtpError> {
        let start_time = Instant::now();

        // Clone interpreter directly from preloaded data
        let clone_start = Instant::now();
        let mut interpreter = clone_interpreter(snapshot_data)?;
        let clone_duration = clone_start.elapsed();

        if clone_duration.as_millis() >= 2 {
            eprintln!("Warning: Clone took {}ms", clone_duration.as_millis());
        }

        // Rest same as process_invocation
        let seed = self.compute_seed(&payload)?;
        inject_effects(&mut interpreter, &seed)?;
        interpreter.set_gas_limit(self.gas_limit);

        let result = self.execute_api_handler(&mut interpreter, &payload)?;

        let response = LambdaResponse {
            status_code: 200,
            headers: HashMap::from([
                ("content-type".to_string(), "application/json".to_string()),
                ("x-mtp-seed".to_string(), hex::encode(&seed)),
                (
                    "x-mtp-gas-used".to_string(),
                    interpreter.gas_used().to_string(),
                ),
            ]),
            body: result,
        };

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seed_computation() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Create a temporary snapshot file
        let mut temp_file = NamedTempFile::new().unwrap();
        let js_content = "function main() { return 42; }";
        temp_file.write_all(js_content.as_bytes()).unwrap();
        let snapshot_path = temp_file.path().to_str().unwrap().to_string();

        let runtime = LambdaRuntime::new(snapshot_path);

        let payload = LambdaPayload {
            request_id: "test-request".to_string(),
            account_id: "123456789".to_string(),
            function_version: "1".to_string(),
            body: serde_json::json!({"test": "data"}),
            headers: HashMap::new(),
        };

        let seed = runtime.compute_seed(&payload).unwrap();
        assert_eq!(seed.len(), 32);

        // Same input should produce same seed
        let seed2 = runtime.compute_seed(&payload).unwrap();
        assert_eq!(seed, seed2);
    }

    #[test]
    fn test_response_creation() {
        let response = LambdaResponse {
            status_code: 200,
            headers: HashMap::from([("content-type".to_string(), "application/json".to_string())]),
            body: r#"{"result": "ok"}"#.to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("200"));
        assert!(json.contains("application/json"));
    }
}
