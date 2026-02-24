/**
 * MTPScript Type Checker
 * Specification §6.0
 *
 * Copyright (c) 2025 My Tech Passport Inc.
 * Author: Ryan Wong
 */

#ifndef MTPSCRIPT_TYPECHECKER_H
#define MTPSCRIPT_TYPECHECKER_H

#include "ast.h"

typedef struct {
    mtpscript_hash_t *env;        // Variable name -> type mapping
    mtpscript_hash_t *declared;   // Variable name -> bool (immutability tracking)
    mtpscript_vector_t *used_effects; // Effects used in this scope
} mtpscript_type_env_t;

mtpscript_type_env_t *mtpscript_type_env_new(void);
void mtpscript_type_env_free(mtpscript_type_env_t *env);
mtpscript_error_t *mtpscript_typecheck_program(mtpscript_program_t *program);

#endif // MTPSCRIPT_TYPECHECKER_H
