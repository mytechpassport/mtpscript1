# MTPScript Detailed Implementation Task Breakdown

## Implementation Strategy: Runtime-First Approach

**Priority**: Runtime foundation first, as it provides the execution environment needed to validate all other phases. All compiled artifacts will be placed in the `build/` folder per specification.

---

## Phase 0: Project Structure Setup (Week 1)
**Reference**: `FOLDER_STRUCTURE.md` lines 8-26

### Task 0.1: Create Complete Directory Structure
**Files**: All directories per `FOLDER_STRUCTURE.md`
**Build Output**: `build/` directory structure

**Implementation Steps**:
```bash
# Create core directories
mkdir -p build/{artifacts,ci,docker,generated,objects,templates}
mkdir -p core/{crypto,db,effects,http,runtime,stdlib,utils}
mkdir -p src/{cli,compiler,decimal,effects,host,lsp,main,snapshot,stdlib}
mkdir -p tests/{unit,integration,fixtures,executables,migration}
mkdir -p scripts
mkdir -p pkg/{decimal,lsp,readline}
mkdir -p tools/{bench,dev}
mkdir -p runtime/{gas,host,snapshot}
mkdir -p compiler/{analysis,backend,cli,frontend,tools}
mkdir -p examples/{basic_commandline,basic_crud,hashing,long_running_loop,third_party_api_call}
mkdir -p extensions/{vscode,cursor}
mkdir -p compliance
mkdir -p docs/{api,compliance,marketing,requirements}
mkdir -p marketing
mkdir -p vendor
```

**Test to Create**: `tests/unit/structure_test.c`
```c
// Test: Verify all required directories exist
int test_directory_structure() {
    assert(dir_exists("build/artifacts"));
    assert(dir_exists("core/runtime"));
    assert(dir_exists("src/compiler"));
    // ... verify all directories
    return PASS;
}
```

**Specification Reference**: `FOLDER_STRUCTURE.md` lines 8-26 (root level structure)

---

### Task 0.2: Build System Configuration
**Files**: `Makefile`, `build/templates/Makefile.template`, `Dockerfile`

**Implementation Pseudocode**:
```makefile
# Makefile structure
CC = gcc
CFLAGS = -O2 -Wall -Wextra -std=c11
BUILD_DIR = build
SRC_DIR = src
CORE_DIR = core

# All targets build to build/ directory
all: $(BUILD_DIR)/mtpsc $(BUILD_DIR)/mtpjs $(BUILD_DIR)/libmtp.a

$(BUILD_DIR)/mtpsc: $(SRC_DIR)/cli/mtpsc.c
	$(CC) $(CFLAGS) -o $@ $<

$(BUILD_DIR)/mtpjs: $(CORE_DIR)/runtime/mtp_js.c
	$(CC) $(CFLAGS) -o $@ $<
```

**Test to Create**: `tests/unit/build_test.c`
```c
int test_build_system() {
    // Test that make produces correct outputs in build/
    assert(system("make") == 0);
    assert(file_exists("build/mtpsc"));
    assert(file_exists("build/mtpjs"));
    return PASS;
}
```

**Specification Reference**: `FOLDER_STRUCTURE.md` lines 158-187 (file placement guidelines)

---

## Phase R: Runtime Foundation (Weeks 2-5)

### Task R.1: MTPJS Integration Layer
**Files**: `core/runtime/mtp_js.c`, `core/runtime/mtp_js.h`
**Build Output**: `build/libmtpjs.a`, `build/mtpjs`

**Specification References**:
- Lines 9-17: JavaScript as execution encoding
- Lines 346-347: Forbidden JS constructs
- Lines 347: MTPJS patched for double-path integers

**Implementation Pseudocode**:
```c
// core/runtime/mtp_js.c
#include "mtpjs.h"
#include "mtp_js.h"

// Initialize MTPJS with security patches
JSRuntime* mtp_js_init_runtime() {
    JSRuntime *rt = JS_NewRuntime();
    // Patch MTPJS to forbid forbidden constructs
    js_patch_forbidden_constructs(rt);
    // Patch double-path for integers > 2^53-1 (line 347)
    js_patch_double_path_integers(rt);
    return rt;
}

// Create VM with memory constraints (line 355)
JSContext* mtp_js_create_context(JSRuntime *rt) {
    JSContext *ctx = JS_NewContext(rt);
    // Set fixed memory budget, no shared heap
    JS_SetMemoryLimit(ctx, MTP_MEMORY_BUDGET);
    return ctx;
}

// Execute JavaScript with security checks
int mtp_js_execute(JSContext *ctx, const char *code) {
    // Check for forbidden constructs (line 346)
    if (contains_forbidden_js(code)) {
        return MTP_ERROR_FORBIDDEN_CONSTRUCT;
    }
    // Execute with gas metering
    return js_execute_with_gas(ctx, code);
}
```

