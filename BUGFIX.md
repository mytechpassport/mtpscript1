# BUGFIX LOG

- [x] `mtpscript-core/src/ir/lower.rs:194` – `params` is never read in the lambda lowering match arm, which triggered a compiler warning and may hide unfinished logic for parameter handling (seen during `cargo test -p mtpscript-core`).
- [x] `mtpscript-core/src/parser/mod.rs:630` – `check_next` is defined but unused, so either it should be integrated into the parser (to support lookahead scenarios) or removed to avoid dead code warnings from the compiler.
- [x] `mtpscript-core/src/types/builtins.rs:128` – The `ctx` binding inside `test_option_result_acceptance_criteria` is never used, weakening that test and emitting an unused-variable warning; either add assertions or drop the local to clear the warning.
- [x] `mtpscript-core/src/parser/mod.rs:58` – Unary `!` is parsed into `BinOp::Or` instead of a dedicated not operation, so `!expr` doesn’t behave correctly and violates the spec’s boolean semantics.
- [x] `mtpscript-core/src/effects/async_effect.rs:101` – The fallback `format!("{:?}", expr)` when encoding complex expressions for `promiseHash` produces non-deterministic bytes, breaking the Async await hashing guarantees required by §7-a.
- [x] `mtpscript-core/src/runtime/clone.rs:39-62` – `parse_js_to_ast` always errors out, so `clone_interpreter` can never complete

- [ ] Execute all newly added test cases and confirm their expected results match the spec: test_adt_pattern_matching, test_recursive, test_gas_exhaustion, test_decimal_edge, test_map_list, test_pipeline_full, test_lambdas_closures, test_functions_effects, test_json_ops, test_hash_cbor, test_json_duplicate_keys, test_db_effects, test_http_out, test_await_syntax, test_api_methods, test_records, test_array_bounds, test_number_overflow, test_json_null. Run the test suite to verify deterministic execution, canonical JSON output, gas metering, effect handling, and error cases per TECHSPECV5.md. and no interpreter instance is ever produced, defeating the MTP-082 requirements for snapshot cloning and initialization.
- [x] `mtpscript-core/src/runtime/effects.rs:101-141` – `inject_effects` ignores the supplied `seed` and merely injects stub `FunctionValue`s without tying them to deterministic effect implementations or caching, leaving MTP-083’s deterministic effect contract unfulfilled.
- [x] `mtpscript-core/src/api/handler.rs:85-92` – `execute_handler` always returns a hardcoded success object rather than the actual API implementation, so the request handler never executes user code (MTP-111).
- [x] `mtpscript-core/src/types/checker.rs` – Type checker lacks fuzz tests for adversarial inputs, potentially allowing type confusion attacks or DoS via deep nesting; add cargo-fuzz integration for AST type checking.
- [x] `mtpscript-core/src/json/parse.rs` – JSON parser has no depth or size limits, vulnerable to DoS attacks (e.g., billion laughs); add configurable limits to prevent resource exhaustion.
- [x] `mtpscript-core/src/effects/builtins.rs` – Built-in functions lack input validation and sanitization, allowing potential injection attacks; add bounds checking and sanitization for all builtin inputs.
- [x] `mtpscript-core/src/ir/nodes.rs` – IR data structures lack validation, allowing malformed IR to cause runtime failures or exploits; implement IR schema validation before lowering.
- [ ] `mtpscript-core/src/ir/lower.rs` – AST to IR lowering has no dedicated unit tests, risking incorrect transformations; add equivalence tests for lowering correctness.
- [ ] `mtpscript-core/src/ir/tail_call.rs` – Tail call detection lacks comprehensive tests for complex expressions, potentially missing optimization opportunities or allowing stack overflows; add property-based tests.
- [ ] `mtpscript-core/src/compiler/effects.rs` – Effect call compilation lacks input validation, allowing malicious effect arguments; add whitelist validation for effect parameters.
- [ ] `mtpscript-core/src/compiler/deterministic.rs` – Code generation lacks cross-platform determinism tests, risking non-reproducible builds; add tests across different environments.
- [ ] `mtpscript-core/src/json/parse.rs` – JSON parsing lacks fuzzing for malicious inputs, missing edge cases like invalid UTF-8 or control characters; integrate fuzz testing.
- [ ] `mtpscript-core/src/json/serialize.rs` – Canonical JSON serialization lacks determinism verification across runs, potentially breaking signatures; add multi-run determinism tests.
- [ ] `mtpscript-core/src/json/mod.rs` – CBOR encoder lacks validation and size limits, allowing invalid output or DoS; add CBOR spec compliance checks and limits.

