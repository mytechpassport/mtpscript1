# MTPScript Implementation Tasks

## Overview

This document tracks the remaining implementation tasks for MTPScript based on the architecture defined in NEWARCHITECTURE.md. Components are categorized by priority and dependency order.

### Key Characteristics
- **Zero ambient authority** – No implicit capabilities
- **Zero hidden I/O** – All effects are explicit
- **Zero cross-request state** – Fresh VM per request
- **Explicit capability declaration** – Effect system controls authority
- **Per-request sandbox isolation** – Sub-millisecond reuse overhead
- **Deterministic Execution**: Same input bytes → same output bytes (SHA-256 verifiable)
- **Minimal Memory Footprint**: Optimized for constrained Lambda environments
- **Tracing Garbage Collector**: Instead of reference counting
- **No Floating-Point**: Uses `Decimal` for deterministic arithmetic
- **ES5 Subset**: Implements a stricter JavaScript subset (no `eval`, `class`, `this`, `try/catch`)

---

## 1. Compiler (Rust) - `compiler/`

### 1.1 Frontend - CRITICAL PATH

| File | Status | Tasks Remaining |
|------|--------|-----------------|
| `frontend/lexer.rs` | 🔴 Stub | Implement `next_token()` - full tokenization of MTPScript syntax |
| `frontend/parser.rs` | 🔴 Stub | Implement `parse_module()`, `parse_pipeline()` (left-associative), expression parsing |
| `frontend/ast.rs` | ✅ Complete | AST type definitions done |

**Acceptance Criteria:**
- [ ] Lexer correctly tokenizes all MTPScript keywords, operators, and literals
- [ ] Parser produces correct AST for all valid fixtures
- [ ] Pipeline operator `|>` is left-associative: `a |> b |> c` ≡ `(a |> b) |> c`
- [ ] Parser rejects invalid syntax with meaningful error messages
- [ ] `if` requires `else` branch (enforced at parse time)

### 1.2 Type Checker

| File | Status | Tasks Remaining |
|------|--------|-----------------|
| `typechecker/types.rs` | 🟡 Partial | Type definitions present |
| `typechecker/checker.rs` | 🔴 Stub | Implement type inference, checking, generics |
| `typechecker/unify.rs` | 🔴 Stub | Implement type unification algorithm |

**Acceptance Criteria:**
- [ ] Type inference for let bindings, function parameters, return types
- [ ] Generic type instantiation (`Option<T>`, `Result<T, E>`)
- [ ] Record structural typing
- [ ] Union type variant type checking
- [ ] No `null`/`undefined` - only `Option<T>` and `Result<T, E>`
- [ ] Pattern match exhaustiveness checking

### 1.3 Effect System

| File | Status | Tasks Remaining |
|------|--------|-----------------|
| `effects/effects.rs` | ✅ Complete | Effect enum defined |
| `effects/checker.rs` | ✅ Complete | Basic effect checking |
| `effects/inference.rs` | 🔴 Stub | Implement effect inference from function bodies |

**Acceptance Criteria:**
- [ ] Lambdas are pure (no effects allowed)
- [ ] Named functions can declare effects with `uses { ... }`
- [ ] Effect usage in body must be subset of declared effects
- [ ] Effect propagation through function calls
- [ ] Reject undeclared effect usage (see `fixtures/invalid/effect_not_declared.mtp`)

### 1.4 IR Transformations

| File | Status | Tasks Remaining |
|------|--------|-----------------|
| `ir/typed_ir.rs` | ✅ Complete | IR definitions done |
| `ir/transforms.rs` | 🔴 Stub | Implement `desugar_await()`, `lower_pipeline()` |
| `ir/pipeline.rs` | 🔴 Stub | Pipeline operator lowering helpers |

**Acceptance Criteria:**
- [ ] `await e` → `Async.await(sha256(cbor(e)), contId, e)`
- [ ] `a |> f` → `f(a)`
- [ ] `a |> f |> g` → `g(f(a))`
- [ ] Continuation IDs are unique and monotonically increasing

### 1.5 Linker