**Security Patches Required**:
```c
// Forbidden JS list (line 346)
static const char* forbidden_constructs[] = {
    "eval", "class", "this", "try", "catch", "for", "while", "do"
};

// Patch to prevent double-path for integers > 2^53-1 (line 347)
void js_patch_double_path_integers(JSRuntime *rt) {
    // Override integer handling to prevent double-path
    // Implementation detail: modify MTPJS internal number handling
}
```

**Test to Create**: `tests/runtime/mtpjs_security_test.c`
```c
int test_forbidden_js_blocked() {
    JSContext *ctx = mtp_js_create_context();
    // Test eval is blocked
    assert(mtp_js_execute(ctx, "eval('1+1')") == MTP_ERROR_FORBIDDEN_CONSTRUCT);
    // Test class is blocked
    assert(mtp_js_execute(ctx, "class X {}") == MTP_ERROR_FORBIDDEN_CONSTRUCT);
    // Test this is blocked
    assert(mtp_js_execute(ctx, "function f() { return this; }") == MTP_ERROR_FORBIDDEN_CONSTRUCT);
    return PASS;
}

int test_double_path_integer_fix() {
    JSContext *ctx = mtp_js_create_context();
    // Test integers > 2^53-1 work correctly
    assert(mtp_js_execute(ctx, "9007199254740993") == MTP_SUCCESS);
    return PASS;
}
```

---

### Task R.2: VM Snapshot System
**Files**: `runtime/snapshot/snapshot.c`, `runtime/snapshot/snapshot.h`
**Build Output**: `build/libsnapshot.a`, `build/snapshot_tool`

**Specification References**:
- Lines 25-28: Execution-isolation model
- Lines 434-440: VM snapshot lifecycle
- Lines 365: ECDSA signature verification

**Implementation Pseudocode**:
```c
// runtime/snapshot/snapshot.c
#include "snapshot.h"

// Create snapshot from compiled bytecode (line 430)
int create_snapshot(const char *input_file, const char *output_file) {
    // Load compiled MTPScript bytecode
    uint8_t *bytecode = load_bytecode(input_file);
    size_t bytecode_size = get_bytecode_size(input_file);

    // Create .msqs format (150-400 kB, line 25)
    Snapshot *snapshot = malloc(sizeof(Snapshot));
    snapshot->magic = MTP_SNAPSHOT_MAGIC;
    snapshot->version = MTP_VERSION_5_1;
    snapshot->bytecode = bytecode;
    snapshot->bytecode_size = bytecode_size;

    // Sign with ECDSA-P256 (line 431)
    if (sign_snapshot_ecdsa(snapshot) != 0) {
        return MTP_ERROR_SIGNATURE_FAILED;
    }

    // Write to file
    return write_snapshot_file(snapshot, output_file);
}

// Clone VM with copy-on-write (line 26, ≤ 1ms requirement)
JSContext* clone_vm(Snapshot *snapshot) {
    struct timespec start, end;
    clock_gettime(CLOCK_MONOTONIC, &start);

    // Map snapshot read-only
    void *mapped = mmap_snapshot(snapshot);

    // Copy-on-write VM clone
    JSContext *ctx = js_clone_context(mapped);

    clock_gettime(CLOCK_MONOTONIC, &end);
    long duration_ms = (end.tv_sec - start.tv_sec) * 1000 +
                      (end.tv_nsec - start.tv_nsec) / 1000000;

    // Verify ≤ 1ms requirement (line 26)
    if (duration_ms > 1) {
        log_warning("VM clone took %ld ms (exceeds 1ms requirement)", duration_ms);
    }

    return ctx;
}

// Verify ECDSA-P256 signature (line 434)
int verify_snapshot_signature(const char *snapshot_file) {
    Snapshot *snapshot = read_snapshot_file(snapshot_file);

    // Load embedded certificate
    ECDSA_KEY *cert = load_embedded_certificate();

    // Verify signature
    int result = ecdsa_verify(cert, snapshot->signature,
                             snapshot->data, snapshot->data_size);

    if (result != 1) {
        // Abort on mismatch (line 434)
        return MTP_ERROR_SIGNATURE_MISMATCH;
    }

    return MTP_SUCCESS;
}

// Secure wipe on sensitive pages (line 27, 439)
void secure_wipe_sensitive_pages(JSContext *ctx) {
    // Identify pages that touched PCI-classified data
    void **sensitive_pages = get_sensitive_pages(ctx);

    // Perform secure wipe
    for (int i = 0; sensitive_pages[i] != NULL; i++) {
        secure_memzero(sensitive_pages[i], PAGE_SIZE);
    }
}
```

