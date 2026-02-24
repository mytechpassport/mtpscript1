/*
 * MTPScript API Routing System Implementation
 * Specification §8 - API System (First-Class)
 */

#include "mquickjs_api.h"
#include <string.h>
#include <stdlib.h>
#include <stdio.h>
#include <stdbool.h>
#include <stdint.h>
#include <strings.h> // for strcasecmp

// Simple URL decoding
static void mtpscript_url_decode(char *str) {
    char *src = str;
    char *dst = str;

    while (*src) {
        if (*src == '%' && *(src + 1) && *(src + 2)) {
            // Convert %XX to character
            char hex[3] = {*(src + 1), *(src + 2), '\0'};
            *dst++ = (char)strtol(hex, NULL, 16);
            src += 3;
        } else if (*src == '+') {
            *dst++ = ' ';
            src++;
        } else {
            *dst++ = *src++;
        }
    }
    *dst = '\0';
}

// Parse query parameters from query string
static void mtpscript_parse_query_params(const char *query_string,
                                       MTPScriptRouteParam **params,
                                       int *param_count) {
    if (!query_string || !*query_string) return;

    char *query_copy = strdup(query_string);
    char *token = strtok(query_copy, "&");
    int count = 0;

    while (token) {
        char *eq = strchr(token, '=');
        if (eq) {
            *eq = '\0';
            char *name = token;
            char *value = eq + 1;

            // URL decode (simplified)
            mtpscript_url_decode(name);
            mtpscript_url_decode(value);

            // Add parameter
            *params = realloc(*params, (count + 1) * sizeof(MTPScriptRouteParam));
            (*params)[count].name = strdup(name);
            (*params)[count].value = strdup(value);
            count++;
        }
        token = strtok(NULL, "&");
    }

    *param_count = count;
    free(query_copy);
}

// Simple route pattern matching
static bool mtpscript_route_pattern_match(const char *pattern, const char *path,
                                         MTPScriptRouteParam *params, int param_count) {
    const char *pattern_ptr = pattern;
    const char *path_ptr = path;
    int param_index = 0;

    while (*pattern_ptr && *path_ptr) {
        if (*pattern_ptr == ':') {
            // Parameter segment
            pattern_ptr++; // Skip ':'
            const char *param_start = pattern_ptr;

            // Find end of parameter name
            while (*pattern_ptr && *pattern_ptr != '/' && *pattern_ptr != '?') {
                pattern_ptr++;
            }

            // Extract parameter value from path
            const char *value_start = path_ptr;
            const char *value_end = path_ptr;

            // Find end of parameter value (until next '/' or end)
            while (*value_end && *value_end != '/' && *value_end != '?') {
                value_end++;
            }

            // Store parameter value
            if (param_index < param_count) {
                int value_len = value_end - value_start;
                params[param_index].value = realloc(params[param_index].value, value_len + 1);
                memcpy(params[param_index].value, value_start, value_len);
                params[param_index].value[value_len] = '\0';
                param_index++;
            }

            path_ptr = value_end;
        } else if (*pattern_ptr == *path_ptr) {
            pattern_ptr++;
            path_ptr++;
        } else {
            return false;
        }
    }

    return *pattern_ptr == '\0' && (*path_ptr == '\0' || *path_ptr == '/' || *path_ptr == '?');
}

// Route registry functions
MTPScriptRouteRegistry *mtpscript_route_registry_new(void) {
    MTPScriptRouteRegistry *registry = calloc(1, sizeof(MTPScriptRouteRegistry));
    if (!registry) return NULL;

    registry->route_capacity = 16;
    registry->routes = calloc(registry->route_capacity, sizeof(MTPScriptRoute));
    if (!registry->routes) {
        free(registry);
        return NULL;
    }

    return registry;
}

void mtpscript_route_registry_free(MTPScriptRouteRegistry *registry) {
    if (!registry) return;

    for (int i = 0; i < registry->route_count; i++) {
        MTPScriptRoute *route = &registry->routes[i];
        free(route->path_pattern);
        free(route->handler_name);
        for (int j = 0; j < route->path_param_count; j++) {
            free(route->path_params[j].name);
        }
        free(route->path_params);
    }

    free(registry->routes);
    free(registry);
}

