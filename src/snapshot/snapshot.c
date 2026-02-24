/**
 * MTPScript Snapshot System Implementation
 * Specification §5.2
 *
 * Copyright (c) 2025 My Tech Passport Inc.
 * Author: Ryan Wong
 */

#include "snapshot.h"
#include <stdio.h>
#include <string.h>

mtpscript_error_t *mtpscript_snapshot_create(const char *bytecode_data, size_t bytecode_size, const char *metadata, const uint8_t *signature, size_t sig_size, const char *output_file) {
    FILE *f = fopen(output_file, "wb");
    if (!f) {
        mtpscript_error_t *error = MTPSCRIPT_MALLOC(sizeof(mtpscript_error_t));
        error->message = mtpscript_string_from_cstr("Failed to open output file");
        return error;
    }

    mtpscript_snapshot_header_t header;
    memcpy(header.magic, "MSQS", 4);
    header.version = 1;
    header.metadata_size = (uint32_t)strlen(metadata);
    header.content_size = (uint32_t)bytecode_size;
    header.signature_size = (uint32_t)sig_size;

    fwrite(&header, sizeof(header), 1, f);
    fwrite(metadata, 1, header.metadata_size, f);
    fwrite(bytecode_data, 1, header.content_size, f);
    if (signature && sig_size > 0) {
        fwrite(signature, 1, sig_size, f);
    }

    fclose(f);
    return NULL;
}

mtpscript_error_t *mtpscript_snapshot_load(const char *input_file, mtpscript_snapshot_t **snapshot_out) {
    FILE *f = fopen(input_file, "rb");
    if (!f) {
        mtpscript_error_t *error = MTPSCRIPT_MALLOC(sizeof(mtpscript_error_t));
        error->message = mtpscript_string_from_cstr("Failed to open input file");
        return error;
    }

    mtpscript_snapshot_t *snapshot = MTPSCRIPT_MALLOC(sizeof(mtpscript_snapshot_t));
    fread(&snapshot->header, sizeof(mtpscript_snapshot_header_t), 1, f);

    snapshot->metadata = MTPSCRIPT_MALLOC(snapshot->header.metadata_size + 1);
    fread(snapshot->metadata, 1, snapshot->header.metadata_size, f);
    snapshot->metadata[snapshot->header.metadata_size] = '\0';

    snapshot->content = MTPSCRIPT_MALLOC(snapshot->header.content_size);
    fread(snapshot->content, 1, snapshot->header.content_size, f);

    if (snapshot->header.signature_size > 0) {
        snapshot->signature = MTPSCRIPT_MALLOC(snapshot->header.signature_size);
        fread(snapshot->signature, 1, snapshot->header.signature_size, f);
    } else {
        snapshot->signature = NULL;
    }

    fclose(f);
    *snapshot_out = snapshot;
    return NULL;
}

void mtpscript_snapshot_free(mtpscript_snapshot_t *snapshot) {
    if (snapshot) {
        if (snapshot->metadata) MTPSCRIPT_FREE(snapshot->metadata);
        if (snapshot->content) MTPSCRIPT_FREE(snapshot->content);
        if (snapshot->signature) MTPSCRIPT_FREE(snapshot->signature);
        MTPSCRIPT_FREE(snapshot);
    }
}
