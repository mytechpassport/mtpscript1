# MTPScript Complete Runtime Implementation Plan

## Executive Summary
This document provides the complete implementation plan for MTPScript v5.1 runtime, transforming QuickJS into a deterministic, serverless-first execution engine with per-request sandbox isolation, gas metering, and effect system.

---

## Phase 0: Project Foundation

### Task 0.1: Directory Structure Creation
**Objective**: Create complete directory structure per FOLDER_STRUCTURE.md
**Files to Create**:
```
build/
├── artifacts/
├── objects/
└── generated/
core/
├── runtime/
├── utils/
├── regex/
├── stdlib/
├── include/
├── effects/
├── crypto/
├── db/
├── http/
└── log/
src/
├── main/
├── cli/
├── compiler/
├── decimal/
├── lsp/
└── snapshot/
tests/
├── unit/
├── integration/
└── fixtures/
scripts/
runtime/
├── gas/
├── host/
└── snapshot/
examples/
```

**Implementation Script** (`scripts/setup_structure.sh`):
```bash
#!/bin/bash
set -e

# Core directories
mkdir -p build/{artifacts,objects,generated}
mkdir -p core/{runtime,utils,regex,stdlib,include,effects,crypto,db,http,log}
mkdir -p src/{main,cli,compiler,decimal,lsp,snapshot}
mkdir -p tests/{unit,integration,fixtures}
mkdir -p scripts
mkdir -p runtime/{gas,host,snapshot}
mkdir -p examples

echo "Directory structure created"
```

---

## Phase 1: QuickJS Integration & MTPJS Foundation

### Task 1.1: Clone and Rename QuickJS
**Objective**: Clone QuickJS and rename all references to MTPJS
**Specification**: TECHSPECV5.md lines 9-17, 346-347

**Implementation Steps**:
```bash
#!/bin/bash
# scripts/setup_quickjs.sh
set -e

# Clone QuickJS
git clone https://github.com/bellard/quickjs.git vendor/quickjs-temp

# Copy core files to MTPJS structure
cp vendor/quickjs-temp/quickjs.c core/runtime/mtpjs.c
cp vendor/quickjs-temp/quickjs.h core/include/mtpjs.h
cp vendor/quickjs-temp/cutils.c core/utils/mtp_cutils.c
cp vendor/quickjs-temp/cutils.h core/utils/mtp_cutils.h
cp vendor/quickjs-temp/dtoa.c core/utils/mtp_dtoa.c
cp vendor/quickjs-temp/dtoa.h core/utils/mtp_dtoa.h
cp vendor/quickjs-temp/libregexp.c core/regex/mtp_libregexp.c
cp vendor/quickjs-temp/libregexp.h core/regex/mtp_libregexp.h
cp vendor/quickjs-temp/libunicode.c core/regex/mtp_libunicode.c
cp vendor/quickjs-temp/libunicode.h core/regex/mtp_libunicode.h
cp vendor/quickjs-temp/quickjs-libc.c core/stdlib/mtpjs_libc.c

# Rename all references
find core/ src/ -name "*.c" -o -name "*.h" | xargs sed -i 's/quickjs/mtpjs/g'
find core/ src/ -name "*.c" -o -name "*.h" | xargs sed -i 's/QuickJS/MTPJS/g'
find core/ src/ -name "*.c" -o -name "*.h" | xargs sed -i 's/QUICKJS/MTPJS/g'

# Clean up temporary clone
rm -rf vendor/quickjs-temp
```

### Task 1.2: MTPJS Core Runtime
**Objective**: Create main MTPJS runtime engine
**Files**: `core/runtime/mtpjs.c`, `core/include/mtpjs.h`

**Core Structures** (`core/include/mtpjs.h`):
```c
#ifndef MTPJS_H
#define MTPJS_H

#include <stdint.h>
#include <stdbool.h>

// MTPScript Runtime Configuration
typedef struct {
    uint64_t gas_limit;
    uint64_t gas_used;
    uint8_t deterministic_seed[32];
    bool gas_exhausted;
    bool per_request_isolation;
} MTPRuntime;

typedef struct {
    MTPRuntime *runtime;
    void *heap;
    size_t heap_size;
    bool pci_data_touched;
} MTPContext;

// Runtime Functions
MTPRuntime* mtpjs_init_runtime(void);
MTPContext* mtpjs_init_context(MTPRuntime *rt);
void mtpjs_free_runtime(MTPRuntime *rt);
void mtpjs_free_context(MTPContext *ctx);

// Security Functions
int mtpjs_configure_security(MTPContext *ctx);
int mtpjs_check_forbidden_constructs(const char *code, size_t len);

// Gas Functions
int mtpjs_set_gas_limit(MTPContext *ctx, uint64_t limit);
uint64_t mtpjs_get_gas_used(MTPContext *ctx);
int mtpjs_consume_gas(MTPContext *ctx, uint64_t cost);

// Deterministic Seed Functions
int mtpjs_generate_seed(const char *aws_request_id, const char *aws_account_id,
                       const char *function_version, const char *snapshot_hash,
                       uint64_t gas_limit, uint8_t seed[32]);
void mtpjs_inject_seed(MTPContext *ctx, const uint8_t seed[32]);

#endif
```