bool mtpscript_route_registry_add(MTPScriptRouteRegistry *registry,
                                 MTPScriptHTTPMethod method,
                                 const char *path_pattern,
                                 const char *handler_name) {
    if (!registry || registry->route_count >= registry->route_capacity) {
        return false;
    }

    MTPScriptRoute *route = &registry->routes[registry->route_count++];
    route->method = method;
    route->path_pattern = strdup(path_pattern);
    route->handler_name = strdup(handler_name);
    route->path_params = NULL;
    route->path_param_count = 0;

    // Parse path parameters from pattern
    const char *ptr = path_pattern;
    while ((ptr = strchr(ptr, ':')) != NULL) {
        ptr++; // Skip ':'
        const char *end = ptr;
        while (*end && *end != '/' && *end != '?') end++;

        // Allocate space for parameter name
        int param_len = end - ptr;
        char *param_name = malloc(param_len + 1);
        memcpy(param_name, ptr, param_len);
        param_name[param_len] = '\0';

        // Add to path params
        route->path_params = realloc(route->path_params,
                                   (route->path_param_count + 1) * sizeof(MTPScriptRouteParam));
        route->path_params[route->path_param_count].name = param_name;
        route->path_params[route->path_param_count].value = NULL;
        route->path_param_count++;

        ptr = end;
    }

    return true;
}

// Calculate route specificity (higher = more specific)
static int mtpscript_route_specificity(const char *pattern) {
    int specificity = 0;
    const char *ptr = pattern;

    while (*ptr) {
        if (*ptr == '/') {
            specificity += 10; // Path segments
        } else if (*ptr == ':') {
            specificity += 5;  // Parameters
        } else {
            specificity += 1;  // Literal characters
        }
        ptr++;
    }

    return specificity;
}

// Route matching and parameter extraction with priority (most-specific wins)
MTPScriptRoute *mtpscript_route_match(MTPScriptRouteRegistry *registry,
                                     MTPScriptHTTPMethod method,
                                     const char *path,
                                     MTPScriptRouteParam **path_params,
                                     int *path_param_count) {
    if (!registry) return NULL;

    MTPScriptRoute *best_match = NULL;
    int best_specificity = -1;

    for (int i = 0; i < registry->route_count; i++) {
        MTPScriptRoute *route = &registry->routes[i];
        if (route->method != method) continue;

        // Test pattern match
        if (mtpscript_route_pattern_match(route->path_pattern, path, route->path_params, route->path_param_count)) {
            int specificity = mtpscript_route_specificity(route->path_pattern);

            // Choose the most specific match
            if (specificity > best_specificity) {
                best_match = route;
                best_specificity = specificity;
            }
        }
    }

    if (best_match) {
        *path_params = best_match->path_params;
        *path_param_count = best_match->path_param_count;
    }

    return best_match;
}

// Parse headers from raw header string
static void mtpscript_parse_headers(const char *headers_raw,
                                   MTPScriptRouteParam **headers,
                                   int *header_count) {
    if (!headers_raw || !*headers_raw) return;

    char *headers_copy = strdup(headers_raw);
    char *line = strtok(headers_copy, "\r\n");

    while (line) {
        char *colon = strchr(line, ':');
        if (colon) {
            *colon = '\0';
            char *name = line;
            char *value = colon + 1;

            // Trim whitespace
            while (*value == ' ' || *value == '\t') value++;

            // Trim trailing whitespace from value
            char *end = value + strlen(value) - 1;
            while (end > value && (*end == ' ' || *end == '\t' || *end == '\r' || *end == '\n')) {
                *end-- = '\0';
            }

            // Add header (case-insensitive storage)
            *headers = realloc(*headers, (*header_count + 1) * sizeof(MTPScriptRouteParam));
            (*headers)[*header_count].name = strdup(name);
            (*headers)[*header_count].value = strdup(value);
            (*header_count)++;
        }
        line = strtok(NULL, "\r\n");
    }

    free(headers_copy);
}

// Request parsing
MTPScriptAPIRequest *mtpscript_api_request_parse(const char *method_str,
                                                const char *path_with_query,
                                                const char *body,
                                                const char *content_type) {
    MTPScriptAPIRequest *request = calloc(1, sizeof(MTPScriptAPIRequest));
    if (!request) return NULL;

    request->method = mtpscript_http_method_from_string(method_str);
    request->body = body ? strdup(body) : NULL;
    request->body_size = body ? strlen(body) : 0;
    request->content_type = content_type ? strdup(content_type) : NULL;

    // Parse path and query
    const char *query_start = strchr(path_with_query, '?');
    if (query_start) {
        int path_len = query_start - path_with_query;
        request->path = malloc(path_len + 1);
        memcpy(request->path, path_with_query, path_len);
        request->path[path_len] = '\0';

        request->query_string = strdup(query_start + 1);

        // Parse query parameters
        mtpscript_parse_query_params(request->query_string,
                                   &request->query_params,
                                   &request->query_param_count);
    } else {
        request->path = strdup(path_with_query);
        request->query_string = NULL;
    }

    return request;
}

