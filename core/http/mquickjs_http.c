/*
 * MTPScript HTTP Effects Implementation
 * Specification §7 - HttpOut Effect
 */

#include "mquickjs_http.h"
#include "mquickjs_crypto.h"
#include <string.h>
#include <stdlib.h>
#include <stdio.h>
#include <stdbool.h>
#include <stdint.h>
#include <curl/curl.h>
#include <openssl/sha.h>

// Thread-local storage for HTTP cache
__thread MTPScriptHTTPCache *g_http_cache = NULL;

// cURL write callback for response body with size limits
static size_t http_write_callback(void *contents, size_t size, size_t nmemb, void *userp) {
    size_t realsize = size * nmemb;
    MTPScriptHTTPResponse *resp = (MTPScriptHTTPResponse *)userp;

    size_t current_size = resp->body ? strlen(resp->body) : 0;
    if (current_size + realsize > MTPSCRIPT_HTTP_MAX_RESPONSE_SIZE) {
        // Response too large
        return 0;
    }

    char *new_body = realloc(resp->body, current_size + realsize + 1);
    if (!new_body) return 0;

    resp->body = new_body;
    if (!resp->body[0]) resp->body[0] = '\0'; // Initialize if empty
    strncat(resp->body, contents, realsize);

    return realsize;
}

// cURL header callback
static size_t http_header_callback(void *contents, size_t size, size_t nmemb, void *userp) {
    size_t realsize = size * nmemb;
    MTPScriptHTTPResponse *resp = (MTPScriptHTTPResponse *)userp;

    // For simplicity, we'll just store the raw headers
    // In a full implementation, we'd parse them into a JSON object
    char *new_headers = realloc(resp->headers, strlen(resp->headers ? resp->headers : "") + realsize + 1);
    if (!new_headers) return 0;

    resp->headers = new_headers;
    if (!resp->headers[0]) resp->headers[0] = '\0';
    strncat(resp->headers, contents, realsize);

    return realsize;
}

// HTTP request functions
MTPScriptHTTPRequest *mtpscript_http_request_new(const char *method, const char *url,
                                                const char *headers, const char *body,
                                                long timeout_ms) {
    MTPScriptHTTPRequest *req = calloc(1, sizeof(MTPScriptHTTPRequest));
    if (!req) return NULL;

    req->method = strdup(method ? method : "GET");
    req->url = strdup(url ? url : "");
    req->headers = headers ? strdup(headers) : NULL;
    req->body = body ? strdup(body) : NULL;
    req->body_size = body ? strlen(body) : 0;
    req->timeout_ms = timeout_ms > 0 ? timeout_ms : 30000; // Default 30 seconds
    req->verify_tls = true; // Enable TLS verification by default

    // Check request body size limit
    if (req->body_size > MTPSCRIPT_HTTP_MAX_REQUEST_SIZE) {
        mtpscript_http_request_free(req);
        return NULL;
    }

    return req;
}

void mtpscript_http_request_free(MTPScriptHTTPRequest *req) {
    if (!req) return;

    free(req->method);
    free(req->url);
    free(req->headers);
    free(req->body);
    free(req);
}