**Test to Create**: `tests/runtime/snapshot_test.c`
```c
int test_snapshot_creation() {
    // Test snapshot creation produces 150-400 kB file
    assert(create_snapshot("test.bc", "test.msqs") == MTP_SUCCESS);
    size_t size = file_size("test.msqs");
    assert(size >= 150000 && size <= 400000);
    return PASS;
}

int test_vm_clone_performance() {
    Snapshot *snapshot = load_snapshot("test.msqs");

    // Test VM cloning meets ≤ 1ms requirement
    struct timespec start, end;
    clock_gettime(CLOCK_MONOTONIC, &start);

    JSContext *ctx = clone_vm(snapshot);

    clock_gettime(CLOCK_MONOTONIC, &end);
    long duration_ms = (end.tv_sec - start.tv_sec) * 1000 +
                      (end.tv_nsec - start.tv_nsec) / 1000000;

    assert(duration_ms <= 1);
    return PASS;
}

int test_signature_verification() {
    // Test valid signature passes
    assert(verify_snapshot_signature("valid.msqs") == MTP_SUCCESS);

    // Test invalid signature fails
    assert(verify_snapshot_signature("invalid.msqs") == MTP_ERROR_SIGNATURE_MISMATCH);

    return PASS;
}
```

---

### Task R.3: Gas Metering Engine
**Files**: `runtime/gas/gas.c`, `runtime/gas/gas.h`, `gas_costs.h`
**Build Output**: `build/libgas.a`

**Specification References**:
- Lines 52-114: Runtime gas limit injection
- Lines 79-89: Gas exhaustion semantics
- Lines 91-94: Gas cost table (Annex A)
- Lines 218-219: Gas bounded recursion

**Implementation Pseudocode**:
```c
// runtime/gas/gas.c
#include "gas.h"
#include "gas_costs.h"

// Gas cost table (Annex A, line 477)
static const gas_cost_t gas_costs[] = {
    {OP_ADD, 10},
    {OP_MULTIPLY, 20},
    {OP_FUNCTION_CALL, 100},
    {OP_DB_READ, 1000},
    {OP_HTTP_OUT, 2000},
    // ... all operations
};

// Initialize gas limit from environment (lines 72-74)
uint64_t initialize_gas_limit() {
    const char *gas_limit_str = getenv("MTP_GAS_LIMIT");

    if (gas_limit_str == NULL) {
        return 10000000; // Default = 10,000,000 (line 73)
    }

    uint64_t gas_limit = strtoull(gas_limit_str, NULL, 10);

    // Validate range 1-2,000,000,000 (line 74)
    if (gas_limit < 1 || gas_limit > 2000000000) {
        // Abort with MTPError: GasLimitOutOfRange (line 74)
        exit(MTP_ERROR_GAS_LIMIT_OUT_OF_RANGE);
    }

    return gas_limit;
}

// Inject gas limit into VM (line 75)
void inject_gas_limit(JSContext *ctx, uint64_t gas_limit) {
    // Write gas limit into VM's internal gasLimit word (line 75)
    JS_SetGasLimit(ctx, gas_limit);

    // Append to audit log (line 76)
    audit_log_append("gasLimit=%llu", gas_limit);
}

// Consume gas for operation
int consume_gas(JSContext *ctx, opcode_t op) {
    uint64_t cost = get_gas_cost(op);
    uint64_t current_gas = JS_GetCurrentGas(ctx);
    uint64_t gas_limit = JS_GetGasLimit(ctx);

    if (current_gas + cost > gas_limit) {
        // Gas exhausted - terminate with deterministic error (lines 81-87)
        return terminate_gas_exhausted(ctx, gas_limit, current_gas);
    }

    JS_AddGasConsumed(ctx, cost);
    return MTP_SUCCESS;
}

// Terminate with gas exhaustion error (lines 81-87)
int terminate_gas_exhausted(JSContext *ctx, uint64_t gas_limit, uint64_t gas_used) {
    // Create deterministic error response
    char error_response[256];
    snprintf(error_response, sizeof(error_response),
             "{ \"error\": \"GasExhausted\", \"gasLimit\": %llu, \"gasUsed\": %llu }",
             gas_limit, gas_used);

    // No stack trace in production (line 89)
    JS_SetReturnValue(ctx, error_response);

    // Response body hashed into SHA-256 (line 89)
    uint8_t response_hash[32];
    sha256(error_response, strlen(error_response), response_hash);

    return MTP_ERROR_GAS_EXHAUSTED;
}

// Get gas cost for operation (Annex A)
uint64_t get_gas_cost(opcode_t op) {
    for (int i = 0; i < sizeof(gas_costs) / sizeof(gas_costs[0]); i++) {
        if (gas_costs[i].opcode == op) {
            return gas_costs[i].cost;
        }
    }
    return 0; // Tail calls cost 0 (line 219)
}
```

