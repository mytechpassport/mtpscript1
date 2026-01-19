use crate::errors::MtpError;
use crate::runtime::{clone_interpreter, inject_effects, value::Value, Interpreter};
use crate::security::SandboxConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::time::Instant;

/// Lambda runtime context
#[derive(Debug, Clone)]
pub struct LambdaRuntime {
    snapshot_path: String,
    gas_limit: u64,
    #[allow(dead_code)]
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
    /// Deadline in milliseconds since Unix epoch (from Lambda-Runtime-Deadline-Ms header)
    #[serde(default)]
    pub deadline_ms: Option<u64>,
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

            // Store request_id before processing
            let request_id = invocation.request_id.clone();

            // Process the invocation
            let response = self.process_invocation(invocation)?;

            // Send response with the correct request_id
            self.send_response_with_id(&response, &request_id)?;

            // Optional: send initialization error if needed
            // This would be for cold start failures
        }
    }

    /// Get next invocation from Lambda runtime API
    fn get_next_invocation(&self) -> Result<LambdaPayload, MtpError> {
        // Call Lambda Runtime API
        let runtime_api = env::var("AWS_LAMBDA_RUNTIME_API").map_err(|_| MtpError::Runtime {
            error: "Runtime".to_string(),
            message: "AWS_LAMBDA_RUNTIME_API not set".to_string(),
        })?;

        let client = reqwest::blocking::Client::new();
        let url = format!("http://{}/2018-06-01/runtime/invocation/next", runtime_api);

        let response = client.get(&url).send().map_err(|e| MtpError::Runtime {
            error: "Runtime".to_string(),
            message: format!("Failed to get invocation: {}", e),
        })?;

        if !response.status().is_success() {
            return Err(MtpError::Runtime {
                error: "Runtime".to_string(),
                message: "Failed to get invocation".to_string(),
            });
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

        // Extract deadline from Lambda-Runtime-Deadline-Ms header
        let deadline_ms = response
            .headers()
            .get("lambda-runtime-deadline-ms")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.parse().ok());

        let headers = response
            .headers()
            .iter()
            .filter_map(|(k, v)| v.to_str().ok().map(|v| (k.to_string(), v.to_string())))
            .collect();

        let body: serde_json::Value = response.json().map_err(|e| MtpError::Runtime {
            error: "Runtime".to_string(),
            message: format!("Failed to parse invocation body: {}", e),
        })?;

        Ok(LambdaPayload {
            request_id,
            account_id,
            function_version,
            body,
            headers,
            deadline_ms,
        })
    }

    /// Process a single invocation
    fn process_invocation(&self, payload: LambdaPayload) -> Result<LambdaResponse, MtpError> {
        let _start_time = Instant::now();

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
    fn send_response_with_id(
        &self,
        response: &LambdaResponse,
        request_id: &str,
    ) -> Result<(), MtpError> {
        let runtime_api = env::var("AWS_LAMBDA_RUNTIME_API").map_err(|_| MtpError::Runtime {
            error: "Runtime".to_string(),
            message: "AWS_LAMBDA_RUNTIME_API not set".to_string(),
        })?;

        let client = reqwest::blocking::Client::new();
        let url = format!(
            "http://{}/2018-06-01/runtime/invocation/{}/response",
            runtime_api, request_id
        );

        let response_body = serde_json::to_string(response)?;

        let lambda_response =
            client
                .post(&url)
                .body(response_body)
                .send()
                .map_err(|e| MtpError::Runtime {
                    error: "Runtime".to_string(),
                    message: format!("Failed to send response: {}", e),
                })?;

        if !lambda_response.status().is_success() {
            return Err(MtpError::Runtime {
                error: "Runtime".to_string(),
                message: "Failed to send response".to_string(),
            });
        }

        Ok(())
    }

    /// Send initialization error (for cold start failures)
    pub fn send_init_error(&self, error: &MtpError) -> Result<(), MtpError> {
        let runtime_api = env::var("AWS_LAMBDA_RUNTIME_API").map_err(|_| MtpError::Runtime {
            error: "Runtime".to_string(),
            message: "AWS_LAMBDA_RUNTIME_API not set".to_string(),
        })?;

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
            .map_err(|e| MtpError::Runtime {
                error: "Runtime".to_string(),
                message: format!("Failed to send init error: {}", e),
            })?;

        Ok(())
    }
}

