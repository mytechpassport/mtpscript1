/* MTPScript Core Type Definitions
 *
 * This header defines the core types and structures used by the MTPScript
 * JavaScript engine, including values, contexts, and effect dispatching.
 */

#ifndef MTPSCRIPT_H
#define MTPSCRIPT_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Forward declaration of mtp_value_t */
typedef struct mtp_value mtp_value_t;

/* MTPScript value type enumeration */
typedef enum {
    MTP_VAL_UNDEFINED = 0,
    MTP_VAL_NULL,
    MTP_VAL_BOOLEAN,
    MTP_VAL_NUMBER,
    MTP_VAL_STRING,
    MTP_VAL_ARRAY,
    MTP_VAL_OBJECT,
    MTP_VAL_FUNCTION,
    MTP_VAL_ERROR
} mtp_value_type_t;

/* MTPScript value union */
typedef union {
    bool boolean_val;
    double number_val;
    char* string_val;
    struct {
        mtp_value_t* elements;
        uint32_t length;
    } array_val;
    struct {
        char** keys;
        mtp_value_t* values;
        uint32_t count;
    } object_val;
    struct {
        void* native_ptr;
        void (*destructor)(void*);
    } external_val;
} mtp_value_data_t;

/* MTPScript value structure */
struct mtp_value {
    mtp_value_type_t type;
    mtp_value_data_t data;
};

/* MTPScript execution context */
typedef struct {
    /* Execution state */
    bool deterministic_mode;        /* Whether deterministic caching is enabled */
    uint64_t gas_consumed;          /* Total gas consumed */
    uint64_t gas_limit;             /* Gas limit for current execution */
    
    /* Runtime state */
    void* runtime;                  /* Pointer to JS runtime */
    void* js_context;               /* Pointer to JS context */
    
    /* Effect system state */
    void* effect_cache;             /* Effect cache data */
    uint64_t effect_cache_hits;     /* Cache hit counter */
    uint64_t effect_cache_misses;   /* Cache miss counter */
    
    /* Error handling */
    mtp_value_t last_error;         /* Last error value */
    bool has_error;                 /* Whether an error occurred */
} mtp_context_t;

/* Effect handler function prototype */
typedef mtp_value_t (*effect_handler_fn)(
    mtp_context_t* ctx,
    const mtp_value_t* effect_args,
    uint32_t gas_limit
);

/* Main effect dispatch function */
mtp_value_t mtp_dispatch_effect(
    mtp_context_t* ctx,
    const char* effect_name,
    const mtp_value_t* effect_args,
    uint32_t gas_limit
);

/* Cache statistics functions */
void mtp_get_cache_stats(uint64_t* hits, uint64_t* misses);
void mtp_clear_effect_cache(void);

/* Context management functions */
mtp_context_t* mtp_create_context(void* runtime, void* js_context);
void mtp_destroy_context(mtp_context_t* ctx);
void mtp_set_deterministic_mode(mtp_context_t* ctx, bool enabled);
void mtp_set_gas_limit(mtp_context_t* ctx, uint64_t limit);

#ifdef __cplusplus
}
#endif

#endif /* MTPSCRIPT_H */