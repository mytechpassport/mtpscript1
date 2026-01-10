/*
 * MTPScript Common Types and Definitions
 * 
 * Core type definitions and shared structures for MTPScript runtime
 */

#ifndef MT5_COMMON_H
#define MT5_COMMON_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Forward declarations */
typedef struct JSRuntime JSRuntime;
typedef struct JSContext JSContext;

/* Result codes */
typedef enum {
    MT5_SUCCESS = 0,
    MT5_ERROR_GENERIC = -1,
    MT5_ERROR_OUT_OF_MEMORY = -2,
    MT5_ERROR_GAS_EXHAUSTED = -3,
    MT5_ERROR_FORBIDDEN_CONSTRUCT = -4,
    MT5_ERROR_SECURITY_VIOLATION = -5,
    MT5_ERROR_INVALID_CONTEXT = -6,
    MT5_ERROR_EFFECT_HANDLER_FAILED = -7
} mt5_result_t;

/* Default gas limit */
#define MT5_DEFAULT_GAS_LIMIT 10000000ULL

/* Runtime context structure */
typedef struct mt5_runtime_ctx {
    JSRuntime* js_rt;
    JSContext* js_ctx;
    
    /* Gas metering */
    uint64_t gas_limit;
    uint64_t gas_used;
    bool gas_exhausted;
    
    /* Execution state */
    uint32_t execution_depth;
    bool ambient_authority;
    
    /* Deterministic execution */
    bool deterministic_mode;
    uint8_t deterministic_seed[32];
    
    /* Security context */
    void* security_ctx;
    
    /* Effect system */
    void* effect_handlers;
    uint32_t effect_count;
} mt5_runtime_ctx_t;

/* Snapshot structure */
typedef struct mt5_snapshot {
    uint64_t gas_used;
    uint32_t execution_depth;
    bool ambient_authority;
    uint8_t hash[32];
    void* data;
    size_t data_size;
} mt5_snapshot_t;

/* Function prototypes */
mt5_runtime_ctx_t* mt5_init_runtime(void);
void mt5_cleanup_runtime(mt5_runtime_ctx_t* ctx);
mt5_result_t mt5_configure_gas_limit(mt5_runtime_ctx_t* ctx, uint64_t limit);
mt5_result_t mt5_configure_zero_ambient_authority(mt5_runtime_ctx_t* ctx);
bool mt5_check_gas_exhaustion(mt5_runtime_ctx_t* ctx);
void mt5_secure_wipe_context(mt5_runtime_ctx_t* ctx);

int mt5_contains_forbidden_constructs(const char* code);
mt5_snapshot_t* mt5_create_snapshot(mt5_runtime_ctx_t* ctx);
void mt5_free_snapshot(mt5_snapshot_t* snapshot);
void mt5_calculate_snapshot_hash(mt5_snapshot_t* snapshot, uint8_t* hash);
char* mt5_serialize_snapshot(mt5_snapshot_t* snapshot);

int mt5_register_console_effect(mt5_runtime_ctx_t* ctx);
int mt5_register_database_effect(mt5_runtime_ctx_t* ctx);
int mt5_get_registered_effect_count(mt5_runtime_ctx_t* ctx);
void mt5_cleanup_effects(mt5_runtime_ctx_t* ctx);

#ifdef __cplusplus
}
#endif

#endif /* MT5_COMMON_H */