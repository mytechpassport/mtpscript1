# FINALFIX - Remaining Issues

**Generated:** 2026-01-19
**Updated:** 2026-01-20
**Sources:** TASK.md, BUGFIX.md

This document consolidates all remaining (uncompleted) issues from TASK.md and BUGFIX.md.

---

## Summary

| Category | Count | Completed |
|----------|-------|-----------|
| **Critical (P0)** | 8 | 0 |
| **High Priority (P1)** | 10 | 0 |
| **Medium Priority** | 6 | 6 |
| **Low Priority / Future** | 12 | 7 |
| **Total** | 36 | 13 |

### Recently Completed (2026-01-20)
- #25: Canonical JSON determinism verification - 14 comprehensive tests added
- #26: CBOR encoder validation - 23 comprehensive tests added
- #27: Lambda lowering type annotations - Already fixed, verified with tests
- #28: PreloadedRuntime graceful shutdown - Added shutdown methods and 5 tests
- #29: Chrono dependency determinism - Using content-hash-derived timestamps, 5 tests added
- #31: Nested function refactoring - Already refactored to proper methods

### Previously Completed (2026-01-19)
- #19: Test execution verified
- #20: AST to IR lowering unit tests - 27 comprehensive tests added
- #21: Tail call detection tests - 13 comprehensive tests added
- #22: Effect call compilation validation - 14 tests added
- #23: Cross-platform determinism tests - 10 tests added
- #24: JSON parser fuzzing tests - 30 tests added
- #30: Audit logging thread safety - Verified with concurrency tests

---

## Critical Priority (P0) - Must Fix Before Production

### 1. MTP-199: External Security Assessment
**Source:** TASK.md
**Effort:** XL
**Description:** Conduct full external security audit and implement all findings before production deployment.

### 2. MTP-202: Centralized Logging and Monitoring
**Source:** TASK.md, BUGFIX.md
**Effort:** L
**Description:** No centralized logging and monitoring system, hindering incident response and compliance. Implement security event detection and alerting.

### 3. MTP-204: Compliance Audit
**Source:** TASK.md, BUGFIX.md
**Effort:** L
**Description:** Compliance gaps (GDPR, PCI-DSS, etc.) unaddressed. Conduct full compliance audit and remediate violations.

### 4. MTP-218: Multiple Security Layers
**Source:** TASK.md
**Effort:** L
**Description:** No defense-in-depth strategies, relying on single points of failure. Implement multiple security layers and fail-safes.

### 5. MTP-220: Adaptive Rate Limiting
**Source:** TASK.md, BUGFIX.md
**Effort:** M
**Description:** No rate limiting or throttling mechanisms across all interfaces, vulnerable to DoS attacks.

### 6. MTP-223: Comprehensive Audit Logging
**Source:** TASK.md, BUGFIX.md
**Effort:** L
**Description:** Missing audit trails for all security-relevant operations. Implement tamper-evident storage.

### 7. MTP-227: Security Requirements Traceability
**Source:** TASK.md
**Effort:** L
**Description:** Missing formal security requirements and acceptance criteria validation. Create security requirements traceability matrix.

### 8. MTP-228: Automated Security Scanning in CI/CD
**Source:** TASK.md, BUGFIX.md
**Effort:** M
**Description:** No continuous security monitoring or vulnerability scanning in CI/CD. Implement automated security scanning and alerting.

---

## High Priority (P1) - Important Before Release

### 9. MTP-200: Chaos Engineering Testing Suite
**Source:** TASK.md, BUGFIX.md
**Effort:** L
**Description:** No chaos engineering or failure injection testing, masking resilience issues under load or adversarial conditions.

### 10. MTP-201: Formal Verification
**Source:** TASK.md, BUGFIX.md
**Effort:** XL
**Description:** Missing formal verification for critical components (parser, type checker, interpreter). Apply model checking and theorem proving where feasible.

### 11. MTP-203: Continuous Profiling
**Source:** TASK.md, BUGFIX.md
**Effort:** L
**Description:** Performance benchmarking absent, allowing undetected bottlenecks and scalability issues.

### 12. MTP-224: Privacy-by-Design
**Source:** TASK.md, BUGFIX.md
**Effort:** L
**Description:** No zero-knowledge or privacy-preserving features, risking data exposure.

