# Phase 2: Production Readiness & Ecosystem (v5.1)

This phase focuses on completing the MTPScript ecosystem for **production deployment**, including full effect implementations, TypeScript migration tooling, package management, and compliance documentation.

## 1. Full Effect Runtime Implementation (P0)

### 1.1 Database Effects (§7)
- [x] **DbRead Effect**: Implement deterministic SQL query execution with result caching
  - [x] Connection pool management with per-request isolation
  - [x] Query parameterization and SQL injection prevention
  - [x] Result serialization to canonical JSON
  - [x] Cache responses keyed by `(seed, query_hash)` for replay determinism
- [x] **DbWrite Effect**: Implement transactional SQL write operations
  - [x] Atomic transaction support with rollback capability
  - [x] Write operation logging for audit trail
  - [x] Idempotency key support for deterministic retries

### 1.2 HTTP Effect (§7)
- [x] **HttpOut Effect**: Implement outbound HTTP client with determinism guarantees
  - [x] Request serialization and canonical form
  - [x] Response caching keyed by `(seed, request_hash)` per §7-a
  - [x] Timeout handling with deterministic error shapes
  - [x] TLS certificate validation
  - [x] Request/response body size limits

### 1.3 Logging Effect (§7)
- [x] **Log Effect**: Implement structured logging with audit compliance
  - [x] Log levels: `debug`, `info`, `warn`, `error`
  - [x] Structured JSON output per §23
  - [x] Correlation ID injection from request seed
  - [x] Log aggregation interface for CloudWatch/external systems
  - [x] No stack traces in production per §16

## 2. Full API Routing System (P0)

### 2.1 Request Handling (§8)
- [x] **Path Parameter Extraction**: `/users/:id` → `{ id: string }`
- [x] **Query Parameter Parsing**: `?page=1&limit=10` → typed parameters
- [x] **Request Body Parsing**: JSON body deserialization with validation
- [x] **Header Access**: Typed header extraction with case normalization
- [x] **Content-Type Negotiation**: `application/json` enforcement

### 2.2 Response Generation (§8)
- [x] **respond json(...)**: Canonical JSON response serialization
- [x] **respond status(...)**: HTTP status code with typed error bodies
- [x] **Response Headers**: Content-Type, Content-Length, custom headers
- [x] **Error Responses**: Deterministic error shapes per §16

### 2.3 Route Matching
- [x] **Static Routes**: Exact path matching (`/users`, `/health`)
- [x] **Dynamic Routes**: Path parameters (`/users/:id/posts/:postId`)
- [x] **Method Routing**: GET, POST, PUT, DELETE, PATCH dispatch
- [x] **Route Priority**: Most-specific match wins

## 3. TypeScript Migration Tooling (P1)

### 3.1 Migration CLI (§17)
- [x] `mtpsc migrate <file.ts>`: Convert TypeScript to MTPScript
- [x] `mtpsc migrate --dir <dir>`: Batch migration of directories
- [x] `mtpsc migrate --check`: Dry-run with compatibility report

### 3.2 Mechanical Transforms (§17)
- [x] **Type Mapping**: `number` → `number`, `string` → `string`, `boolean` → `boolean`
- [x] **Null Handling**: `null | T` → `Option<T>`, `throws` → `Result<T, E>`
- [x] **Class Removal**: Convert classes to records and functions (basic implementation with manual intervention flags)
- [x] **Loop Conversion**: `for`/`while` → recursive functions (basic implementation with manual intervention flags)
- [x] **Effect Inference**: Detect I/O and annotate with `uses { ... }`
- [x] **Import Rewriting**: npm imports → audit manifest entries (basic implementation with manual intervention flags)
- [x] **Generics**: `T<U>` → parametric types (limited support with compatibility issue flags)
- [x] **Enums**: Convert to union types with content hashing (basic implementation with manual intervention flags)
- [x] **Interface Conversion**: Interfaces → structural records
- [x] **Method Extraction**: Class methods → top-level functions (basic implementation with manual intervention flags)

### 3.3 Migration Reports
- [x] **Compatibility Analysis**: List unsupported TypeScript features
- [x] **Manual Intervention Points**: Flag code requiring human review
- [x] **Effect Suggestions**: Recommend effect declarations based on I/O patterns
- [x] **TypeScript AST Parser**: Parse TypeScript files to AST for migration

