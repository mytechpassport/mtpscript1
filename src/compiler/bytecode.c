/**
 * MTPScript Bytecode Generator Implementation
 * Specification §5.2
 *
 * Copyright (c) 2025 My Tech Passport Inc.
 * Author: Ryan Wong
 */

#include "bytecode.h"
#include "../../cutils.h"
#include "../../mquickjs.h"
#include <string.h>
#include <stdio.h>
#include <stdlib.h>

#define BYTECODE_COMPILE_MEM_SIZE (8 * 1024 * 1024) // 8MB

mtpscript_error_t *mtpscript_bytecode_compile(const char *js_source, const char *filename, mtpscript_bytecode_t **bytecode_out) {
    // Initialize MicroQuickJS context for bytecode compilation and validation
    uint8_t *mem_buf = malloc(BYTECODE_COMPILE_MEM_SIZE);
    if (!mem_buf) {
        mtpscript_error_t *error = MTPSCRIPT_MALLOC(sizeof(mtpscript_error_t));
        error->message = mtpscript_string_from_cstr("Failed to allocate memory for bytecode compilation");
        error->location = (mtpscript_location_t){0, 0, "bytecode_compilation"};
        return error;
    }

    JSContext *ctx = JS_NewContext(mem_buf, BYTECODE_COMPILE_MEM_SIZE, NULL);
    if (!ctx) {
        free(mem_buf);
        mtpscript_error_t *error = MTPSCRIPT_MALLOC(sizeof(mtpscript_error_t));
        error->message = mtpscript_string_from_cstr("Failed to create JS context for bytecode compilation");
        error->location = (mtpscript_location_t){0, 0, "bytecode_compilation"};
        return error;
    }

    // Parse the JavaScript source to validate syntax and compile it
    JSValue parsed_code = JS_Parse(ctx, js_source, strlen(js_source), filename, 0);
    if (JS_IsException(parsed_code)) {
        // JavaScript parsing failed - this indicates invalid syntax
        JS_FreeContext(ctx);
        free(mem_buf);

        mtpscript_error_t *error = MTPSCRIPT_MALLOC(sizeof(mtpscript_error_t));
        error->message = mtpscript_string_from_cstr("JavaScript parsing failed during bytecode compilation");
        error->location = (mtpscript_location_t){0, 0, "bytecode_compilation"};
        return error;
    }

    // JavaScript parsed successfully - create bytecode object
    // For Phase 1, we store the validated JavaScript source as "bytecode"
    // Full MicroQuickJS bytecode compilation would require more complex handling
    // The snapshot system handles signing and binary storage of the result
    mtpscript_bytecode_t *bytecode = MTPSCRIPT_MALLOC(sizeof(mtpscript_bytecode_t));
    size_t source_len = strlen(js_source);
    bytecode->size = source_len;
    bytecode->data = MTPSCRIPT_MALLOC(source_len);
    memcpy(bytecode->data, js_source, source_len);

    // Clean up
    JS_FreeContext(ctx);
    free(mem_buf);

    *bytecode_out = bytecode;
    return NULL;
}

void mtpscript_bytecode_free(mtpscript_bytecode_t *bytecode) {
    if (bytecode) {
        if (bytecode->data) MTPSCRIPT_FREE(bytecode->data);
        MTPSCRIPT_FREE(bytecode);
    }
}