| File | Status | Tasks Remaining |
|------|--------|-----------------|
| `linker/linker.rs` | 🟡 Partial | Reference resolution TODO |
| `linker/symbol_table.rs` | 🔴 Stub | Build symbol table from modules |
| `linker/union_check.rs` | ✅ Complete | Union hash verification done |

**Acceptance Criteria:**
- [ ] Build global symbol table from all modules
- [ ] Detect duplicate exports
- [ ] Detect unresolved imports
- [ ] Detect cyclic dependencies
- [ ] Union variant hash verification across modules

### 1.6 Code Generation

| File | Status | Tasks Remaining |
|------|--------|-----------------|
| `backend/js_subset.rs` | 🔴 Stub | Generate deterministic JS subset |
| `backend/canonical.rs` | 🔴 Stub | Canonicalization rules |
| `backend/bytecode.rs` | 🔴 Stub | Generate MTP bytecode |

**Acceptance Criteria:**
- [ ] Generated JS contains NO: `eval`, `class`, `this`, `try/catch`, loops
- [ ] Only recursion for iteration (with gas limits)
- [ ] Deterministic output (same input → same JS)
- [ ] Bytecode generation with gas instrumentation

### 1.7 Snapshot & Signing

| File | Status | Tasks Remaining |
|------|--------|-----------------|
| `snapshot/snapshot.rs` | 🔴 Stub | Generate .msqs snapshots |
| `snapshot/sign.rs` | 🔴 Stub | ECDSA-P256 signing |

**Acceptance Criteria:**
- [ ] Generate .msqs snapshot from bytecode (150-400 kB)
- [ ] ECDSA-P256 signature generation
- [ ] Signature appended as .msqs.sig

### 1.8 CLI

| File | Status | Tasks Remaining |
|------|--------|-----------------|
| `cli/main.rs` | 🟡 Partial | Implement full compilation pipeline |

**Acceptance Criteria:**
- [ ] `mtpc <input.mtp>` compiles to snapshot
- [ ] `--emit-js` outputs JavaScript for debugging
- [ ] `--snapshot` generates .msqs
- [ ] `--sign <key>` signs the snapshot
- [ ] Meaningful error messages with source locations

---

## 2. Core Runtime (C) - `core/`

### 2.1 Runtime Engine

| File | Status | Tasks Remaining |
|------|--------|-----------------|
| `runtime/mtp_js.h` | ✅ Complete | Public API header done |
| `runtime/mtp_js.c` | 🔴 Stub | Full JS engine implementation (~18k lines) |
| `runtime/mtp_js_priv.h` | 🔴 Missing/Stub | Private definitions |
| `runtime/mtp_js_opcode.h` | 🔴 Missing/Stub | Bytecode opcode definitions |

**Acceptance Criteria:**
- [ ] `MTP_NewContext()` creates context with deterministic seed
- [ ] `MTP_CloneSnapshot()` implements COW cloning (60µs - 1ms)
- [ ] `MTP_FreeContext()` with optional secure wipe
- [ ] No double-path for integers > 2⁵³-1
- [ ] Tracing garbage collector with compacting
- [ ] Gas metering at bytecode level
- [ ] No ambient I/O
- [ ] Compact value representation: 61 bits value | 3 bits tag
- [ ] Memory block types: JS_MTAG_OBJECT, JS_MTAG_STRING, JS_MTAG_FUNCTION_BYTECODE, etc.

### 2.2 Sandbox

| File | Status | Tasks Remaining |
|------|--------|-----------------|
| `sandbox/mtp_js_clone.c` | 🔴 Stub | COW VM cloning |
| `sandbox/mtp_js_gas.c` | ✅ Complete | Gas metering done |
| `sandbox/mtp_js_wipe.c` | 🔴 Stub | Secure memory wipe |

**Acceptance Criteria:**
- [ ] VM cloning using copy-on-write pages
- [ ] Snapshot signature verification before mapping
- [ ] Secure wipe on pages touching PCI-classified data
- [ ] ≤60µs best-case clone, ≤1ms worst-case

### 2.3 Crypto

| File | Status | Tasks Remaining |
|------|--------|-----------------|
| `crypto/mtp_js_seed.c` | 🟡 Partial | `mtp_js_sha256()` forward declared but not implemented |
| `crypto/mtp_js_crypto.c` | 🔴 Stub | SHA-256, ECDSA-P256 implementation |

