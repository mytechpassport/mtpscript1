/**
 * MTPScript TypeScript Migration Implementation
 * Specification §17
 *
 * Copyright (c) 2025 My Tech Passport Inc.
 * Author: Ryan Wong
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>
#include <unistd.h>
#include <dirent.h>
#include <sys/stat.h>
#include <limits.h>
#include <errno.h>
#include "mtpscript.h"
#include "cutils.h"

// Migration context for tracking issues and suggestions
typedef struct {
    bool check_only;
    bool batch_mode;
    mtpscript_vector_t *compatibility_issues;
    mtpscript_vector_t *manual_interventions;
    mtpscript_vector_t *effect_suggestions;
} mtpscript_migration_context_t;

mtpscript_migration_context_t *mtpscript_migration_context_new() {
    mtpscript_migration_context_t *ctx = calloc(1, sizeof(mtpscript_migration_context_t));
    ctx->compatibility_issues = mtpscript_vector_new();
    ctx->manual_interventions = mtpscript_vector_new();
    ctx->effect_suggestions = mtpscript_vector_new();
    return ctx;
}

void mtpscript_migration_context_free(mtpscript_migration_context_t *ctx) {
    if (!ctx) return;

    for (size_t i = 0; i < ctx->compatibility_issues->size; i++) {
        mtpscript_string_free(ctx->compatibility_issues->items[i]);
    }
    mtpscript_vector_free(ctx->compatibility_issues);

    for (size_t i = 0; i < ctx->manual_interventions->size; i++) {
        mtpscript_string_free(ctx->manual_interventions->items[i]);
    }
    mtpscript_vector_free(ctx->manual_interventions);

    for (size_t i = 0; i < ctx->effect_suggestions->size; i++) {
        mtpscript_string_free(ctx->effect_suggestions->items[i]);
    }
    mtpscript_vector_free(ctx->effect_suggestions);

    free(ctx);
}

// String replacement utility
static char *str_replace(const char *orig, const char *rep, const char *with) {
    char *result;
    const char *ins;
    char *tmp;
    int len_rep;
    int len_with;
    int len_front;
    int count;

    if (!orig || !rep) {
        return NULL;
    }
    len_rep = strlen(rep);
    if (len_rep == 0) {
        return NULL;
    }
    if (!with) {
        with = "";
    }
    len_with = strlen(with);

    ins = orig;
    for (count = 0; (tmp = strstr(ins, rep)); ++count) {
        ins = tmp + len_rep;
    }

    tmp = result = malloc(strlen(orig) + (len_with - len_rep) * count + 1);

    if (!result) {
        return NULL;
    }

    while (count--) {
        ins = strstr(orig, rep);
        len_front = ins - orig;
        tmp = strncpy(tmp, orig, len_front) + len_front;
        tmp = strcpy(tmp, with) + len_with;
        orig += len_front + len_rep;
    }
    strcpy(tmp, orig);
    return result;
}

// TypeScript AST-based migration
char *mtpscript_migrate_typescript_ast(const char *source, mtpscript_migration_context_t *ctx) {
    // TODO: Implement full AST-based migration
    // For now, just return a copy
    if (ctx && ctx->effect_suggestions) {
        mtpscript_vector_push(ctx->effect_suggestions,
            mtpscript_string_from_cstr("Add appropriate effects based on the functionality being migrated"));
    }
    return strdup(source);
}

// Line-by-line TypeScript migration
char *mtpscript_migrate_typescript_line(const char *line, mtpscript_migration_context_t *ctx) {
    char *migrated = strdup(line);

    // Type mapping: number -> Int, string -> String, boolean -> Bool
    if (strstr(line, ": number") != NULL) {
        char *new_line = str_replace(migrated, ": number", ": Int");
        free(migrated);
        migrated = new_line;
    }

    if (strstr(line, ": string") != NULL) {
        char *new_line = str_replace(migrated, ": string", ": String");
        free(migrated);
        migrated = new_line;
    }

    if (strstr(line, ": boolean") != NULL) {
        char *new_line = str_replace(migrated, ": boolean", ": Bool");
        free(migrated);
        migrated = new_line;
    }

    // Null handling: T | null -> Option<T>
    if (strstr(line, " | null") != NULL) {
        // Simple replacement: T | null -> Option<T>
        char *new_line = str_replace(migrated, " | null", "");
        free(migrated);
        migrated = new_line;

        // Now replace the type part with Option<type>
        char *colon_pos = strstr(migrated, ": ");
        if (colon_pos) {
            colon_pos += 2; // Skip ": "
            char *equals_pos = strstr(colon_pos, " =");
            if (equals_pos) {
                // Extract the type
                size_t type_len = equals_pos - colon_pos;
                char *type_str = strndup(colon_pos, type_len);

                // Create new string with Option<type>
                size_t prefix_len = colon_pos - migrated;
                char *new_migrated = malloc(prefix_len + 7 + strlen(type_str) + strlen(equals_pos) + 1);
                memcpy(new_migrated, migrated, prefix_len);
                sprintf(new_migrated + prefix_len, "Option<%s>%s", type_str, equals_pos);
                free(type_str);
                free(migrated);
                migrated = new_migrated;
            }
        }
    }

    // Interface conversion: interfaces -> structural records
    if (strstr(line, "interface ") != NULL) {
        char *new_line = str_replace(migrated, "interface ", "record ");
        free(migrated);
        migrated = new_line;
    }

    // Class removal: convert to records and functions (basic implementation)
    if (strstr(line, "class ") != NULL && ctx && ctx->manual_interventions) {
        mtpscript_vector_push(ctx->manual_interventions,
            mtpscript_string_from_cstr("Classes must be manually converted to records and functions"));
    }

    // Loop conversion: for/while -> recursive functions (basic implementation)
    if ((strstr(line, "for (") != NULL || strstr(line, "while (") != NULL) && ctx && ctx->manual_interventions) {
        mtpscript_vector_push(ctx->manual_interventions,
            mtpscript_string_from_cstr("Loops must be converted to recursive functions"));
    }

    // Enum conversion: enums -> union types (basic implementation)
    if (strstr(line, "enum ") != NULL && ctx && ctx->manual_interventions) {
        mtpscript_vector_push(ctx->manual_interventions,
            mtpscript_string_from_cstr("Enums should be converted to union types"));
    }

    // Import rewriting: npm imports -> audit manifest entries (basic implementation)
    if ((strstr(line, "import ") != NULL && strstr(line, "from ") != NULL) && ctx && ctx->manual_interventions) {
        mtpscript_vector_push(ctx->manual_interventions,
            mtpscript_string_from_cstr("Imports must be manually added to audit manifest"));
    }

    // Generics: T<U> -> parametric types (basic implementation)
    if ((strstr(line, "<") != NULL && strstr(line, ">") != NULL) && ctx && ctx->compatibility_issues) {
        mtpscript_vector_push(ctx->compatibility_issues,
            mtpscript_string_from_cstr("Generics have limited support - manual review required"));
    }

    // Method extraction: class methods -> top-level functions (basic implementation)
    if (strstr(line, "  ") != NULL && strstr(line, "(") != NULL && strstr(line, ")") != NULL && ctx && ctx->manual_interventions) {
        mtpscript_vector_push(ctx->manual_interventions,
            mtpscript_string_from_cstr("Class methods should be extracted to top-level functions"));
    }

    // Effect inference: detect I/O patterns
    if (strstr(line, "fetch(") != NULL || strstr(line, "axios.") != NULL) {
        if (ctx && ctx->effect_suggestions) {
            mtpscript_vector_push(ctx->effect_suggestions,
                mtpscript_string_from_cstr("Add HttpOut effect for HTTP requests"));
        }
    }

    if (strstr(line, "fs.") != NULL || strstr(line, "readFile") != NULL || strstr(line, "writeFile") != NULL) {
        if (ctx && ctx->effect_suggestions) {
            mtpscript_vector_push(ctx->effect_suggestions,
                mtpscript_string_from_cstr("Add file system effects for I/O operations"));
        }
    }

    if (strstr(line, "mysql") != NULL || strstr(line, "postgres") != NULL || strstr(line, "db.") != NULL) {
        if (ctx && ctx->effect_suggestions) {
            mtpscript_vector_push(ctx->effect_suggestions,
                mtpscript_string_from_cstr("Add DbRead/DbWrite effects for database operations"));
        }
    }

    if (strstr(line, "console.log") != NULL || strstr(line, "logger.") != NULL) {
        if (ctx && ctx->effect_suggestions) {
            mtpscript_vector_push(ctx->effect_suggestions,
                mtpscript_string_from_cstr("Add Log effect for logging operations"));
        }
    }

    return migrated;
}

// Generate migration report
void mtpscript_migration_report(mtpscript_migration_context_t *ctx) {
    printf("\n=== TypeScript Migration Report ===\n");

    printf("\nCompatibility Issues (%zu):\n", ctx->compatibility_issues->size);
    for (size_t i = 0; i < ctx->compatibility_issues->size; i++) {
        printf("  - %s\n", mtpscript_string_cstr(ctx->compatibility_issues->items[i]));
    }

    printf("\nManual Interventions Required (%zu):\n", ctx->manual_interventions->size);
    for (size_t i = 0; i < ctx->manual_interventions->size; i++) {
        printf("  - %s\n", mtpscript_string_cstr(ctx->manual_interventions->items[i]));
    }

    printf("\nEffect Suggestions (%zu):\n", ctx->effect_suggestions->size);
    for (size_t i = 0; i < ctx->effect_suggestions->size; i++) {
        printf("  - %s\n", mtpscript_string_cstr(ctx->effect_suggestions->items[i]));
    }

    printf("\n===================================\n");
}

// File migration function
int mtpscript_migrate_file(const char *input_file, const char *output_file,
                          mtpscript_migration_context_t *ctx) {
    FILE *in = fopen(input_file, "r");
    if (!in) {
        fprintf(stderr, "Error: Cannot open input file %s\n", input_file);
        return 1;
    }

    FILE *out = fopen(output_file, "w");
    if (!out) {
        fprintf(stderr, "Error: Cannot open output file %s\n", output_file);
        fclose(in);
        return 1;
    }

    char line[4096];
    while (fgets(line, sizeof(line), in)) {
        char *migrated = mtpscript_migrate_typescript_line(line, ctx);
        fprintf(out, "%s", migrated);
        free(migrated);
    }

    fclose(in);
    fclose(out);
    return 0;
}

// Directory migration function
int mtpscript_migrate_directory(const char *input_dir, const char *output_dir,
                              mtpscript_migration_context_t *ctx, bool check_only) {
    DIR *dir = opendir(input_dir);
    if (!dir) {
        fprintf(stderr, "Error: Cannot open directory %s\n", input_dir);
        return -1;
    }

    struct dirent *entry;
    int total_files = 0;
    int migrated_files = 0;
    int failed_files = 0;

    // First pass: count TypeScript files
    while ((entry = readdir(dir)) != NULL) {
        if (entry->d_type == DT_REG) {
            const char *ext = strrchr(entry->d_name, '.');
            if (ext && strcmp(ext, ".ts") == 0) {
                total_files++;
            }
        } else if (entry->d_type == DT_DIR && strcmp(entry->d_name, ".") != 0 && strcmp(entry->d_name, "..") != 0) {
            // Recursively process subdirectories
            char sub_input_dir[PATH_MAX];
            char sub_output_dir[PATH_MAX];
            snprintf(sub_input_dir, sizeof(sub_input_dir), "%s/%s", input_dir, entry->d_name);
            snprintf(sub_output_dir, sizeof(sub_output_dir), "%s/%s", output_dir, entry->d_name);

            // Create output subdirectory
            if (!check_only) {
                mkdir(sub_output_dir, 0755);
            }

            int sub_result = mtpscript_migrate_directory(sub_input_dir, sub_output_dir, ctx, check_only);
            if (sub_result < 0) {
                failed_files++;
            } else {
                migrated_files += sub_result;
                total_files += sub_result; // Count files in subdirectories
            }
        }
    }

    // Reset directory stream
    rewinddir(dir);

    // Second pass: migrate files
    while ((entry = readdir(dir)) != NULL) {
        if (entry->d_type == DT_REG) {
            const char *ext = strrchr(entry->d_name, '.');
            if (ext && strcmp(ext, ".ts") == 0) {
                char input_file[PATH_MAX];
                char output_file[PATH_MAX];

                snprintf(input_file, sizeof(input_file), "%s/%s", input_dir, entry->d_name);

                if (check_only) {
                    snprintf(output_file, sizeof(output_file), "/tmp/migration_check_%s_%s",
                            entry->d_name, input_dir + (input_dir[0] == '/' ? 1 : 0));
                    // Replace path separators with underscores for temp filename
                    for (char *c = output_file; *c; c++) {
                        if (*c == '/' || *c == '\\') *c = '_';
                    }
                } else {
                    // Generate output filename by replacing .ts with .mtp
                    char base_name[PATH_MAX];
                    snprintf(base_name, sizeof(base_name), "%.*s",
                            (int)(ext - entry->d_name), entry->d_name);
                    snprintf(output_file, sizeof(output_file), "%s/%s.mtp", output_dir, base_name);
                }

                printf("Migrating %s -> %s\n", input_file, check_only ? "(check mode)" : output_file);

                int result = mtpscript_migrate_file(input_file, output_file, ctx);
                if (result == 0) {
                    migrated_files++;
                } else {
                    failed_files++;
                    fprintf(stderr, "Failed to migrate: %s\n", input_file);
                }
            }
        }
    }

    closedir(dir);

    if (failed_files > 0) {
        fprintf(stderr, "Migration completed with %d failures out of %d files\n", failed_files, total_files);
        return -1;
    }

    return migrated_files;
}
