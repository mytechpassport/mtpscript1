/**
 * MTPScript Language Server Protocol (LSP) Implementation
 *
 * Copyright (c) 2025 My Tech Passport Inc.
 * Author: Ryan Wong
 */

#include "lsp.h"
#include <unistd.h>

// JSON parsing helper functions (simplified - in production would use proper JSON parser)
static char *json_get_string(const char *json, const char *key) {
    char *start = strstr(json, key);
    if (!start) return NULL;

    start = strchr(start, ':');
    if (!start) return NULL;

    start = strchr(start, '"');
    if (!start) return NULL;
    start++;

    char *end = strchr(start, '"');
    if (!end) return NULL;

    size_t len = end - start;
    char *result = malloc(len + 1);
    memcpy(result, start, len);
    result[len] = '\0';
    return result;
}

static int json_get_int(const char *json, const char *key) {
    char *start = strstr(json, key);
    if (!start) return -1;

    start = strchr(start, ':');
    if (!start) return -1;

    return atoi(start + 1);
}

// LSP Server implementation
mtpscript_lsp_server_t *mtpscript_lsp_server_new() {
    mtpscript_lsp_server_t *server = calloc(1, sizeof(mtpscript_lsp_server_t));
    server->diagnostics = mtpscript_vector_new();
    server->initialized = false;
    return server;
}

void mtpscript_lsp_server_free(mtpscript_lsp_server_t *server) {
    if (server) {
        if (server->current_program) {
            mtpscript_program_free(server->current_program);
        }
        if (server->diagnostics) {
            for (size_t i = 0; i < server->diagnostics->size; i++) {
                mtpscript_lsp_diagnostic_t *diag = server->diagnostics->items[i];
                free(diag->code);
                free(diag->source);
                free(diag->message);
                free(diag);
            }
            mtpscript_vector_free(server->diagnostics);
        }
        free(server->current_uri);
        free(server);
    }
}

// Message I/O functions
char *mtpscript_lsp_read_message() {
    // Read Content-Length header
    char header[1024];
    if (!fgets(header, sizeof(header), stdin)) {
        return NULL;
    }

    if (strncmp(header, "Content-Length: ", 16) != 0) {
        return NULL;
    }

    int content_length = atoi(header + 16);

    // Skip empty line
    if (!fgets(header, sizeof(header), stdin)) {
        return NULL;
    }

    // Read content
    char *content = malloc(content_length + 1);
    size_t bytes_read = fread(content, 1, content_length, stdin);
    content[bytes_read] = '\0';

    return content;
}

void mtpscript_lsp_write_message(const char *message) {
    fprintf(stdout, "Content-Length: %zu\r\n\r\n%s", strlen(message), message);
    fflush(stdout);
}

// Diagnostic functions
mtpscript_vector_t *mtpscript_lsp_get_diagnostics(mtpscript_lsp_server_t *server, const char *uri) {
    // Clear existing diagnostics
    for (size_t i = 0; i < server->diagnostics->size; i++) {
        mtpscript_lsp_diagnostic_t *diag = server->diagnostics->items[i];
        free(diag->code);
        free(diag->source);
        free(diag->message);
        free(diag);
    }
    mtpscript_vector_free(server->diagnostics);
    server->diagnostics = mtpscript_vector_new();

    if (!server->current_program) {
        return server->diagnostics;
    }

    // Generate diagnostics from program
    // This is a simplified implementation - in production would analyze AST for errors
    for (size_t i = 0; i < server->current_program->declarations->size; i++) {
        mtpscript_declaration_t *decl = server->current_program->declarations->items[i];

        // Check for serve declarations (basic validation)
        if (decl->kind == MTPSCRIPT_DECL_SERVE) {
            mtpscript_serve_decl_t *serve = &decl->data.serve;

            // Check if port is valid
            if (serve->port <= 0 || serve->port > 65535) {
                mtpscript_lsp_diagnostic_t *diag = calloc(1, sizeof(mtpscript_lsp_diagnostic_t));
                diag->range.start.line = 0; // Would need proper line tracking
                diag->range.start.character = 0;
                diag->range.end.line = 0;
                diag->range.end.character = 10;
                diag->severity = MTPSCRIPT_LSP_DIAGNOSTIC_ERROR;
                diag->code = strdup("invalid-port");
                diag->source = strdup("mtpscript");
                diag->message = strdup("Port must be between 1 and 65535");
                mtpscript_vector_push(server->diagnostics, diag);
            }

            // Check if routes exist
            if (!serve->routes || serve->routes->size == 0) {
                mtpscript_lsp_diagnostic_t *diag = calloc(1, sizeof(mtpscript_lsp_diagnostic_t));
                diag->range.start.line = 0;
                diag->range.start.character = 0;
                diag->range.end.line = 0;
                diag->range.end.character = 10;
                diag->severity = MTPSCRIPT_LSP_DIAGNOSTIC_WARNING;
                diag->code = strdup("no-routes");
                diag->source = strdup("mtpscript");
                diag->message = strdup("Serve declaration has no routes defined");
                mtpscript_vector_push(server->diagnostics, diag);
            }
        }
    }

    return server->diagnostics;
}