**Acceptance Criteria:**
- [ ] SHA-256 implementation
- [ ] ECDSA-P256 signature generation and verification
- [ ] Deterministic seed computation per spec

### 2.4 Effects

| File | Status | Tasks Remaining |
|------|--------|-----------------|
| `effects/mtp_js_effects.c` | 🟡 Partial | Registry works, needs integration |
| `effects/mtp_js_db.c` | 🔴 Stub | DbRead/DbWrite handlers |
| `effects/mtp_js_http.c` | 🔴 Stub | HttpOut handler |
| `effects/mtp_js_log.c` | 🔴 Stub | Log handler |
| `effects/mtp_js_async.c` | 🔴 Stub | Deterministic async handler |

**Acceptance Criteria:**
- [ ] Effect injection after context creation
- [ ] DbRead executes SQL SELECT
- [ ] DbWrite executes SQL INSERT/UPDATE/DELETE
- [ ] HttpOut makes outbound HTTP requests
- [ ] Log produces structured log entries
- [ ] Async implements deterministic await with replay cache

### 2.5 Utilities

| File | Status | Tasks Remaining |
|------|--------|-----------------|
| `utils/canonical_json.c` | 🔴 Stub | RFC 8785 canonical JSON |
| `utils/cbor.c` | 🔴 Stub | RFC 7049 §3.9 deterministic CBOR |
| `utils/cutils.c` | 🔴 Stub | Common utilities |
| `utils/dtoa.c` | 🔴 Stub | Float64 printing (Decimal only) |

**Acceptance Criteria:**
- [ ] Canonical JSON: keys sorted, no -0/NaN/Infinity, shortest decimal
- [ ] Duplicate key rejection at parse
- [ ] Deterministic CBOR: shortest-form integers, sorted map keys
- [ ] FNV-1a 64-bit hashing of CBOR

---

## 3. Runtime Host - `runtime/host/`

| File | Status | Tasks Remaining |
|------|--------|-----------------|
| `host/mtp_js_lambda.c` | 🟡 Partial | Full Lambda runtime loop |
| `host/mtp_js_local.c` | 🔴 Stub | Local dev server |

**Acceptance Criteria:**
- [ ] Load and verify snapshot signature
- [ ] Event loop: get invocation → seed → clone → inject → execute → respond → cleanup
- [ ] Gas limit from `MTP_GAS_LIMIT` env var
- [ ] Canonical JSON response (RFC 8785)
- [ ] Audit log entry with responseHash
- [ ] Cold-start ≤2ms worst-case

---

## 4. Security & Audit Posture

**Purpose**: Ensure MTPScript meets compliance requirements for regulated environments.

### Compliance Support
- [ ] SOC 2 compliance
- [ ] SOX compliance
- [ ] ISO 27001 compliance
- [ ] PCI-DSS compliance (with secure wipe)

### Security Properties
- [ ] Explicit authority – All effects declared
- [ ] Deterministic behavior – SHA-256 verifiable
- [ ] Sealed runtime – No ambient I/O
- [ ] Minimal surface – Constrained JS subset

### Reproducible Builds
- [ ] Containerised build image pinned by SHA-256
- [ ] Signed `build-info.json`
- [ ] All dependencies content-hash verified

### Audit Manifest
- [ ] Lists unsafe dependencies with content-hashes
- [ ] Includes snapshot hash, compiler version, gas limit
- [ ] Every request logs: gasLimit, gasUsed, responseHash

---

## 5. Module System

**Purpose**: Static, reproducible module imports with security guarantees.

### Import Rules
- [ ] Static imports only
- [ ] Git-hash pinned dependencies
- [ ] Signed tag required
- [ ] Vendored at build time
- [ ] Order-independent compilation

### Package Manager (`mtpkg`)
- [ ] Git-hash based versioning
- [ ] Git-tag signature verification
- [ ] No runtime network access
- [ ] Produces audit manifest
- [ ] npm bridge via explicit unsafe adapters