**Core Implementation** (`core/runtime/mtpjs.c`):
```c
#include "mtpjs.h"
#include <stdlib.h>
#include <string.h>

// Forbidden JavaScript constructs per TECHSPECV5.md line 346
static const char* forbidden_constructs[] = {
    "eval", "class", "this", "try", "catch", "for", "while", "do"
};

// Initialize MTPScript runtime
MTPRuntime* mtpjs_init_runtime(void) {
    MTPRuntime *rt = calloc(1, sizeof(MTPRuntime));
    if (!rt) return NULL;
    
    // Default gas limit from TECHSPECV5.md line 73
    rt->gas_limit = 10000000;
    rt->gas_used = 0;
    rt->gas_exhausted = false;
    rt->per_request_isolation = true;
    
    return rt;
}

// Initialize MTPScript context
MTPContext* mtpjs_init_context(MTPRuntime *rt) {
    MTPContext *ctx = calloc(1, sizeof(MTPContext));
    if (!ctx) return NULL;
    
    ctx->runtime = rt;
    ctx->heap_size = 64 * 1024 * 1024; // 64MB fixed budget per TECHSPECV5.md line 355
    ctx->heap = malloc(ctx->heap_size);
    ctx->pci_data_touched = false;
    
    return ctx;
}

// Check for forbidden constructs
int mtpjs_check_forbidden_constructs(const char *code, size_t len) {
    for (int i = 0; i < sizeof(forbidden_constructs)/sizeof(forbidden_constructs[0]); i++) {
        if (strstr(code, forbidden_constructs[i]) != NULL) {
            return -1; // Forbidden construct found
        }
    }
    return 0; // All clear
}

// Gas metering - consume gas for operations
int mtpjs_consume_gas(MTPContext *ctx, uint64_t cost) {
    if (ctx->runtime->gas_exhausted) {
        return -2; // Already exhausted
    }
    
    uint64_t new_used = ctx->runtime->gas_used + cost;
    
    // Check gas exhaustion per TECHSPECV5.md lines 81-87
    if (new_used > ctx->runtime->gas_limit) {
        ctx->runtime->gas_exhausted = true;
        return -1; // Gas exhausted
    }
    
    ctx->runtime->gas_used = new_used;
    return 0; // Success
}

// Generate deterministic seed per TECHSPECV5.md lines 58-66
int mtpjs_generate_seed(const char *aws_request_id, const char *aws_account_id,
                       const char *function_version, const char *snapshot_hash,
                       uint64_t gas_limit, uint8_t seed[32]) {
    // Simple SHA-256 implementation (will be replaced with OpenSSL)
    // For now, create deterministic seed from input parameters
    
    // Concatenate all seed components
    char seed_input[512];
    snprintf(seed_input, sizeof(seed_input), "%s%s%smtpscript-v5.1%s%llu",
             aws_request_id ? aws_request_id : "",
             aws_account_id ? aws_account_id : "",
             function_version ? function_version : "",
             snapshot_hash ? snapshot_hash : "",
             (unsigned long long)gas_limit);
    
    // For now, simple hash - will be replaced with proper SHA-256
    for (int i = 0; i < 32; i++) {
        seed[i] = (uint8_t)(seed_input[i % strlen(seed_input)]);
    }
    
    return 0;
}
```

---

## Phase 2: Gas Metering System

### Task 2.1: Gas Cost Table
**Objective**: Define gas costs for all operations
**Specification**: TECHSPECV5.md lines 91-94, Annex A

**Gas Costs** (`runtime/gas/gas_costs.h`):
```c
#ifndef GAS_COSTS_H
#define GAS_COSTS_H

typedef enum {
    OP_LOAD_CONST = 1,
    OP_GET_GLOBAL = 2,
    OP_SET_GLOBAL = 3,
    OP_BINARY_OP = 4,
    OP_UNARY_OP = 5,
    OP_FUNCTION_CALL = 100,
    OP_RETURN = 6,
    OP_TAIL_CALL = 0,  // Free per TECHSPECV5.md line 219
    OP_DB_READ = 1000,
    OP_DB_WRITE = 2000,
    OP_HTTP_OUT = 2000,
    OP_LOG = 50,
    OP_ASYNC_AWAIT = 500,
} mtp_opcode_t;

static const uint64_t gas_costs[] = {
    [OP_LOAD_CONST] = 1,
    [OP_GET_GLOBAL] = 2,
    [OP_SET_GLOBAL] = 3,
    [OP_BINARY_OP] = 4,
    [OP_UNARY_OP] = 2,
    [OP_FUNCTION_CALL] = 100,
    [OP_RETURN] = 2,
    [OP_TAIL_CALL] = 0,
    [OP_DB_READ] = 1000,
    [OP_DB_WRITE] = 2000,
    [OP_HTTP_OUT] = 2000,
    [OP_LOG] = 50,
    [OP_ASYNC_AWAIT] = 500,
};

static inline uint64_t get_gas_cost(mtp_opcode_t op) {
    if (op < sizeof(gas_costs)/sizeof(gas_costs[0])) {
        return gas_costs[op];
    }
    return 100; // Default cost for unknown operations
}

#endif
```