// Completion functions
mtpscript_vector_t *mtpscript_lsp_get_completions(mtpscript_lsp_server_t *server,
                                                const char *uri,
                                                mtpscript_lsp_position_t position) {
    mtpscript_vector_t *completions = mtpscript_vector_new();

    // Add keyword completions
    const char *keywords[] = {
        "func", "record", "union", "enum", "if", "else", "match", "return",
        "let", "uses", "serve", "true", "false", "Int", "String", "Bool", NULL
    };

    for (int i = 0; keywords[i]; i++) {
        mtpscript_lsp_completion_item_t *item = calloc(1, sizeof(mtpscript_lsp_completion_item_t));
        item->label = strdup(keywords[i]);
        item->kind = MTPSCRIPT_LSP_COMPLETION_KEYWORD;
        item->detail = strdup("keyword");
        item->insert_text = strdup(keywords[i]);
        mtpscript_vector_push(completions, item);
    }

    // Add built-in function completions
    const char *functions[] = {
        "println", "readln", "to_string", "length", "append", NULL
    };

    for (int i = 0; functions[i]; i++) {
        mtpscript_lsp_completion_item_t *item = calloc(1, sizeof(mtpscript_lsp_completion_item_t));
        item->label = strdup(functions[i]);
        item->kind = MTPSCRIPT_LSP_COMPLETION_FUNCTION;
        item->detail = strdup("built-in function");
        item->insert_text = strdup(functions[i]);
        mtpscript_vector_push(completions, item);
    }

    // Add effect completions
    const char *effects[] = {
        "DbRead", "DbWrite", "HttpOut", "Log", NULL
    };

    for (int i = 0; effects[i]; i++) {
        mtpscript_lsp_completion_item_t *item = calloc(1, sizeof(mtpscript_lsp_completion_item_t));
        item->label = strdup(effects[i]);
        item->kind = MTPSCRIPT_LSP_COMPLETION_CLASS;
        item->detail = strdup("effect");
        item->insert_text = strdup(effects[i]);
        mtpscript_vector_push(completions, item);
    }

    return completions;
}

// Hover functions
mtpscript_lsp_hover_t *mtpscript_lsp_get_hover(mtpscript_lsp_server_t *server,
                                             const char *uri,
                                             mtpscript_lsp_position_t position) {
    // This is a simplified implementation - in production would analyze AST
    mtpscript_lsp_hover_t *hover = calloc(1, sizeof(mtpscript_lsp_hover_t));

    // Basic hover information for common constructs
    hover->contents = strdup("**MTPScript**\n\nA deterministic programming language for serverless functions.");

    return hover;
}

// Go to definition functions
mtpscript_vector_t *mtpscript_lsp_find_definition(mtpscript_lsp_server_t *server,
                                                const char *uri,
                                                mtpscript_lsp_position_t position) {
    mtpscript_vector_t *locations = mtpscript_vector_new();

    if (!server->current_program) {
        return locations;
    }

    // This is a simplified implementation - in production would search AST for definitions
    // For now, just return empty list
    return locations;
}

// Find references functions
mtpscript_vector_t *mtpscript_lsp_find_references(mtpscript_lsp_server_t *server,
                                                const char *uri,
                                                mtpscript_lsp_position_t position) {
    mtpscript_vector_t *locations = mtpscript_vector_new();

    if (!server->current_program) {
        return locations;
    }

    // This is a simplified implementation - in production would search AST for references
    // For now, just return empty list
    return locations;
}

// LSP method handlers
void mtpscript_lsp_initialize(mtpscript_lsp_server_t *server, mtpscript_lsp_request_t *request) {
    // Send initialize response
    const char *response = "{"
        "\"jsonrpc\":\"2.0\","
        "\"id\":1,"
        "\"result\":{"
            "\"capabilities\":{"
                "\"textDocumentSync\":1,"
                "\"completionProvider\":{\"resolveProvider\":false,\"triggerCharacters\":[\".\"]},"
                "\"hoverProvider\":true,"
                "\"definitionProvider\":true,"
                "\"referencesProvider\":true,"
                "\"diagnosticProvider\":{\"interFileDependencies\":false,\"workspaceDiagnostics\":false}"
            "}"
        "}"
    "}";

    mtpscript_lsp_write_message(response);
    server->initialized = true;
}

void mtpscript_lsp_shutdown(mtpscript_lsp_server_t *server, mtpscript_lsp_request_t *request) {
    const char *response = "{"
        "\"jsonrpc\":\"2.0\","
        "\"id\":2,"
        "\"result\":null"
    "}";

    mtpscript_lsp_write_message(response);
}