### npm Bridging (Unsafe Boundary)
- [ ] Adapters live in `host/unsafe/*.js`
- [ ] Pure functions of arguments + seed
- [ ] Type signature enforced: `function adapterName(seed: Uint8Array, ...args: JsonValue[]): JsonValue`
- [ ] No `require()` inside MTPScript
- [ ] No shared state
- [ ] No exceptions escaping
- [ ] Audit manifest lists every unsafe dependency with content-hash

---

## 6. Standard Library - `src/stdlib/`

| File | Status | Tasks Remaining |
|------|--------|-----------------|
| `prelude.mtp` | 🔴 Stub | Auto-imported definitions |
| `option.mtp` | ✅ Complete | Option<T> type |
| `result.mtp` | ✅ Complete | Result<T, E> type |
| `decimal.mtp` | 🟡 Partial | Function shells exist, implementations TODO |
| `json.mtp` | 🔴 Stub | JSON type and operations |
| `list.mtp` | 🔴 Stub | List operations |
| `string.mtp` | 🔴 Stub | String utilities |

**Acceptance Criteria:**
- [ ] Decimal: IEEE-754-2008 decimal128 compatible
- [ ] Decimal: round-half-even rounding
- [ ] Decimal: constant-time comparison
- [ ] Decimal: overflow returns `Result<Decimal, Overflow>`
- [ ] JSON type matching spec (JsonNull, JsonBool, etc.)
- [ ] List: map, filter, fold, take, etc.

---

## 7. Build System

| File | Status | Tasks Remaining |
|------|--------|-----------------|
| `Makefile` | 🟡 Partial | Complete build rules |
| `scripts/build.sh` | 🔴 Stub | Build automation |
| `scripts/test.sh` | 🔴 Stub | Test runner |
| `scripts/generate_stdlib.sh` | 🔴 Stub | Stdlib header generation |

**Acceptance Criteria:**
- [ ] `make all` builds mtpjs, mtpc, mtpkg
- [ ] `mtpjs` - REPL/runner tool for interactive MTPScript execution
- [ ] `mtpc` - Compiler CLI tool
- [ ] `mtpkg` - Package manager tool
- [ ] `make test` runs all tests
- [ ] Incremental builds with .d dependency files
- [ ] Build artifacts in `build/` directory

---

## 8. Package Manager - `mtpkg`

| Status | Tasks Remaining |
|--------|-----------------|
| 🔴 Not Started | Full implementation |

**Acceptance Criteria:**
- [ ] Git-hash based versioning
- [ ] Git-tag signature verification
- [ ] Vendor dependencies at build time
- [ ] Generate audit manifest
- [ ] mtp.lock file with content hashes

---

## Priority Order

1. **Phase 1 - Compiler Frontend** (Blocks everything)
   - Lexer tokenization
   - Parser implementation
   - Type checker core

2. **Phase 2 - Compiler Backend**
   - Effect inference
   - IR transforms
   - JS code generation

3. **Phase 3 - Core Runtime**
   - mtp_js.c full implementation
   - Gas metering integration
   - Crypto primitives

4. **Phase 4 - Snapshot & Deployment**
   - Bytecode generation
   - Snapshot creation & signing
   - Lambda adapter

5. **Phase 5 - Standard Library & Tooling**
   - Decimal implementation
   - JSON/CBOR utilities
   - Package manager

6. **Phase 6 - Security & Compliance**
   - Security properties implementation
   - Audit manifest generation
   - Reproducible build verification

---

## Legend

- ✅ Complete - Fully implemented
- 🟡 Partial - Structure exists, core logic TODO
- 🔴 Stub - Function shells only, no implementation

---

## 9. Error System

### Typed Error Codes
All errors are typed and deterministic:

```mtp
type ApiError {
  | NotFound(string)
  | Unauthorized
  | ValidationError(List<FieldError>)
  | GasExhausted(number, number)  // limit, used
}
```

### Error Properties
- [ ] No stack traces in production
- [ ] Deterministic error shapes (canonical JSON)
- [ ] All errors flow through `Result<T, E>`

---

## 10. Formal Determinism Claim

> For every MTPScript program P, compiler version C, input byte sequence I, and operator-supplied gasLimit L, the SHA-256 of the canonical JSON response is identical across all conforming runtimes.

