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
- [ ] `mtpscript-core/src/api/openapi.rs:72-115,170-294` – OpenAPI `$ref` generation currently hashes only the record name via `sha256_ref(&record.name)`, so Annex B (§8) cannot be satisfied because identical schemas with different names produce distinct references and refs drift when type names change. Compute the SHA-256 over the canonical schema (field/variant definitions) and reuse that hash in both the component name and `$ref` so deterministic OpenAPI output matches the spec’s folding rules.
 - [ ] `mtpscript-core/src/json/serialize.rs:2-69` – Canonical JSON serialization claims to obey the §§5/23 ordering rules but actually sorts object keys lexicographically instead of comparing type tag → FNV-1a hash → CBOR tie-break values, so the normative map order is not enforced and determinism can break for keys requiring the tie breaker. Implement the multi-stage comparator, add regression tests for complex keys, and re-run serialization before hashing responses.
 - [ ] `host/unsafe/*` – Section 21 (requirements/TECHSPECV5.md:58-68) requires npm bridge adapters to live in `host/unsafe/*.js` and be deterministic wrappers for unsafe dependencies, but the repo has no `host` directory or adapters, preventing npm bridging entirely. Add the host/unsafe adapters, wire them into the runtime’s effect registry, and generate the required audit manifest per §21.
 - [ ] `mtpscript-core/src/runtime/interpreter.rs, mtpscript-core/src/errors/mod.rs, mtpscript-core/src/lambda/runtime.rs, mtpscript-core/src/modules/*.rs` – Multiple compilation errors blocking build: MtpError enum variants misused (require {error, message} struct syntax), interpreter eval_expr returns Result in unit context, missing inject_builtin_objects method, call_function/eval_unaryop called as free functions instead of methods, type inference failures in closures; fix to enable compiling and running sample programs.

## New Bugs Found During Comprehensive Audit (2026-01-11)

### Critical Bugs

- [ ] `mtpscript-core/src/ir/tail_call.rs:26-32` – Tail call detection logic is inverted. The `is_tail_recursive_call` function requires ALL branches to contain tail calls (uses `&&` at line 27-28 for if branches, and `all()` at line 30-32 for match cases). This is semantically wrong: a function is tail-recursive if ANY terminal branch is a tail call to itself, not ALL branches. This causes valid tail-recursive functions to be flagged as non-tail-recursive, breaking the §6 optimization guarantee and causing 2-gas penalties on recursive calls that should cost 0-gas.

- [ ] `mtpscript-core/src/runtime/effects.rs:121-178` – Effect injection is non-functional. `inject_effects` injects `FunctionValue` objects into `global_scope`, but these functions have no bodies registered in `function_bodies`. When the interpreter calls an effect like `DbRead()`, `call_function()` at interpreter.rs:471-481 looks up the body in `function_bodies` and fails with "Function body not found". Effects cannot execute, violating MTP-083 deterministic effect contract.

- [ ] `mtpscript-core/src/lambda/runtime.rs:213-224` – Lambda response uses wrong request_id. The `send_response` method reads `_X_AMZN_TRACE_ID` from environment (line 218) instead of using the `request_id` from the actual invocation payload. This causes responses to be sent with incorrect request identifiers, breaking Lambda runtime protocol compliance.

- [ ] `mtpscript-core/src/lambda/adapter.rs:53-81` – Lambda effect injection is empty. The three helper methods `inject_environment_effect`, `inject_logging_effect`, and `inject_time_effect` all return `Ok(())` without actually injecting anything. Lambda-specific effects (environment variable access, structured logging, deterministic time) are non-functional.

### Medium Bugs

- [ ] `mtpscript-core/src/gas/costs.rs:36-38` – DbWrite gas cost is guessed, not spec-compliant. Line 37 has comment "assuming DbWrite also costs 50 like DbRead" but Annex A may specify different costs. The hardcoded assumption may violate gas metering requirements. Verify against spec and update accordingly.

- [ ] `mtpscript-core/src/json/parse.rs:154-155,183-185` – JSON parser accepts trailing commas which is non-standard. Lines 154-155 (arrays) and 183-185 (objects) allow trailing commas before closing brackets. While lenient parsing may be intentional, this violates RFC 8259 strict JSON and could cause determinism issues if other implementations reject such input.

- [ ] `mtpscript-core/src/runtime/clone.rs:60-61` – Signature verification still bypassed. The `clone_interpreter` function calls `verify_snapshot()` (line 60) which only checks magic/version/CRC (lines 6-38) but does NOT call ECDSA signature verification from `security::verify::verify_snapshot()`. Tampered snapshots with valid CRC but forged signatures will load. Must integrate signature verification per §14.

- [ ] `mtpscript-core/src/api/openapi.rs:173,197,281,288` – Schema refs hash name not content. The `sha256_ref()` calls hash the record/ADT *name* (e.g., `&record.name` at line 173), not the canonical schema content. Two structurally identical types with different names produce different `$ref` URIs, and renaming a type changes all refs even if schema unchanged. Violates Annex B §8 deterministic schema folding.

- [ ] `mtpscript-core/src/security/verify.rs:14-31` – Signature extraction offset may be incorrect. Line 18 calculates `sig_start = snapshot.len() - 132` which assumes signature is 128 bytes, but ECDSA-P256 signatures are 64 bytes (DER-encoded P1363). Need to verify offset calculation matches snapshot format: JS content ends at `len - 68` (64 sig + 4 CRC) per snapshot/mod.rs:93.

### Low Priority / Testing Gaps

- [ ] `mtpscript-core/src/ir/lower.rs:285-293` – Lambda lowering discards type annotations. Line 289 extracts only parameter names, losing type expressions. While TypeExpr is parsed, it's not preserved in IrExpr::Lambda, which may cause type checking gaps for lambda parameters.

- [ ] `mtpscript-core/src/lambda/runtime.rs:283-294` – PreloadedRuntime infinite loop has no exit condition. The `run()` method loops forever with no way to gracefully shutdown. Lambda functions should respect context deadline and exit cleanly.

- [ ] `mtpscript-core/src/modules/npm_bridge.rs:139` – Chrono dependency used without feature flags. Line 139 calls `chrono::Utc::now().to_rfc3339()` but chrono's clock feature may not be enabled, and wall-clock time in audit manifest violates determinism. Should use seed-derived timestamp.

- [ ] `mtpscript-core/src/audit/logger.rs:28-33` – Audit logging to stderr may interleave. Multiple threads writing to `io::stderr()` can produce interleaved output. For compliance audit trails, need atomic line writes or structured log framing.

- [ ] `mtpscript-core/src/runtime/interpreter.rs:335` – ExprStmt evaluation discards value but returns Null. Line 335 matches `JsExpr::ExprStmt(expr)` but returns `Ok(Value::Null)` without evaluating `expr`. Expression statements should evaluate for side effects even if discarding result.

- [ ] `mtpscript-core/src/interpreter.rs:338-416` – eval_binop and eval_unaryop defined as nested functions inside eval_expr. While valid Rust, this unconventional structure with nested fn definitions at lines 338-416 and 418-427 inside a match arm makes the code harder to maintain and could cause scope confusion. Should be extracted as regular methods on Interpreter.

complete