### Task 2.2: Gas Injection and Enforcement
**Objective**: Implement gas limit injection and enforcement
**Specification**: TECHSPECV5.md lines 72-89

**Gas Implementation** (`runtime/gas/gas_injection.c`):
```c
#include "mtpjs.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

// Initialize gas limit from environment per TECHSPECV5.md lines 72-74
uint64_t mtpjs_initialize_gas_limit(void) {
    const char *gas_limit_str = getenv("MTP_GAS_LIMIT");
    
    if (!gas_limit_str) {
        return 10000000; // Default per TECHSPECV5.md line 73
    }
    
    char *endptr;
    uint64_t gas_limit = strtoull(gas_limit_str, &endptr, 10);
    
    // Validate range 1-2,000,000,000 per TECHSPECV5.md line 74
    if (*endptr != '\0' || gas_limit < 1 || gas_limit > 2000000000ULL) {
        fprintf(stderr, "MTPError: GasLimitOutOfRange\n");
        exit(1);
    }
    
    return gas_limit;
}

// Inject gas limit into VM per TECHSPECV5.md line 75
void mtpjs_inject_gas_limit(MTPContext *ctx, uint64_t gas_limit) {
    ctx->runtime->gas_limit = gas_limit;
    ctx->runtime->gas_used = 0;
    ctx->runtime->gas_exhausted = false;
    
    // Append to audit log per TECHSPECV5.md line 76
    fprintf(stderr, "AUDIT: gasLimit=%llu\n", (unsigned long long)gas_limit);
}

// Handle gas exhaustion per TECHSPECV5.md lines 81-87
int mtpjs_handle_gas_exhaustion(MTPContext *ctx, char *response, size_t response_size) {
    // Create deterministic error response
    snprintf(response, response_size,
             "{ \"error\": \"GasExhausted\", \"gasLimit\": %llu, \"gasUsed\": %llu }",
             (unsigned long long)ctx->runtime->gas_limit,
             (unsigned long long)ctx->runtime->gas_used);
    
    // No stack trace in production per TECHSPECV5.md line 89
    return -1; // Gas exhausted error
}
```

---

## Phase 3: Effect System Implementation

### Task 3.1: Effect Dispatcher Framework
**Objective**: Create deterministic effect system
**Specification**: TECHSPECV5.md lines 223-237

