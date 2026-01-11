use crate::errors::MtpError;
use std::collections::{HashMap, HashSet};

/// Taint level for data
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaintLevel {
    /// Data is safe and trusted
    Clean = 0,
    /// Data may be tainted (user input, etc.)
    Tainted = 1,
    /// Data is definitely malicious
    Poisoned = 2,
}

/// A taint source identifier
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct TaintSource {
    pub id: String,
    pub description: String,
}

/// Tainted value with source tracking
#[derive(Debug, Clone)]
pub struct TaintedValue {
    pub level: TaintLevel,
    pub sources: HashSet<TaintSource>,
}

/// Static taint analysis for AST
pub struct StaticTaintAnalyzer {
    taint_map: HashMap<String, TaintedValue>,
    sources: HashSet<TaintSource>,
}

impl StaticTaintAnalyzer {
    /// Create a new static taint analyzer
    pub fn new() -> Self {
        StaticTaintAnalyzer {
            taint_map: HashMap::new(),
            sources: HashSet::new(),
        }
    }

    /// Add a taint source (e.g., user input)
    pub fn add_source(&mut self, source: TaintSource) {
        self.sources.insert(source);
    }

    /// Mark a variable as tainted
    pub fn taint_variable(&mut self, var: &str, level: TaintLevel, sources: HashSet<TaintSource>) {
        self.taint_map
            .insert(var.to_string(), TaintedValue { level, sources });
    }

    /// Propagate taint through assignment: target = expr
    pub fn propagate_taint(&mut self, target: &str, sources: &[&str]) {
        let mut max_level = TaintLevel::Clean;
        let mut combined_sources = HashSet::new();

        for &src in sources {
            if let Some(tainted) = self.taint_map.get(src) {
                if tainted.level > max_level {
                    max_level = tainted.level;
                }
                combined_sources.extend(tainted.sources.clone());
            }
        }

        if max_level > TaintLevel::Clean || !combined_sources.is_empty() {
            self.taint_map.insert(
                target.to_string(),
                TaintedValue {
                    level: max_level,
                    sources: combined_sources,
                },
            );
        }
    }

    /// Check if a variable is tainted
    pub fn is_tainted(&self, var: &str) -> Option<&TaintedValue> {
        self.taint_map.get(var)
    }

    /// Get taint report
    pub fn get_report(&self) -> String {
        let mut report = String::from("# Static Taint Analysis Report\n\n");

        let mut tainted_vars: Vec<_> = self
            .taint_map
            .iter()
            .filter(|(_, v)| v.level > TaintLevel::Clean)
            .collect();

        tainted_vars.sort_by_key(|(k, _)| *k);

        report.push_str(&format!("Tainted variables: {}\n\n", tainted_vars.len()));

        for (var, tainted) in tainted_vars {
            report.push_str(&format!("- {}: {:?}\n", var, tainted.level));
            for source in &tainted.sources {
                report.push_str(&format!(
                    "  Source: {} ({})\n",
                    source.id, source.description
                ));
            }
            report.push_str("\n");
        }

        report
    }
}

/// Dynamic taint analysis for runtime
pub struct DynamicTaintTracker {
    taint_map: HashMap<String, TaintedValue>,
    call_stack: Vec<String>,
}

impl DynamicTaintTracker {
    /// Create a new dynamic taint tracker
    pub fn new() -> Self {
        DynamicTaintTracker {
            taint_map: HashMap::new(),
            call_stack: Vec::new(),
        }
    }

    /// Mark data as tainted at runtime
    pub fn mark_tainted(&mut self, key: &str, level: TaintLevel, source: TaintSource) {
        let mut sources = HashSet::new();
        sources.insert(source);

        self.taint_map
            .insert(key.to_string(), TaintedValue { level, sources });
    }

    /// Check if data is tainted
    pub fn check_taint(&self, key: &str) -> Option<&TaintedValue> {
        self.taint_map.get(key)
    }

