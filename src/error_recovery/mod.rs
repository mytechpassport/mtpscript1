use crate::errors::MtpError;
use std::collections::HashMap;
use std::sync::Arc;

/// Error recovery strategy
#[derive(Debug, Clone, PartialEq)]
pub enum RecoveryStrategy {
    /// Skip the failing operation and continue
    Skip,
    /// Use a default value instead
    UseDefault,
    /// Retry the operation with backoff
    Retry { max_attempts: u32, backoff_ms: u64 },
    /// Log the error and continue
    LogAndContinue,
    /// Fail fast on critical errors
    FailFast,
    /// Attempt to repair the state
    Repair,
}

/// Recovery configuration for different error types
pub struct ErrorRecoveryConfig {
    pub strategies: HashMap<String, RecoveryStrategy>,
    pub default_strategy: RecoveryStrategy,
}

impl Default for ErrorRecoveryConfig {
    fn default() -> Self {
        let mut strategies = HashMap::new();

        // Configure recovery strategies for different error types
        strategies.insert("ParseError".to_string(), RecoveryStrategy::Skip);
        strategies.insert("TypeError".to_string(), RecoveryStrategy::FailFast);
        strategies.insert("GasExhausted".to_string(), RecoveryStrategy::FailFast);
        strategies.insert(
            "RuntimeError".to_string(),
            RecoveryStrategy::Retry {
                max_attempts: 3,
                backoff_ms: 100,
            },
        );
        strategies.insert(
            "ValidationError".to_string(),
            RecoveryStrategy::LogAndContinue,
        );
        strategies.insert(
            "IoError".to_string(),
            RecoveryStrategy::Retry {
                max_attempts: 5,
                backoff_ms: 500,
            },
        );
        strategies.insert("SecurityError".to_string(), RecoveryStrategy::FailFast);

        ErrorRecoveryConfig {
            strategies,
            default_strategy: RecoveryStrategy::LogAndContinue,
        }
    }
}

/// Error recovery engine
pub struct ErrorRecoveryEngine {
    config: ErrorRecoveryConfig,
    recovery_stats: HashMap<String, u64>,
}

impl ErrorRecoveryEngine {
    pub fn new(config: ErrorRecoveryConfig) -> Self {
        ErrorRecoveryEngine {
            config,
            recovery_stats: HashMap::new(),
        }
    }

    /// Handle an error with appropriate recovery strategy
    pub fn handle_error<F, T>(
        &mut self,
        error: &MtpError,
        recovery_action: F,
    ) -> Result<T, MtpError>
    where
        F: Fn() -> Result<T, MtpError>,
    {
        let error_type = self.get_error_type(error);
        let strategy = self
            .config
            .strategies
            .get(&error_type)
            .unwrap_or(&self.config.default_strategy)
            .clone();

        // Update recovery statistics
        *self.recovery_stats.entry(error_type.clone()).or_insert(0) += 1;

        match strategy {
            RecoveryStrategy::Skip => {
                eprintln!("Skipping error: {}", error);
                Err(error.clone()) // Still return the error, but logged
            }
            RecoveryStrategy::UseDefault => {
                eprintln!("Using default value for error: {}", error);
                // This would need to be handled by the caller
                Err(error.clone())
            }
            RecoveryStrategy::Retry {
                max_attempts,
                backoff_ms,
            } => self.handle_retry(error, recovery_action, max_attempts, backoff_ms),
            RecoveryStrategy::LogAndContinue => {
                eprintln!("Logging error and continuing: {}", error);
                Err(error.clone())
            }
            RecoveryStrategy::FailFast => {
                eprintln!("Failing fast on critical error: {}", error);
                Err(error.clone())
            }
            RecoveryStrategy::Repair => self.handle_repair(error, recovery_action),
        }
    }

