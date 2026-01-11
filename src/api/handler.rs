use crate::errors::MtpError;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub struct Request {
    pub method: String,
    pub path: String,
    pub body: String,
    pub headers: HashMap<String, String>,
    pub client_ip: String,
    pub timestamp: Instant,
}

pub struct Response {
    pub status: u16,
    pub body: String,
    pub headers: HashMap<String, String>,
}

/// Rate limiter for API requests
pub struct RateLimiter {
    requests: Mutex<HashMap<String, Vec<Instant>>>,
    max_requests: u32,
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window_seconds: u64) -> Self {
        RateLimiter {
            requests: Mutex::new(HashMap::new()),
            max_requests,
            window: Duration::from_secs(window_seconds),
        }
    }

    /// Check if request is allowed under rate limit
    pub fn check_rate_limit(&self, client_id: &str) -> Result<(), MtpError> {
        let now = Instant::now();
        let mut requests = self.requests.lock().unwrap();

        let client_requests = requests
            .entry(client_id.to_string())
            .or_insert_with(Vec::new);

        // Remove old requests outside the window
        client_requests.retain(|&time| now.duration_since(time) < self.window);

        if client_requests.len() >= self.max_requests as usize {
            return Err(MtpError {
                error: "RateLimitError".to_string(),
                message: Some("Too many requests".to_string()),
                gasLimit: None,
                gasUsed: None,
            });
        }

        client_requests.push(now);
        Ok(())
    }

    /// Clean up old entries periodically
    pub fn cleanup(&self) {
        let now = Instant::now();
        let mut requests = self.requests.lock().unwrap();
        let initial_len = requests.len();

        requests.retain(|_, times| {
            times.retain(|&time| now.duration_since(time) < self.window);
            !times.is_empty()
        });

        let final_len = requests.len();
        if initial_len != final_len {
            println!("Cleaned up {} rate limit entries", initial_len - final_len);
        }
    }
}

/// Input validator for API requests
pub struct InputValidator {
    max_body_size: usize,
    max_path_length: usize,
    max_header_count: usize,
    max_header_value_length: usize,
}

impl Default for InputValidator {
    fn default() -> Self {
        InputValidator {
            max_body_size: 10 * 1024 * 1024, // 10MB
            max_path_length: 2048,
            max_header_count: 50,
            max_header_value_length: 4096,
        }
    }
}

impl InputValidator {
    /// Validate entire request
    pub fn validate_request(&self, req: &Request) -> Result<(), MtpError> {
        self.validate_method(&req.method)?;
        self.validate_path(&req.path)?;
        self.validate_headers(&req.headers)?;
        self.validate_body(&req.body)?;
        self.validate_client_ip(&req.client_ip)?;

        Ok(())
    }

    fn validate_method(&self, method: &str) -> Result<(), MtpError> {
        match method {
            "GET" | "POST" | "PUT" | "DELETE" | "PATCH" | "HEAD" | "OPTIONS" => Ok(()),
            _ => Err(MtpError {
                error: "ValidationError".to_string(),
                message: Some(format!("Invalid HTTP method: {}", method)),
                gasLimit: None,
                gasUsed: None,
            }),
        }
    }

    fn validate_path(&self, path: &str) -> Result<(), MtpError> {
        if path.len() > self.max_path_length {
            return Err(MtpError::ValidationError {
                error: "ValidationError".to_string(),
                message: "Path too long".to_string(),
            });
        }

        // Check for directory traversal attempts
        if path.contains("..") || path.contains("\\") {
            return Err(MtpError::ValidationError {
                error: "ValidationError".to_string(),
                message: "Path contains invalid characters".to_string(),
            });
        }

        // Check for null bytes
        if path.contains('\0') {
            return Err(MtpError::ValidationError {
                error: "ValidationError".to_string(),
                message: "Path contains null bytes".to_string(),
            });
        }

        // Basic path validation
        if !path.starts_with('/') {
            return Err(MtpError::ValidationError {
                error: "ValidationError".to_string(),
                message: "Path must start with /".to_string(),
            });
        }

        Ok(())
    }

    fn validate_headers(&self, headers: &HashMap<String, String>) -> Result<(), MtpError> {
        if headers.len() > self.max_header_count {
            return Err(MtpError::ValidationError {
                error: "ValidationError".to_string(),
                message: "Too many headers".to_string(),
            });
        }

        for (key, value) in headers {
            // Validate header name
            if key.is_empty() || key.len() > 256 {
                return Err(MtpError::ValidationError {
                    error: "ValidationError".to_string(),
                    message: format!("Invalid header name: {}", key),
                });
            }

            // Check for control characters in header name
            if key.chars().any(|c| c.is_control()) {
                return Err(MtpError::ValidationError {
                    error: "ValidationError".to_string(),
                    message: format!("Header name contains control characters: {}", key),
                });
            }

            // Validate header value
            if value.len() > self.max_header_value_length {
                return Err(MtpError::ValidationError {
                    error: "ValidationError".to_string(),
                    message: format!("Header value too long: {}", key),
                });
            }

            // Check for CR/LF injection
            if value.contains('\r') || value.contains('\n') {
                return Err(MtpError::ValidationError {
                    error: "ValidationError".to_string(),
                    message: format!("Header value contains CRLF: {}", key),
                });
            }
        }

        Ok(())
    }