### 13. MTP-226: Side-Channel Attack Mitigations
**Source:** TASK.md, BUGFIX.md
**Effort:** M
**Description:** No defense against side-channel attacks (timing, power, etc.). Implement constant-time algorithms and mitigations.

### 14. MTP-229: Incident Response Plan
**Source:** TASK.md, BUGFIX.md
**Effort:** L
**Description:** Lack of incident response plan and security operations procedures. Develop and test incident response capabilities.

### 15. Comprehensive Cryptography Audit
**Source:** BUGFIX.md (line 40)
**File:** `mtpscript-core/src/security/mod.rs`
**Description:** No comprehensive cryptography audit or key management policies. Implement FIPS-compliant crypto and key rotation.

### 16. Comprehensive Threat Model
**Source:** BUGFIX.md (line 29)
**File:** `mtpscript-core/src/`
**Description:** System lacks comprehensive threat model and security audit.

### 17. Data Flow Analysis / Taint Tracking
**Source:** BUGFIX.md (line 43)
**File:** `mtpscript-core/src/`
**Description:** No data flow analysis or taint tracking, allowing information leaks through implicit channels.

### 18. Supply Chain Security
**Source:** BUGFIX.md (line 45)
**File:** `mtpscript-core/src/`
**Description:** No supply chain security measures, vulnerable to dependency injection attacks. Implement SBOM generation and dependency scanning.

---

## Medium Priority - Testing & Validation Gaps

### 19. Execute All Newly Added Test Cases
**Source:** BUGFIX.md (line 10)
**Description:** Execute all newly added test cases and confirm their expected results match the spec: test_adt_pattern_matching, test_recursive, test_gas_exhaustion, test_decimal_edge, test_map_list, test_pipeline_full, test_lambdas_closures, test_functions_effects, test_json_ops, test_hash_cbor, test_json_duplicate_keys, test_db_effects, test_http_out, test_await_syntax, test_api_methods, test_records, test_array_bounds, test_number_overflow, test_json_null.

### 20. AST to IR Lowering Unit Tests ✅ COMPLETED
**Source:** BUGFIX.md (line 17)
**File:** `mtpscript-core/src/ir/lower.rs`
**Description:** AST to IR lowering has no dedicated unit tests, risking incorrect transformations.
**Resolution:** Added 27 comprehensive unit tests covering pipelines, binary ops, functions, literals, arrays, objects, pattern matching, lambdas, and more.

### 21. Tail Call Detection Comprehensive Tests ✅ COMPLETED
**Source:** BUGFIX.md (line 18)
**File:** `mtpscript-core/src/ir/tail_call.rs`
**Description:** Tail call detection lacks comprehensive tests for complex expressions, potentially missing optimization opportunities.
**Resolution:** Added 13 comprehensive tests including non-tail-recursive cases, match expressions, nested if statements, and explicit tail call markers.

### 22. Effect Call Compilation Validation ✅ COMPLETED
**Source:** BUGFIX.md (line 19)
**File:** `mtpscript-core/src/compiler/effects.rs`
**Description:** Effect call compilation lacks input validation, allowing malicious effect arguments.
**Resolution:** Added 14 validation tests covering all effect types (DbRead, DbWrite, HttpOut, Log, Async), argument validation, type checking, and unknown effect rejection.

### 23. Cross-Platform Determinism Tests ✅ COMPLETED
**Source:** BUGFIX.md (line 20)
**File:** `mtpscript-core/src/compiler/deterministic.rs`
**Description:** Code generation lacks cross-platform determinism tests, risking non-reproducible builds.
**Resolution:** Added 10 tests verifying deterministic name generation across instances, idempotent transformations, and consistency across 100+ runs.

### 24. JSON Parser Fuzzing ✅ COMPLETED
**Source:** BUGFIX.md (line 21)
**File:** `mtpscript-core/src/json/parse.rs`
**Description:** JSON parsing lacks fuzzing for malicious inputs, missing edge cases like invalid UTF-8 or control characters.
**Resolution:** Added 30 fuzzing-style tests covering edge cases: invalid inputs, escape sequences, unicode, unterminated structures, nesting limits, and more. Documented parser limitations with TODOs.

---

## Low Priority - Code Quality & Future Improvements

