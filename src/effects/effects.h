/**
 * MTPScript Effect System
 * Specification §9.0
 *
 * Copyright (c) 2025 My Tech Passport Inc.
 * Author: Ryan Wong
 */

#ifndef MTPSCRIPT_EFFECTS_H
#define MTPSCRIPT_EFFECTS_H

#include "../compiler/mtpscript.h"

typedef enum {
    MTPSCRIPT_EFFECT_DB_READ,
    MTPSCRIPT_EFFECT_DB_WRITE,
    MTPSCRIPT_EFFECT_HTTP_OUT,
    MTPSCRIPT_EFFECT_LOG,
    MTPSCRIPT_EFFECT_ASYNC
} mtpscript_effect_kind_t;

typedef struct {
    mtpscript_effect_kind_t kind;
    mtpscript_string_t *name;
} mtpscript_effect_t;

mtpscript_error_t *mtpscript_effect_validate(mtpscript_vector_t *declared_effects, mtpscript_vector_t *actual_effects);

#endif // MTPSCRIPT_EFFECTS_H