    /// Handle retry with backoff
    fn handle_retry<F, T>(
        &self,
        error: &MtpError,
        recovery_action: F,
        max_attempts: u32,
        backoff_ms: u64,
    ) -> Result<T, MtpError>
    where
        F: Fn() -> Result<T, MtpError>,
    {
        for attempt in 1..=max_attempts {
            eprintln!("Retry attempt {} for error: {}", attempt, error);

            match recovery_action() {
                Ok(result) => {
                    eprintln!("Recovery successful on attempt {}", attempt);
                    return Ok(result);
                }
                Err(e) => {
                    if attempt == max_attempts {
                        eprintln!("All retry attempts failed");
                        return Err(e);
                    }

                    std::thread::sleep(std::time::Duration::from_millis(
                        backoff_ms * attempt as u64,
                    ));
                }
            }
        }

        Err(error.clone())
    }

    /// Handle repair strategy
    fn handle_repair<F, T>(&self, error: &MtpError, recovery_action: F) -> Result<T, MtpError>
    where
        F: Fn() -> Result<T, MtpError>,
    {
        eprintln!("Attempting to repair state for error: {}", error);

        // For now, just try the action once
        // In a real implementation, this would attempt to fix the state
        match recovery_action() {
            Ok(result) => {
                eprintln!("Repair successful");
                Ok(result)
            }
            Err(e) => {
                eprintln!("Repair failed");
                Err(e)
            }
        }
    }

    /// Extract error type from MtpError
    fn get_error_type(&self, error: &MtpError) -> String {
        match error {
            MtpError::ParseError(_) => "ParseError".to_string(),
            MtpError::TypeError(_) => "TypeError".to_string(),
            MtpError::RuntimeError { .. } => "RuntimeError".to_string(),
            MtpError::GasExhausted { .. } => "GasExhausted".to_string(),
            MtpError::IoError(_) => "IoError".to_string(),
            MtpError::ValidationError { .. } => "ValidationError".to_string(),
            MtpError::SecurityError { .. } => "SecurityError".to_string(),
        }
    }

    /// Get recovery statistics
    pub fn get_recovery_stats(&self) -> &HashMap<String, u64> {
        &self.recovery_stats
    }

    /// Generate recovery report
    pub fn generate_recovery_report(&self) -> String {
        let mut report = String::from("# Error Recovery Report\n\n");

        if self.recovery_stats.is_empty() {
            report.push_str("No errors encountered.\n");
        } else {
            report.push_str("Error recovery statistics:\n\n");

            let mut sorted_stats: Vec<_> = self.recovery_stats.iter().collect();
            sorted_stats.sort_by(|a, b| b.1.cmp(a.1)); // Sort by count descending

            for (error_type, count) in sorted_stats {
                let strategy = self
                    .config
                    .strategies
                    .get(error_type)
                    .unwrap_or(&self.config.default_strategy);

                report.push_str(&format!(
                    "- {}: {} occurrences (strategy: {:?})\n",
                    error_type, count, strategy
                ));
            }
        }

        report
    }
}

/// Circuit breaker for preventing cascade failures
pub struct CircuitBreaker {
    failure_count: u64,
    last_failure_time: Option<std::time::Instant>,
    threshold: u64,
    timeout: std::time::Duration,
    state: CircuitBreakerState,
}

#[derive(Debug, Clone, PartialEq)]
enum CircuitBreakerState {
    Closed,   // Normal operation
    Open,     // Failing, reject requests
    HalfOpen, // Testing if service recovered
}

impl CircuitBreaker {
    pub fn new(threshold: u64, timeout_secs: u64) -> Self {
        CircuitBreaker {
            failure_count: 0,
            last_failure_time: None,
            threshold,
            timeout: std::time::Duration::from_secs(timeout_secs),
            state: CircuitBreakerState::Closed,
        }
    }

    /// Check if request should be allowed
    pub fn allow_request(&mut self) -> bool {
        match self.state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                if let Some(last_failure) = self.last_failure_time {
                    if last_failure.elapsed() > self.timeout {
                        self.state = CircuitBreakerState::HalfOpen;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitBreakerState::HalfOpen => true,
        }
    }

    /// Record successful operation
    pub fn record_success(&mut self) {
        self.failure_count = 0;
        self.last_failure_time = None;
        self.state = CircuitBreakerState::Closed;
    }

    /// Record failed operation
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = Some(std::time::Instant::now());

        if self.failure_count >= self.threshold {
            self.state = CircuitBreakerState::Open;
        }
    }

    /// Get current state
    pub fn get_state(&self) -> &CircuitBreakerState {
        &self.state
    }
}