// Enhanced request parsing with headers
MTPScriptAPIRequest *mtpscript_api_request_parse_full(const char *method_str,
                                                    const char *path_with_query,
                                                    const char *body,
                                                    const char *content_type,
                                                    const char *headers_raw) {
    MTPScriptAPIRequest *request = mtpscript_api_request_parse(method_str, path_with_query, body, content_type);
    if (!request) return NULL;

    // Parse headers
    if (headers_raw) {
        mtpscript_parse_headers(headers_raw, &request->headers, &request->header_count);
    }

    return request;
}

void mtpscript_api_request_free(MTPScriptAPIRequest *request) {
    if (!request) return;

    free(request->path);
    free(request->query_string);
    free(request->body);
    free(request->content_type);

    for (int i = 0; i < request->query_param_count; i++) {
        free(request->query_params[i].name);
        free(request->query_params[i].value);
    }
    free(request->query_params);

    free(request);
}

// Response generation
MTPScriptAPIResponse *mtpscript_api_response_new(void) {
    MTPScriptAPIResponse *response = calloc(1, sizeof(MTPScriptAPIResponse));
    if (!response) return NULL;

    response->status_code = 200;
    response->content_type = strdup("application/json");
    return response;
}

void mtpscript_api_response_free(MTPScriptAPIResponse *response) {
    if (!response) return;

    free(response->body);
    free(response->content_type);

    for (int i = 0; i < response->header_count; i++) {
        free(response->headers[i].name);
        free(response->headers[i].value);
    }
    free(response->headers);

    free(response);
}

void mtpscript_api_response_set_json(MTPScriptAPIResponse *response, JSValue json_value, JSContext *ctx) {
    // Simplified JSON serialization for demonstration
    // In a full implementation, this would use QuickJS JSON.stringify
    free(response->body);
    response->body = strdup("{\"message\": \"JSON response generated\"}");
    response->body_size = strlen(response->body);

    // Set content type if not already set
    if (!response->content_type) {
        response->content_type = strdup("application/json");
    }

    // Set content-length header
    char content_length[32];
    sprintf(content_length, "%zu", response->body_size);
    mtpscript_api_set_header(response, "Content-Length", content_length);
}

void mtpscript_api_response_set_status(MTPScriptAPIResponse *response, int status_code) {
    response->status_code = status_code;
}

// Utility functions
MTPScriptHTTPMethod mtpscript_http_method_from_string(const char *method_str) {
    if (strcmp(method_str, "GET") == 0) return MTPSCRIPT_HTTP_GET;
    if (strcmp(method_str, "POST") == 0) return MTPSCRIPT_HTTP_POST;
    if (strcmp(method_str, "PUT") == 0) return MTPSCRIPT_HTTP_PUT;
    if (strcmp(method_str, "DELETE") == 0) return MTPSCRIPT_HTTP_DELETE;
    if (strcmp(method_str, "PATCH") == 0) return MTPSCRIPT_HTTP_PATCH;
    return MTPSCRIPT_HTTP_GET; // Default
}

const char *mtpscript_http_method_to_string(MTPScriptHTTPMethod method) {
    switch (method) {
        case MTPSCRIPT_HTTP_GET: return "GET";
        case MTPSCRIPT_HTTP_POST: return "POST";
        case MTPSCRIPT_HTTP_PUT: return "PUT";
        case MTPSCRIPT_HTTP_DELETE: return "DELETE";
        case MTPSCRIPT_HTTP_PATCH: return "PATCH";
        default: return "GET";
    }
}

// JSON parsing and validation
JSValue mtpscript_api_parse_json_body(JSContext *ctx, const char *body, size_t body_size) {
    if (!body || body_size == 0) {
        return JS_NULL;
    }

    // Simplified JSON parsing - just return a string for demonstration
    // In a full implementation, this would use QuickJS JSON.parse
    return JS_NewStringLen(ctx, body, body_size);
}

bool mtpscript_api_validate_json(JSContext *ctx, JSValue json_value) {
    // Basic validation - check if it's a valid JSON value
    return !JS_IsException(json_value);
}

// Header access functions (case-insensitive)
const char *mtpscript_api_get_header(const MTPScriptAPIRequest *request, const char *name) {
    if (!request || !name) return NULL;

    for (int i = 0; i < request->header_count; i++) {
        if (strcasecmp(request->headers[i].name, name) == 0) {
            return request->headers[i].value;
        }
    }
    return NULL;
}

