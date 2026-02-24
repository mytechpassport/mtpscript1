/**
 * MTPScript Effect System Implementation
 * Specification §9.0
 *
 * Copyright (c) 2025 My Tech Passport Inc.
 * Author: Ryan Wong
 */

#include "effects.h"
#include <string.h>

mtpscript_error_t *mtpscript_effect_validate(mtpscript_vector_t *declared_effects, mtpscript_vector_t *actual_effects) {
    for (size_t i = 0; i < actual_effects->size; i++) {
        mtpscript_effect_t *actual = mtpscript_vector_get(actual_effects, i);
        bool found = false;
        for (size_t j = 0; j < declared_effects->size; j++) {
            mtpscript_effect_t *declared = mtpscript_vector_get(declared_effects, j);
            if (actual->kind == declared->kind) {
                found = true;
                break;
            }
        }
        if (!found) {
            mtpscript_error_t *error = MTPSCRIPT_MALLOC(sizeof(mtpscript_error_t));
            error->message = mtpscript_string_from_cstr("Undeclared effect detected");
            return error;
        }
    }
    return NULL;
}