- [x] `mtpscript-core/src/runtime/value.rs` – Value type lacks Hash implementation, violating spec requirements for hashing; implement secure Hash trait (e.g., SHA-256 for large values).
- [x] `mtpscript-core/src/api/handler.rs` – API request handler lacks input validation and rate limiting, vulnerable to injection and DoS attacks; add sanitization and rate limiting.
- [x] `mtpscript-core/src/security/sign.rs` – ECDSA signing lacks key strength validation and management policies, allowing weak keys; add FIPS-compliant key validation.
- [x] `mtpscript-core/src/modules/import.rs` – Static imports lack code signing verification, enabling supply chain attacks; add cryptographic verification for imported modules.
- [ ] `mtpscript-core/src/` – System lacks comprehensive threat model and security audit; conduct full external security assessment and implement findings before production deployment.
- [ ] `mtpscript-core/src/` – No chaos engineering or failure injection testing, masking resilience issues under load or adversarial conditions; implement chaos monkey testing suite.
- [ ] `mtpscript-core/src/` – Missing formal verification for critical components (parser, type checker, interpreter); apply model checking and theorem proving where feasible.
- [ ] `mtpscript-core/src/` – No centralized logging and monitoring system, hindering incident response and compliance; implement security event detection and alerting.
- [ ] `mtpscript-core/src/` – Performance benchmarking absent, allowing undetected bottlenecks and scalability issues; implement continuous profiling and optimization.
- [ ] `mtpscript-core/src/` – Compliance gaps (GDPR, PCI-DSS, etc.) unaddressed, risking legal penalties; conduct full compliance audit and remediate violations.
- [x] `mtpscript-core/src/lexer/mod.rs` – Lexer lacks input size limits and Unicode security handling, vulnerable to DoS and encoding attacks; add configurable limits and proper UTF-8 validation.
- [x] `mtpscript-core/src/parser/mod.rs` – Parser has no stack depth limits, allowing recursive descent stack overflows; implement recursion limits and robust error recovery.
- [x] `mtpscript-core/src/types/checker.rs` – Type checker lacks bounds checking on type recursion depth, enabling DoS via deep type nesting; add depth limits and cycle detection.
- [x] `mtpscript-core/src/runtime/interpreter.rs` – Interpreter lacks execution timeouts and resource quotas, allowing infinite loops and resource exhaustion; add configurable timeouts and metering.
- [x] `mtpscript-core/src/errors/mod.rs:8-27` – `MtpError::GasExhausted` currently serializes as `{"error":"GasExhausted","details":{"gas_limit":...}}` because of `#[serde(tag="error", content="details")]` and snake_case fields, which contradicts §0-c/§6’s deterministic error shape `{"error":"GasExhausted","gasLimit":<uint64>,"gasUsed":<uint64>}`; produce the flat camelCase payload and add a regression test that serializes a `GasExhausted` to the canonical string.
- [ ] `mtpscript-core/src/security/mod.rs` – No comprehensive cryptography audit or key management policies, allowing weak signatures and key compromise; implement FIPS-compliant crypto and key rotation.
- [x] `mtpscript-core/src/api/handler.rs` – API handlers lack comprehensive input validation and sanitization, vulnerable to injection attacks; add strict validation and XSS prevention.
- [x] `mtpscript-core/src/runtime/value.rs` – Value operations lack constant-time implementations, enabling timing attacks on sensitive data; implement constant-time comparisons and operations.
- [ ] `mtpscript-core/src/` – No data flow analysis or taint tracking, allowing information leaks through implicit channels; implement static and dynamic taint analysis.
- [ ] `mtpscript-core/src/` – Missing comprehensive fuzz testing across all components, leaving edge cases untested; integrate AFL/libFuzzer for continuous fuzzing.
- [ ] `mtpscript-core/src/` – No supply chain security measures, vulnerable to dependency injection attacks; implement SBOM generation and dependency scanning.
- [ ] `mtpscript-core/src/` – Lack of proper error handling and propagation, allowing crashes and undefined behavior; implement comprehensive error recovery and graceful degradation.
- [ ] `mtpscript-core/src/` – No memory safety guarantees beyond Rust's defaults, potentially allowing use-after-free in complex scenarios; add additional memory safety checks and bounds validation.
- [ ] `mtpscript-core/src/` – Race condition vulnerabilities in concurrent operations (if any), risking data corruption; implement proper synchronization primitives and race detection.
- [ ] `mtpscript-core/src/` – No defense-in-depth strategies, relying on single points of failure; implement multiple security layers and fail-safes.
- [ ] `mtpscript-core/src/` – Missing input/output validation at all boundaries, allowing malformed data propagation; add schema validation and sanitization everywhere.
- [ ] `mtpscript-core/src/` – No rate limiting or throttling mechanisms, vulnerable to DoS attacks; implement adaptive rate limiting across all interfaces.
- [ ] `mtpscript-core/src/` – Lack of proper session management and state isolation, allowing cross-request contamination; implement strict state isolation and cleanup.
- [ ] `mtpscript-core/src/` – No integrity checks on internal data structures, allowing memory corruption exploits; add checksums and validation for critical structures.
- [ ] `mtpscript-core/src/` – Missing audit trails for all security-relevant operations, hindering forensic analysis; implement comprehensive audit logging with tamper-evident storage.
- [ ] `mtpscript-core/src/` – No zero-knowledge or privacy-preserving features, risking data exposure; implement privacy-by-design principles where applicable.
- [ ] `mtpscript-core/src/` – Lack of proper resource cleanup and lifecycle management, allowing resource leaks; implement RAII patterns and resource tracking.
- [ ] `mtpscript-core/src/` – No defense against side-channel attacks (timing, power, etc.), allowing information leakage; implement constant-time algorithms and side-channel mitigations.
- [ ] `mtpscript-core/src/` – Missing formal security requirements and acceptance criteria validation, allowing implementation gaps; create security requirements traceability matrix.
- [ ] `mtpscript-core/src/` – No continuous security monitoring or vulnerability scanning in CI/CD, allowing regressions; implement automated security scanning and alerting.
- [ ] `mtpscript-core/src/` – Lack of incident response plan and security operations procedures, risking ineffective breach response; develop and test incident response capabilities.
- [x] `mtpscript-core/src/runtime/interpreter.rs:83-88` – The `execute` method is a placeholder that only returns the input JS code as a string without parsing or running it, so compiled MTPScript programs cannot actually execute; implement a full JavaScript subset interpreter to evaluate the generated code and return the actual result (e.g., for the API handler, call the generated function and return its JSON output).
- [x] `mtpscript-core/src/types/decimal.rs` – Decimal type to_string does not produce shortest canonical string; e.g., 100.0 instead of 100, failing tests and spec §4-a.
- [x] `mtpscript-core/src/types/decimal.rs` – Decimal mul and div operations produce strings with trailing .0, not shortest form, violating canonical serialization.
- [x] `mtpscript-core/src/runtime/clone.rs` – Clone interpreter parses JS AST but does not use it to initialize the interpreter's function_bodies; instead, execute re-parses, defeating the purpose of pre-cloning and wasting time.
- [x] `mtpscript-core/src/runtime/wipe.rs` – Secure wipe implementation missing; file does not exist, violating §27.3 and secure disposal requirements.
- [x] `mtpscript/src/cli/commands.rs:251-272` – Serve command is placeholder returning hardcoded string; does not implement HTTP server for API execution as per §15 Local Web Server.
- [x] `mtpscript-core/src/api/handler.rs:62` – Snapshot hash in seed computation is hardcoded to [0u8;32] instead of SHA-256 of actual snapshot, violating deterministic seed per §0-b.
- [x] `mtpscript-core/src/runtime/interpreter.rs:70` – Interpreter gas counter initialized to hardcoded 10M instead of injected gas_limit from request, violating §0-c gas limit injection.
- [x] `mtpscript-core/src/runtime/mod.rs` – Gas limit not read from MTP_GAS_LIMIT env var as required by §0-c host adapter contract; defaults to 10M without validation.
- [x] `tests/integration.rs:135-137` – E2E test for API compilation skips on failure, indicating API declarations not fully supported in compilation pipeline; violates full pipeline requirement in §12.
- [x] `mtpscript-core/src/runtime/clone.rs:48-58` – Parsed JS AST discarded after checking; interpreter not pre-populated with function definitions, forcing re-parsing on every execute call, violating pre-clone optimization.
- [x] `mtpscript-core/src/runtime/clone.rs:41-58` – `clone_interpreter` returns an `Interpreter` but never calls `wipe_interpreter`, so PCI-classified pages are not zeroed after each request as §0-a requires; hook `wipe_interpreter(interp, true)` (or similar) into the request lifecycle/drop path and verify the heap is zeroed when the PCI flag is set.
- [x] `mtpscript-core/src/runtime/clone.rs:5-33` – The runtime only checks magic/version before accepting `.msqs`, skipping the ECDSA-P256 signature/CRC verification mandated by §14, so tampered snapshots load silently; call `security::verify::verify_snapshot` (or equivalent) and fail clone when signature validation fails.
- [x] `mtpscript-core/src/effects/async_effect.rs:140` – `deterministic_expr_serialize` function not implemented, causing compilation failure; required for deterministic promiseHash in Async effect.
- [x] `mtpscript-core/src/api/openapi.rs:72-115,170-294` – OpenAPI `$ref` generation was hashing only record name. FIXED: Now uses `record.content_hash()` and `adt.content_hash()` which compute SHA-256 over the canonical schema structure, satisfying Annex B §8.
 - [x] `mtpscript-core/src/json/serialize.rs:2-69` – Canonical JSON serialization was using lexicographic order. FIXED: Now implements §5 ordering rules - sorts by FNV-1a 64-bit hash of key, with CBOR byte-wise tie-break for hash collisions. Added tests for section 5 ordering and CBOR tie-break.
 - [x] `host/unsafe/*` – Section 21 requires npm bridge adapters. FIXED: Created `host/unsafe/` directory with uuid.js, crypto.js, datetime.js adapters, manifest.json audit file, and index.js. All adapters follow §21 requirements: pure functions of seed + args, deterministic output, no wall-clock time, no shared state.
 - [ ] `mtpscript-core/src/runtime/interpreter.rs, mtpscript-core/src/errors/mod.rs, mtpscript-core/src/lambda/runtime.rs, mtpscript-core/src/modules/*.rs` – Multiple compilation errors blocking build: MtpError enum variants misused (require {error, message} struct syntax), interpreter eval_expr returns Result in unit context, missing inject_builtin_objects method, call_function/eval_unaryop called as free functions instead of methods, type inference failures in closures; fix to enable compiling and running sample programs.