MTPScriptHTTPResponse *mtpscript_http_request_execute(MTPScriptHTTPRequest *req) {
    if (!req || !req->url) return NULL;

    MTPScriptHTTPResponse *resp = calloc(1, sizeof(MTPScriptHTTPResponse));
    if (!resp) return NULL;

    CURL *curl = curl_easy_init();
    if (!curl) {
        free(resp);
        return NULL;
    }

    // Set URL
    curl_easy_setopt(curl, CURLOPT_URL, req->url);

    // Set method
    if (strcmp(req->method, "POST") == 0) {
        curl_easy_setopt(curl, CURLOPT_POST, 1L);
        if (req->body) {
            curl_easy_setopt(curl, CURLOPT_POSTFIELDS, req->body);
        }
    } else if (strcmp(req->method, "PUT") == 0) {
        curl_easy_setopt(curl, CURLOPT_CUSTOMREQUEST, "PUT");
        if (req->body) {
            curl_easy_setopt(curl, CURLOPT_POSTFIELDS, req->body);
        }
    } else if (strcmp(req->method, "DELETE") == 0) {
        curl_easy_setopt(curl, CURLOPT_CUSTOMREQUEST, "DELETE");
    } else if (strcmp(req->method, "PATCH") == 0) {
        curl_easy_setopt(curl, CURLOPT_CUSTOMREQUEST, "PATCH");
        if (req->body) {
            curl_easy_setopt(curl, CURLOPT_POSTFIELDS, req->body);
        }
    }
    // Default is GET

    // Set headers (simplified - in real implementation, parse JSON headers)
    struct curl_slist *header_list = NULL;
    if (req->headers) {
        // For now, just set Content-Type if body exists
        if (req->body) {
            header_list = curl_slist_append(header_list, "Content-Type: application/json");
        }
    } else if (req->body) {
        header_list = curl_slist_append(header_list, "Content-Type: application/json");
    }

    if (header_list) {
        curl_easy_setopt(curl, CURLOPT_HTTPHEADER, header_list);
    }

    // Set callbacks
    curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, http_write_callback);
    curl_easy_setopt(curl, CURLOPT_WRITEDATA, resp);
    curl_easy_setopt(curl, CURLOPT_HEADERFUNCTION, http_header_callback);
    curl_easy_setopt(curl, CURLOPT_HEADERDATA, resp);

    // Set timeout
    curl_easy_setopt(curl, CURLOPT_TIMEOUT_MS, req->timeout_ms);

    // Configure TLS certificate validation
    if (req->verify_tls) {
        curl_easy_setopt(curl, CURLOPT_SSL_VERIFYPEER, 1L);
        curl_easy_setopt(curl, CURLOPT_SSL_VERIFYHOST, 2L);
        // Use system CA certificates
        curl_easy_setopt(curl, CURLOPT_CAINFO, NULL);
        curl_easy_setopt(curl, CURLOPT_CAPATH, NULL);
    } else {
        // Disable SSL verification (for development/testing only)
        curl_easy_setopt(curl, CURLOPT_SSL_VERIFYPEER, 0L);
        curl_easy_setopt(curl, CURLOPT_SSL_VERIFYHOST, 0L);
    }

    // Perform request
    CURLcode res = curl_easy_perform(curl);

    if (res != CURLE_OK) {
        resp->error = strdup(curl_easy_strerror(res));
        resp->status_code = 0; // Error
    } else {
        // Get status code
        curl_easy_getinfo(curl, CURLINFO_RESPONSE_CODE, &resp->status_code);
    }

    // Cleanup
    curl_easy_cleanup(curl);
    if (header_list) {
        curl_slist_free_all(header_list);
    }

    return resp;
}

void mtpscript_http_response_free(MTPScriptHTTPResponse *resp) {
    if (!resp) return;

    free(resp->headers);
    free(resp->body);
    free(resp->error);
    free(resp);
}

// HTTP cache management
MTPScriptHTTPCache *mtpscript_http_cache_new(void) {
    if (g_http_cache) return g_http_cache;

    g_http_cache = calloc(1, sizeof(MTPScriptHTTPCache));
    return g_http_cache;
}

void mtpscript_http_cache_free(MTPScriptHTTPCache *cache) {
    if (!cache) return;

    // Note: JSValue objects should be freed by the JS runtime
    free(cache);

    if (cache == g_http_cache) {
        g_http_cache = NULL;
    }
}

JSValue mtpscript_http_cache_get(MTPScriptHTTPCache *cache, const uint8_t *request_hash) {
    if (!cache || !cache->has_seed) return JS_UNDEFINED;

    for (int i = 0; i < cache->count; i++) {
        if (memcmp(cache->entries[i].request_hash, request_hash, 32) == 0) {
            return cache->entries[i].response;
        }
    }
    return JS_UNDEFINED;
}

void mtpscript_http_cache_put(MTPScriptHTTPCache *cache, const uint8_t *request_hash, JSValue response) {
    if (!cache || !cache->has_seed || cache->count >= 1024) return;

    // Simple eviction: replace oldest entry if full
    int index = cache->count < 1024 ? cache->count : 0;
    if (cache->count >= 1024) {
        // Note: Old JSValue would need to be freed, but we don't have context here
    } else {
        cache->count++;
    }

    memcpy(cache->entries[index].request_hash, request_hash, 32);
    cache->entries[index].response = response;
    cache->entries[index].has_response = true;
}

void mtpscript_http_cache_set_seed(MTPScriptHTTPCache *cache, const uint8_t *seed, size_t seed_len) {
    if (!cache || seed_len != 32) return;

    memcpy(cache->execution_seed, seed, 32);
    cache->has_seed = true;
}

// Generate request hash for caching
void mtpscript_http_generate_request_hash(const uint8_t *seed, size_t seed_len,
                                         const MTPScriptHTTPRequest *req,
                                         uint8_t out_hash[32]) {
    uint8_t hash_input[4096];
    size_t hash_len = 0;

    // Add seed
    if (seed_len > 0) {
        memcpy(hash_input + hash_len, seed, seed_len);
        hash_len += seed_len;
    }

    // Add method
    if (req->method) {
        memcpy(hash_input + hash_len, req->method, strlen(req->method));
        hash_len += strlen(req->method);
    }

    // Add URL
    if (req->url) {
        memcpy(hash_input + hash_len, req->url, strlen(req->url));
        hash_len += strlen(req->url);
    }

    // Add headers
    if (req->headers) {
        memcpy(hash_input + hash_len, req->headers, strlen(req->headers));
        hash_len += strlen(req->headers);
    }

    // Add body
    if (req->body) {
        memcpy(hash_input + hash_len, req->body, strlen(req->body));
        hash_len += strlen(req->body);
    }

    SHA256(hash_input, hash_len, out_hash);
}

