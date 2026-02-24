/*
 * MTPScript Log Effects Implementation
 * Specification §7 - Log Effect
 */

#include "mquickjs_log.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <sys/time.h>

// Thread-local log aggregator
__thread MTPScriptLogAggregator *g_log_aggregator = NULL;

// Log aggregation functions
void mtpscript_log_set_aggregator(MTPScriptLogAggregator *aggregator) {
    g_log_aggregator = aggregator;
}

MTPScriptLogAggregator *mtpscript_log_get_aggregator(void) {
    return g_log_aggregator;
}

// Convert log level to string
const char *mtpscript_log_level_to_string(MTPScriptLogLevel level) {
    switch (level) {
        case MTPSCRIPT_LOG_DEBUG: return "DEBUG";
        case MTPSCRIPT_LOG_INFO:  return "INFO";
        case MTPSCRIPT_LOG_WARN:  return "WARN";
        case MTPSCRIPT_LOG_ERROR: return "ERROR";
        default: return "UNKNOWN";
    }
}

// Write log entry to stdout and/or aggregator
void mtpscript_log_write(MTPScriptLogLevel level, const char *message,
                        const char *correlation_id, JSValue data) {
    // Get current timestamp
    struct timeval tv;
    gettimeofday(&tv, NULL);
    int64_t timestamp = (int64_t)tv.tv_sec * 1000 + tv.tv_usec / 1000;

    // Create structured JSON log entry
    char *log_json = NULL;
    size_t log_size = 0;
    FILE *log_stream = open_memstream(&log_json, &log_size);

    if (log_stream) {
        fprintf(log_stream, "{\"timestamp\":%lld,\"level\":\"%s\",\"message\":\"%s\"",
                timestamp,
                mtpscript_log_level_to_string(level),
                message ? message : "");

        if (correlation_id) {
            fprintf(log_stream, ",\"correlationId\":\"%s\"", correlation_id);
        }

        // Add additional data if provided
        if (!JS_IsUndefined(data) && !JS_IsNull(data)) {
            // In a full implementation, we'd serialize the JSValue to JSON
            // For now, just indicate that additional data is present
            fprintf(log_stream, ",\"hasAdditionalData\":true");
        }

        fprintf(log_stream, "}\n");
        fclose(log_stream);

        // Send to aggregator if configured
        if (g_log_aggregator && g_log_aggregator->enabled && g_log_aggregator->send_logs) {
            g_log_aggregator->send_logs(log_json, 1);
        } else {
            // Default: write to stdout
            printf("%s", log_json);
            fflush(stdout);
        }

        free(log_json);
    }
}

// Log effect handler - supports aggregation interface
JSValue mtpscript_log_effect(JSContext *ctx, const uint8_t *seed, size_t seed_len, JSValue args) {
    // Generate correlation ID from seed
    char correlation_id[65];
    if (seed_len >= 32) {
        for (int i = 0; i < 32; i++) {
            sprintf(correlation_id + (i * 2), "%02x", seed[i]);
        }
        correlation_id[64] = '\0';
    } else {
        strcpy(correlation_id, "unknown");
    }

    // Simple implementation with aggregation support
    const char *message = "Log effect called with aggregation support";
    MTPScriptLogLevel level = MTPSCRIPT_LOG_INFO;

    // Create structured data for demonstration
    JSValue data = JS_NewObject(ctx);
    JSValue aggregation_val = JS_NewString(ctx, "CloudWatch");
    JSValue enabled_val = JS_NewBool(g_log_aggregator ? g_log_aggregator->enabled : false);

    JS_SetPropertyStr(ctx, data, "aggregationTarget", aggregation_val);
    JS_SetPropertyStr(ctx, data, "aggregationEnabled", enabled_val);

    // Write log entry with structured data support
    mtpscript_log_write(level, message, correlation_id, data);

    // Return undefined (logging doesn't return a value)
    return JS_UNDEFINED;
}

// Register log effects
void mtpscript_log_register_effects(JSContext *ctx) {
    // Register Log effect
    JS_RegisterEffect(ctx, "Log", mtpscript_log_effect);
}
