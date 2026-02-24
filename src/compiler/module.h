/**
 * MTPScript Module System
 * Specification §10
 *
 * Copyright (c) 2025 My Tech Passport Inc.
 * Author: Ryan Wong
 */

#ifndef MTPSCRIPT_MODULE_H
#define MTPSCRIPT_MODULE_H

#include "mtpscript.h"
#include "ast.h"

// Module structure
typedef struct mtpscript_module_t {
    mtpscript_string_t *name;
    mtpscript_string_t *git_url;
    mtpscript_string_t *git_hash;
    mtpscript_string_t *tag;
    mtpscript_program_t *program;  // parsed AST
    mtpscript_hash_t *exports;     // exported symbols
} mtpscript_module_t;

// Module resolver
typedef struct mtpscript_module_resolver_t {
    mtpscript_hash_t *module_cache;  // git_hash -> module
    mtpscript_hash_t *verified_tags; // tag -> verified git_hash
} mtpscript_module_resolver_t;

// Module system functions
mtpscript_module_resolver_t *mtpscript_module_resolver_new(void);
void mtpscript_module_resolver_free(mtpscript_module_resolver_t *resolver);

mtpscript_error_t *mtpscript_module_resolve(mtpscript_module_resolver_t *resolver,
                                         mtpscript_import_decl_t *import_decl,
                                         mtpscript_module_t **module_out);

mtpscript_error_t *mtpscript_module_verify_git_hash(const char *git_url,
                                                  const char *expected_hash,
                                                  char *actual_hash_out,
                                                  size_t hash_size);

mtpscript_error_t *mtpscript_module_verify_tag(const char *git_url,
                                             const char *tag,
                                             char *verified_hash_out,
                                             size_t hash_size);

#endif // MTPSCRIPT_MODULE_H