**Test to Create**: `tests/runtime/gas_metering_test.c`
```c
int test_gas_limit_injection() {
    JSContext *ctx = mtp_js_create_context();

    // Test default gas limit
    setenv("MTP_GAS_LIMIT", "", 1); // Clear environment
    uint64_t gas_limit = initialize_gas_limit();
    assert(gas_limit == 10000000);

    // Test custom gas limit
    setenv("MTP_GAS_LIMIT", "5000000", 1);
    gas_limit = initialize_gas_limit();
    assert(gas_limit == 5000000);

    // Test out of range gas limit
    setenv("MTP_GAS_LIMIT", "3000000000", 1);
    // Should abort with MTPError: GasLimitOutOfRange
    assert(initialize_gas_limit() == MTP_ERROR_GAS_LIMIT_OUT_OF_RANGE);

    return PASS;
}

int test_gas_exhaustion() {
    JSContext *ctx = mtp_js_create_context();
    inject_gas_limit(ctx, 1000); // Very low gas limit

    // Execute code that consumes more than 1000 gas
    const char *code = "for(let i=0; i<1000; i++) { i + i; }";
    int result = mtp_js_execute(ctx, code);

    // Should terminate with GasExhausted error
    assert(result == MTP_ERROR_GAS_EXHAUSTED);

    // Check error response format
    const char *response = JS_GetReturnValue(ctx);
    assert(strstr(response, "\"error\": \"GasExhausted\"") != NULL);
    assert(strstr(response, "\"gasLimit\": 1000") != NULL);

    return PASS;
}
```

---

### Task R.4: Deterministic Seed System
**Files**: `runtime/host/seed.c`, `runtime/host/seed.h`
**Build Output**: `build/libseed.a`

**Specification References**:
- Lines 34-66: Deterministic seed algorithm
- Lines 47-48: Seed uniqueness requirements

**Implementation Pseudocode**:
```c
// runtime/host/seed.c
#include "seed.h"

// Generate deterministic seed (lines 58-66)
int generate_deterministic_seed(const char *aws_request_id,
                                const char *aws_account_id,
                                const char *function_version,
                                const char *snapshot_content_hash,
                                uint64_t gas_limit,
                                uint8_t seed[32]) {
    SHA256_CTX ctx;
    SHA256_Init(&ctx);

    // Concatenate all seed components (line 58-65)
    SHA256_Update(&ctx, aws_request_id, strlen(aws_request_id));
    SHA256_Update(&ctx, aws_account_id, strlen(aws_account_id));
    SHA256_Update(&ctx, function_version, strlen(function_version));
    SHA256_Update(&ctx, "mtpscript-v5.1", 14); // Literal constant (line 62)
    SHA256_Update(&ctx, snapshot_content_hash, strlen(snapshot_content_hash));

    // Convert gas_limit to ASCII decimal (line 64)
    char gas_limit_ascii[21];
    snprintf(gas_limit_ascii, sizeof(gas_limit_ascii), "%llu", gas_limit);
    SHA256_Update(&ctx, gas_limit_ascii, strlen(gas_limit_ascii));

    // Generate 32-byte seed
    SHA256_Final(seed, &ctx);

    return MTP_SUCCESS;
}

// Verify seed uniqueness (line 48)
int verify_seed_uniqueness(const uint8_t *seed, const seed_history_t *history) {
    // Check seed was never reused across requests
    for (int i = 0; i < history->count; i++) {
        if (memcmp(seed, history->seeds[i], 32) == 0) {
            return MTP_ERROR_SEED_REUSE;
        }
    }
    return MTP_SUCCESS;
}

// Inject seed into VM
void inject_seed_into_vm(JSContext *ctx, const uint8_t seed[32]) {
    // Set seed for deterministic randomness
    JS_SetDeterministicSeed(ctx, seed);

    // Log seed for audit purposes
    char seed_hex[65];
    bytes_to_hex(seed, 32, seed_hex);
    audit_log_append("seed=%s", seed_hex);
}
```

