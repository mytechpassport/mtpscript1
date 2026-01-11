use crate::audit::logger::{log_audit, AuditEntry};
use std::time::{Duration, Instant};

pub struct RequestTrace {
    pub request_id: String,
    pub account_id: String,
    pub function_version: String,
    pub gas_limit: u64,
    pub start_time: Instant,
    pub effects: Vec<String>,
    pub gas_used: u64,
}

impl RequestTrace {
    pub fn new(
        request_id: String,
        account_id: String,
        function_version: String,
        gas_limit: u64,
    ) -> Self {
        Self {
            request_id,
            account_id,
            function_version,
            gas_limit,
            start_time: Instant::now(),
            effects: Vec::new(),
            gas_used: 0,
        }
    }

    pub fn add_effect(&mut self, effect: String) {
        self.effects.push(effect);
    }

    pub fn set_gas_used(&mut self, gas_used: u64) {
        self.gas_used = gas_used;
    }

    pub fn finish(self, response_hash: String) {
        let duration = self.start_time.elapsed();
        let entry = AuditEntry {
            request_id: self.request_id,
            account_id: self.account_id,
            function_version: self.function_version,
            gas_limit: self.gas_limit,
            gas_used: self.gas_used,
            response_hash,
            timestamp: chrono::Utc::now().to_rfc3339(), // need chrono crate
            duration_ms: duration.as_millis() as u64,
            effects_used: self.effects,
        };
        log_audit(&entry);
    }
}
