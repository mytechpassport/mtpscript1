/*
 * MTPScript HTTP Effects Implementation
 * Specification §7 - HttpOut Effect
 */

#ifndef MQUICKJS_HTTP_H
#define MQUICKJS_HTTP_H

#include "mquickjs.h"
#include <curl/curl.h>
#include <stdbool.h>
#include <stdint.h>

// HTTP request structure
typedef struct {
    char *method;       // HTTP method (GET, POST, etc.)
    char *url;          // Request URL
    char *headers;      // JSON string of headers
    char *body;         // Request body (optional)
    size_t body_size;   // Size of request body
    long timeout_ms;    // Request timeout in milliseconds
    bool verify_tls;    // Whether to verify TLS certificates
} MTPScriptHTTPRequest;

// HTTP size limits
#define MTPSCRIPT_HTTP_MAX_REQUEST_SIZE  (10 * 1024 * 1024)  // 10MB
#define MTPSCRIPT_HTTP_MAX_RESPONSE_SIZE (50 * 1024 * 1024)  // 50MB

// HTTP response structure
typedef struct {
    long status_code;   // HTTP status code
    char *headers;      // JSON string of response headers
    char *body;         // Response body
    char *error;        // Error message (if any)
} MTPScriptHTTPResponse;

// HTTP cache entry
typedef struct {
    uint8_t request_hash[32];  // SHA-256 of (seed, request)
    JSValue response;          // Cached JS response value
    bool has_response;
} MTPScriptHTTPCacheEntry;

// HTTP cache
typedef struct {
    MTPScriptHTTPCacheEntry entries[1024];  // Simple array cache
    int count;
    uint8_t execution_seed[32];
    bool has_seed;
} MTPScriptHTTPCache;

// HTTP request functions
MTPScriptHTTPRequest *mtpscript_http_request_new(const char *method, const char *url,
                                                const char *headers, const char *body,
                                                long timeout_ms);
void mtpscript_http_request_free(MTPScriptHTTPRequest *req);

MTPScriptHTTPResponse *mtpscript_http_request_execute(MTPScriptHTTPRequest *req);
void mtpscript_http_response_free(MTPScriptHTTPResponse *resp);

// HTTP cache management
MTPScriptHTTPCache *mtpscript_http_cache_new(void);
void mtpscript_http_cache_free(MTPScriptHTTPCache *cache);
JSValue mtpscript_http_cache_get(MTPScriptHTTPCache *cache, const uint8_t *request_hash);
void mtpscript_http_cache_put(MTPScriptHTTPCache *cache, const uint8_t *request_hash, JSValue response);
void mtpscript_http_cache_set_seed(MTPScriptHTTPCache *cache, const uint8_t *seed, size_t seed_len);

// Generate request hash for caching
void mtpscript_http_generate_request_hash(const uint8_t *seed, size_t seed_len,
                                         const MTPScriptHTTPRequest *req,
                                         uint8_t out_hash[32]);

// HTTP effect handler
JSValue mtpscript_http_out(JSContext *ctx, const uint8_t *seed, size_t seed_len, JSValue args);

// Register HTTP effects
void mtpscript_http_register_effects(JSContext *ctx);

#endif /* MQUICKJS_HTTP_H */