**Test to Create**: `tests/runtime/seed_test.c`
```c
int test_deterministic_seed_generation() {
    const char *aws_request_id = "test-request-123";
    const char *aws_account_id = "123456789012";
    const char *function_version = "1";
    const char *snapshot_hash = "abcd1234";
    uint64_t gas_limit = 10000000;

    uint8_t seed1[32], seed2[32];

    // Generate seed twice with same inputs
    generate_deterministic_seed(aws_request_id, aws_account_id,
                                function_version, snapshot_hash,
                                gas_limit, seed1);
    generate_deterministic_seed(aws_request_id, aws_account_id,
                                function_version, snapshot_hash,
                                gas_limit, seed2);

    // Seeds should be identical (deterministic)
    assert(memcmp(seed1, seed2, 32) == 0);

    // Different gas limit should produce different seed
    generate_deterministic_seed(aws_request_id, aws_account_id,
                                function_version, snapshot_hash,
                                20000000, seed2);
    assert(memcmp(seed1, seed2, 32) != 0);

    return PASS;
}

int test_seed_uniqueness() {
    seed_history_t history = {0};
    uint8_t seed1[32] = {1, 2, 3, /* ... */};
    uint8_t seed2[32] = {1, 2, 3, /* ... */}; // Same seed

    // Add first seed to history
    history.seeds[history.count++] = seed1;

    // Verify unique seed passes
    assert(verify_seed_uniqueness(seed1, &history) == MTP_SUCCESS);

    // Verify reused seed fails
    assert(verify_seed_uniqueness(seed2, &history) == MTP_ERROR_SEED_REUSE);

    return PASS;
}
```

---

### Task R.5: Effect Host Framework
**Files**: `core/effects/mtp_js_effects.c`, `core/crypto/mtp_js_crypto.c`, `core/db/mtp_js_db.c`, `core/http/mtp_js_http.c`, `core/effects/mtp_js_log.c`
**Build Output**: `build/libeffects.a`

**Specification References**:
- Lines 223-237: Effect system authority model
- Lines 241-276: Async effect implementation
- Lines 27-28: Host effects re-injection per VM

**Implementation Pseudocode**:
```c
// core/effects/mtp_js_effects.c
#include "mtp_js_effects.h"

// Effect dispatch table
static const effect_handler_t effect_handlers[] = {
    {"DbRead", handle_db_read},
    {"DbWrite", handle_db_write},
    {"HttpOut", handle_http_out},
    {"Log", handle_log},
    {"Async", handle_async},
    {NULL, NULL}
};

// Dispatch effect call (deterministic based on seed)
JSValue dispatch_effect(JSContext *ctx, const char *effect_name,
                        JSValue args, const uint8_t seed[32]) {
    // Find effect handler
    for (int i = 0; effect_handlers[i].name != NULL; i++) {
        if (strcmp(effect_handlers[i].name, effect_name) == 0) {
            // Call effect handler with seed for determinism
            return effect_handlers[i].handler(ctx, args, seed);
        }
    }

    return JS_ThrowTypeError(ctx, "Unknown effect: %s", effect_name);
}

// Database read effect (deterministic)
JSValue handle_db_read(JSContext *ctx, JSValue args, const uint8_t seed[32]) {
    // Extract SQL query from args
    char *sql_query = js_to_string(ctx, JS_GetPropertyStr(ctx, args, "sql"));

    // Create deterministic cache key from seed + query
    char cache_key[256];
    snprintf(cache_key, sizeof(cache_key), "db_read_%s_%s",
             seed_to_hex(seed), sql_query);

    // Check cache first (deterministic behavior)
    JSValue cached_result = effect_cache_get(cache_key);
    if (!JS_IsNull(cached_result)) {
        return cached_result;
    }

    // Execute SQL query deterministically
    JSValue result = execute_sql_deterministic(ctx, sql_query, seed);

    // Cache result for future calls
    effect_cache_put(cache_key, result);

    return result;
}

// HTTP out effect (deterministic)
JSValue handle_http_out(JSContext *ctx, JSValue args, const uint8_t seed[32]) {
    // Extract URL and method from args
    char *url = js_to_string(ctx, JS_GetPropertyStr(ctx, args, "url"));
    char *method = js_to_string(ctx, JS_GetPropertyStr(ctx, args, "method"));

    // Create deterministic cache key (line 273)
    char cache_key[512];
    snprintf(cache_key, sizeof(cache_key), "http_%s_%s_%s",
             method, seed_to_hex(seed), url);

    // Check cache (deterministic behavior)
    JSValue cached_response = effect_cache_get(cache_key);
    if (!JS_IsNull(cached_response)) {
        return cached_response;
    }

    // Block-synchronous execution (line 272)
    JSValue response = execute_http_request_deterministic(ctx, method, url, seed);

    // Cache response bytes keyed by (seed, contId) (line 273)
    effect_cache_put(cache_key, response);

    return response;
}

// Async effect implementation (lines 241-276)
JSValue handle_async(JSContext *ctx, JSValue args, const uint8_t seed[32]) {
    // Extract promiseHash, contId, effectArgs (line 244)
    char *promise_hash = js_to_string(ctx, JS_GetPropertyStr(ctx, args, "promiseHash"));
    int cont_id = js_to_int(ctx, JS_GetPropertyStr(ctx, args, "contId"));
    JSValue effect_args = JS_GetPropertyStr(ctx, args, "effectArgs");

    // Create cache key from (seed, contId) (line 273)
    char cache_key[256];
    snprintf(cache_key, sizeof(cache_key), "async_%s_%d",
             seed_to_hex(seed), cont_id);

    // Check cache for identical response (line 274)
    JSValue cached_result = effect_cache_get(cache_key);
    if (!JS_IsNull(cached_result)) {
        return cached_result;
    }

    // Execute async I/O block-synchronously (line 272)
    JSValue result = execute_async_io_deterministic(ctx, effect_args, seed);

    // Cache for replay (line 273)
    effect_cache_put(cache_key, result);

    return result;
}
```

