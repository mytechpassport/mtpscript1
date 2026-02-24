/**
 * MTPScript AWS Lambda Host Adapter
 * Specification §11.0
 *
 * Copyright (c) 2025 My Tech Passport Inc.
 * Author: Ryan Wong
 */

#ifndef MTPSCRIPT_HOST_LAMBDA_H
#define MTPSCRIPT_HOST_LAMBDA_H

#include "../compiler/mtpscript.h"
#include "../snapshot/snapshot.h"

typedef struct {
    mtpscript_string_t *method;
    mtpscript_string_t *path;
    mtpscript_hash_t *headers;
    mtpscript_string_t *body;
} mtpscript_lambda_event_t;

typedef struct {
    int status_code;
    mtpscript_hash_t *headers;
    mtpscript_string_t *body;
} mtpscript_lambda_response_t;

mtpscript_error_t *mtpscript_host_lambda_run(mtpscript_snapshot_t *snapshot, mtpscript_lambda_event_t *event, mtpscript_lambda_response_t **response);

#endif // MTPSCRIPT_HOST_LAMBDA_H