### 25. Canonical JSON Determinism Verification ✅ COMPLETED
**Source:** BUGFIX.md (line 22)
**File:** `mtpscript-core/src/json/serialize.rs`
**Description:** Canonical JSON serialization lacks determinism verification across runs.
**Resolution:** Added 14 comprehensive determinism tests covering: different insertion orders, deeply nested structures, many keys (100+), unicode keys, special characters, empty structures, mixed types, hash collision tiebreaks, numeric string keys, RFC 8785 whitespace compliance, CBOR encoding boundaries, and fresh instance verification.

### 26. CBOR Encoder Validation ✅ COMPLETED
**Source:** BUGFIX.md (line 23)
**File:** `mtpscript-core/src/json/mod.rs`
**Description:** CBOR encoder lacks validation and size limits.
**Resolution:** Added 23 comprehensive tests covering: small integers (0-23), boundary integers, negative integers, zero, empty strings/arrays/objects, unicode strings, long strings, nested structures, decimals, deterministic map key ordering, multiple run determinism, large arrays, all types combined, special characters, deeply nested structures, max i64, min safe i64, string length boundaries, and output size validation.

### 27. Lambda Lowering Type Annotations ✅ COMPLETED
**Source:** BUGFIX.md (line 106)
**File:** `mtpscript-core/src/ir/lower.rs:285-293`
**Description:** Lambda lowering discards type annotations. Line 289 extracts only parameter names, losing type expressions.
**Resolution:** Lambda lowering now properly calls `resolve_type_expr(type_expr)` to preserve type annotations. Tests verify type preservation for multiple typed parameters.

### 28. PreloadedRuntime Graceful Shutdown ✅ COMPLETED
**Source:** BUGFIX.md (line 108)
**File:** `mtpscript-core/src/lambda/runtime.rs:283-294`
**Description:** PreloadedRuntime infinite loop has no exit condition. Lambda functions should respect context deadline and exit cleanly.
**Resolution:** Added `shutdown_handle()`, `stop()`, and `is_shutdown_requested()` methods for external shutdown control. Added 5 tests for shutdown flag, external handle setting, shared handles, cross-thread shutdown, and deadline calculation.

### 29. Chrono Dependency Feature Flags ✅ COMPLETED
**Source:** BUGFIX.md (line 110)
**File:** `mtpscript-core/src/modules/npm_bridge.rs:139`
**Description:** Chrono dependency used without feature flags. Wall-clock time in audit manifest violates determinism. Should use seed-derived timestamp.
**Resolution:** Audit manifest now uses content-hash-derived deterministic timestamps instead of wall-clock time. Added 5 tests verifying deterministic timestamp derivation, different content produces different timestamps, timestamp range validation, manifest serialization determinism, and no wall-clock time usage.

### 30. Audit Logging Thread Safety ✅ COMPLETED
**Source:** BUGFIX.md (line 112)
**File:** `mtpscript-core/src/audit/logger.rs:28-33`
**Description:** Audit logging to stderr may interleave. Multiple threads writing to `io::stderr()` can produce interleaved output.
**Resolution:** Added global mutex for thread-safe logging and 5 comprehensive tests including concurrent logging with 10 threads x 10 requests each, mutex poison recovery, and roundtrip serialization.

### 31. Nested Function Refactoring ✅ COMPLETED
**Source:** BUGFIX.md (line 116)
**File:** `mtpscript-core/src/runtime/interpreter.rs`
**Description:** eval_binop and eval_unaryop defined as nested functions inside eval_expr. Should be extracted as regular methods on Interpreter.
**Resolution:** Already refactored - `eval_binop` and `eval_unaryop` are proper methods on the Interpreter struct (lines 1274 and 1346), not nested functions.

### 32. Error Handling and Propagation
**Source:** BUGFIX.md (line 46)
**File:** `mtpscript-core/src/`
**Description:** Lack of proper error handling and propagation, allowing crashes and undefined behavior.

### 33. Memory Safety Beyond Rust Defaults
**Source:** BUGFIX.md (line 47)
**File:** `mtpscript-core/src/`
**Description:** No memory safety guarantees beyond Rust's defaults, potentially allowing use-after-free in complex scenarios.