    fn validate_body(&self, body: &str) -> Result<(), MtpError> {
        if body.len() > self.max_body_size {
            return Err(MtpError::ValidationError {
                error: "ValidationError".to_string(),
                message: "Request body too large".to_string(),
            });
        }

        // Check for null bytes
        if body.contains('\0') {
            return Err(MtpError::ValidationError {
                error: "ValidationError".to_string(),
                message: "Body contains null bytes".to_string(),
            });
        }

        // Check for control characters that shouldn't be in JSON
        for (i, c) in body.chars().enumerate() {
            if c.is_control() && c != '\n' && c != '\r' && c != '\t' {
                return Err(MtpError::ValidationError {
                    error: "ValidationError".to_string(),
                    message: format!("Body contains invalid control character at position {}", i),
                });
            }
        }

        // Basic JSON structure validation
        self.validate_json_structure(body)?;

        Ok(())
    }

    fn validate_json_structure(&self, body: &str) -> Result<(), MtpError> {
        if body.is_empty() {
            return Ok(());
        }

        // Basic bracket/brace matching
        let mut brace_count = 0;
        let mut bracket_count = 0;
        let mut in_string = false;
        let mut escaped = false;

        for c in body.chars() {
            if escaped {
                escaped = false;
                continue;
            }

            match c {
                '"' => {
                    if !escaped {
                        in_string = !in_string;
                    }
                }
                '\\' => {
                    if in_string {
                        escaped = true;
                    }
                }
                '{' => {
                    if !in_string {
                        brace_count += 1;
                    }
                }
                '}' => {
                    if !in_string {
                        brace_count -= 1;
                        if brace_count < 0 {
                            return Err(MtpError::ValidationError {
                                error: "ValidationError".to_string(),
                                message: "Mismatched braces in JSON".to_string(),
                            });
                        }
                    }
                }
                '[' => {
                    if !in_string {
                        bracket_count += 1;
                    }
                }
                ']' => {
                    if !in_string {
                        bracket_count -= 1;
                        if bracket_count < 0 {
                            return Err(MtpError::ValidationError {
                                error: "ValidationError".to_string(),
                                message: "Mismatched brackets in JSON".to_string(),
                            });
                        }
                    }
                }
                _ => {}
            }
        }

        if brace_count != 0 {
            return Err(MtpError::ValidationError {
                error: "ValidationError".to_string(),
                message: "Unclosed braces in JSON".to_string(),
            });
        }

        if bracket_count != 0 {
            return Err(MtpError::ValidationError {
                error: "ValidationError".to_string(),
                message: "Unclosed brackets in JSON".to_string(),
            });
        }

        if in_string {
            return Err(MtpError::ValidationError {
                error: "ValidationError".to_string(),
                message: "Unclosed string in JSON".to_string(),
            });
        }

        Ok(())
    }

    fn validate_client_ip(&self, ip: &str) -> Result<(), MtpError> {
        // Basic IP validation
        if ip.is_empty() || ip.len() > 45 {
            // IPv6 max length
            return Err(MtpError::ValidationError {
                error: "ValidationError".to_string(),
                message: "Invalid client IP".to_string(),
            });
        }

        // Check for basic IP format (could be more sophisticated)
        if !ip.contains('.') && !ip.contains(':') {
            return Err(MtpError::ValidationError {
                error: "ValidationError".to_string(),
                message: "Invalid IP format".to_string(),
            });
        }

        Ok(())
    }
}

/// Enhanced API handler with validation and rate limiting
pub fn handle_request(
    req: Request,
    rate_limiter: &RateLimiter,
    validator: &InputValidator,
) -> Result<Response, MtpError> {
    // Validate input
    validator.validate_request(&req)?;

    // Check rate limit
    rate_limiter.check_rate_limit(&req.client_ip)?;

    // Clean up rate limiter periodically (could be done in background)
    rate_limiter.cleanup();

    // Process request
    match (req.method.as_str(), req.path.as_str()) {
        ("GET", "/health") => Ok(Response {
            status: 200,
            body: r#"{"status": "ok"}"#.to_string(),
            headers: [("Content-Type".to_string(), "application/json".to_string())].into(),
        }),
        ("POST", "/echo") => {
            // Additional validation for POST body
            if req.body.len() > 1024 * 1024 {
                // 1MB for echo endpoint
                return Err(MtpError::ValidationError {
                    error: "ValidationError".to_string(),
                    message: "Echo body too large".to_string(),
                });
            }

            Ok(Response {
                status: 200,
                body: req.body,
                headers: [("Content-Type".to_string(), "application/json".to_string())].into(),
            })
        }
        _ => Ok(Response {
            status: 404,
            body: r#"{"error": "Not found"}"#.to_string(),
            headers: [("Content-Type".to_string(), "application/json".to_string())].into(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_rate_limiting() {
        let limiter = RateLimiter::new(2, 60); // 2 requests per minute

        let client = "127.0.0.1";
        assert!(limiter.check_rate_limit(client).is_ok());
        assert!(limiter.check_rate_limit(client).is_ok());
        assert!(limiter.check_rate_limit(client).is_err()); // Should be rate limited
    }

    #[test]
    fn test_input_validation() {
        let validator = InputValidator::default();

        let valid_req = Request {
            method: "GET".to_string(),
            path: "/test".to_string(),
            body: "test".to_string(),
            headers: HashMap::new(),
            client_ip: "127.0.0.1".to_string(),
            timestamp: Instant::now(),
        };

        assert!(validator.validate_request(&valid_req).is_ok());

        // Test invalid path
        let invalid_req = Request {
            path: "../../../etc/passwd".to_string(),
            ..valid_req
        };

        assert!(validator.validate_request(&invalid_req).is_err());
    }
}