// Serialize request to canonical form for caching
static char *mtpscript_http_serialize_request(const MTPScriptHTTPRequest *req) {
    // Create canonical representation of the request
    // Format: METHOD URL\nHEADERS\nBODY
    size_t total_size = strlen(req->method) + 1 + strlen(req->url) + 2; // method + space + url + \n\n

    if (req->headers) total_size += strlen(req->headers) + 1; // headers + \n
    if (req->body) total_size += req->body_size + 1; // body + \n

    char *serialized = malloc(total_size + 1);
    if (!serialized) return NULL;

    sprintf(serialized, "%s %s\n", req->method, req->url);

    if (req->headers) {
        strcat(serialized, req->headers);
        strcat(serialized, "\n");
    } else {
        strcat(serialized, "\n");
    }

    if (req->body) {
        strcat(serialized, req->body);
        strcat(serialized, "\n");
    } else {
        strcat(serialized, "\n");
    }

    return serialized;
}

// HTTP effect handler
JSValue mtpscript_http_out(JSContext *ctx, const uint8_t *seed, size_t seed_len, JSValue args) {
    MTPScriptHTTPCache *cache = mtpscript_http_cache_new();

    if (!cache) {
        return JS_ThrowError(ctx, JS_CLASS_INTERNAL_ERROR, "HTTP system not initialized");
    }

    // Set execution seed for caching
    mtpscript_http_cache_set_seed(cache, seed, seed_len);

    // Simple implementation: GET request to httpbin.org with TLS validation and size limits
    MTPScriptHTTPRequest *req = mtpscript_http_request_new("GET",
                                                         "https://httpbin.org/get",
                                                         "Accept: application/json\r\nUser-Agent: MTPScript/1.0",
                                                         NULL, 10000);

    // Enable TLS verification
    req->verify_tls = true;

    // Generate request hash
    uint8_t request_hash[32];
    mtpscript_http_generate_request_hash(seed, seed_len, req, request_hash);

    // Check cache first
    JSValue cached_response = mtpscript_http_cache_get(cache, request_hash);
    if (!JS_IsUndefined(cached_response)) {
        mtpscript_http_request_free(req);
        return cached_response;
    }

    // Execute request
    MTPScriptHTTPResponse *resp = mtpscript_http_request_execute(req);
    mtpscript_http_request_free(req);

    if (!resp) {
        return JS_ThrowError(ctx, JS_CLASS_INTERNAL_ERROR, "Failed to execute HTTP request");
    }

    // Check response size limit
    if (resp->body && strlen(resp->body) > MTPSCRIPT_HTTP_MAX_RESPONSE_SIZE) {
        mtpscript_http_response_free(resp);
        return JS_ThrowError(ctx, JS_CLASS_INTERNAL_ERROR, "Response body too large");
    }

    // Convert response to JS object
    JSValue js_response = JS_NewObject(ctx);

    // Add status code
    JSValue status_val = JS_NewInt32(ctx, resp->status_code);
    JS_SetPropertyStr(ctx, js_response, "statusCode", status_val);

    // Add headers
    JSValue headers_val = JS_NewString(ctx, resp->headers ? resp->headers : "");
    JS_SetPropertyStr(ctx, js_response, "headers", headers_val);

    // Add body
    JSValue body_val = JS_NewString(ctx, resp->body ? resp->body : "");
    JS_SetPropertyStr(ctx, js_response, "body", body_val);

    // Add error if any
    if (resp->error) {
        JSValue error_val = JS_NewString(ctx, resp->error);
        JS_SetPropertyStr(ctx, js_response, "error", error_val);
    }

    // Cache the response
    mtpscript_http_cache_put(cache, request_hash, js_response);

    mtpscript_http_response_free(resp);

    return js_response;
}

// Register HTTP effects
void mtpscript_http_register_effects(JSContext *ctx) {
    // Initialize HTTP cache
    mtpscript_http_cache_new();

    // Initialize cURL (only once)
    static bool curl_initialized = false;
    if (!curl_initialized) {
        curl_global_init(CURL_GLOBAL_DEFAULT);
        curl_initialized = true;
    }

    // Register HttpOut effect
    JS_RegisterEffect(ctx, "HttpOut", mtpscript_http_out);
}