**Test to Create**: `tests/runtime/effects_test.c`
```c
int test_effect_determinism() {
    JSContext *ctx = mtp_js_create_context();
    uint8_t seed[32] = {1, 2, 3, /* ... */};

    // Call same effect twice with same seed
    JSValue args1 = js_parse_json(ctx, "{\"sql\": \"SELECT * FROM users\"}");
    JSValue result1 = handle_db_read(ctx, args1, seed);

    JSValue args2 = js_parse_json(ctx, "{\"sql\": \"SELECT * FROM users\"}");
    JSValue result2 = handle_db_read(ctx, args2, seed);

    // Results should be identical (deterministic)
    assert(js_values_equal(result1, result2));

    return PASS;
}

int test_async_effect_caching() {
    JSContext *ctx = mtp_js_create_context();
    uint8_t seed[32] = {1, 2, 3, /* ... */};

    // Call async effect with same contId
    JSValue args = js_parse_json(ctx, "{\"url\": \"https://api.example.com\"}");
    JSValue async_args = JS_NewObject(ctx);
    JS_SetPropertyStr(ctx, async_args, "promiseHash", JS_NewString(ctx, "hash123"));
    JS_SetPropertyStr(ctx, async_args, "contId", JS_NewInt32(ctx, 1));
    JS_SetPropertyStr(ctx, async_args, "effectArgs", args);

    JSValue result1 = handle_async(ctx, async_args, seed);
    JSValue result2 = handle_async(ctx, async_args, seed);

    // Should return identical cached response
    assert(js_values_equal(result1, result2));

    return PASS;
}
```

---

### Task R.6: Security Isolation Mechanism
**Files**: `runtime/host/isolation.c`, `runtime/host/isolation.h`
**Build Output**: `build/libisolation.a`

**Specification References**:
- Lines 11-15: Zero ambient authority, zero hidden I/O, zero cross-request state
- Lines 25-28: Per-request sandbox isolation
- Lines 355-356: Fixed memory budget, VM discarded after response

**Implementation Pseudocode**:
```c
// runtime/host/isolation.c
#include "isolation.h"

// Enforce zero ambient authority (line 11)
int enforce_zero_ambient_authority(JSContext *ctx) {
    // Disable all ambient capabilities
    js_disable_file_system(ctx);
    js_disable_network_access(ctx);
    js_disable_environment_variables(ctx);
    js_disable_process_info(ctx);

    // Only allow explicitly declared effects
    return MTP_SUCCESS;
}

// Enforce zero hidden I/O (line 12)
int enforce_zero_hidden_io(JSContext *ctx) {
    // Monitor all I/O operations
    js_monitor_io_operations(ctx);

    // Ensure all I/O goes through effect system
    js_redirect_io_to_effects(ctx);

    return MTP_SUCCESS;
}

// Enforce zero cross-request state (line 13)
int enforce_zero_cross_request_state(JSContext *ctx) {
    // Clear all global state between requests
    js_clear_global_state(ctx);

    // Reset static variables
    js_reset_static_variables(ctx);

    // Clear caches (except effect cache which is deterministic)
    js_clear_non_deterministic_caches(ctx);

    return MTP_SUCCESS;
}

// Create per-request sandbox (line 25)
JSContext* create_per_request_sandbox(Snapshot *snapshot, const request_context_t *req_ctx) {
    // Clone VM from snapshot
    JSContext *ctx = clone_vm(snapshot);

    // Enforce security boundaries
    enforce_zero_ambient_authority(ctx);
    enforce_zero_hidden_io(ctx);
    enforce_zero_cross_request_state(ctx);

    // Set fixed memory budget (line 355)
    JS_SetMemoryLimit(ctx, MTP_FIXED_MEMORY_BUDGET);

    return ctx;
}

// Discard VM after response (line 355, 439)
void discard_vm(JSContext *ctx, const response_metadata_t *resp_meta) {
    // Check if response contained PCI-classified data
    if (resp_meta->contains_pci_data) {
        // Secure wipe performed on sensitive pages (line 27, 439)
        secure_wipe_sensitive_pages(ctx);
    }

    // Free VM context
    JS_FreeContext(ctx);
}

// Inject host effects per VM (line 27, 356)
void inject_host_effects(JSContext *ctx, const uint8_t seed[32]) {
    // Re-inject effects for each VM (never reused)
    js_register_effect_handlers(ctx, effect_handlers);

    // Set deterministic seed for effects
    js_set_effect_seed(ctx, seed);

    // Run static initializers once per VM (line 28)
    js_run_static_initializers(ctx);
}
```

