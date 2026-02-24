/**
 * MTPScript Type Checker Implementation
 * Specification §6.0
 *
 * Copyright (c) 2025 My Tech Passport Inc.
 * Author: Ryan Wong
 */

#include "typechecker.h"
#include <string.h>
#include <stdio.h>

// Forward declarations
static void record_effect_usage(mtpscript_type_env_t *env, const char *effect);
static mtpscript_error_t *validate_map_key_type(mtpscript_type_t *key_type);
static mtpscript_error_t *check_union_exhaustiveness(mtpscript_type_t *union_type, mtpscript_vector_t *match_arms);

// Check union exhaustiveness for match expressions
static mtpscript_error_t *check_union_exhaustiveness(mtpscript_type_t *union_type, mtpscript_vector_t *match_arms) {
    if (union_type->kind != MTPSCRIPT_TYPE_UNION) {
        return NULL; // Not a union type, no exhaustiveness checking needed
    }

    // Create a set of covered variants
    mtpscript_hash_t *covered_variants = mtpscript_hash_new();

    // Collect all patterns from match arms
    for (size_t i = 0; i < match_arms->size; i++) {
        mtpscript_match_arm_t *arm = match_arms->items[i];
        const char *pattern = mtpscript_string_cstr(arm->pattern);
        mtpscript_hash_set(covered_variants, pattern, (void*)1);
    }

    // Check that all union variants are covered
    for (size_t i = 0; i < union_type->union_variants->size; i++) {
        mtpscript_string_t *variant = union_type->union_variants->items[i];
        if (!mtpscript_hash_get(covered_variants, mtpscript_string_cstr(variant))) {
            mtpscript_hash_free(covered_variants);
            mtpscript_error_t *error = MTPSCRIPT_MALLOC(sizeof(mtpscript_error_t));
            char msg[256];
            sprintf(msg, "Non-exhaustive match: union variant '%s' not covered", mtpscript_string_cstr(variant));
            error->message = mtpscript_string_from_cstr(msg);
            error->location = (mtpscript_location_t){0, 0, "union_exhaustiveness_check"};
            return error;
        }
    }

    mtpscript_hash_free(covered_variants);
    return NULL;
}

mtpscript_type_env_t *mtpscript_type_env_new(void) {
    mtpscript_type_env_t *env = MTPSCRIPT_MALLOC(sizeof(mtpscript_type_env_t));
    env->env = mtpscript_hash_new();
    env->declared = mtpscript_hash_new();
    env->used_effects = mtpscript_vector_new();
    return env;
}

void mtpscript_type_env_free(mtpscript_type_env_t *env) {
    if (env) {
        mtpscript_hash_free(env->env);
        mtpscript_hash_free(env->declared);
        mtpscript_vector_free(env->used_effects);
        MTPSCRIPT_FREE(env);
    }
}

