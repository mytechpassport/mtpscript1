/*
 * MTPScript API Routing System
 * Specification §8 - API System (First-Class)
 */

#ifndef MQUICKJS_API_H
#define MQUICKJS_API_H

#include "mquickjs.h"
#include <stdbool.h>
#include <stdint.h>

// HTTP methods
typedef enum {
    MTPSCRIPT_HTTP_GET,
    MTPSCRIPT_HTTP_POST,
    MTPSCRIPT_HTTP_PUT,
    MTPSCRIPT_HTTP_DELETE,
    MTPSCRIPT_HTTP_PATCH
} MTPScriptHTTPMethod;

// Route parameter
typedef struct {
    char *name;
    char *value;
} MTPScriptRouteParam;

// Route definition
typedef struct {
    MTPScriptHTTPMethod method;
    char *path_pattern;        // e.g., "/users/:id"
    char *handler_name;        // Name of the MTPScript function to call
    MTPScriptRouteParam *path_params;
    int path_param_count;
} MTPScriptRoute;

// API request
typedef struct {
    MTPScriptHTTPMethod method;
    char *path;
    char *query_string;
    char *body;
    size_t body_size;
    char *content_type;
    MTPScriptRouteParam *query_params;
    int query_param_count;
    MTPScriptRouteParam *headers;
    int header_count;
} MTPScriptAPIRequest;

// API response
typedef struct {
    int status_code;
    char *body;
    size_t body_size;
    char *content_type;
    MTPScriptRouteParam *headers;
    int header_count;
} MTPScriptAPIResponse;

// Route registry
typedef struct {
    MTPScriptRoute *routes;
    int route_count;
    int route_capacity;
} MTPScriptRouteRegistry;

// Route registry functions
MTPScriptRouteRegistry *mtpscript_route_registry_new(void);
void mtpscript_route_registry_free(MTPScriptRouteRegistry *registry);
bool mtpscript_route_registry_add(MTPScriptRouteRegistry *registry,
                                 MTPScriptHTTPMethod method,
                                 const char *path_pattern,
                                 const char *handler_name);

// Route matching and parameter extraction
MTPScriptRoute *mtpscript_route_match(MTPScriptRouteRegistry *registry,
                                     MTPScriptHTTPMethod method,
                                     const char *path,
                                     MTPScriptRouteParam **path_params,
                                     int *path_param_count);

// Request parsing
MTPScriptAPIRequest *mtpscript_api_request_parse(const char *method,
                                                const char *path_with_query,
                                                const char *body,
                                                const char *content_type);
MTPScriptAPIRequest *mtpscript_api_request_parse_full(const char *method,
                                                    const char *path_with_query,
                                                    const char *body,
                                                    const char *content_type,
                                                    const char *headers_raw);
void mtpscript_api_request_free(MTPScriptAPIRequest *request);

// Response generation
MTPScriptAPIResponse *mtpscript_api_response_new(void);
void mtpscript_api_response_free(MTPScriptAPIResponse *response);
void mtpscript_api_response_set_json(MTPScriptAPIResponse *response, JSValue json_value, JSContext *ctx);
void mtpscript_api_response_set_status(MTPScriptAPIResponse *response, int status_code);

// Utility functions
MTPScriptHTTPMethod mtpscript_http_method_from_string(const char *method_str);
const char *mtpscript_http_method_to_string(MTPScriptHTTPMethod method);

// JSON parsing and validation
JSValue mtpscript_api_parse_json_body(JSContext *ctx, const char *body, size_t body_size);
bool mtpscript_api_validate_json(JSContext *ctx, JSValue json_value);

// Header access functions
const char *mtpscript_api_get_header(const MTPScriptAPIRequest *request, const char *name);
void mtpscript_api_set_header(MTPScriptAPIResponse *response, const char *name, const char *value);

// Response generation functions
JSValue mtpscript_api_respond_json(JSContext *ctx, JSValue json_data);
JSValue mtpscript_api_respond_status(JSContext *ctx, int status_code, const char *message);
JSValue mtpscript_api_respond_error(JSContext *ctx, int status_code, const char *error_type, const char *message);

// API routing handler
JSValue mtpscript_api_route(JSContext *ctx, MTPScriptRouteRegistry *registry,
                           MTPScriptAPIRequest *request);

// Register API routing
void mtpscript_api_register(JSContext *ctx, MTPScriptRouteRegistry *registry);

#endif /* MQUICKJS_API_H */