### 34. Race Condition Vulnerabilities
**Source:** BUGFIX.md (line 48)
**File:** `mtpscript-core/src/`
**Description:** Race condition vulnerabilities in concurrent operations (if any).

### 35. Proper Session Management
**Source:** BUGFIX.md (line 52)
**File:** `mtpscript-core/src/`
**Description:** Lack of proper session management and state isolation.

### 36. Resource Cleanup and Lifecycle Management
**Source:** BUGFIX.md (line 56)
**File:** `mtpscript-core/src/`
**Description:** Lack of proper resource cleanup and lifecycle management, allowing resource leaks.

---

## Placeholder Implementations (from MTP-231)

These are lower priority placeholders that may need attention:

- `src/parser/mod.rs`: Array access and stub implementations
- `src/effects/builtins.rs`: Placeholder JSON serialization
- `src/sbom/mod.rs`: Mock Cargo.toml content for testing
- `src/modules/mod.rs`, `src/lambda/mod.rs`, `src/compiler/mod.rs`, `src/effects/mod.rs`, `src/types/mod.rs`: Stub modules
- `src/api/router.rs`, `src/api/openapi.rs`: Placeholder types
- `src/lexer/mod.rs`: Panics for unexpected tokens (acceptable for invalid input)
- `src/compiler/effects.rs`, `src/ir/tail_call.rs`, `src/compiler/deterministic.rs`: Placeholder definitions
- `mtpscript-core/src/compiler/pattern.rs`: Pattern matching in compiler
- `mtpscript-core/src/types/checker.rs`: Some placeholder types
- `mtpscript-core/src/snapshot/mod.rs`: Placeholder fields
- `mtpscript-core/src/security/*`: Various placeholder implementations
- `mtpscript-core/src/ir/lower.rs`: Some panics for unexpected expressions
- `mtpscript-core/src/api/handler.rs`: Rate limiting, SHA-256
- `mtpscript-core/src/parser/mod.rs`: Panics for unexpected declarations
- `mtpscript-core/src/compiler/effects.rs`: Placeholder returns
- `mtpscript-core/src/security/fuzz.rs`: Placeholder fuzzing
- `mtpscript-core/src/types/builtins.rs`: Panics for unexpected ADT types
- `mtpscript-core/src/compiler/respond.rs`: Expression type support

---

## Quick Reference: Issues by File

| File | Issue Count |
|------|-------------|
| `mtpscript-core/src/` (general) | 9 |
| `mtpscript-core/src/security/` | 3 |
| `mtpscript-core/src/ir/lower.rs` | 2 |
| `mtpscript-core/src/json/` | 3 |
| `mtpscript-core/src/compiler/` | 2 |
| `mtpscript-core/src/lambda/runtime.rs` | 1 |
| `mtpscript-core/src/modules/npm_bridge.rs` | 1 |
| `mtpscript-core/src/audit/logger.rs` | 1 |
| `mtpscript-core/src/interpreter.rs` | 1 |
| External/Process | 13 |

---

## Recommended Action Order

### Phase 1: Security & Compliance (Critical)
1. MTP-199: External Security Assessment
2. MTP-204: Compliance Audit
3. MTP-227: Security Requirements Traceability
4. MTP-228: Automated Security Scanning

### Phase 2: Infrastructure (P0)
5. MTP-202: Centralized Logging and Monitoring
6. MTP-223: Comprehensive Audit Logging
7. MTP-218: Multiple Security Layers
8. MTP-220: Adaptive Rate Limiting

### Phase 3: Testing & Validation (P1)
9. Execute All Test Cases (#19)
10. Cross-Platform Determinism Tests (#23)
11. JSON Parser Fuzzing (#24)
12. AST to IR Lowering Unit Tests (#20)
13. Tail Call Detection Tests (#21)

### Phase 4: Hardening (P1)
14. MTP-226: Side-Channel Attack Mitigations
15. Comprehensive Cryptography Audit (#15)
16. Supply Chain Security (#18)
17. Data Flow Analysis (#17)

### Phase 5: Resilience & Operations
18. MTP-200: Chaos Engineering
19. MTP-203: Continuous Profiling
20. MTP-229: Incident Response Plan
21. MTP-201: Formal Verification
22. MTP-224: Privacy-by-Design

### Phase 6: Code Quality (Low Priority)
23-36. Remaining items as time permits
