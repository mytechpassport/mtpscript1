/**
 * MTPScript Language Server Protocol (LSP) Implementation
 * Provides IDE support for MTPScript including diagnostics, completion, hover, etc.
 *
 * Copyright (c) 2025 My Tech Passport Inc.
 * Author: Ryan Wong
 */

#ifndef MTPSCRIPT_LSP_H
#define MTPSCRIPT_LSP_H

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>
#include "cutils.h"
#include "../compiler/lexer.h"
#include "../compiler/parser.h"
#include "../compiler/typechecker.h"

// LSP Message structures
typedef struct {
    char *jsonrpc;
    int id;
    char *method;
    void *params;
} mtpscript_lsp_request_t;

typedef struct {
    char *jsonrpc;
    int id;
    void *result;
    void *error;
} mtpscript_lsp_response_t;

typedef struct {
    char *jsonrpc;
    char *method;
    void *params;
} mtpscript_lsp_notification_t;

// Position in document
typedef struct {
    int line;
    int character;
} mtpscript_lsp_position_t;

// Range in document
typedef struct {
    mtpscript_lsp_position_t start;
    mtpscript_lsp_position_t end;
} mtpscript_lsp_range_t;

// Diagnostic severity levels
typedef enum {
    MTPSCRIPT_LSP_DIAGNOSTIC_ERROR = 1,
    MTPSCRIPT_LSP_DIAGNOSTIC_WARNING = 2,
    MTPSCRIPT_LSP_DIAGNOSTIC_INFORMATION = 3,
    MTPSCRIPT_LSP_DIAGNOSTIC_HINT = 4
} mtpscript_lsp_diagnostic_severity_t;

// Diagnostic structure
typedef struct {
    mtpscript_lsp_range_t range;
    mtpscript_lsp_diagnostic_severity_t severity;
    char *code;
    char *source;
    char *message;
} mtpscript_lsp_diagnostic_t;

// Completion item kind
typedef enum {
    MTPSCRIPT_LSP_COMPLETION_TEXT = 1,
    MTPSCRIPT_LSP_COMPLETION_METHOD = 2,
    MTPSCRIPT_LSP_COMPLETION_FUNCTION = 3,
    MTPSCRIPT_LSP_COMPLETION_CONSTRUCTOR = 4,
    MTPSCRIPT_LSP_COMPLETION_FIELD = 5,
    MTPSCRIPT_LSP_COMPLETION_VARIABLE = 6,
    MTPSCRIPT_LSP_COMPLETION_CLASS = 7,
    MTPSCRIPT_LSP_COMPLETION_INTERFACE = 8,
    MTPSCRIPT_LSP_COMPLETION_MODULE = 9,
    MTPSCRIPT_LSP_COMPLETION_PROPERTY = 10,
    MTPSCRIPT_LSP_COMPLETION_UNIT = 11,
    MTPSCRIPT_LSP_COMPLETION_VALUE = 12,
    MTPSCRIPT_LSP_COMPLETION_ENUM = 13,
    MTPSCRIPT_LSP_COMPLETION_KEYWORD = 14,
    MTPSCRIPT_LSP_COMPLETION_SNIPPET = 15,
    MTPSCRIPT_LSP_COMPLETION_COLOR = 16,
    MTPSCRIPT_LSP_COMPLETION_FILE = 17,
    MTPSCRIPT_LSP_COMPLETION_REFERENCE = 18,
    MTPSCRIPT_LSP_COMPLETION_FOLDER = 19,
    MTPSCRIPT_LSP_COMPLETION_ENUMMEMBER = 20,
    MTPSCRIPT_LSP_COMPLETION_CONSTANT = 21,
    MTPSCRIPT_LSP_COMPLETION_STRUCT = 22,
    MTPSCRIPT_LSP_COMPLETION_EVENT = 23,
    MTPSCRIPT_LSP_COMPLETION_OPERATOR = 24,
    MTPSCRIPT_LSP_COMPLETION_TYPEPARAMETER = 25
} mtpscript_lsp_completion_item_kind_t;

// Completion item
typedef struct {
    char *label;
    mtpscript_lsp_completion_item_kind_t kind;
    char *detail;
    char *documentation;
    char *insert_text;
} mtpscript_lsp_completion_item_t;

// Hover information
typedef struct {
    mtpscript_lsp_range_t range;
    char *contents;
} mtpscript_lsp_hover_t;

// Location (for go to definition, references)
typedef struct {
    char *uri;
    mtpscript_lsp_range_t range;
} mtpscript_lsp_location_t;

// LSP Server state
typedef struct {
    mtpscript_program_t *current_program;
    mtpscript_vector_t *diagnostics; // mtpscript_lsp_diagnostic_t
    char *current_uri;
    bool initialized;
} mtpscript_lsp_server_t;

// Core LSP functions
mtpscript_lsp_server_t *mtpscript_lsp_server_new();
void mtpscript_lsp_server_free(mtpscript_lsp_server_t *server);

// Message handling
void mtpscript_lsp_process_message(mtpscript_lsp_server_t *server, const char *message);
void mtpscript_lsp_send_response(const mtpscript_lsp_response_t *response);
void mtpscript_lsp_send_notification(const mtpscript_lsp_notification_t *notification);

// LSP method handlers
void mtpscript_lsp_initialize(mtpscript_lsp_server_t *server, mtpscript_lsp_request_t *request);
void mtpscript_lsp_shutdown(mtpscript_lsp_server_t *server, mtpscript_lsp_request_t *request);
void mtpscript_lsp_text_document_did_open(mtpscript_lsp_server_t *server, mtpscript_lsp_notification_t *notification);
void mtpscript_lsp_text_document_did_change(mtpscript_lsp_server_t *server, mtpscript_lsp_notification_t *notification);
void mtpscript_lsp_text_document_completion(mtpscript_lsp_server_t *server, mtpscript_lsp_request_t *request);
void mtpscript_lsp_text_document_hover(mtpscript_lsp_server_t *server, mtpscript_lsp_request_t *request);
void mtpscript_lsp_text_document_definition(mtpscript_lsp_server_t *server, mtpscript_lsp_request_t *request);
void mtpscript_lsp_text_document_references(mtpscript_lsp_server_t *server, mtpscript_lsp_request_t *request);

// Diagnostic functions
mtpscript_vector_t *mtpscript_lsp_get_diagnostics(mtpscript_lsp_server_t *server, const char *uri);

// Completion functions
mtpscript_vector_t *mtpscript_lsp_get_completions(mtpscript_lsp_server_t *server,
                                                const char *uri,
                                                mtpscript_lsp_position_t position);

// Hover functions
mtpscript_lsp_hover_t *mtpscript_lsp_get_hover(mtpscript_lsp_server_t *server,
                                             const char *uri,
                                             mtpscript_lsp_position_t position);

// Go to definition functions
mtpscript_vector_t *mtpscript_lsp_find_definition(mtpscript_lsp_server_t *server,
                                                const char *uri,
                                                mtpscript_lsp_position_t position);

// Find references functions
mtpscript_vector_t *mtpscript_lsp_find_references(mtpscript_lsp_server_t *server,
                                                const char *uri,
                                                mtpscript_lsp_position_t position);

// Utility functions
mtpscript_lsp_position_t mtpscript_lsp_offset_to_position(const char *text, size_t offset);
size_t mtpscript_lsp_position_to_offset(const char *text, mtpscript_lsp_position_t position);
char *mtpscript_lsp_read_message();
void mtpscript_lsp_write_message(const char *message);

#endif // MTPSCRIPT_LSP_H