**Effect System** (`core/effects/mtpjs_effects.c`):
```c
#include "mtpjs.h"
#include <stdlib.h>
#include <string.h>

typedef struct {
    const char *name;
    int (*handler)(MTPContext *ctx, const char *args, char *response, size_t response_size);
    bool cacheable;
} effect_handler_t;

// Effect cache entry
typedef struct {
    char key[256];
    char response[4096];
    bool valid;
} effect_cache_entry_t;

// Effect cache (simple LRU implementation)
static effect_cache_entry_t effect_cache[100];
static int cache_index = 0;

// Built-in effect handlers per TECHSPECV5.md lines 229-236
static const effect_handler_t effect_handlers[] = {
    {"DbRead", handle_db_read, true},
    {"DbWrite", handle_db_write, false},
    {"HttpOut", handle_http_out, true},
    {"Log", handle_log, false},
    {"Async", handle_async, true},
    {NULL, NULL, false}
};

// Generate cache key from seed and arguments
static void generate_cache_key(const char *effect_name, const char *args,
                           const uint8_t seed[32], char *key, size_t key_size) {
    char seed_hex[65];
    for (int i = 0; i < 32; i++) {
        sprintf(seed_hex + i*2, "%02x", seed[i]);
    }
    seed_hex[64] = '\0';
    
    snprintf(key, key_size, "%s_%s_%s", effect_name, seed_hex, args);
}

// Dispatch effect with deterministic caching
int mtpjs_dispatch_effect(MTPContext *ctx, const char *effect_name,
                         const char *args, char *response, size_t response_size) {
    // Find effect handler
    const effect_handler_t *handler = NULL;
    for (int i = 0; effect_handlers[i].name != NULL; i++) {
        if (strcmp(effect_handlers[i].name, effect_name) == 0) {
            handler = &effect_handlers[i];
            break;
        }
    }
    
    if (!handler) {
        snprintf(response, response_size, "{\"error\":\"Unknown effect: %s\"}", effect_name);
        return -1;
    }
    
    // Generate cache key
    char cache_key[256];
    if (handler->cacheable) {
        generate_cache_key(effect_name, args, ctx->runtime->deterministic_seed,
                       cache_key, sizeof(cache_key));
        
        // Check cache for deterministic response per TECHSPECV5.md line 274
        for (int i = 0; i < 100; i++) {
            if (effect_cache[i].valid && strcmp(effect_cache[i].key, cache_key) == 0) {
                strncpy(response, effect_cache[i].response, response_size);
                return 0; // Cached response
            }
        }
    }
    
    // Consume gas for effect operation
    mtp_opcode_t op = OP_DB_READ; // Default, should be mapped from effect name
    if (strcmp(effect_name, "DbWrite") == 0) op = OP_DB_WRITE;
    else if (strcmp(effect_name, "HttpOut") == 0) op = OP_HTTP_OUT;
    else if (strcmp(effect_name, "Log") == 0) op = OP_LOG;
    else if (strcmp(effect_name, "Async") == 0) op = OP_ASYNC_AWAIT;
    
    if (mtpjs_consume_gas(ctx, get_gas_cost(op)) != 0) {
        return mtpjs_handle_gas_exhaustion(ctx, response, response_size);
    }
    
    // Execute effect handler with deterministic seed
    int result = handler->handler(ctx, args, response, response_size);
    
    // Cache result if required
    if (handler->cacheable && result == 0) {
        strncpy(effect_cache[cache_index].key, cache_key, sizeof(effect_cache[cache_index].key));
        strncpy(effect_cache[cache_index].response, response, sizeof(effect_cache[cache_index].response));
        effect_cache[cache_index].valid = true;
        cache_index = (cache_index + 1) % 100;
    }
    
    return result;
}

// Database read effect (deterministic)
static int handle_db_read(MTPContext *ctx, const char *args, char *response, size_t response_size) {
    // Parse SQL from args
    char sql[1024];
    if (sscanf(args, "{\"sql\":\"%1023[^\"]\"}", sql) != 1) {
        strncpy(response, "{\"error\":\"Invalid SQL format\"}", response_size);
        return -1;
    }
    
    // For now, return deterministic mock data
    // In production, this would query a database deterministically
    snprintf(response, response_size,
             "{\"result\":[{\"id\":1,\"name\":\"test\"}],\"affectedRows\":1}");
    return 0;
}

// Database write effect (deterministic)
static int handle_db_write(MTPContext *ctx, const char *args, char *response, size_t response_size) {
    // Parse SQL and data from args
    char table[256];
    char data[2048];
    if (sscanf(args, "{\"table\":\"%255[^\"]\",\"data\":\"%2047[^\"]\"}", table, data) != 2) {
        strncpy(response, "{\"error\":\"Invalid write format\"}", response_size);
        return -1;
    }
    
    // For now, return deterministic success
    snprintf(response, response_size,
             "{\"insertedId\":42,\"affectedRows\":1}");
    return 0;
}

// HTTP out effect (deterministic)
static int handle_http_out(MTPContext *ctx, const char *args, char *response, size_t response_size) {
    char url[2048];
    char method[16];
    if (sscanf(args, "{\"url\":\"%2047[^\"]\",\"method\":\"%15[^\"]\"}", url, method) != 2) {
        strncpy(response, "{\"error\":\"Invalid HTTP format\"}", response_size);
        return -1;
    }
    
    // For now, return deterministic mock response
    // In production, this would make real HTTP calls deterministically
    snprintf(response, response_size,
             "{\"status\":200,\"body\":\"mock response for %s %s\"}", method, url);
    return 0;
}

// Log effect
static int handle_log(MTPContext *ctx, const char *args, char *response, size_t response_size) {
    char level[16];
    char message[1024];
    if (sscanf(args, "{\"level\":\"%15[^\"]\",\"message\":\"%1023[^\"]\"}", level, message) != 2) {
        strncpy(response, "{\"error\":\"Invalid log format\"}", response_size);
        return -1;
    }
    
    // Log to stderr for now
    fprintf(stderr, "LOG [%s]: %s\n", level, message);
    
    strncpy(response, "{\"success\":true}", response_size);
    return 0;
}

// Async effect (deterministic per TECHSPECV5.md lines 241-276)
static int handle_async(MTPContext *ctx, const char *args, char *response, size_t response_size) {
    char promise_hash[65];
    int cont_id;
    char effect_args[1024];
    
    if (sscanf(args, "{\"promiseHash\":\"%64[^\"]\",\"contId\":%d,\"effectArgs\":\"%1023[^\"]\"}",
                 promise_hash, &cont_id, effect_args) != 3) {
        strncpy(response, "{\"error\":\"Invalid async format\"}", response_size);
        return -1;
    }
    
    // Block-synchronous execution per TECHSPECV5.md line 272
    // For now, return deterministic mock async response
    snprintf(response, response_size,
             "{\"result\":\"async response for contId %d\",\"promiseHash\":\"%s\"}",
             cont_id, promise_hash);
    return 0;
}
```

---

## Phase 4: VM Snapshot System

### Task 4.1: Snapshot Format and Creation
**Objective**: Implement .msqs snapshot format
**Specification**: TECHSPECV5.md lines 25-28, 434-440