static mtpscript_error_t *typecheck_expression(mtpscript_expression_t *expr, mtpscript_type_env_t *env, mtpscript_type_t **type_out) {
    switch (expr->kind) {
        case MTPSCRIPT_EXPR_INT_LITERAL:
            *type_out = mtpscript_type_new(MTPSCRIPT_TYPE_INT);
            break;
        case MTPSCRIPT_EXPR_STRING_LITERAL:
            *type_out = mtpscript_type_new(MTPSCRIPT_TYPE_STRING);
            break;
        case MTPSCRIPT_EXPR_BOOL_LITERAL:
            *type_out = mtpscript_type_new(MTPSCRIPT_TYPE_BOOL);
            break;
        case MTPSCRIPT_EXPR_DECIMAL_LITERAL:
            *type_out = mtpscript_type_new(MTPSCRIPT_TYPE_DECIMAL);
            break;
        case MTPSCRIPT_EXPR_VARIABLE: {
            mtpscript_type_t *type = mtpscript_hash_get(env->env, mtpscript_string_cstr(expr->data.variable.name));
            if (!type) {
                // Return error
                return NULL;
            }
            *type_out = type;
            break;
        }
        case MTPSCRIPT_EXPR_FUNCTION_CALL: {
            // Basic effect tracking for known functions
            const char *func_name = mtpscript_string_cstr(expr->data.call.function_name);

            // Record effects based on function name
            if (strcmp(func_name, "log") == 0) {
                record_effect_usage(env, "Log");
            } else if (strcmp(func_name, "http_get") == 0 || strcmp(func_name, "http_post") == 0) {
                record_effect_usage(env, "HttpOut");
            } else if (strcmp(func_name, "db_read") == 0) {
                record_effect_usage(env, "DbRead");
            } else if (strcmp(func_name, "db_write") == 0) {
                record_effect_usage(env, "DbWrite");
            }

            // TODO: Add proper function call type checking
            *type_out = mtpscript_type_new(MTPSCRIPT_TYPE_INT); // Placeholder
            break;
        }
        case MTPSCRIPT_EXPR_AWAIT_EXPR: {
            // Await expressions require Async effect (§7-a)
            record_effect_usage(env, "Async");

            // Type check the inner expression
            mtpscript_error_t *inner_error = typecheck_expression(expr->data.await.expression, env, type_out);
            if (inner_error) return inner_error;

            // Await returns the same type as the awaited expression
            break;
        }
        case MTPSCRIPT_EXPR_MATCH_EXPR: {
            // Type check the scrutinee (expression being matched)
            mtpscript_type_t *scrutinee_type;
            mtpscript_error_t *scrutinee_error = typecheck_expression(expr->data.match.scrutinee, env, &scrutinee_type);
            if (scrutinee_error) return scrutinee_error;

            // Check union exhaustiveness if scrutinee is a union type
            mtpscript_error_t *exhaustive_error = check_union_exhaustiveness(scrutinee_type, expr->data.match.arms);
            if (exhaustive_error) return exhaustive_error;

            // Type check all arms and ensure they have compatible return types
            mtpscript_type_t *arm_type = NULL;
            for (size_t i = 0; i < expr->data.match.arms->size; i++) {
                mtpscript_match_arm_t *arm = mtpscript_vector_get(expr->data.match.arms, i);
                mtpscript_type_t *current_arm_type;
                mtpscript_error_t *arm_error = typecheck_expression(arm->body, env, &current_arm_type);
                if (arm_error) return arm_error;

                if (!arm_type) {
                    arm_type = current_arm_type;
                } else if (!mtpscript_type_equals(arm_type, current_arm_type)) {
                    return MTPSCRIPT_MALLOC(sizeof(mtpscript_error_t)); // Type mismatch in match arms
                }
            }

            if (!arm_type) {
                *type_out = mtpscript_type_new(MTPSCRIPT_TYPE_INT); // Default if no arms
            } else {
                *type_out = arm_type;
            }
            break;
        }
        // TODO: Add type checking for Option/Result construction and access
        default: break;
    }
    return NULL;
}

static void record_effect_usage(mtpscript_type_env_t *env, const char *effect) {
    // Check if effect is already recorded
    for (size_t i = 0; i < env->used_effects->size; i++) {
        mtpscript_string_t *existing = mtpscript_vector_get(env->used_effects, i);
        if (strcmp(mtpscript_string_cstr(existing), effect) == 0) {
            return; // Already recorded
        }
    }
    // Add new effect
    mtpscript_vector_push(env->used_effects, mtpscript_string_from_cstr(effect));
}