void mtpscript_lsp_text_document_did_open(mtpscript_lsp_server_t *server, mtpscript_lsp_notification_t *notification) {
    // Parse the document content and update diagnostics
    // This is a simplified implementation
    mtpscript_lsp_get_diagnostics(server, server->current_uri);

    // Send diagnostics notification
    char diagnostics_json[4096];
    snprintf(diagnostics_json, sizeof(diagnostics_json),
        "{"
            "\"jsonrpc\":\"2.0\","
            "\"method\":\"textDocument/publishDiagnostics\","
            "\"params\":{"
                "\"uri\":\"%s\","
                "\"diagnostics\":[]"
            "}"
        "}",
        server->current_uri ? server->current_uri : "file:///tmp/test.mtp"
    );

    mtpscript_lsp_write_message(diagnostics_json);
}

void mtpscript_lsp_text_document_did_change(mtpscript_lsp_server_t *server, mtpscript_lsp_notification_t *notification) {
    // Update document content and refresh diagnostics
    mtpscript_lsp_get_diagnostics(server, server->current_uri);
}

void mtpscript_lsp_text_document_completion(mtpscript_lsp_server_t *server, mtpscript_lsp_request_t *request) {
    mtpscript_vector_t *completions = mtpscript_lsp_get_completions(server, server->current_uri, (mtpscript_lsp_position_t){0, 0});

    // Send completion response (simplified)
    const char *response = "{"
        "\"jsonrpc\":\"2.0\","
        "\"id\":3,"
        "\"result\":{"
            "\"items\":["
                "{\"label\":\"func\",\"kind\":14,\"detail\":\"keyword\"},"
                "{\"label\":\"record\",\"kind\":14,\"detail\":\"keyword\"},"
                "{\"label\":\"DbRead\",\"kind\":7,\"detail\":\"effect\"}"
            "]"
        "}"
    "}";

    mtpscript_lsp_write_message(response);

    // Clean up completions
    for (size_t i = 0; i < completions->size; i++) {
        mtpscript_lsp_completion_item_t *item = completions->items[i];
        free(item->label);
        free(item->detail);
        free(item->documentation);
        free(item->insert_text);
        free(item);
    }
    mtpscript_vector_free(completions);
}

void mtpscript_lsp_text_document_hover(mtpscript_lsp_server_t *server, mtpscript_lsp_request_t *request) {
    mtpscript_lsp_hover_t *hover = mtpscript_lsp_get_hover(server, server->current_uri, (mtpscript_lsp_position_t){0, 0});

    const char *response = "{"
        "\"jsonrpc\":\"2.0\","
        "\"id\":4,"
        "\"result\":{"
            "\"contents\":\"**MTPScript**\\n\\nA deterministic programming language for serverless functions.\""
        "}"
    "}";

    mtpscript_lsp_write_message(response);
    free(hover->contents);
    free(hover);
}

void mtpscript_lsp_text_document_definition(mtpscript_lsp_server_t *server, mtpscript_lsp_request_t *request) {
    const char *response = "{"
        "\"jsonrpc\":\"2.0\","
        "\"id\":5,"
        "\"result\":[]"
    "}";

    mtpscript_lsp_write_message(response);
}

void mtpscript_lsp_text_document_references(mtpscript_lsp_server_t *server, mtpscript_lsp_request_t *request) {
    const char *response = "{"
        "\"jsonrpc\":\"2.0\","
        "\"id\":6,"
        "\"result\":[]"
    "}";

    mtpscript_lsp_write_message(response);
}

// Main message processing
void mtpscript_lsp_process_message(mtpscript_lsp_server_t *server, const char *message) {
    // Parse JSON-RPC message (simplified)
    if (strstr(message, "initialize")) {
        mtpscript_lsp_request_t request = {.id = 1, .method = "initialize"};
        mtpscript_lsp_initialize(server, &request);
    } else if (strstr(message, "shutdown")) {
        mtpscript_lsp_request_t request = {.id = 2, .method = "shutdown"};
        mtpscript_lsp_shutdown(server, &request);
    } else if (strstr(message, "textDocument/didOpen")) {
        mtpscript_lsp_notification_t notification = {.method = "textDocument/didOpen"};
        mtpscript_lsp_text_document_did_open(server, &notification);
    } else if (strstr(message, "textDocument/didChange")) {
        mtpscript_lsp_notification_t notification = {.method = "textDocument/didChange"};
        mtpscript_lsp_text_document_did_change(server, &notification);
    } else if (strstr(message, "textDocument/completion")) {
        mtpscript_lsp_request_t request = {.id = 3, .method = "textDocument/completion"};
        mtpscript_lsp_text_document_completion(server, &request);
    } else if (strstr(message, "textDocument/hover")) {
        mtpscript_lsp_request_t request = {.id = 4, .method = "textDocument/hover"};
        mtpscript_lsp_text_document_hover(server, &request);
    } else if (strstr(message, "textDocument/definition")) {
        mtpscript_lsp_request_t request = {.id = 5, .method = "textDocument/definition"};
        mtpscript_lsp_text_document_definition(server, &request);
    } else if (strstr(message, "textDocument/references")) {
        mtpscript_lsp_request_t request = {.id = 6, .method = "textDocument/references"};
        mtpscript_lsp_text_document_references(server, &request);
    }
}