**Test to Create**: `tests/runtime/isolation_test.c`
```c
int test_zero_ambient_authority() {
    JSContext *ctx = mtp_js_create_context();

    // Test file system access is blocked
    assert(mtp_js_execute(ctx, "require('fs')") == MTP_ERROR_FORBIDDEN_CONSTRUCT);

    // Test network access is blocked
    assert(mtp_js_execute(ctx, "fetch('http://example.com')") == MTP_ERROR_FORBIDDEN_CONSTRUCT);

    // Test environment variables are blocked
    assert(mtp_js_execute(ctx, "process.env") == MTP_ERROR_FORBIDDEN_CONSTRUCT);

    return PASS;
}

int test_per_request_isolation() {
    Snapshot *snapshot = load_snapshot("test.msqs");
    request_context_t req_ctx = {/* ... */};

    // Create two sandboxes from same snapshot
    JSContext *ctx1 = create_per_request_sandbox(snapshot, &req_ctx);
    JSContext *ctx2 = create_per_request_sandbox(snapshot, &req_ctx);

    // Modify state in first sandbox
    mtp_js_execute(ctx1, "globalVar = 'test1'");

    // Verify state is not shared with second sandbox
    JSValue result = mtp_js_eval(ctx2, "globalVar");
    assert(JS_IsNull(result) || JS_IsUndefined(result));

    // Discard VMs
    discard_vm(ctx1, &(response_metadata_t){.contains_pci_data = false});
    discard_vm(ctx2, &(response_metadata_t){.contains_pci_data = false});

    return PASS;
}
```

---

## Phase 1: Parser + AST (Weeks 6-7)

### Task 1.1: Lexer Implementation
**Files**: `src/compiler/lexer.mtp`, `src/compiler/lexer.h`
**Build Output**: `build/liblexer.a`

**Specification References**:
- Lines 153-161: Syntax & Grammar (EBNF)
- Line 159: await expression addition