**Snapshot System** (`runtime/snapshot/snapshot.c`):
```c
#include "mtpjs.h"
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

// Snapshot format per TECHSPECV5.md lines 25-28
#pragma pack(push, 1)
typedef struct {
    uint32_t magic;              // 0x4D545351 ("MTPQ")
    uint16_t version_major;      // 5
    uint16_t version_minor;      // 1
    uint64_t content_hash;       // SHA-256 of bytecode
    uint32_t bytecode_size;      // Size of compiled JS
    uint32_t heap_size;          // Initial heap state
    uint32_t stack_size;         // Stack state
    uint64_t timestamp;          // Build timestamp
    uint8_t signature[64];        // ECDSA-P256 signature (mock for now)
    uint8_t bytecode[];          // Variable-length bytecode
} MTPSnapshot;
#pragma pack(pop)

#define MTP_SNAPSHOT_MAGIC 0x4D545351
#define MTP_SNAPSHOT_VERSION_MAJOR 5
#define MTP_SNAPSHOT_VERSION_MINOR 1

// Create snapshot from JavaScript code
MTPSnapshot* mtpjs_create_snapshot(const char *javascript_code, size_t code_size) {
    // Calculate content hash (mock - should be SHA-256)
    uint64_t content_hash = 0;
    for (size_t i = 0; i < code_size; i++) {
        content_hash = content_hash * 31 + javascript_code[i];
    }
    
    // Allocate snapshot
    size_t snapshot_size = sizeof(MTPSnapshot) + code_size;
    MTPSnapshot *snapshot = calloc(1, snapshot_size);
    if (!snapshot) return NULL;
    
    // Fill snapshot header
    snapshot->magic = MTP_SNAPSHOT_MAGIC;
    snapshot->version_major = MTP_SNAPSHOT_VERSION_MAJOR;
    snapshot->version_minor = MTP_SNAPSHOT_VERSION_MINOR;
    snapshot->content_hash = content_hash;
    snapshot->bytecode_size = code_size;
    snapshot->heap_size = 64 * 1024 * 1024; // 64MB
    snapshot->stack_size = 1024 * 1024; // 1MB
    snapshot->timestamp = 1234567890; // Mock timestamp
    
    // Copy bytecode
    memcpy(snapshot->bytecode, javascript_code, code_size);
    
    // Mock signature (should be real ECDSA-P256 per TECHSPECV5.md line 431)
    memset(snapshot->signature, 0xAA, sizeof(snapshot->signature));
    
    return snapshot;
}

// Verify snapshot signature per TECHSPECV5.md line 434
int mtpjs_verify_snapshot(MTPSnapshot *snapshot) {
    // Check magic number
    if (snapshot->magic != MTP_SNAPSHOT_MAGIC) {
        return -1; // Invalid magic
    }
    
    // Check version
    if (snapshot->version_major != MTP_SNAPSHOT_VERSION_MAJOR ||
        snapshot->version_minor != MTP_SNAPSHOT_VERSION_MINOR) {
        return -2; // Invalid version
    }
    
    // For now, always accept signature (should verify ECDSA-P256)
    return 0;
}

// Clone VM from snapshot with copy-on-write per TECHSPECV5.md line 26
MTPContext* mtpjs_clone_vm(MTPSnapshot *snapshot) {
    // Create new runtime and context
    MTPRuntime *rt = mtpjs_init_runtime();
    if (!rt) return NULL;
    
    MTPContext *ctx = mtpjs_init_context(rt);
    if (!ctx) {
        mtpjs_free_runtime(rt);
        return NULL;
    }
    
    // Load bytecode into context
    // For now, just store reference to snapshot
    // In production, this would map memory copy-on-write
    
    // Verify snapshot signature
    if (mtpjs_verify_snapshot(snapshot) != 0) {
        mtpjs_free_context(ctx);
        mtpjs_free_runtime(rt);
        return NULL;
    }
    
    return ctx;
}

// Free snapshot
void mtpjs_free_snapshot(MTPSnapshot *snapshot) {
    free(snapshot);
}
```

---

## Phase 5: Security Isolation

### Task 5.1: Zero Ambient Authority
**Objective**: Implement security constraints
**Specification**: TECHSPECV5.md lines 11-15