/* Validate type recursively - ensure Map key constraints (§5) */
static mtpscript_error_t *validate_type(mtpscript_type_t *type) {
    switch (type->kind) {
        case MTPSCRIPT_TYPE_MAP:
            // Validate Map key type - must be primitive with deterministic ordering
            if (!type->key) return NULL; // Skip if not fully initialized
            return validate_map_key_type(type->key);
        case MTPSCRIPT_TYPE_OPTION:
        case MTPSCRIPT_TYPE_LIST:
            // Recursively validate inner types
            if (type->inner) {
                return validate_type(type->inner);
            }
            break;
        case MTPSCRIPT_TYPE_RESULT:
            // Validate both value and error types
            if (type->value) {
                mtpscript_error_t *err = validate_type(type->value);
                if (err) return err;
            }
            if (type->error) {
                return validate_type(type->error);
            }
            break;
        default:
            break;
    }
    return NULL;
}

/* Validate Map key type - ensure no function types and deterministic ordering (§5) */
static mtpscript_error_t *validate_map_key_type(mtpscript_type_t *key_type) {
    // Map keys cannot be functions (function exclusion constraint)
    // This is a simplified check - in a full implementation, this would recursively
    // check all nested types to ensure no function types exist in map keys

    // For now, we just ensure the key type is one of the allowed primitive types
    // that have deterministic ordering: Int, String, Bool, Decimal
    switch (key_type->kind) {
        case MTPSCRIPT_TYPE_INT:
        case MTPSCRIPT_TYPE_STRING:
        case MTPSCRIPT_TYPE_BOOL:
        case MTPSCRIPT_TYPE_DECIMAL:
            // These types have deterministic ordering via Tag → Hash → CBOR
            return NULL;
        default:
            {
                mtpscript_error_t *error = MTPSCRIPT_MALLOC(sizeof(mtpscript_error_t));
                error->message = mtpscript_string_from_cstr("Map keys must be primitive types with deterministic ordering (Int, String, Bool, Decimal)");
                error->location = (mtpscript_location_t){0, 0, "map_key_validation"};
                return error;
            }
    }
}

static mtpscript_error_t *validate_function_effects(mtpscript_function_decl_t *func, mtpscript_vector_t *used_effects) {
    // Check that all used effects are declared
    for (size_t i = 0; i < used_effects->size; i++) {
        mtpscript_string_t *used_effect = mtpscript_vector_get(used_effects, i);
        bool declared = false;

        for (size_t j = 0; j < func->effects->size; j++) {
            mtpscript_string_t *declared_effect = mtpscript_vector_get(func->effects, j);
            if (strcmp(mtpscript_string_cstr(used_effect), mtpscript_string_cstr(declared_effect)) == 0) {
                declared = true;
                break;
            }
        }

        if (!declared) {
            // Return error: undeclared effect usage
            mtpscript_error_t *error = MTPSCRIPT_MALLOC(sizeof(mtpscript_error_t));
            error->message = mtpscript_string_from_cstr("Function uses undeclared effect");
            error->location = (mtpscript_location_t){0, 0, NULL}; // Would need proper location
            return error;
        }
    }

    // Check that all declared effects are actually used (optional, but good practice)
    for (size_t i = 0; i < func->effects->size; i++) {
        mtpscript_string_t *declared_effect = mtpscript_vector_get(func->effects, i);
        bool used = false;

        for (size_t j = 0; j < used_effects->size; j++) {
            mtpscript_string_t *used_effect = mtpscript_vector_get(used_effects, j);
            if (strcmp(mtpscript_string_cstr(declared_effect), mtpscript_string_cstr(used_effect)) == 0) {
                used = true;
                break;
            }
        }

        // For now, we allow declared but unused effects (could be future-proofing)
        // In a strict implementation, this might be an error
        (void)used;
    }

    return NULL;
}

