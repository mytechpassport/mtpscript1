/**
 * MTPScript Snapshot System
 * Specification §5.2
 *
 * Copyright (c) 2025 My Tech Passport Inc.
 * Author: Ryan Wong
 */

#ifndef MTPSCRIPT_SNAPSHOT_H
#define MTPSCRIPT_SNAPSHOT_H

#include "mtpscript.h"

typedef struct {
    uint8_t magic[4];
    uint32_t version;
    uint32_t metadata_size;
    uint32_t content_size;
    uint32_t signature_size;
} mtpscript_snapshot_header_t;

typedef struct {
    mtpscript_snapshot_header_t header;
    char *metadata;
    uint8_t *content;
    uint8_t *signature;
} mtpscript_snapshot_t;

mtpscript_error_t *mtpscript_snapshot_create(const char *bytecode_data, size_t bytecode_size, const char *metadata, const uint8_t *signature, size_t sig_size, const char *output_file);
mtpscript_error_t *mtpscript_snapshot_load(const char *input_file, mtpscript_snapshot_t **snapshot);
void mtpscript_snapshot_free(mtpscript_snapshot_t *snapshot);

#endif // MTPSCRIPT_SNAPSHOT_H