### 3.2 Mechanical Transforms (§17)
- [x] **Type Mapping**: `number` → `number`, `string` → `string`, `boolean` → `boolean`
- [x] **Null Handling**: `null | T` → `Option<T>`, `throws` → `Result<T, E>`
- [x] **Class Removal**: Convert classes to records and functions
- [x] **Loop Conversion**: `for`/`while` → recursive functions
- [x] **Effect Inference**: Detect I/O and annotate with `uses { ... }`
- [x] **Import Rewriting**: npm imports → audit manifest entries
- [x] **Generics**: `T<U>` → parametric types (limited support)
- [x] **Enums**: Convert to union types with content hashing
- [x] **Interface Conversion**: Interfaces → structural records
- [x] **Method Extraction**: Class methods → top-level functions

### 3.3 Migration Reports
- [x] **Compatibility Analysis**: List unsupported TypeScript features
- [x] **Manual Intervention Points**: Flag code requiring human review
- [x] **Effect Suggestions**: Recommend effect declarations based on I/O patterns
- [x] **TypeScript AST Parser**: Parse TypeScript files to AST for migration

## 4. Package Manager CLI (P1)

### 4.1 Dependency Management (§11)
- [x] `mtpsc add <package>[@version]`: Add git-pinned dependency
- [x] `mtpsc remove <package>`: Remove dependency
- [x] `mtpsc update <package>`: Update to latest signed tag
- [x] `mtpsc list`: List all dependencies with versions and hashes

### 4.2 Lock File Management
- [x] **mtp.lock**: Deterministic lock file with git hashes and signatures
- [x] **Integrity Verification**: SHA-256 content hash validation
- [x] **Signature Verification**: Git tag signature validation per §10