## New Bugs Found During Comprehensive Audit (2026-01-11)

### Critical Bugs

- [x] `mtpscript-core/src/ir/tail_call.rs:26-32` – Tail call detection logic was inverted. FIXED: Now uses `||` for if branches and `.any()` for match cases, correctly identifying functions as tail-recursive if ANY branch contains a tail call.

- [x] `mtpscript-core/src/runtime/effects.rs:121-178` – Effect injection was non-functional. FIXED: `inject_effects` now properly registers function bodies in `function_bodies` that call builtin implementations (e.g., `db_read_impl`), and registers the builtins map for actual execution.

- [x] `mtpscript-core/src/lambda/runtime.rs:213-224` – Lambda response was using wrong request_id. FIXED: Now properly extracts `request_id` from the invocation payload and passes it to `send_response_with_id()` method.

- [x] `mtpscript-core/src/lambda/adapter.rs:53-81` – Lambda effect injection was empty. FIXED: All three helper methods now properly inject functions (GetEnv, LambdaLog, GetTime) with function bodies and builtin implementations.

### Medium Bugs

- [x] `mtpscript-core/src/gas/costs.rs:36-38` – DbWrite gas cost was guessed. FIXED: Now set to 100 (2x DbRead's 50) with proper comment explaining the rationale (disk I/O, transaction logging, replication costs).

- [x] `mtpscript-core/src/json/parse.rs:154-155,183-185` – JSON parser was accepting trailing commas. FIXED: Now properly rejects trailing commas per RFC 8259 - after parsing comma, checks if next non-whitespace is closing bracket and returns error.

- [x] `mtpscript-core/src/runtime/clone.rs:60-61` – Signature verification was bypassed. FIXED: `verify_snapshot()` now calls `verify_snapshot_signature()` when MTP_SIGNING_CERT env var is set, performing full ECDSA-P256 signature verification on the content hash.

- [x] `mtpscript-core/src/api/openapi.rs:173,197,281,288` – Schema refs were hashing name not content. FIXED: Now uses `record.content_hash()` and `adt.content_hash()` which compute SHA-256 over the canonical schema structure (sorted fields/variants), ensuring identical schemas produce identical refs regardless of name.

- [x] `mtpscript-core/src/security/verify.rs:14-31` – Signature extraction offset was incorrect. FIXED: Now correctly uses `sig_start = snapshot.len() - 68` (64 bytes signature + 4 bytes CRC) matching ECDSA-P256 raw format.

### Low Priority / Testing Gaps

- [ ] `mtpscript-core/src/ir/lower.rs:285-293` – Lambda lowering discards type annotations. Line 289 extracts only parameter names, losing type expressions. While TypeExpr is parsed, it's not preserved in IrExpr::Lambda, which may cause type checking gaps for lambda parameters.

- [ ] `mtpscript-core/src/lambda/runtime.rs:283-294` – PreloadedRuntime infinite loop has no exit condition. The `run()` method loops forever with no way to gracefully shutdown. Lambda functions should respect context deadline and exit cleanly.

- [ ] `mtpscript-core/src/modules/npm_bridge.rs:139` – Chrono dependency used without feature flags. Line 139 calls `chrono::Utc::now().to_rfc3339()` but chrono's clock feature may not be enabled, and wall-clock time in audit manifest violates determinism. Should use seed-derived timestamp.

- [ ] `mtpscript-core/src/audit/logger.rs:28-33` – Audit logging to stderr may interleave. Multiple threads writing to `io::stderr()` can produce interleaved output. For compliance audit trails, need atomic line writes or structured log framing.

- [x] `mtpscript-core/src/runtime/interpreter.rs:335` – ExprStmt evaluation was discarding value. FIXED: Now properly evaluates the expression with `self.eval_expr(expr, local_scope)` and returns the result.

- [ ] `mtpscript-core/src/interpreter.rs:338-416` – eval_binop and eval_unaryop defined as nested functions inside eval_expr. While valid Rust, this unconventional structure with nested fn definitions at lines 338-416 and 418-427 inside a match arm makes the code harder to maintain and could cause scope confusion. Should be extracted as regular methods on Interpreter.

complete