    /// Propagate taint through operations
    pub fn propagate(&mut self, result_key: &str, input_keys: &[&str]) {
        let mut max_level = TaintLevel::Clean;
        let mut combined_sources = HashSet::new();

        for &input_key in input_keys {
            if let Some(tainted) = self.taint_map.get(input_key) {
                if tainted.level > max_level {
                    max_level = tainted.level;
                }
                combined_sources.extend(tainted.sources.clone());
            }
        }

        if max_level > TaintLevel::Clean || !combined_sources.is_empty() {
            self.taint_map.insert(
                result_key.to_string(),
                TaintedValue {
                    level: max_level,
                    sources: combined_sources,
                },
            );
        }
    }

    /// Enter a function call
    pub fn enter_function(&mut self, func_name: &str) {
        self.call_stack.push(func_name.to_string());
    }

    /// Exit a function call
    pub fn exit_function(&mut self) {
        self.call_stack.pop();
    }

    /// Check for dangerous operations on tainted data
    pub fn check_dangerous_operation(
        &self,
        operation: &str,
        data_key: &str,
    ) -> Result<(), MtpError> {
        if let Some(tainted) = self.check_taint(data_key) {
            if tainted.level >= TaintLevel::Poisoned {
                return Err(MtpError::SecurityError {
                    error: "TaintedDataUsage".to_string(),
                    message: format!(
                        "Attempted {} on poisoned data from sources: {:?}",
                        operation, tainted.sources
                    ),
                });
            }

            // Log warning for tainted data usage
            eprintln!(
                "Warning: {} operation on tainted data (level: {:?}) from sources: {:?}",
                operation, tainted.level, tainted.sources
            );
        }
        Ok(())
    }

    /// Get current call stack
    pub fn get_call_stack(&self) -> &[String] {
        &self.call_stack
    }
}

/// Global dynamic taint tracker (simplified for single-threaded use)
static mut DYNAMIC_TRACKER: Option<DynamicTaintTracker> = None;

/// Initialize global dynamic taint tracker
pub fn init_dynamic_taint_tracking() {
    unsafe {
        DYNAMIC_TRACKER = Some(DynamicTaintTracker::new());
    }
}

/// Get global dynamic taint tracker
pub fn get_dynamic_tracker() -> Option<&'static mut DynamicTaintTracker> {
    unsafe { DYNAMIC_TRACKER.as_mut() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_taint_analysis() {
        let mut analyzer = StaticTaintAnalyzer::new();

        let user_input = TaintSource {
            id: "user_input".to_string(),
            description: "HTTP request body".to_string(),
        };
        analyzer.add_source(user_input.clone());

        analyzer.taint_variable("input", TaintLevel::Tainted, {
            let mut sources = HashSet::new();
            sources.insert(user_input);
            sources
        });

        analyzer.propagate_taint("processed", &["input"]);

        assert!(analyzer.is_tainted("processed").is_some());
        let tainted = analyzer.is_tainted("processed").unwrap();
        assert_eq!(tainted.level, TaintLevel::Tainted);
    }

    #[test]
    fn test_dynamic_taint_tracking() {
        let mut tracker = DynamicTaintTracker::new();

        let source = TaintSource {
            id: "network".to_string(),
            description: "Network input".to_string(),
        };

        tracker.mark_tainted("data", TaintLevel::Tainted, source);
        tracker.propagate("result", &["data"]);

        assert!(tracker.check_taint("result").is_some());
        assert_eq!(
            tracker.check_taint("result").unwrap().level,
            TaintLevel::Tainted
        );
    }

    #[test]
    fn test_dangerous_operation_detection() {
        let mut tracker = DynamicTaintTracker::new();

        let source = TaintSource {
            id: "malicious".to_string(),
            description: "Potentially malicious input".to_string(),
        };

        tracker.mark_tainted("evil_data", TaintLevel::Poisoned, source);

        assert!(tracker
            .check_dangerous_operation("eval", "evil_data")
            .is_err());
    }
}