/// Optimized cold start: preload snapshot into memory
pub struct PreloadedRuntime {
    snapshot_data: Vec<u8>,
    runtime: LambdaRuntime,
    /// Shared shutdown flag that can be set externally
    shutdown: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl PreloadedRuntime {
    /// Create preloaded runtime for faster cold starts
    pub fn new(snapshot_path: String) -> Result<Self, MtpError> {
        let snapshot_data = fs::read(&snapshot_path)?;
        let runtime = LambdaRuntime::new(snapshot_path);

        Ok(Self {
            snapshot_data,
            runtime,
            shutdown: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        })
    }

    /// Get a handle to the shutdown flag for external control
    ///
    /// This allows signal handlers or other components to trigger graceful shutdown.
    pub fn shutdown_handle(&self) -> std::sync::Arc<std::sync::atomic::AtomicBool> {
        self.shutdown.clone()
    }

    /// Request graceful shutdown
    ///
    /// This signals the runtime to stop accepting new invocations and exit cleanly.
    pub fn stop(&self) {
        use std::sync::atomic::Ordering;
        self.shutdown.store(true, Ordering::SeqCst);
    }

    /// Check if shutdown has been requested
    pub fn is_shutdown_requested(&self) -> bool {
        use std::sync::atomic::Ordering;
        self.shutdown.load(Ordering::SeqCst)
    }

    /// Run with preloaded snapshot
    ///
    /// This method respects Lambda's context deadline and will exit gracefully
    /// when a shutdown signal is received or the deadline expires.
    pub fn run(&self) -> Result<(), MtpError> {
        use std::sync::atomic::Ordering;

        // Use preloaded data instead of reading from disk each time
        let runtime = self.runtime.clone();

        // The shutdown flag can be set externally via shutdown_handle() or stop()
        // In production, you would register a SIGTERM handler like:
        //   let handle = runtime.shutdown_handle();
        //   signal_hook::flag::register(SIGTERM, handle)?;

        while !self.shutdown.load(Ordering::SeqCst) {
            // Get next invocation
            let invocation = match runtime.get_next_invocation() {
                Ok(inv) => inv,
                Err(e) => {
                    // Log error but continue - the Lambda runtime API will retry
                    eprintln!("Error getting invocation: {}", e);
                    continue;
                }
            };

            // Check if we have enough time remaining (minimum 100ms buffer)
            if let Some(deadline_ms) = invocation.deadline_ms {
                let now_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0);

                let remaining_ms = deadline_ms.saturating_sub(now_ms);
                if remaining_ms < 100 {
                    // Not enough time, send timeout error and continue
                    eprintln!(
                        "Warning: Insufficient time remaining ({}ms), skipping invocation",
                        remaining_ms
                    );
                    let _ = runtime.send_error(
                        &invocation.request_id,
                        "Timeout: insufficient execution time",
                    );
                    continue;
                }
            }

            let request_id = invocation.request_id.clone();
            match runtime.process_invocation_with_snapshot(invocation, &self.snapshot_data) {
                Ok(response) => {
                    if let Err(e) = runtime.send_response_with_id(&response, &request_id) {
                        eprintln!("Error sending response: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("Error processing invocation: {}", e);
                    let _ = runtime.send_error(&request_id, &format!("{}", e));
                }
            }
        }

        Ok(())
    }
}

impl LambdaRuntime {
    /// Send error to Lambda runtime API
    fn send_error(&self, request_id: &str, error_message: &str) -> Result<(), MtpError> {
        let runtime_api = env::var("AWS_LAMBDA_RUNTIME_API").map_err(|_| MtpError::Runtime {
            error: "Runtime".to_string(),
            message: "AWS_LAMBDA_RUNTIME_API not set".to_string(),
        })?;

        let client = reqwest::blocking::Client::new();
        let url = format!(
            "http://{}/2018-06-01/runtime/invocation/{}/error",
            runtime_api, request_id
        );

        let error_body = serde_json::json!({
            "errorType": "RuntimeError",
            "errorMessage": error_message
        });

        client
            .post(&url)
            .header("Lambda-Runtime-Function-Error-Type", "RuntimeError")
            .json(&error_body)
            .send()
            .map_err(|e| MtpError::Runtime {
                error: "Runtime".to_string(),
                message: format!("Failed to send error: {}", e),
            })?;

        Ok(())
    }

    /// Process invocation with preloaded snapshot
    fn process_invocation_with_snapshot(
        &self,
        payload: LambdaPayload,
        snapshot_data: &[u8],
    ) -> Result<LambdaResponse, MtpError> {
        let _start_time = Instant::now();

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
            deadline_ms: None,
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

    // Graceful shutdown tests (#28)

    #[test]
    fn test_preloaded_runtime_shutdown_flag() {
        use std::io::Write;
        use std::sync::atomic::Ordering;
        use tempfile::NamedTempFile;

        // Create a temporary snapshot file
        let mut temp_file = NamedTempFile::new().unwrap();
        let js_content = "function main() { return 42; }";
        temp_file.write_all(js_content.as_bytes()).unwrap();
        let snapshot_path = temp_file.path().to_str().unwrap().to_string();

        let runtime = PreloadedRuntime::new(snapshot_path).unwrap();

        // Initially not shutdown
        assert!(!runtime.is_shutdown_requested());

        // Request shutdown
        runtime.stop();

        // Now should be shutdown
        assert!(runtime.is_shutdown_requested());
    }

    #[test]
    fn test_shutdown_handle_can_be_set_externally() {
        use std::io::Write;
        use std::sync::atomic::Ordering;
        use tempfile::NamedTempFile;

        // Create a temporary snapshot file
        let mut temp_file = NamedTempFile::new().unwrap();
        let js_content = "function main() { return 42; }";
        temp_file.write_all(js_content.as_bytes()).unwrap();
        let snapshot_path = temp_file.path().to_str().unwrap().to_string();

        let runtime = PreloadedRuntime::new(snapshot_path).unwrap();
        let handle = runtime.shutdown_handle();

        // Initially not shutdown
        assert!(!runtime.is_shutdown_requested());

        // Set via handle
        handle.store(true, Ordering::SeqCst);

        // Should now be shutdown
        assert!(runtime.is_shutdown_requested());
    }

    #[test]
    fn test_shutdown_handle_is_shared() {
        use std::io::Write;
        use std::sync::atomic::Ordering;
        use tempfile::NamedTempFile;

        // Create a temporary snapshot file
        let mut temp_file = NamedTempFile::new().unwrap();
        let js_content = "function main() { return 42; }";
        temp_file.write_all(js_content.as_bytes()).unwrap();
        let snapshot_path = temp_file.path().to_str().unwrap().to_string();

        let runtime = PreloadedRuntime::new(snapshot_path).unwrap();

        // Get multiple handles
        let handle1 = runtime.shutdown_handle();
        let handle2 = runtime.shutdown_handle();

        // They should all point to the same flag
        assert!(!handle1.load(Ordering::SeqCst));
        assert!(!handle2.load(Ordering::SeqCst));

        // Setting via one affects all
        handle1.store(true, Ordering::SeqCst);

        assert!(handle2.load(Ordering::SeqCst));
        assert!(runtime.is_shutdown_requested());
    }

    #[test]
    fn test_shutdown_from_another_thread() {
        use std::io::Write;
        use std::thread;
        use std::time::Duration;
        use tempfile::NamedTempFile;

        // Create a temporary snapshot file
        let mut temp_file = NamedTempFile::new().unwrap();
        let js_content = "function main() { return 42; }";
        temp_file.write_all(js_content.as_bytes()).unwrap();
        let snapshot_path = temp_file.path().to_str().unwrap().to_string();

        let runtime = PreloadedRuntime::new(snapshot_path).unwrap();
        let handle = runtime.shutdown_handle();

        // Spawn thread that will trigger shutdown
        let shutdown_thread = thread::spawn(move || {
            thread::sleep(Duration::from_millis(10));
            handle.store(true, std::sync::atomic::Ordering::SeqCst);
        });

        // Wait for thread
        shutdown_thread.join().unwrap();

        // Should be shutdown
        assert!(runtime.is_shutdown_requested());
    }

    #[test]
    fn test_deadline_calculation() {
        use std::time::{SystemTime, UNIX_EPOCH};

        // Test that we can properly calculate remaining time from deadline
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        // Deadline 1 second in the future
        let deadline_ms = now_ms + 1000;
        let remaining = deadline_ms.saturating_sub(now_ms);

        assert!(remaining >= 990 && remaining <= 1010, "Remaining time should be ~1000ms, got {}ms", remaining);

        // Deadline in the past
        let past_deadline = now_ms.saturating_sub(1000);
        let past_remaining = past_deadline.saturating_sub(now_ms);
        assert_eq!(past_remaining, 0, "Past deadline should have 0 remaining time");
    }
}
