use std::collections::HashMap;

use crate::errors::runtime::RuntimeError;
use crate::runtime::clone::clone_interpreter;
use crate::runtime::effects::inject_effects;
use crate::runtime::seed::{compute_seed, SeedRequest};
use crate::runtime::Interpreter;

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
}

impl RequestHandler {
    pub fn new(snapshot: Vec<u8>, gas_limit: u64) -> Self {
        Self {
            snapshot,
            gas_limit,
        }
    }

    pub fn handle_request(&self, req: HttpRequest) -> Result<HttpResponse, RuntimeError> {
        // 1. Parse request (already parsed)

        // 2. Extract metadata
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
        let snapshot_hash = [0u8; 32]; // Placeholder - would compute from snapshot
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

        // 5. Inject effects
        inject_effects(&mut interp, &seed)?;

        // 6. Execute handler (placeholder - would route to correct function)
        let result = self.execute_handler(&mut interp, &req)?;

        // 7. Serialize response to canonical JSON
        let response_body = result.to_json_string()?;

        // 8. Hash response
        let response_hash = sha256(&response_body);

        // 9. Log audit
        self.log_audit(&request_id, &response_hash);

        // 10. Return HTTP response
        Ok(HttpResponse {
            status: 200,
            headers: HashMap::from([("Content-Type".to_string(), "application/json".to_string())]),
            body: response_body.into_bytes(),
        })
    }

    fn execute_handler(
        &self,
        _interp: &mut Interpreter,
        _req: &HttpRequest,
    ) -> Result<crate::runtime::Value, RuntimeError> {
        // Placeholder: would execute the matched API handler function
        // For now, return a mock response
        Ok(crate::runtime::Value::Object(HashMap::from([(
            "status".to_string(),
            crate::runtime::Value::String("success".to_string()),
        )])))
    }

    fn log_audit(&self, request_id: &str, response_hash: &[u8]) {
        // Placeholder: would log to audit system
        println!(
            "AUDIT: request_id={}, response_hash={}",
            request_id,
            hex::encode(response_hash)
        );
    }
}

// Placeholder SHA-256 function
fn sha256(data: &str) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    let result = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}
