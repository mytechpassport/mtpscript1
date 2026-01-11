#[cfg(test)]
mod trace_tests {
    use mtpscript_core::audit::trace::{execute_with_tracing, RequestTrace, RequestTracer};

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
        std::thread::sleep(std::time::Duration::from_millis(10));

        let trace = tracer.complete(75000, "response_hash".to_string());

        assert_eq!(trace.request_id, "test-456");
        assert_eq!(trace.gas_limit, 2000000);
        assert_eq!(trace.gas_used, 75000);
        assert_eq!(trace.response_hash, "response_hash");
        assert!(trace.duration_ms >= 10); // At least 10ms due to sleep
        assert!(trace.timestamp <= chrono::Utc::now());
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

    #[test]
    fn test_trace_fields_completeness() {
        let mut tracer = RequestTracer::new("complete-test".to_string(), 1000000);
        tracer.record_effect_call("DbRead");
        tracer.record_effect_call("DbWrite");
        tracer.record_effect_call("HttpOut");

        let trace = tracer.complete(50000, "resp_hash".to_string());

        // Verify all required fields are present and correct
        assert_eq!(trace.request_id, "complete-test");
        assert_eq!(trace.gas_limit, 1000000);
        assert_eq!(trace.gas_used, 50000);
        assert_eq!(trace.response_hash, "resp_hash");
        assert!(trace.duration_ms >= 0);
        assert_eq!(trace.effect_calls.len(), 3);

        // Verify effect counts
        let db_read_count = trace
            .effect_calls
            .iter()
            .find(|c| c.effect_name == "DbRead")
            .map(|c| c.call_count)
            .unwrap_or(0);
        assert_eq!(db_read_count, 1);
    }
}
