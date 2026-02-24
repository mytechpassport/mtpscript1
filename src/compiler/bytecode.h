/**
 * MTPScript Bytecode Generator
 * Specification §5.2
 *
 * Copyright (c) 2025 My Tech Passport Inc.
 * Author: Ryan Wong
 */

#ifndef MTPSCRIPT_BYTECODE_H
#define MTPSCRIPT_BYTECODE_H

#include "mtpscript.h"

typedef struct {
    uint8_t *data;
    size_t size;
} mtpscript_bytecode_t;

mtpscript_error_t *mtpscript_bytecode_compile(const char *js_source, const char *filename, mtpscript_bytecode_t **bytecode);
void mtpscript_bytecode_free(mtpscript_bytecode_t *bytecode);

#endif // MTPSCRIPT_BYTECODE_H
