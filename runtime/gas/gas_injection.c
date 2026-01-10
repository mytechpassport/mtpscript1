#include <stdlib.h>
#include <string.h>
#include <stdio.h>

// MTPScript Runtime Configuration
typedef struct {
    uint64_t gas_limit;
    uint64_t gas_used;
    uint8_t deterministic_seed[32];
    int gas_exhausted; // Using int instead of bool for C compatibility
    int per_request_isolation; // Using int instead of bool for C compatibility
} MTPRuntime;

typedef struct {
    MTPRuntime *runtime;
    void *heap;
    size_t heap_size;
    int pci_data_touched; // Using int instead of bool for C compatibility
} MTPContext;

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
    ctx->runtime->gas_exhausted = 0;
    
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