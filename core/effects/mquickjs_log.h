/*
 * MTPScript Log Effects Implementation
 * Specification §7 - Log Effect
 */

#ifndef MQUICKJS_LOG_H
#define MQUICKJS_LOG_H

#include "mquickjs.h"
#include <stdbool.h>
#include <stdint.h>

// Log levels
typedef enum {
    MTPSCRIPT_LOG_DEBUG,
    MTPSCRIPT_LOG_INFO,
    MTPSCRIPT_LOG_WARN,
    MTPSCRIPT_LOG_ERROR
} MTPScriptLogLevel;

// Log entry structure
typedef struct {
    MTPScriptLogLevel level;
    const char *message;
    const char *correlation_id;  // From request seed
    int64_t timestamp;          // Unix timestamp in milliseconds
    JSValue data;              // Additional structured data
} MTPScriptLogEntry;

// Log aggregation interface
typedef struct {
    void (*send_logs)(const char *logs_json, size_t count);  // Callback to send logs
    void *user_data;           // User data for callback
    bool enabled;              // Whether aggregation is enabled
} MTPScriptLogAggregator;

// Log effect handler
JSValue mtpscript_log_effect(JSContext *ctx, const uint8_t *seed, size_t seed_len, JSValue args);

// Register log effects
void mtpscript_log_register_effects(JSContext *ctx);

// Logging functions (for internal use)
void mtpscript_log_write(MTPScriptLogLevel level, const char *message,
                        const char *correlation_id, JSValue data);

// Log aggregation functions
void mtpscript_log_set_aggregator(MTPScriptLogAggregator *aggregator);
MTPScriptLogAggregator *mtpscript_log_get_aggregator(void);

// Convert log level to string
const char *mtpscript_log_level_to_string(MTPScriptLogLevel level);

#endif /* MQUICKJS_LOG_H */
