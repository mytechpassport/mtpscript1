use std::collections::HashMap;

use crate::api::router::Router;
use crate::errors::runtime::RuntimeError;
use crate::runtime::clone::clone_interpreter;
use crate::runtime::effects::inject_effects;
use crate::runtime::seed::{compute_seed, SeedRequest};
use crate::runtime::Interpreter;
use crate::runtime::Value;

#[derive(Debug)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

#[derive(Debug)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

pub struct RequestHandler {
    pub snapshot: Vec<u8>,
    pub gas_limit: u64,
    pub router: Router,
}

impl RequestHandler {
    pub fn new(snapshot: Vec<u8>, gas_limit: u64, router: Router) -> Self {
        Self {
            snapshot,
            gas_limit,
            router,
        }
    }

    pub fn handle_request(&self, req: HttpRequest) -> Result<HttpResponse, RuntimeError> {
        // 1. Validate input
        self.validate_request(&req)?;

        // 2. Check rate limit (placeholder - would check against rate limiter)
        self.check_rate_limit(&req)?;

        // 3. Parse request (already parsed)

        // 4. Extract metadata
        let request_id = req
            .headers
            .get("x-request-id")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        let account_id = req
            .headers
            .get("x-account-id")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        let function_version = req
            .headers
            .get("x-function-version")
            .cloned()
            .unwrap_or_else(|| "1".to_string());

        // 3. Compute seed
        let snapshot_hash = sha256_bytes(&self.snapshot);
        let seed_req = SeedRequest::new(
            request_id.clone(),
            account_id,
            function_version,
            snapshot_hash,
            self.gas_limit,
        );
        let seed = compute_seed(&seed_req)?;

        // 4. Clone interpreter
        let mut interp = clone_interpreter(&self.snapshot)?;

        // 5. Set gas limit
        interp.set_gas_limit(self.gas_limit);

        // 7. Inject effects
        inject_effects(&mut interp, &seed)?;

        // 8. Route and execute handler
        let result = self.execute_handler(&mut interp, &req)?;

        // 7. Serialize response to canonical JSON
        let response_body = result.to_json_string()?;

        // 8. Hash response
        let response_hash = sha256(&response_body);

        // 9. Log audit
        self.log_audit(&request_id, &response_hash, self.gas_limit);

        // 10. Return HTTP response
        Ok(HttpResponse {
            status: 200,
            headers: HashMap::from([("Content-Type".to_string(), "application/json".to_string())]),
            body: response_body.into_bytes(),
        })
    }

    fn execute_handler(
        &self,
        interp: &mut Interpreter,
        req: &HttpRequest,
    ) -> Result<Value, RuntimeError> {
        // Match route using the router
        let route_match = self
            .router
            .match_route(&req.method, &req.path)
            .ok_or_else(|| {
                RuntimeError::ValueError(format!("No route found for {} {}", req.method, req.path))
            })?;

        // Prepare arguments for the API handler function
        let mut args = Vec::new();

        // Add request object
        let request_obj = HashMap::from([
            ("method".to_string(), Value::String(req.method.clone())),
            ("path".to_string(), Value::String(req.path.clone())),
            (
                "body".to_string(),
                Value::String(String::from_utf8_lossy(&req.body).to_string()),
            ),
        ]);
        args.push(Value::Object(request_obj));

        // Add path parameters
        let params_obj: HashMap<String, Value> = route_match
            .params
            .iter()
            .map(|(k, v)| (k.clone(), Value::String(v.clone())))
            .collect();
        args.push(Value::Object(params_obj));

        // Call the API handler function
        interp.call_global_function(&route_match.api.handler, args)
    }

    fn log_audit(&self, request_id: &str, response_hash: &[u8], gas_limit: u64) {
        // Log to audit system as JSON line to stderr (forwarded to CloudWatch in Lambda)
        let audit_entry = serde_json::json!({
            "request_id": request_id,
            "gas_limit": gas_limit,
            "response_hash": hex::encode(response_hash),
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        eprintln!("{}", audit_entry);
    }

    fn validate_request(&self, req: &HttpRequest) -> Result<(), RuntimeError> {
        // Validate method
        let valid_methods = ["GET", "POST", "PUT", "DELETE", "PATCH"];
        if !valid_methods.contains(&req.method.as_str()) {
            return Err(RuntimeError::ValueError(format!(
                "Invalid HTTP method: {}",
                req.method
            )));
        }

        // Validate path length
        if req.path.is_empty() || req.path.len() > 2048 {
            return Err(RuntimeError::ValueError("Invalid path length".to_string()));
        }

        // Validate path characters (basic)
        if !req
            .path
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "/-_.".contains(c))
        {
            return Err(RuntimeError::ValueError(
                "Invalid path characters".to_string(),
            ));
        }

        // Validate headers
        for (name, value) in &req.headers {
            if name.len() > 128 || value.len() > 4096 {
                return Err(RuntimeError::ValueError("Header too long".to_string()));
            }
            // Check for suspicious headers
            if name.to_lowercase().contains("script") || value.contains("<script") {
                return Err(RuntimeError::ValueError(
                    "Suspicious header content".to_string(),
                ));
            }
        }

        // Validate body size
        if req.body.len() > 10_000_000 {
            // 10MB limit
            return Err(RuntimeError::ValueError(
                "Request body too large".to_string(),
            ));
        }

        Ok(())
    }

    fn check_rate_limit(&self, _req: &HttpRequest) -> Result<(), RuntimeError> {
        // Placeholder: would check against a rate limiter
        // For now, always allow
        Ok(())
    }
}

// SHA-256 function for bytes
fn sha256_bytes(data: &[u8]) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}

// Placeholder SHA-256 function for strings
fn sha256(data: &str) -> [u8; 32] {
    sha256_bytes(data.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::router::{ApiDeclaration, HttpMethod, Router};

    #[test]
    fn test_request_handler_creation() {
        let snapshot = vec![1, 2, 3, 4]; // Mock snapshot
        let gas_limit = 10_000_000;
        let router = Router::new();

        let handler = RequestHandler::new(snapshot.clone(), gas_limit, router);
        assert_eq!(handler.snapshot, snapshot);
        assert_eq!(handler.gas_limit, gas_limit);
    }

    #[test]
    fn test_http_request_creation() {
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());

        let req = HttpRequest {
            method: "POST".to_string(),
            path: "/users".to_string(),
            headers,
            body: b"{\"name\":\"Alice\"}".to_vec(),
        };

        assert_eq!(req.method, "POST");
        assert_eq!(req.path, "/users");
        assert_eq!(req.body, b"{\"name\":\"Alice\"}");
    }

    #[test]
    fn test_http_response_creation() {
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());

        let resp = HttpResponse {
            status: 200,
            headers,
            body: b"{\"success\":true}".to_vec(),
        };

        assert_eq!(resp.status, 200);
        assert_eq!(resp.body, b"{\"success\":true}");
    }

    #[test]
    fn test_sha256_function() {
        let hash = sha256("test");
        assert_eq!(hash.len(), 32);

        // Same input should produce same hash
        let hash2 = sha256("test");
        assert_eq!(hash, hash2);

        // Different input should produce different hash
        let hash3 = sha256("different");
        assert_ne!(hash, hash3);
    }
}