**Implementation Pseudocode**:
```mtp
// src/compiler/lexer.mtp
type Token {
    | TokenNumber(number)
    | TokenString(string)
    | TokenIdentifier(string)
    | TokenKeyword(string)  // if, else, function, type, api, uses, etc.
    | TokenOperator(string) // +, -, *, /, ==, !=, <, >, <=, >=
    | TokenPunctuation(string) // (, ), {, }, [, ], ;, ,, :, |
    | TokenAwait  // await keyword (line 159)
    | TokenEOF
}

type Lexer {
    source: string,
    position: number,
    current_char: char
}

// Initialize lexer
function create_lexer(source: string): Lexer {
    return Lexer {
        source: source,
        position: 0,
        current_char: source[0] if source.length > 0 else '\0'
    }
}

// Get next token
function next_token(lexer: Lexer): Token {
    // Skip whitespace
    while (is_whitespace(lexer.current_char)) {
        advance(lexer)
    }

    // Handle comments
    if (lexer.current_char == '/' && peek_char(lexer) == '/') {
        skip_line_comment(lexer)
        return next_token(lexer)
    }

    if (lexer.current_char == '/' && peek_char(lexer) == '*') {
        skip_block_comment(lexer)
        return next_token(lexer)
    }

    // Handle numbers (signed 64-bit, line 169)
    if (is_digit(lexer.current_char) ||
        (lexer.current_char == '-' && is_digit(peek_char(lexer)))) {
        return scan_number(lexer)
    }

    // Handle strings (UTF-8, line 171)
    if (lexer.current_char == '"') {
        return scan_string(lexer)
    }

    // Handle identifiers and keywords
    if (is_identifier_start(lexer.current_char)) {
        return scan_identifier(lexer)
    }

    // Handle operators
    if (is_operator_start(lexer.current_char)) {
        return scan_operator(lexer)
    }

    // Handle punctuation
    if (is_punctuation(lexer.current_char)) {
        return scan_punctuation(lexer)
    }

    // Handle await keyword (line 159)
    if (starts_with(lexer.source, lexer.position, "await")) {
        advance_n(lexer, 5)
        return TokenAwait
    }

    // EOF
    if (lexer.current_char == '\0') {
        return TokenEOF
    }

    // Unknown character
    return TokenError("Unexpected character: " + lexer.current_char)
}

// Scan number (signed 64-bit with overflow checking)
function scan_number(lexer: Lexer): Token {
    let start_pos = lexer.position
    let is_negative = false

    if (lexer.current_char == '-') {
        is_negative = true
        advance(lexer)
    }

    if (!is_digit(lexer.current_char)) {
        return TokenError("Invalid number format")
    }

    let value = 0
    while (is_digit(lexer.current_char)) {
        value = value * 10 + (lexer.current_char - '0')

        // Check for 64-bit overflow (line 169)
        if (value > 9223372036854775807 && !is_negative) {
            return TokenError("Number overflow: exceeds 64-bit signed integer")
        }
        if (value > 9223372036854775808 && is_negative) {
            return TokenError("Number overflow: exceeds 64-bit signed integer")
        }

        advance(lexer)
    }

    if (is_negative) {
        value = -value
    }

    return TokenNumber(value)
}

// Scan string (UTF-8, line 171)
function scan_string(lexer: Lexer): Token {
    advance(lexer) // Skip opening quote

    let value = ""
    while (lexer.current_char != '"' && lexer.current_char != '\0') {
        // Handle escape sequences
        if (lexer.current_char == '\\') {
            advance(lexer)
            match (lexer.current_char) {
                | 'n' => value += '\n'
                | 't' => value += '\t'
                | 'r' => value += '\r'
                | '\\' => value += '\\'
                | '"' => value += '"'
                | 'u' => {
                    // Handle UTF-8 escape sequences
                    advance(lexer)
                    let code_point = scan_unicode_escape(lexer)
                    value += utf8_from_code_point(code_point)
                }
                | _ => return TokenError("Invalid escape sequence")
            }
        } else {
            value += lexer.current_char
        }
        advance(lexer)
    }

    if (lexer.current_char != '"') {
        return TokenError("Unterminated string")
    }

    advance(lexer) // Skip closing quote
    return TokenString(value)
}
```

**Test to Create**: `tests/unit/lexer_test.c`
```c
int test_lexer_tokenization() {
    const char *source = "let x = 42 + \"hello\"";
    Lexer *lexer = create_lexer(source);

    // Test token sequence
    Token *token = next_token(lexer);
    assert(token->type == TOKEN_KEYWORD);
    assert(strcmp(token->value, "let") == 0);

    token = next_token(lexer);
    assert(token->type == TOKEN_IDENTIFIER);
    assert(strcmp(token->value, "x") == 0);

    token = next_token(lexer);
    assert(token->type == TOKEN_OPERATOR);
    assert(strcmp(token->value, "=") == 0);

    token = next_token(lexer);
    assert(token->type == TOKEN_NUMBER);
    assert(token->number_value == 42);

    token = next_token(lexer);
    assert(token->type == TOKEN_OPERATOR);
    assert(strcmp(token->value, "+") == 0);

    token = next_token(lexer);
    assert(token->type == TOKEN_STRING);
    assert(strcmp(token->string_value, "hello") == 0);

    return PASS;
}

int test_lexer_number_overflow() {
    Lexer *lexer = create_lexer("9223372036854775808"); // Max int64 + 1

    Token *token = next_token(lexer);
    assert(token->type == TOKEN_ERROR);
    assert(strstr(token->error_message, "overflow") != NULL);

    return PASS;
}

int test_lexer_await_keyword() {
    Lexer *lexer = create_lexer("await x");

    Token *token = next_token(lexer);
    assert(token->type == TOKEN_AWAIT);

    return PASS;
}
```

---

[Continue with remaining tasks... Due to length constraints, I'll provide the next phase in a separate response. Would you like me to continue with the detailed breakdown for the remaining phases (Parser, Type System, Code Generation, Tooling)?]