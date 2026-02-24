# Phase 0: MicroQuickJS Hardening & Determinism (v5.1)

This phase focuses on transforming the standard QuickJS engine into **MicroQuickJS**, a sealed, per-request sandbox runtime that guarantees **deterministic execution semantics** with **sub-millisecond reuse overhead** for MTPScript v5.1.

## 1. Snapshot-Based Execution Model

### 1.1 VM Snapshot Creation & Signing
- [x] Implement build-time snapshot generation (`.msqs` format) - immutable, 150–400 kB
- [x] Add ECDSA-P256 signature verification for snapshots before runtime mapping
- [x] Implement `clone_vm()` with copy-on-write semantics (≤ 60 µs best-case, ≤ 1 ms worst-case under EFS/FUSE cold page fault)
- [x] Add secure wipe functionality for pages containing PCI-classified data

### 1.2 Per-Request Sandbox Isolation
- [x] Ensure one fresh VM per request with no cross-request state leakage
- [x] Implement VM discard after every request with secure memory wipe
- [x] Add host effect re-injection per VM after static initializers
- [x] Guarantee zero ambient authority and zero hidden I/O

## 2. Deterministic Seed Injection

### 2.1 Seed Algorithm Implementation
- [x] Implement SHA-256 deterministic seed from canonical concatenation
- [x] Ensure seed is never reused across requests
- [x] Bind seed into VM state before any static initializer execution

### 2.2 Runtime Determinism Guarantees
- [x] Remove all non-deterministic features (Date.now, Math.random, etc.)
- [x] Implement total structural equality (no reference identity) for all types
- [x] Include closure environments in structural equality of functions
- [x] Implement deterministic CBOR serialization (RFC 7049 §3.9) for hashing
- [x] Ensure FNV-1a 64-bit hashing for structural equality
- [x] Implement deterministic object key iteration order per §5 (Type tag → Hash → CBOR tie-break)
- [x] Explicitly exclude functions from being used as map keys

## 3. Runtime Gas Limit Injection

### 3.1 Host Adapter Gas Contract
- [x] Add `MTP_GAS_LIMIT` environment variable reading (default: 10,000,000)
- [x] Implement gas limit range validation (1–2,000,000,000 inclusive)
- [x] Inject 64-bit unsigned `gasLimit` into VM state
- [x] **Ensure `gasLimit` is invisible to guest code** (cannot be read or queried) (§7)
- [x] Add gas limit to audit log (`gasLimit=<value>`)

### 3.2 Gas Exhaustion Semantics
- [x] Implement gas cost table (Annex A) with fixed β-reduction units
- [x] Add cumulative gas tracking against injected `gasLimit`
- [x] Implement deterministic gas exhaustion error:
  ```json
  {
    "error": "GasExhausted",
    "gasLimit": <uint64>,
    "gasUsed": <uint64>
  }
  ```
- [x] Ensure tail calls cost 0 gas units

## 4. IEEE-754 Decimal Arithmetic

### 4.1 Decimal Type Implementation
- [x] Integrate C decimal library with IEEE-754-2008 decimal128 compliance
- [x] Implement `Decimal` type with 1–34 digit significand and 0–28 scale
- [x] Add round-half-even rounding (ties to even per clause 4.3.2)
- [x] Implement overflow handling with `Result<Decimal, Overflow>`

### 4.2 Decimal Operations
- [x] Support constant-time comparison algorithm with scale normalization
- [x] Implement shortest canonical string serialization (no trailing zeros)
- [x] Expose decimal arithmetic as MicroQuickJS globals
- [x] Ensure deterministic serialization/deserialization

## 5. Engine Hardening & Security

### 5.1 Forbidden JavaScript Features
- [x] Disable `eval()`, `new Function()`, and dynamic code loading
- [x] Remove `class`, `this`, `try/catch/finally` constructs
- [x] Forbid loops, global mutation, and reflection/introspection
- [x] **Disable all floating-point math and JS `Number` (double) support** (§1.2)
- [x] **Disable implicit type coercions** (strict equality and typing only) (§1.2)
- [x] Patch MicroQuickJS to forbid double-path for integers > 2⁵³-1
- [x] Ensure `JsonNull` is inhabited only through parsing (no literal support)
- [x] **Remove JavaScript event loop visibility** (no `setTimeout`, `Promise` microtasks, etc.)

### 5.2 Memory & Resource Constraints
- [x] Implement strict heap allocation tracking with fixed memory budget
- [x] Remove all OS-level access (filesystem, network, process)
- [x] Implement **sensitive page tracking** for selective secure wipe (§22)
- [x] Implement **block-synchronous host effect execution** for `Async.await`
- [x] Add immutable `Object.prototype` and globals to prevent prototype pollution
- [x] Ensure no shared mutable state across requests

## 6. Canonical JSON Output

### 6.1 Deterministic Serialization
- [x] Implement RFC 8785 canonical JSON with duplicate-key rejection
- [x] Ensure object keys ordered by §5 rules (type tag, hash, CBOR tie-break)
- [x] Add decimal shortest form serialization (no `-0`, `NaN`, `Infinity`)
- [x] Preserve array order from source literals (left-associative)

### 6.2 Response Hashing
- [x] Generate SHA-256 of canonical JSON response body
- [x] Ensure bit-exact SHA-256 response across conforming runtimes
- [x] Include gas exhaustion errors in deterministic response format

## Acceptance Criteria (v5.1)

### Functional Requirements
- [x] All MicroQuickJS bytecode produces identical SHA-256 hash across platforms
- [x] Per-request sandbox isolation with ≤ 1 ms cold-start overhead
- [x] Deterministic seed algorithm produces same 32-byte seed for identical inputs
- [x] Gas limit injection and exhaustion work within 1–2,000,000,000 range
- [x] Decimal arithmetic compliant with IEEE-754-2008 decimal128

### Security & Audit Requirements
- [x] Zero cross-request state leakage (secure wipe verified)
- [x] All forbidden JavaScript features properly disabled
- [x] ECDSA-P256 snapshot signature verification functional
- [x] Host effects injected deterministically using request seed
- [x] Canonical JSON output enables deterministic response hashing

### Performance Requirements
- [x] VM clone time: ≤ 60 µs best-case, ≤ 1 ms worst-case under EFS page fault
- [x] Memory usage stays within configured heap limit (no shared heap)
- [x] Gas counting accurately terminates execution at budget exhaustion
- [x] Deterministic CBOR and FNV-1a hashing performance adequate for serverless

### Test Coverage
- [x] All tests in `tests/` pass with zero failures
- [x] Snapshot lifecycle (build → sign → verify → clone → execute → wipe) functional
- [x] Cross-platform determinism verified on different OS/CPU architectures
- [x] Gas cost table (Annex A) accurately implemented and tested