void mtpscript_api_set_header(MTPScriptAPIResponse *response, const char *name, const char *value) {
    if (!response || !name) return;

    // Resize headers array
    response->headers = realloc(response->headers,
                               (response->header_count + 1) * sizeof(MTPScriptRouteParam));

    MTPScriptRouteParam *header = &response->headers[response->header_count++];
    header->name = strdup(name);
    header->value = value ? strdup(value) : NULL;
}

// Response generation functions
JSValue mtpscript_api_respond_json(JSContext *ctx, JSValue json_data) {
    // Create response object with JSON serialization
    JSValue response = JS_NewObject(ctx);

    // Set status code to 200
    JSValue status_val = JS_NewInt32(ctx, 200);
    JS_SetPropertyStr(ctx, response, "statusCode", status_val);

    // Simplified body
    JSValue body_val = JS_NewString(ctx, "{\"message\": \"JSON response\"}");
    JS_SetPropertyStr(ctx, response, "body", body_val);

    // Set content type
    JSValue content_type_val = JS_NewString(ctx, "application/json");
    JS_SetPropertyStr(ctx, response, "contentType", content_type_val);

    return response;
}

JSValue mtpscript_api_respond_status(JSContext *ctx, int status_code, const char *message) {
    JSValue response = JS_NewObject(ctx);

    JSValue status_val = JS_NewInt32(ctx, status_code);
    JS_SetPropertyStr(ctx, response, "statusCode", status_val);

    if (message) {
        JSValue body_val = JS_NewString(ctx, message);
        JS_SetPropertyStr(ctx, response, "body", body_val);
    }

    return response;
}

JSValue mtpscript_api_respond_error(JSContext *ctx, int status_code, const char *error_type, const char *message) {
    // Create deterministic error response per §16
    char error_body[256];
    sprintf(error_body, "{\"type\": \"%s\", \"message\": \"%s\"}",
            error_type ? error_type : "Error",
            message ? message : "An error occurred");

    JSValue response = JS_NewObject(ctx);
    JSValue status_val = JS_NewInt32(ctx, status_code);
    JS_SetPropertyStr(ctx, response, "statusCode", status_val);

    // Set content type to JSON
    JSValue content_type_val = JS_NewString(ctx, "application/json");
    JS_SetPropertyStr(ctx, response, "contentType", content_type_val);

    // Set error body
    JSValue body_val = JS_NewString(ctx, error_body);
    JS_SetPropertyStr(ctx, response, "body", body_val);

    return response;
}

// API routing handler
JSValue mtpscript_api_route(JSContext *ctx, MTPScriptRouteRegistry *registry,
                           MTPScriptAPIRequest *request) {
    MTPScriptRouteParam *path_params = NULL;
    int path_param_count = 0;

    MTPScriptRoute *route = mtpscript_route_match(registry, request->method, request->path,
                                                 &path_params, &path_param_count);

    if (!route) {
        // Return 404
        MTPScriptAPIResponse *response = mtpscript_api_response_new();
        response->status_code = 404;
        free(response->body);
        response->body = strdup("{\"error\": \"Not Found\"}");

        // Convert to JS object for return
        JSValue js_response = JS_NewObject(ctx);
        JSValue status_val = JS_NewInt32(ctx, response->status_code);
        JSValue body_val = JS_NewString(ctx, response->body);

        JS_SetPropertyStr(ctx, js_response, "statusCode", status_val);
        JS_SetPropertyStr(ctx, js_response, "body", body_val);

        mtpscript_api_response_free(response);
        return js_response;
    }

    // Call the handler function (simplified - would need to pass params)
    // For now, just return a success response
    MTPScriptAPIResponse *response = mtpscript_api_response_new();
    free(response->body);
    response->body = strdup("{\"message\": \"API route matched\"}");

    // Convert to JS object
    JSValue js_response = JS_NewObject(ctx);
    JSValue status_val = JS_NewInt32(ctx, response->status_code);
    JSValue body_val = JS_NewString(ctx, response->body);

    JS_SetPropertyStr(ctx, js_response, "statusCode", status_val);
    JS_SetPropertyStr(ctx, js_response, "body", body_val);

    mtpscript_api_response_free(response);
    return js_response;
}

// Register API routing
void mtpscript_api_register(JSContext *ctx, MTPScriptRouteRegistry *registry) {
    // Add some default routes for testing
    mtpscript_route_registry_add(registry, MTPSCRIPT_HTTP_GET, "/health", "health_handler");
    mtpscript_route_registry_add(registry, MTPSCRIPT_HTTP_GET, "/users/:id", "get_user_handler");
    mtpscript_route_registry_add(registry, MTPSCRIPT_HTTP_POST, "/users", "create_user_handler");
}
