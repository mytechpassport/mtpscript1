use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Effect call record
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EffectCall {
    /// Name of the effect called
    pub effect_name: String,
    /// Number of times this effect was called
    pub call_count: u64,
}

/// Request trace for detailed execution logging
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequestTrace {
    /// Unique request identifier
    pub request_id: String,
    /// Gas limit for this request
    #[serde(rename = "gasLimit")]
    pub gas_limit: u64,
    /// Gas actually used
    #[serde(rename = "gasUsed")]
    pub gas_used: u64,
    /// SHA-256 hash of the canonical JSON response body
    #[serde(rename = "responseHash")]
    pub response_hash: String,
    /// Duration of request processing in milliseconds
    pub duration_ms: i64,
    /// Effect calls made during execution
    pub effect_calls: Vec<EffectCall>,
    /// ISO 8601 timestamp when tracing started
    pub timestamp: DateTime<Utc>,
}

/// Tracer for recording request execution details
pub struct RequestTracer {
    pub request_id: String,
    pub gas_limit: u64,
    start_time: DateTime<Utc>,
    pub effect_calls: HashMap<String, u64>,
}

impl RequestTracer {
    /// Create a new tracer for a request
    pub fn new(request_id: String, gas_limit: u64) -> Self {
        Self {
            request_id,
            gas_limit,
            start_time: Utc::now(),
            effect_calls: HashMap::new(),
        }
    }

    /// Record an effect call
    pub fn record_effect_call(&mut self, effect_name: &str) {
        *self
            .effect_calls
            .entry(effect_name.to_string())
            .or_insert(0) += 1;
    }

    /// Complete the trace with final results
    pub fn complete(self, gas_used: u64, response_hash: String) -> RequestTrace {
        let end_time = Utc::now();
        let duration = end_time.signed_duration_since(self.start_time);

        let effect_calls = self
            .effect_calls
            .into_iter()
            .map(|(effect_name, call_count)| EffectCall {
                effect_name,
                call_count,
            })
            .collect();

        RequestTrace {
            request_id: self.request_id,
            gas_limit: self.gas_limit,
            gas_used,
            response_hash,
            duration_ms: duration.num_milliseconds(),
            effect_calls,
            timestamp: self.start_time,
        }
    }
}

/// Execute a request with tracing enabled
pub fn execute_with_tracing<F>(
    request_id: String,
    gas_limit: u64,
    execution_fn: F,
) -> Result<RequestTrace, Box<dyn std::error::Error>>
where
    F: FnOnce(&mut RequestTracer) -> Result<(u64, String), Box<dyn std::error::Error>>,
{
    let mut tracer = RequestTracer::new(request_id, gas_limit);
    let (gas_used, response_hash) = execution_fn(&mut tracer)?;
    Ok(tracer.complete(gas_used, response_hash))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_request_tracer_creation() {
        let tracer = RequestTracer::new("test-123".to_string(), 1000000);
        assert_eq!(tracer.request_id, "test-123");
        assert_eq!(tracer.gas_limit, 1000000);
        assert!(tracer.effect_calls.is_empty());
    }

    #[test]
    fn test_effect_call_recording() {
        let mut tracer = RequestTracer::new("test-123".to_string(), 1000000);

        tracer.record_effect_call("DbRead");
        tracer.record_effect_call("DbRead");
        tracer.record_effect_call("HttpOut");

        let trace = tracer.complete(50000, "hash123".to_string());

        assert_eq!(trace.effect_calls.len(), 2);
        let db_read = trace
            .effect_calls
            .iter()
            .find(|c| c.effect_name == "DbRead")
            .unwrap();
        let http_out = trace
            .effect_calls
            .iter()
            .find(|c| c.effect_name == "HttpOut")
            .unwrap();

        assert_eq!(db_read.call_count, 2);
        assert_eq!(http_out.call_count, 1);
    }

    #[test]
    fn test_trace_completion() {
        let tracer = RequestTracer::new("test-456".to_string(), 2000000);

        // Simulate some delay
        thread::sleep(Duration::from_millis(10));

        let trace = tracer.complete(75000, "response_hash".to_string());

        assert_eq!(trace.request_id, "test-456");
        assert_eq!(trace.gas_limit, 2000000);
        assert_eq!(trace.gas_used, 75000);
        assert_eq!(trace.response_hash, "response_hash");
        assert!(trace.duration_ms >= 10); // At least 10ms due to sleep
        assert!(trace.timestamp <= Utc::now());
    }

    #[test]
    fn test_execute_with_tracing() {
        let result = execute_with_tracing("trace-test".to_string(), 1000000, |tracer| {
            tracer.record_effect_call("DbRead");
            tracer.record_effect_call("Log");
            Ok((25000, "test_hash".to_string()))
        });

        assert!(result.is_ok());
        let trace = result.unwrap();

        assert_eq!(trace.request_id, "trace-test");
        assert_eq!(trace.gas_limit, 1000000);
        assert_eq!(trace.gas_used, 25000);
        assert_eq!(trace.response_hash, "test_hash");
        assert_eq!(trace.effect_calls.len(), 2);
    }

    #[test]
    fn test_trace_serialization() {
        let mut tracer = RequestTracer::new("serialize-test".to_string(), 500000);
        tracer.record_effect_call("Async");

        let trace = tracer.complete(10000, "serial_hash".to_string());

        let json = serde_json::to_string(&trace).unwrap();
        assert!(json.contains(r#""request_id":"serialize-test""#));
        assert!(json.contains(r#""gasLimit":500000"#));
        assert!(json.contains(r#""gasUsed":10000"#));
        assert!(json.contains(r#""responseHash":"serial_hash""#));
        assert!(json.contains(r#""duration_ms""#));
        assert!(json.contains(r#""effect_calls""#));
        assert!(json.contains(r#""Async""#));
    }
}