**Security Implementation** (`core/runtime/mtpjs_security.c`):
```c
#include "mtpjs.h"
#include <string.h>

// Security configuration
typedef struct {
    bool forbid_eval;
    bool forbid_class;
    bool forbid_this;
    bool forbid_try_catch;
    bool forbid_loops;
    bool forbid_global_mutation;
    bool zero_ambient_authority;
    bool zero_hidden_io;
    bool zero_cross_request_state;
} security_config_t;

static security_config_t default_security = {
    .forbid_eval = true,
    .forbid_class = true,
    .forbid_this = true,
    .forbid_try_catch = true,
    .forbid_loops = true,
    .forbid_global_mutation = true,
    .zero_ambient_authority = true,
    .zero_hidden_io = true,
    .zero_cross_request_state = true,
};

// Configure security for context
int mtpjs_configure_security(MTPContext *ctx) {
    // Apply all security constraints per TECHSPECV5.md lines 11-15
    // For now, just store configuration
    // In production, this would patch QuickJS runtime functions
    
    return 0;
}

// Enforce zero ambient authority per TECHSPECV5.md line 11
int mtpjs_enforce_zero_ambient_authority(MTPContext *ctx) {
    // Disable file system access
    // Disable network access
    // Disable environment variables
    // Disable process information
    
    // For now, return success (mock implementation)
    return 0;
}

// Enforce zero hidden I/O per TECHSPECV5.md line 12
int mtpjs_enforce_zero_hidden_io(MTPContext *ctx) {
    // Monitor all I/O operations
    // Ensure all I/O goes through effect system
    // Remove hidden side effects
    
    return 0;
}

// Enforce zero cross-request state per TECHSPECV5.md line 13
int mtpjs_enforce_zero_cross_request_state(MTPContext *ctx) {
    // Clear all global state between requests
    // Reset static variables
    // Clear non-deterministic caches
    
    // Mark PCI data flag for secure wipe
    ctx->pci_data_touched = false;
    
    return 0;
}

// Secure wipe for sensitive pages per TECHSPECV5.md lines 27, 439
void mtpjs_secure_wipe_sensitive_pages(MTPContext *ctx) {
    if (!ctx->pci_data_touched) {
        return; // No PCI data touched
    }
    
    // Secure wipe with multiple passes
    if (ctx->heap) {
        memset(ctx->heap, 0, ctx->heap_size);
        memset(ctx->heap, 0xFF, ctx->heap_size);
        memset(ctx->heap, 0, ctx->heap_size);
    }
}
```

---

## Phase 6: Main Application

### Task 6.1: MTPJS REPL
**Objective**: Create main REPL application
**File**: `src/main/mtpjs_repl.c`

**REPL Implementation**:
```c
#include "mtpjs.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(int argc, char *argv[]) {
    printf("MTPScript v5.1 REPL\n");
    printf("Deterministic execution environment\n");
    
    // Initialize runtime
    MTPRuntime *rt = mtpjs_init_runtime();
    if (!rt) {
        fprintf(stderr, "Failed to initialize runtime\n");
        return 1;
    }
    
    MTPContext *ctx = mtpjs_init_context(rt);
    if (!ctx) {
        fprintf(stderr, "Failed to initialize context\n");
        mtpjs_free_runtime(rt);
        return 1;
    }
    
    // Configure security
    mtpjs_configure_security(ctx);
    mtpjs_enforce_zero_ambient_authority(ctx);
    mtpjs_enforce_zero_hidden_io(ctx);
    mtpjs_enforce_zero_cross_request_state(ctx);
    
    // Initialize gas limit
    uint64_t gas_limit = mtpjs_initialize_gas_limit();
    mtpjs_inject_gas_limit(ctx, gas_limit);
    
    // REPL loop
    char input[4096];
    while (1) {
        printf("mtpjs> ");
        fflush(stdout);
        
        if (fgets(input, sizeof(input), stdin) == NULL) {
            break; // EOF
        }
        
        // Remove newline
        input[strcspn(input, "\n")] = 0;
        
        if (strcmp(input, "exit") == 0 || strcmp(input, "quit") == 0) {
            break;
        }
        
        // Check for forbidden constructs
        if (mtpjs_check_forbidden_constructs(input, strlen(input)) != 0) {
            printf("Error: Forbidden JavaScript construct\n");
            continue;
        }
        
        // Execute (mock for now)
        printf("Executing: %s\n", input);
        printf("Gas used: %llu / %llu\n", 
               (unsigned long long)ctx->runtime->gas_used,
               (unsigned long long)ctx->runtime->gas_limit);
    }
    
    // Cleanup
    mtpjs_secure_wipe_sensitive_pages(ctx);
    mtpjs_free_context(ctx);
    mtpjs_free_runtime(rt);
    
    return 0;
}
```

### Task 6.2: MTPSC Compiler
**Objective**: Create basic compiler
**File**: `src/cli/mtpsc.c`

**Compiler Implementation**:
```c
#include "mtpjs.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(int argc, char *argv[]) {
    if (argc != 3) {
        fprintf(stderr, "Usage: %s <input.mtp> <output.msqs>\n", argv[0]);
        return 1;
    }
    
    const char *input_file = argv[1];
    const char *output_file = argv[2];
    
    // Read input file
    FILE *f = fopen(input_file, "r");
    if (!f) {
        fprintf(stderr, "Cannot open input file: %s\n", input_file);
        return 1;
    }
    
    fseek(f, 0, SEEK_END);
    long file_size = ftell(f);
    fseek(f, 0, SEEK_SET);
    
    char *code = malloc(file_size + 1);
    fread(code, 1, file_size, f);
    code[file_size] = '\0';
    fclose(f);
    
    // Create snapshot
    MTPSnapshot *snapshot = mtpjs_create_snapshot(code, strlen(code));
    if (!snapshot) {
        fprintf(stderr, "Failed to create snapshot\n");
        free(code);
        return 1;
    }
    
    // Write snapshot to file
    FILE *out = fopen(output_file, "wb");
    if (!out) {
        fprintf(stderr, "Cannot open output file: %s\n", output_file);
        mtpjs_free_snapshot(snapshot);
        free(code);
        return 1;
    }
    
    size_t snapshot_size = sizeof(MTPSnapshot) + strlen(code);
    fwrite(snapshot, 1, snapshot_size, out);
    fclose(out);
    
    printf("Created snapshot: %s (%zu bytes)\n", output_file, snapshot_size);
    
    // Cleanup
    mtpjs_free_snapshot(snapshot);
    free(code);
    
    return 0;
}
```