This claim holds because:
1. Seed is deterministically computed from inputs
2. All effects are pure functions of seed + arguments
3. Gas limit is bound into the seed
4. Output is canonical JSON (RFC 8785)
5. Response is hashed with SHA-256

---

## 11. Detailed API Specifications

### `core/runtime/mtp_js.h` - Public API Header

**Key Type Definitions:**
- [ ] `JSContext` - Opaque JavaScript execution context
- [ ] `JSValue` - JavaScript value (64-bit tagged)
- [ ] `JSGCRef` - Garbage collector reference holder
- [ ] `JSCFunction` - C function callable from JavaScript
- [ ] `MTPSeed` - 32-byte deterministic seed

**Context Management:**
- [ ] `MTP_NewContext()` - Create context with deterministic seed
- [ ] `MTP_CloneSnapshot()` - Clone from snapshot (COW)
- [ ] `MTP_FreeContext()` - Free context with optional secure wipe

**Effect Injection:**
- [ ] `MTP_InjectEffect()` - Inject effect handlers after context creation

**Gas Metering:**
- [ ] `MTP_GasRemaining()` - Check remaining gas
- [ ] `MTP_GasUsed()` - Get gas used

### `core/runtime/mtp_js.c` - Runtime Engine Core

**Major Components:**
- [ ] Custom allocator within fixed buffer
- [ ] Tracing and compacting garbage collector
- [ ] Memory block tagging system
- [ ] Determinism patches (no double-path, deterministic object ordering, FNV-1a hashing)
- [ ] Gas metering at bytecode level
- [ ] Effect system integration

### `core/utils/canonical_json.c` - Canonical JSON (RFC 8785)

**Serialization Rules:**
- [ ] Object keys sorted by: type tag → hash → CBOR byte-wise
- [ ] No `-0`, no `NaN`, no `Infinity`
- [ ] Decimal in shortest form (no trailing zeros)
- [ ] No duplicate keys (rejected at parse)
- [ ] UTF-8 encoding, no BOM

### `core/utils/cbor.c` - Deterministic CBOR (RFC 7049 §3.9)

**Features:**
- [ ] Shortest-form integer encoding
- [ ] Sorted map keys (length-prefixed, then lexicographic)
- [ ] Consistent type representation
- [ ] FNV-1a 64-bit hashing of CBOR encoding

### `compiler/linker/` - Link-Time Verification

**Linker Phases:**
- [ ] Build global symbol table
- [ ] Verify union exhaustiveness
- [ ] Resolve cross-module references

**Link Errors:**
- [ ] `UnionVariantMismatch` - Two modules reference same union with different variant sets
- [ ] `UnresolvedSymbol` - Import references non-existent export
- [ ] `DuplicateExport` - Same symbol exported from multiple modules
- [ ] `CyclicDependency` - Circular module imports detected

### `core/crypto/mtp_js_seed.c` - Deterministic Seed Generation

**Seed Algorithm:**
```
seed = SHA-256(
  AWS_Request_Id       ||
  AWS_Account_Id       ||
  Function_Version     ||
  "mtpscript-v5.1"     ||
  Snapshot_Content_Hash||
  Gas_Limit_ASCII
)
```

---

## Annexes

### Annex A – Gas Cost Table

Available as machine-readable CSV in `/gas-v5.1.csv`

| Category | Example | Cost (β-reductions) |
|----------|---------|---------------------|
| Arithmetic | `add`, `sub`, `mul` | 1 |
| Comparison | `eq`, `lt`, `gt` | 1 |
| Property access | `get_prop` | 3 |
| Function call | `call` | 5 |
| Tail call | `tail_call` | 0 |
| String concat | `concat` | 2 + len/100 |
| Array access | `get_elem` | 2 |
| Decimal ops | `decimal_add` | 10 |

### Annex B – Deterministic OpenAPI Generation

Available as JSON schema in `/openapi-rules-v5.1.json`

Rules:
1. Fields ordered alphabetically
2. $ref folding for repeated types
3. Deterministic operation IDs
4. No examples in production builds

---

*Last updated: January 6, 2026*

