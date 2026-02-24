/**
 * MTPScript AST Implementation
 * Specification §4.2
 *
 * Copyright (c) 2025 My Tech Passport Inc.
 * Author: Ryan Wong
 */

#include "ast.h"
#include <string.h>
#include <stdio.h>
#include <openssl/sha.h>

mtpscript_type_t *mtpscript_type_new(mtpscript_type_kind_t kind) {
    mtpscript_type_t *type = MTPSCRIPT_MALLOC(sizeof(mtpscript_type_t));
    type->kind = kind;
    type->name = NULL;
    type->inner = NULL;
    type->key = NULL;
    type->value = NULL;
    type->error = NULL;
    type->union_variants = NULL;
    type->union_hash = NULL;
    return type;
}

// Create a union type with content hashing for exhaustiveness checking
mtpscript_type_t *mtpscript_type_union_new(mtpscript_vector_t *variants) {
    mtpscript_type_t *type = mtpscript_type_new(MTPSCRIPT_TYPE_UNION);
    type->union_variants = variants;

    // Generate SHA-256 hash of union variants for exhaustiveness checking
    SHA256_CTX sha256;
    SHA256_Init(&sha256);

    // Sort variants alphabetically for deterministic hashing
    mtpscript_vector_t *sorted_variants = mtpscript_vector_new();
    for (size_t i = 0; i < variants->size; i++) {
        mtpscript_string_t *variant = mtpscript_vector_get(variants, i);
        mtpscript_vector_push(sorted_variants, variant);
    }

    // Simple bubble sort for determinism (small number of variants expected)
    for (size_t i = 0; i < sorted_variants->size; i++) {
        for (size_t j = i + 1; j < sorted_variants->size; j++) {
            mtpscript_string_t *a = sorted_variants->items[i];
            mtpscript_string_t *b = sorted_variants->items[j];
            if (strcmp(mtpscript_string_cstr(a), mtpscript_string_cstr(b)) > 0) {
                // Swap pointers
                sorted_variants->items[i] = b;
                sorted_variants->items[j] = a;
            }
        }
    }

    // Hash the sorted variant names
    for (size_t i = 0; i < sorted_variants->size; i++) {
        mtpscript_string_t *variant = sorted_variants->items[i];
        SHA256_Update(&sha256, mtpscript_string_cstr(variant), variant->length);
        SHA256_Update(&sha256, "|", 1); // Separator
    }

    unsigned char hash[SHA256_DIGEST_LENGTH];
    SHA256_Final(hash, &sha256);

    // Convert to hex string
    char hex_hash[SHA256_DIGEST_LENGTH * 2 + 1];
    for (int i = 0; i < SHA256_DIGEST_LENGTH; i++) {
        sprintf(hex_hash + (i * 2), "%02x", hash[i]);
    }
    hex_hash[SHA256_DIGEST_LENGTH * 2] = '\0';

    type->union_hash = mtpscript_string_from_cstr(hex_hash);
    mtpscript_vector_free(sorted_variants);

    return type;
}

mtpscript_expression_t *mtpscript_expression_new(mtpscript_expression_kind_t kind) {
    mtpscript_expression_t *expr = MTPSCRIPT_MALLOC(sizeof(mtpscript_expression_t));
    expr->kind = kind;
    memset(&expr->data, 0, sizeof(expr->data));
    return expr;
}

mtpscript_statement_t *mtpscript_statement_new(mtpscript_statement_kind_t kind) {
    mtpscript_statement_t *stmt = MTPSCRIPT_MALLOC(sizeof(mtpscript_statement_t));
    stmt->kind = kind;
    memset(&stmt->data, 0, sizeof(stmt->data));
    return stmt;
}

mtpscript_declaration_t *mtpscript_declaration_new(mtpscript_declaration_kind_t kind) {
    mtpscript_declaration_t *decl = MTPSCRIPT_MALLOC(sizeof(mtpscript_declaration_t));
    decl->kind = kind;
    memset(&decl->data, 0, sizeof(decl->data));
    return decl;
}

mtpscript_program_t *mtpscript_program_new(void) {
    mtpscript_program_t *program = MTPSCRIPT_MALLOC(sizeof(mtpscript_program_t));
    program->declarations = mtpscript_vector_new();
    return program;
}

bool mtpscript_type_equals(mtpscript_type_t *a, mtpscript_type_t *b) {
    if (!a || !b) return false;
    if (a->kind != b->kind) return false;

    switch (a->kind) {
        case MTPSCRIPT_TYPE_INT:
        case MTPSCRIPT_TYPE_STRING:
        case MTPSCRIPT_TYPE_BOOL:
        case MTPSCRIPT_TYPE_DECIMAL:
            return true; // Primitive types are equal if kinds match

        case MTPSCRIPT_TYPE_OPTION:
        case MTPSCRIPT_TYPE_LIST:
            return a->inner && b->inner && mtpscript_type_equals(a->inner, b->inner);

        case MTPSCRIPT_TYPE_RESULT:
            return a->inner && b->inner && mtpscript_type_equals(a->inner, b->inner) &&
                   a->error && b->error && mtpscript_type_equals(a->error, b->error);

        case MTPSCRIPT_TYPE_MAP:
            return a->key && b->key && mtpscript_type_equals(a->key, b->key) &&
                   a->value && b->value && mtpscript_type_equals(a->value, b->value);

        case MTPSCRIPT_TYPE_CUSTOM:
            return a->name && b->name &&
                   strcmp(mtpscript_string_cstr(a->name), mtpscript_string_cstr(b->name)) == 0;

        default:
            return false;
    }
}