/// Transaction rollback mechanism
pub struct TransactionManager {
    operations: Vec<Box<dyn Fn() -> Result<(), MtpError> + Send + Sync>>,
    compensation_actions: Vec<Box<dyn Fn() -> Result<(), MtpError> + Send + Sync>>,
}

impl TransactionManager {
    pub fn new() -> Self {
        TransactionManager {
            operations: Vec::new(),
            compensation_actions: Vec::new(),
        }
    }

    /// Add an operation with its compensation action
    pub fn add_operation<F, C>(&mut self, operation: F, compensation: C)
    where
        F: Fn() -> Result<(), MtpError> + Send + Sync + 'static,
        C: Fn() -> Result<(), MtpError> + Send + Sync + 'static,
    {
        self.operations.push(Box::new(operation));
        self.compensation_actions.push(Box::new(compensation));
    }

    /// Execute all operations, rollback on failure
    pub fn execute_transaction(&self) -> Result<(), MtpError> {
        let mut completed_operations = 0;

        for operation in &self.operations {
            match operation() {
                Ok(()) => {
                    completed_operations += 1;
                }
                Err(e) => {
                    eprintln!("Operation failed, rolling back: {}", e);
                    self.rollback(completed_operations)?;
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    /// Rollback completed operations
    fn rollback(&self, completed_count: usize) -> Result<(), MtpError> {
        for i in (0..completed_count).rev() {
            if let Some(compensation) = self.compensation_actions.get(i) {
                if let Err(e) = compensation() {
                    eprintln!("Compensation action failed: {}", e);
                    // Continue with other compensations even if one fails
                }
            }
        }
        Ok(())
    }
}

/// Health check system
pub struct HealthChecker {
    checks: Vec<Box<dyn Fn() -> Result<(), MtpError> + Send + Sync>>,
    last_check_time: Option<std::time::Instant>,
    check_interval: std::time::Duration,
    healthy: bool,
}

impl HealthChecker {
    pub fn new(check_interval_secs: u64) -> Self {
        HealthChecker {
            checks: Vec::new(),
            last_check_time: None,
            check_interval: std::time::Duration::from_secs(check_interval_secs),
            healthy: true,
        }
    }

    /// Add a health check
    pub fn add_check<F>(&mut self, check: F)
    where
        F: Fn() -> Result<(), MtpError> + Send + Sync + 'static,
    {
        self.checks.push(Box::new(check));
    }

    /// Perform health checks
    pub fn check_health(&mut self) -> bool {
        let now = std::time::Instant::now();

        // Skip if we checked recently
        if let Some(last_check) = self.last_check_time {
            if now.duration_since(last_check) < self.check_interval {
                return self.healthy;
            }
        }

        self.last_check_time = Some(now);
        self.healthy = true;

        for check in &self.checks {
            if check().is_err() {
                self.healthy = false;
                break;
            }
        }

        self.healthy
    }

    /// Get current health status
    pub fn is_healthy(&self) -> bool {
        self.healthy
    }
}

/// Global error recovery manager
pub struct ErrorRecoveryManager {
    engine: ErrorRecoveryEngine,
    circuit_breaker: CircuitBreaker,
    health_checker: HealthChecker,
}

impl ErrorRecoveryManager {
    pub fn new() -> Self {
        ErrorRecoveryManager {
            engine: ErrorRecoveryEngine::new(ErrorRecoveryConfig::default()),
            circuit_breaker: CircuitBreaker::new(5, 60), // 5 failures, 60 second timeout
            health_checker: HealthChecker::new(30),      // Check every 30 seconds
        }
    }

    /// Execute an operation with comprehensive error recovery
    pub fn execute_with_recovery<F, T>(&mut self, operation: F) -> Result<T, MtpError>
    where
        F: Fn() -> Result<T, MtpError>,
    {
        // Check circuit breaker
        if !self.circuit_breaker.allow_request() {
            return Err(MtpError::RuntimeError {
                error: "CircuitBreakerOpen".to_string(),
                message: "Circuit breaker is open, rejecting request".to_string(),
            });
        }

        // Check health
        if !self.health_checker.check_health() {
            self.circuit_breaker.record_failure();
            return Err(MtpError::RuntimeError {
                error: "UnhealthySystem".to_string(),
                message: "System is unhealthy, rejecting request".to_string(),
            });
        }

        match operation() {
            Ok(result) => {
                self.circuit_breaker.record_success();
                Ok(result)
            }
            Err(e) => {
                self.circuit_breaker.record_failure();

                // Try recovery
                self.engine.handle_error(&e, operation)
            }
        }
    }

    /// Get recovery report
    pub fn get_recovery_report(&self) -> String {
        let mut report = String::from("# Comprehensive Error Recovery Report\n\n");

        report.push_str("## Recovery Statistics\n\n");
        report.push_str(&self.engine.generate_recovery_report());

        report.push_str("\n## Circuit Breaker Status\n\n");
        report.push_str(&format!("State: {:?}\n", self.circuit_breaker.get_state()));
        report.push_str(&format!(
            "Failure count: {}\n",
            self.circuit_breaker.failure_count
        ));

        report.push_str("\n## Health Status\n\n");
        report.push_str(&format!("Healthy: {}\n", self.health_checker.is_healthy()));

        report
    }
}

/// Initialize global error recovery
pub fn init_error_recovery() -> ErrorRecoveryManager {
    let mut manager = ErrorRecoveryManager::new();

    // Add some default health checks
    manager.health_checker.add_check(|| {
        // Check if we can allocate memory
        let _test_vec: Vec<u8> = vec![0; 1024];
        Ok(())
    });

    manager.health_checker.add_check(|| {
        // Check if crypto audit is initialized
        if crate::security::crypto_audit::get_crypto_audit().is_some() {
            Ok(())
        } else {
            Err(MtpError::RuntimeError {
                error: "HealthCheckFailed".to_string(),
                message: "Crypto audit not initialized".to_string(),
            })
        }
    });

    manager
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_recovery_engine() {
        let config = ErrorRecoveryConfig::default();
        let mut engine = ErrorRecoveryEngine::new(config);

        let error = MtpError::ParseError("Test error".to_string());
        let result: Result<i32, MtpError> = engine.handle_error(&error, || Ok(42));

        // Since strategy is Skip, it should return the error
        assert!(result.is_err());

        let stats = engine.get_recovery_stats();
        assert_eq!(stats.get("ParseError"), Some(&1));
    }

    #[test]
    fn test_circuit_breaker() {
        let mut breaker = CircuitBreaker::new(3, 1);

        // Should allow requests initially
        assert!(breaker.allow_request());

        // Record failures
        for _ in 0..3 {
            breaker.record_failure();
        }

        // Should be open now
        assert!(!breaker.allow_request());
        assert_eq!(breaker.get_state(), &CircuitBreakerState::Open);

        // Wait for timeout (simulate)
        breaker.last_failure_time =
            Some(std::time::Instant::now() - std::time::Duration::from_secs(2));

        // Should allow request again (half-open)
        assert!(breaker.allow_request());
        assert_eq!(breaker.get_state(), &CircuitBreakerState::HalfOpen);

        // Record success to close
        breaker.record_success();
        assert_eq!(breaker.get_state(), &CircuitBreakerState::Closed);
    }

    #[test]
    fn test_transaction_rollback() {
        let mut tm = TransactionManager::new();

        let mut executed = Vec::new();
        let mut rolled_back = Vec::new();

        tm.add_operation(
            || {
                executed.push(1);
                Ok(())
            },
            || {
                rolled_back.push(1);
                Ok(())
            },
        );

        tm.add_operation(
            || {
                executed.push(2);
                Err(MtpError::RuntimeError {
                    error: "TestError".to_string(),
                    message: "Failing operation".to_string(),
                })
            },
            || {
                rolled_back.push(2);
                Ok(())
            },
        );

        tm.add_operation(
            || {
                executed.push(3);
                Ok(())
            },
            || {
                rolled_back.push(3);
                Ok(())
            },
        );

        let result = tm.execute_transaction();
        assert!(result.is_err());

        assert_eq!(executed, vec![1, 2]); // First two operations ran
        assert_eq!(rolled_back, vec![2, 1]); // Rolled back in reverse order
    }
}
