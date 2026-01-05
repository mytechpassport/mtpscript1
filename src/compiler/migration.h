/**
 * MTPScript TypeScript Migration Header
 * Specification §17
 *
 * Copyright (c) 2025 My Tech Passport Inc.
 * Author: Ryan Wong
 */

#ifndef MTPSCRIPT_MIGRATION_H
#define MTPSCRIPT_MIGRATION_H

#include "cutils.h"
#include "mtpscript.h"

// Migration context for tracking issues and suggestions
typedef struct {
    bool check_only;
    bool batch_mode;
    mtpscript_vector_t *compatibility_issues;
    mtpscript_vector_t *manual_interventions;
    mtpscript_vector_t *effect_suggestions;
} mtpscript_migration_context_t;

mtpscript_migration_context_t *mtpscript_migration_context_new();
void mtpscript_migration_context_free(mtpscript_migration_context_t *ctx);
char *mtpscript_migrate_typescript_line(const char *line, mtpscript_migration_context_t *ctx);
char *mtpscript_migrate_typescript_ast(const char *source, mtpscript_migration_context_t *ctx);
int mtpscript_migrate_file(const char *input_file, const char *output_file,
                          mtpscript_migration_context_t *ctx);
int mtpscript_migrate_directory(const char *input_dir, const char *output_dir,
                              mtpscript_migration_context_t *ctx, bool check_only);
void mtpscript_migration_report(mtpscript_migration_context_t *ctx);

#endif // MTPSCRIPT_MIGRATION_H