void mtpscript_type_free(mtpscript_type_t *type) {
    if (type) {
        if (type->name) mtpscript_string_free(type->name);
        if (type->inner) mtpscript_type_free(type->inner);
        if (type->key) mtpscript_type_free(type->key);
        if (type->value) mtpscript_type_free(type->value);
        MTPSCRIPT_FREE(type);
    }
}

void mtpscript_expression_free(mtpscript_expression_t *expr) {
    if (expr) {
        switch (expr->kind) {
            case MTPSCRIPT_EXPR_STRING_LITERAL:
            case MTPSCRIPT_EXPR_DECIMAL_LITERAL:
                if (expr->data.string_val) mtpscript_string_free(expr->data.string_val);
                break;
            case MTPSCRIPT_EXPR_VARIABLE:
                if (expr->data.variable.name) mtpscript_string_free(expr->data.variable.name);
                break;
            case MTPSCRIPT_EXPR_BINARY_EXPR:
                mtpscript_expression_free(expr->data.binary.left);
                mtpscript_expression_free(expr->data.binary.right);
                break;
            case MTPSCRIPT_EXPR_FUNCTION_CALL:
                if (expr->data.call.function_name) mtpscript_string_free(expr->data.call.function_name);
                if (expr->data.call.arguments) {
                    for (size_t i = 0; i < expr->data.call.arguments->size; i++) {
                        mtpscript_expression_free(mtpscript_vector_get(expr->data.call.arguments, i));
                    }
                    mtpscript_vector_free(expr->data.call.arguments);
                }
                break;
            case MTPSCRIPT_EXPR_BLOCK_EXPR:
                if (expr->data.block.statements) {
                    for (size_t i = 0; i < expr->data.block.statements->size; i++) {
                        mtpscript_statement_free(mtpscript_vector_get(expr->data.block.statements, i));
                    }
                    mtpscript_vector_free(expr->data.block.statements);
                }
                break;
            default: break;
        }
        MTPSCRIPT_FREE(expr);
    }
}

void mtpscript_statement_free(mtpscript_statement_t *stmt) {
    if (stmt) {
        switch (stmt->kind) {
            case MTPSCRIPT_STMT_VAR_DECL:
                if (stmt->data.var_decl.name) mtpscript_string_free(stmt->data.var_decl.name);
                if (stmt->data.var_decl.type) mtpscript_type_free(stmt->data.var_decl.type);
                if (stmt->data.var_decl.initializer) mtpscript_expression_free(stmt->data.var_decl.initializer);
                break;
            case MTPSCRIPT_STMT_RETURN_STMT:
                if (stmt->data.return_stmt.expression) mtpscript_expression_free(stmt->data.return_stmt.expression);
                break;
            case MTPSCRIPT_STMT_EXPRESSION_STMT:
                if (stmt->data.expression_stmt.expression) mtpscript_expression_free(stmt->data.expression_stmt.expression);
                break;
        }
        MTPSCRIPT_FREE(stmt);
    }
}

void mtpscript_declaration_free(mtpscript_declaration_t *decl) {
    if (decl) {
        if (decl->kind == MTPSCRIPT_DECL_FUNCTION) {
            if (decl->data.function.name) mtpscript_string_free(decl->data.function.name);
            if (decl->data.function.params) {
                for (size_t i = 0; i < decl->data.function.params->size; i++) {
                    mtpscript_param_t *param = mtpscript_vector_get(decl->data.function.params, i);
                    if (param->name) mtpscript_string_free(param->name);
                    if (param->type) mtpscript_type_free(param->type);
                    MTPSCRIPT_FREE(param);
                }
                mtpscript_vector_free(decl->data.function.params);
            }
            if (decl->data.function.return_type) mtpscript_type_free(decl->data.function.return_type);
            if (decl->data.function.body) {
                for (size_t i = 0; i < decl->data.function.body->size; i++) {
                    mtpscript_statement_free(mtpscript_vector_get(decl->data.function.body, i));
                }
                mtpscript_vector_free(decl->data.function.body);
            }
            if (decl->data.function.effects) mtpscript_vector_free(decl->data.function.effects);
        } else if (decl->kind == MTPSCRIPT_DECL_API) {
            if (decl->data.api.method) mtpscript_string_free(decl->data.api.method);
            if (decl->data.api.path) mtpscript_string_free(decl->data.api.path);
            if (decl->data.api.handler) {
                // handler is a mtpscript_function_decl_t, but it's part of the union?
                // Actually in the header api.handler is a pointer to function_decl.
                // But the function_decl itself is inside the union. This needs care.
                // Let's assume handler is managed separately if it's a pointer.
            }
        }
        MTPSCRIPT_FREE(decl);
    }
}

void mtpscript_program_free(mtpscript_program_t *program) {
    if (program) {
        if (program->declarations) {
            for (size_t i = 0; i < program->declarations->size; i++) {
                mtpscript_declaration_free(mtpscript_vector_get(program->declarations, i));
            }
            mtpscript_vector_free(program->declarations);
        }
        MTPSCRIPT_FREE(program);
    }
}