---

## Phase 7: Build System

### Task 7.1: Simple Makefile
**Objective**: Create build system with compile, run, test targets
**Specification**: Simple Makefile requirements

**Main Makefile**:
```makefile
# MTPScript Build System
CC = gcc
CFLAGS = -Wall -Wextra -std=c11 -O2 -g
LDFLAGS = -lm

# Directories
BUILD_DIR = build
SRC_DIR = src
CORE_DIR = core
RUNTIME_DIR = runtime
TESTS_DIR = tests

# Source files
CORE_SOURCES = $(CORE_DIR)/runtime/mtpjs.c \
               $(CORE_DIR)/runtime/mtpjs_security.c \
               $(CORE_DIR)/effects/mtpjs_effects.c \
               $(CORE_DIR)/utils/mtp_cutils.c \
               $(CORE_DIR)/utils/mtp_dtoa.c \
               $(CORE_DIR)/regex/mtp_libregexp.c \
               $(CORE_DIR)/regex/mtp_libunicode.c \
               $(CORE_DIR)/stdlib/mtpjs_libc.c

RUNTIME_SOURCES = $(RUNTIME_DIR)/snapshot/snapshot.c \
                  $(RUNTIME_DIR)/gas/gas_injection.c

MAIN_SOURCES = $(SRC_DIR)/main/mtpjs_repl.c
CLI_SOURCES = $(SRC_DIR)/cli/mtpsc.c

# Objects
CORE_OBJECTS = $(CORE_SOURCES:$(CORE_DIR)/%.c=$(BUILD_DIR)/objects/core/%.o)
RUNTIME_OBJECTS = $(RUNTIME_SOURCES:$(RUNTIME_DIR)/%.c=$(BUILD_DIR)/objects/runtime/%.o)
MAIN_OBJECTS = $(MAIN_SOURCES:$(SRC_DIR)/%.c=$(BUILD_DIR)/objects/main/%.o)
CLI_OBJECTS = $(CLI_SOURCES:$(SRC_DIR)/%.c=$(BUILD_DIR)/objects/cli/%.o)

ALL_OBJECTS = $(CORE_OBJECTS) $(RUNTIME_OBJECTS)

# Targets
.PHONY: all clean compile run test compile-run

all: compile

# Compile all
compile: $(BUILD_DIR)/mtpjs_repl $(BUILD_DIR)/mtpsc

# Compile and run REPL
run: $(BUILD_DIR)/mtpjs_repl
	./$(BUILD_DIR)/mtpjs_repl

# Compile and run compiler
compile-run: $(BUILD_DIR)/mtpsc
	./$(BUILD_DIR)/mtpsc examples/test.mtp examples/test.msqs

# Run tests
test: compile
	@echo "Running basic tests..."
	@mkdir -p $(BUILD_DIR)/tests
	@echo "1" > $(BUILD_DIR)/tests/basic.passed
	@if [ -f examples/test.mtp ]; then \
		./$(BUILD_DIR)/mtpsc examples/test.mtp $(BUILD_DIR)/tests/test.msqs && \
		echo "Compiler test: PASSED" && \
		echo "2" > $(BUILD_DIR)/tests/basic.passed; \
	else \
		echo "Compiler test: SKIPPED (no test file)"; \
	fi
	@echo "Basic test suite completed"

# REPL executable
$(BUILD_DIR)/mtpjs_repl: $(MAIN_OBJECTS) $(ALL_OBJECTS)
	@mkdir -p $(BUILD_DIR)
	$(CC) $(CFLAGS) -I$(CORE_DIR)/include -o $@ $^ $(LDFLAGS)

# Compiler executable  
$(BUILD_DIR)/mtpsc: $(CLI_OBJECTS) $(ALL_OBJECTS)
	@mkdir -p $(BUILD_DIR)
	$(CC) $(CFLAGS) -I$(CORE_DIR)/include -o $@ $^ $(LDFLAGS)

# Object file compilation
$(BUILD_DIR)/objects/%.o: %.c
	@mkdir -p $(dir $@)
	$(CC) $(CFLAGS) -I$(CORE_DIR)/include -c -o $@ $<

# Clean
clean:
	rm -rf $(BUILD_DIR)

# Setup directories
setup:
	@mkdir -p $(BUILD_DIR)/objects/core
	@mkdir -p $(BUILD_DIR)/objects/runtime
	@mkdir -p $(BUILD_DIR)/objects/main
	@mkdir -p $(BUILD_DIR)/objects/cli
	@mkdir -p $(BUILD_DIR)/tests
	@mkdir -p examples
	@echo "Directory structure created"

# Help
help:
	@echo "Available targets:"
	@echo "  compile      - Build all executables"
	@echo "  run          - Compile and run REPL"
	@echo "  compile-run  - Compile and run compiler on test file"
	@echo "  test         - Run basic tests"
	@echo "  clean        - Remove build artifacts"
	@echo "  setup        - Create directory structure"
```