### 4.3 Vendoring System (§10)
- [x] **vendor/**: Local copy of all dependencies
- [x] **Offline Builds**: No network access required after vendor
- [x] **Audit Manifest Generation**: `audit-manifest.json` with content hashes

### 4.4 npm Bridge CLI (§21)
- [x] `mtpsc npm-bridge <package>`: Generate unsafe adapter wrapper
- [x] **Adapter Template**: Generate `host/unsafe/<package>.js` skeleton
- [x] **Type Signature Enforcement**: Validate `(seed, ...args) => JsonValue` contract
- [x] **Audit Manifest Update**: Auto-add to `unsafeDeps` list

## 5. Production AWS Lambda Deployment (P1)

### 5.1 Custom Runtime Packaging (§14)
- [x] **Native Binary Build**: Statically linked `bootstrap` executable
- [x] **Lambda Layer**: Reusable layer with MicroQuickJS runtime
- [x] **Deployment Package**: `app.msqs` + `app.msqs.sig` + certificate

### 5.2 Infrastructure Templates
- [x] **SAM Template**: AWS SAM `template.yaml` for deployment
- [x] **CDK Construct**: AWS CDK construct for MTPScript functions
- [x] **Terraform Module**: Terraform module for MTPScript deployment

### 5.3 Cold Start Optimization (§14)
- [x] **Provisioned Concurrency**: Configuration for warm starts
- [x] **EFS Integration**: Snapshot storage on EFS with page fault handling
- [x] **Memory Tuning**: Optimal memory/CPU allocation recommendations

## 6. Annex Files & Documentation (P1)

### 6.1 Gas Cost Table (Annex A)
- [x] Create `/gas-v5.1.csv` with all opcode and built-in costs
  - Format: `opcode,name,cost_beta_units,category`
  - Include all IR opcodes
  - Include all built-in function costs
  - Document tail call 0-cost exception

### 6.2 OpenAPI Generation Rules (Annex B)
- [x] Create `/openapi-rules-v5.1.json` schema
  - Deterministic field ordering rules
  - `$ref` folding algorithm
  - Schema deduplication rules
  - Path parameter ordering

### 6.3 Compliance Documentation (§18)
- [x] **SOC 2 Mapping**: Control mapping document
- [x] **SOX Compliance**: Financial control attestation guide
- [x] **ISO 27001**: Information security controls mapping
- [x] **PCI-DSS**: Payment card data handling controls

## 7. Union Exhaustiveness Checking (P1)

### 7.1 Content Hashing (§24)
- [x] **Union Type Content Hashing**: Generate SHA-256 hash of variant list for each union type
- [x] **Link-Time Verification**: Fail compilation if any unit sees different variant sets
- [x] **Union ADT Definition**: Extend type system to support union types with exhaustive checking

### 7.2 Exhaustive Match Enforcement (§24)
- [x] **Compile-Time Exhaustiveness**: Verify all union variants covered in match expressions
- [x] **Link-Time Guarantees**: Runtime checks not needed due to link-time verification
- [x] **Pattern Matching Infrastructure**: Support destructuring patterns for union variants

## 8. Full HTTP Server Syntax & Support (P1)

### 8.1 Server Declaration Parsing (§15, §20)
- [x] Parse `serve { port: 8080, routes: [...] }` MTPScript syntax
- [x] Route configuration with path patterns and handlers
- [x] Server configuration options (port, host, timeouts)
- [x] Hot reload on source file changes with snapshot recompilation

### 8.2 Server Runtime Implementation (§20)
- [x] **Snapshot-Clone Isolation**: Same per-request VM cloning as Lambda runtime
- [x] **Not User-Programmable**: Server is reference implementation only
- [x] **Development Tools**: Request logging, error handling, debugging support

## 9. Pipeline Operator Associativity Verification (P1)

### 9.1 Left-Associative Generation (§25)
- [x] **α-Equivalent JS Output**: Ensure `a |> b |> c ≡ (a |> b) |> c` generates identical JS across compilers
- [x] **Deterministic Code Generation**: Pipeline lowering produces consistent AST structure
- [x] **Test Coverage**: Comprehensive tests for associativity edge cases

### 9.2 Local Development Server (P2)

## 10. Cross-Platform Testing & CI/CD (P2)

### 10.1 Platform Matrix
- [x] **Linux x86_64**: Primary CI target
- [x] **Linux ARM64**: AWS Graviton support
- [x] **macOS x86_64**: Development support
- [x] **macOS ARM64 (Apple Silicon)**: Development support

### 10.2 Determinism Verification
- [x] **Cross-Platform SHA-256 Tests**: Verify identical output hashes
- [x] **Endianness Tests**: Verify big/little endian consistency
- [x] **Floating-Point Absence Tests**: Verify no FP operations leak through

### 10.3 CI/CD Pipeline
- [x] **GitHub Actions**: Multi-platform build and test
- [x] **Release Automation**: Signed binary releases
- [x] **Reproducible Build Verification**: Hash comparison across builds

## 11. Performance & Benchmarking (P2)

### 11.1 Benchmarks
- [x] **VM Clone Time**: Measure and optimize `clone_vm()` performance
- [x] **Request Throughput**: Requests/second under load
- [x] **Memory Usage**: Per-request memory consumption
- [x] **Gas Metering Overhead**: Cost of gas counting

### 11.2 Profiling Tools
- [x] `mtpsc profile <file.mtp>`: Gas consumption profile
- [x] `mtpsc benchmark <file.mtp>`: Performance benchmark
- [x] Memory allocation tracking

## 12. Language Server Protocol (P2)

### 12.1 LSP Implementation
- [x] **Diagnostics**: Real-time error reporting
- [x] **Completion**: Auto-complete for types, functions, effects
- [x] **Hover**: Type information on hover
- [x] **Go to Definition**: Navigate to declarations
- [x] **Find References**: Find all usages

### 12.2 Editor Extensions
- [x] **VS Code Extension**: Syntax highlighting + LSP client
- [x] **Cursor Extension**: Native integration
- [x] **Syntax Grammar**: TextMate grammar for `.mtp` files

## 13. Formal Determinism Verification (P1)

### 13.1 Determinism Claim Testing (§26)
- [x] **Response SHA-256 Verification**: Verify identical SHA-256 hashes across conforming runtimes
- [x] **Canonical JSON Compliance**: Ensure all output follows RFC 8785 with duplicate-key rejection
- [x] **Seed Algorithm Validation**: Test deterministic seed generation per §0-b (updated by §0-c with gas limit)
- [x] **CBOR Determinism**: Verify RFC 7049 §3.9 compliance for all serialization
- [x] **Gas Limit Determinism**: Verify identical responses for same program, input, and gasLimit L

### 13.2 Cross-Runtime Testing Infrastructure (§26)
- [x] **Runtime Conformance Suite**: Test programs against multiple runtime implementations
- [x] **Deterministic Replay Testing**: Verify request/response determinism across platforms
- [x] **Gas Limit Determinism**: Ensure identical gas exhaustion behavior

## 14. Advanced Security & Audit Features (P1)

### 14.1 VM Snapshot Security (§22)
- [x] **Secure Memory Wipe**: Selective wipe of pages containing PCI-classified data
- [x] **Zero Cross-Request Leakage**: Guaranteed memory isolation between requests
- [x] **Snapshot Lifecycle Audit**: Complete audit trail from build to execution

### 14.2 Audit Trail Implementation (§18)
- [x] **Request Audit Logging**: All requests logged with deterministic correlation IDs
- [x] **Gas Usage Audit**: Gas consumption logged for every request with gasLimit field (§0-c.5)
- [x] **Effect Usage Tracking**: Runtime verification of declared vs actual effects
- [x] **OpenAPI Audit Schema**: Every request log includes gasLimit field in audit stream

### 14.3 Regulatory Compliance (§18)
- [x] **SOC 2 Controls**: Security, availability, and confidentiality controls
- [x] **SOX Compliance**: Financial reporting controls and audit trails
- [x] **ISO 27001**: Information security management system
- [x] **PCI-DSS**: Payment card industry data security standards

## 15. Build Info & Signing Infrastructure (P1)

### 15.1 Containerized Build Environment (§18)
- [x] **Dockerfile**: Reproducible build container pinned by SHA-256
- [x] **Build Info Generation**: `build-info.json` with all build artifacts and hashes
- [x] **Build Signing**: ECDSA-P256 signature of build-info.json

### 15.2 Reproducible Builds (§18)
- [x] **Deterministic Compilation**: Identical binaries from identical source + environment
- [x] **Source Code Verification**: Git hash inclusion in build-info.json
- [x] **Dependency Pinning**: All build dependencies version-pinned and hashed

### 15.3 Runtime Verification (§22)
- [x] **Snapshot Signature Verification**: ECDSA signature validation before mapping
- [x] **Build Info Audit**: Runtime verification of build provenance
- [x] **Certificate Management**: Embedded certificate validation chain

## Acceptance Criteria (v5.1)

### P0 Requirements (Must Have)
- [x] All four built-in effects (DbRead, DbWrite, HttpOut, Log) fully implemented
- [x] API routing handles path params, query params, and request bodies
- [x] Effect implementations cache responses for deterministic replay
- [x] All effects produce canonical JSON output per §23

### P1 Requirements (Should Have)
- [x] `mtpsc migrate` converts basic TypeScript files to MTPScript with full mechanical transforms and reports
- [x] Package manager can add/remove/update git-pinned dependencies
- [x] AWS Lambda deployment works with provided templates
- [x] `/gas-v5.1.csv` and `/openapi-rules-v5.1.json` exist and are valid
- [x] Basic compliance documentation available

### P2 Requirements (Nice to Have)
- [x] Hot reload in development server
- [x] Cross-platform CI/CD with determinism verification
- [x] Performance benchmarks establish baselines
- [x] LSP provides basic IDE support
- [x] VS Code extension with syntax highlighting + LSP client
- [x] Cursor extension with native integration
- [x] TextMate grammar for .mtp files

### Test Coverage
- [x] Integration tests for all effect implementations
- [x] End-to-end tests for API routing
- [x] Migration tests with TypeScript fixture files
- [x] Cross-platform determinism tests in CI
- [x] Union exhaustiveness checking tests
- [x] HTTP server syntax parsing tests
- [x] Pipeline associativity verification tests
- [x] Formal determinism claim validation tests
- [x] Hot reload functionality tests
- [x] LSP server functionality tests
- [x] VS Code extension file structure tests
- [x] Cursor extension file structure tests
- [x] TextMate grammar content tests

---

## Priority Order

1. **P0 - Critical**: Effect implementations and API routing (blocks production use)
2. **P1 - Important**: Migration tools, package manager, Lambda deployment, documentation
3. **P2 - Desirable**: Dev server improvements, CI/CD, performance, LSP

## Dependencies

- Phase 0 & Phase 1 must be complete (verified ✅)
- Database driver selection (PostgreSQL recommended for determinism)
- HTTP client library selection (libcurl or custom minimal client)
- AWS account access for Lambda testing