static mtpscript_error_t *typecheck_statement(mtpscript_statement_t *stmt, mtpscript_type_env_t *env) {
    switch (stmt->kind) {
        case MTPSCRIPT_STMT_RETURN_STMT: {
            mtpscript_type_t *type;
            return typecheck_expression(stmt->data.return_stmt.expression, env, &type);
        }
        case MTPSCRIPT_STMT_VAR_DECL: {
            const char *var_name = mtpscript_string_cstr(stmt->data.var_decl.name);

            // Check immutability: variable cannot be redeclared in same scope
            if (mtpscript_hash_get(env->declared, var_name)) {
                // Return error: variable already declared (immutability violation)
                mtpscript_error_t *error = MTPSCRIPT_MALLOC(sizeof(mtpscript_error_t));
                error->message = mtpscript_string_from_cstr("Variable already declared in this scope (immutability violation)");
                error->location = stmt->location;
                return error;
            }

            mtpscript_type_t *init_type;
            mtpscript_error_t *expr_error = typecheck_expression(stmt->data.var_decl.initializer, env, &init_type);
            if (expr_error) return expr_error;

            mtpscript_hash_set(env->env, var_name, init_type);
            mtpscript_hash_set(env->declared, var_name, (void*)1); // Mark as declared
            break;
        }
        default: break;
    }
    return NULL;
}

static mtpscript_error_t *typecheck_declaration(mtpscript_declaration_t *decl, mtpscript_type_env_t *env) {
    if (decl->kind == MTPSCRIPT_DECL_IMPORT) {
        // Import declarations don't add to the local type environment
        // They would be resolved by the module system during compilation
        // For now, just validate the import syntax
        return NULL;
    } else if (decl->kind == MTPSCRIPT_DECL_FUNCTION) {
        mtpscript_type_env_t *local_env = mtpscript_type_env_new();
        // Add params to local env (mark as declared for immutability)
        for (size_t i = 0; i < decl->data.function.params->size; i++) {
            mtpscript_param_t *param = mtpscript_vector_get(decl->data.function.params, i);
            const char *param_name = mtpscript_string_cstr(param->name);
            mtpscript_hash_set(local_env->env, param_name, param->type);
            mtpscript_hash_set(local_env->declared, param_name, (void*)1); // Mark as declared
        }

        // Type check function body
        for (size_t i = 0; i < decl->data.function.body->size; i++) {
            mtpscript_error_t *stmt_error = typecheck_statement(mtpscript_vector_get(decl->data.function.body, i), local_env);
            if (stmt_error) {
                mtpscript_type_env_free(local_env);
                return stmt_error;
            }
        }

        // Validate effects: check that used effects are declared
        mtpscript_error_t *effect_error = validate_function_effects(&decl->data.function, local_env->used_effects);
        mtpscript_type_env_free(local_env);
        if (effect_error) return effect_error;
    }
    return NULL;
}

mtpscript_error_t *mtpscript_typecheck_program(mtpscript_program_t *program) {
    mtpscript_type_env_t *env = mtpscript_type_env_new();

    // First pass: validate all types in the program for Map constraints
    for (size_t i = 0; i < program->declarations->size; i++) {
        mtpscript_declaration_t *decl = mtpscript_vector_get(program->declarations, i);
        if (decl->kind == MTPSCRIPT_DECL_FUNCTION) {
            // Validate function parameter types
            for (size_t j = 0; j < decl->data.function.params->size; j++) {
                mtpscript_param_t *param = mtpscript_vector_get(decl->data.function.params, j);
                mtpscript_error_t *type_error = validate_type(param->type);
                if (type_error) {
                    mtpscript_type_env_free(env);
                    return type_error;
                }
            }
            // Validate return type
            if (decl->data.function.return_type) {
                mtpscript_error_t *type_error = validate_type(decl->data.function.return_type);
                if (type_error) {
                    mtpscript_type_env_free(env);
                    return type_error;
                }
            }
        }
    }

    // Second pass: normal type checking
    for (size_t i = 0; i < program->declarations->size; i++) {
        mtpscript_error_t *decl_error = typecheck_declaration(mtpscript_vector_get(program->declarations, i), env);
        if (decl_error) {
            mtpscript_type_env_free(env);
            return decl_error;
        }
    }

    mtpscript_type_env_free(env);
    return NULL;
}