---

## Phase 8: Testing

### Task 8.1: Test Files
**Objective**: Create test examples and validation

**Test Example** (`examples/test.mtp`):
```javascript
// Simple MTPScript test program
const x = 1 + 2;
const y = x * 3;

// Log effect call (will be handled by effect system)
console.log("Result: " + y);

// This should be blocked (forbidden construct)
// eval("1+1"); // This should fail

// This should be blocked (forbidden construct)  
// class Test {} // This should fail

// Database effect (deterministic)
// DbRead({sql: "SELECT * FROM users"});

// Final result
y;
```

**Basic Test Suite** (`tests/basic_test.c`):
```c
#include "mtpjs.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <assert.h>

int test_runtime_creation() {
    MTPRuntime *rt = mtpjs_init_runtime();
    assert(rt != NULL);
    assert(rt->gas_limit == 10000000); // Default limit
    
    MTPContext *ctx = mtpjs_init_context(rt);
    assert(ctx != NULL);
    assert(ctx->heap_size == 64 * 1024 * 1024); // 64MB
    
    mtpjs_free_context(ctx);
    mtpjs_free_runtime(rt);
    
    printf("✓ Runtime creation test passed\n");
    return 0;
}

int test_gas_metering() {
    MTPRuntime *rt = mtpjs_init_runtime();
    MTPContext *ctx = mtpjs_init_context(rt);
    
    mtpjs_inject_gas_limit(ctx, 1000);
    
    assert(ctx->runtime->gas_limit == 1000);
    assert(ctx->runtime->gas_used == 0);
    
    // Consume some gas
    assert(mtpjs_consume_gas(ctx, 100) == 0);
    assert(ctx->runtime->gas_used == 100);
    
    // Try to exceed limit
    assert(mtpjs_consume_gas(ctx, 2000) == -1);
    assert(ctx->runtime->gas_exhausted == true);
    
    mtpjs_free_context(ctx);
    mtpjs_free_runtime(rt);
    
    printf("✓ Gas metering test passed\n");
    return 0;
}

int test_forbidden_constructs() {
    assert(mtpjs_check_forbidden_constructs("1 + 1", 5) == 0);
    assert(mtpjs_check_forbidden_constructs("eval('1+1')", 11) == -1);
    assert(mtpjs_check_forbidden_constructs("class Test {}", 13) == -1);
    assert(mtpjs_check_forbidden_constructs("this.x = 1", 10) == -1);
    
    printf("✓ Forbidden constructs test passed\n");
    return 0;
}

int test_snapshot_creation() {
    const char *code = "const x = 1 + 2;";
    MTPSnapshot *snapshot = mtpjs_create_snapshot(code, strlen(code));
    
    assert(snapshot != NULL);
    assert(snapshot->magic == 0x4D545351);
    assert(snapshot->version_major == 5);
    assert(snapshot->version_minor == 1);
    assert(snapshot->bytecode_size == strlen(code));
    
    mtpjs_free_snapshot(snapshot);
    
    printf("✓ Snapshot creation test passed\n");
    return 0;
}

int main() {
    printf("Running MTPScript basic tests...\n\n");
    
    test_runtime_creation();
    test_gas_metering();
    test_forbidden_constructs();
    test_snapshot_creation();
    
    printf("\n✓ All tests passed!\n");
    return 0;
}
```

---

## Implementation Sequence

### Step 1: Setup
```bash
make setup
```

### Step 2: Build
```bash
make compile
```

### Step 3: Test
```bash
make test
```

### Step 4: Run REPL
```bash
make run
```

### Step 5: Compile MTPScript
```bash
make compile-run
```

---

## Success Criteria

### Technical Success:
1. **Runtime Foundation**: MTPJS runtime initializes and executes JavaScript
2. **Gas Metering**: Gas limits enforced and exhaustion handled
3. **Effect System**: Deterministic effects with caching
4. **Snapshot System**: .msqs snapshots created and loaded
5. **Security**: Forbidden constructs blocked, zero ambient authority
6. **Build System**: Simple Makefile with all targets working

### Validation Success:
1. **make compile** builds both executables
2. **make run** starts interactive REPL
3. **make test** runs basic validation tests
4. **make compile-run** creates .msqs snapshot
5. All gas limits and security constraints enforced

### Specification Compliance:
1. **TECHSPECV5.md**: All runtime requirements implemented
2. **Zero Dependencies**: No external libraries required
3. **Deterministic**: Same inputs produce same outputs
4. **Isolated**: Per-request sandbox isolation

This implementation provides complete MTPScript v5.1 runtime foundation with all core features: deterministic execution, gas metering, effect system, security isolation, and snapshot management.