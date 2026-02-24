/**
 * MTPScript Code Generator Implementation
 * Specification §5.0
 *
 * Copyright (c) 2025 My Tech Passport Inc.
 * Author: Ryan Wong
 */

#include "codegen.h"
#include <string.h>
#include <stdio.h>

static void codegen_expression(mtpscript_expression_t *expr, mtpscript_string_t *out) {
    switch (expr->kind) {
        case MTPSCRIPT_EXPR_INT_LITERAL: {
            char buf[32];
            sprintf(buf, "%lld", expr->data.int_val);
            mtpscript_string_append_cstr(out, buf);
            break;
        }
        case MTPSCRIPT_EXPR_STRING_LITERAL:
            mtpscript_string_append_cstr(out, "\"");
            mtpscript_string_append_cstr(out, mtpscript_string_cstr(expr->data.string_val));
            mtpscript_string_append_cstr(out, "\"");
            break;
        case MTPSCRIPT_EXPR_BOOL_LITERAL:
            mtpscript_string_append_cstr(out, expr->data.bool_val ? "true" : "false");
            break;
        case MTPSCRIPT_EXPR_DECIMAL_LITERAL:
            mtpscript_string_append_cstr(out, mtpscript_string_cstr(expr->data.decimal_val));
            break;
        case MTPSCRIPT_EXPR_VARIABLE:
            mtpscript_string_append_cstr(out, mtpscript_string_cstr(expr->data.variable.name));
            break;
        case MTPSCRIPT_EXPR_BINARY_EXPR:
            codegen_expression(expr->data.binary.left, out);
            mtpscript_string_append_cstr(out, " ");
            mtpscript_string_append_cstr(out, expr->data.binary.op);
            mtpscript_string_append_cstr(out, " ");
            codegen_expression(expr->data.binary.right, out);
            break;
        case MTPSCRIPT_EXPR_FUNCTION_CALL:
            mtpscript_string_append_cstr(out, mtpscript_string_cstr(expr->data.call.function_name));
            mtpscript_string_append_cstr(out, "(");
            for (size_t i = 0; i < expr->data.call.arguments->size; i++) {
                codegen_expression(mtpscript_vector_get(expr->data.call.arguments, i), out);
                if (i < expr->data.call.arguments->size - 1) mtpscript_string_append_cstr(out, ", ");
            }
            mtpscript_string_append_cstr(out, ")");
            break;
        case MTPSCRIPT_EXPR_PIPE_EXPR:
            // Left-associative: right(left)
            codegen_expression(expr->data.pipe.right, out);
            mtpscript_string_append_cstr(out, "(");
            codegen_expression(expr->data.pipe.left, out);
            mtpscript_string_append_cstr(out, ")");
            break;
        case MTPSCRIPT_EXPR_AWAIT_EXPR:
            // Desugar await e into Async.await(ph, contId, e) (§7-a)
            mtpscript_string_append_cstr(out, "Async.await(ph, contId, ");
            codegen_expression(expr->data.await.expression, out);
            mtpscript_string_append_cstr(out, ")");
            break;
        case MTPSCRIPT_EXPR_MATCH_EXPR:
            // Generate JavaScript switch-like construct for match
            mtpscript_string_append_cstr(out, "(function() {\n");
            mtpscript_string_append_cstr(out, "  const scrutinee = ");
            codegen_expression(expr->data.match.scrutinee, out);
            mtpscript_string_append_cstr(out, ";\n");

            for (size_t i = 0; i < expr->data.match.arms->size; i++) {
                mtpscript_match_arm_t *arm = mtpscript_vector_get(expr->data.match.arms, i);
                mtpscript_string_append_cstr(out, "  ");
                if (i == 0) {
                    mtpscript_string_append_cstr(out, "if");
                } else {
                    mtpscript_string_append_cstr(out, "} else if");
                }
                mtpscript_string_append_cstr(out, " (scrutinee === ");
                mtpscript_string_append_cstr(out, mtpscript_string_cstr(arm->pattern));
                mtpscript_string_append_cstr(out, ") {\n");
                mtpscript_string_append_cstr(out, "    return ");
                codegen_expression(arm->body, out);
                mtpscript_string_append_cstr(out, ";\n");
            }

            mtpscript_string_append_cstr(out, "  } else {\n");
            mtpscript_string_append_cstr(out, "    throw new Error('Non-exhaustive match');\n");
            mtpscript_string_append_cstr(out, "  }\n");
            mtpscript_string_append_cstr(out, "})()");
            break;
        default: break;
    }
}

static void codegen_statement(mtpscript_statement_t *stmt, mtpscript_string_t *out) {
    switch (stmt->kind) {
        case MTPSCRIPT_STMT_RETURN_STMT:
            mtpscript_string_append_cstr(out, "  return ");
            codegen_expression(stmt->data.return_stmt.expression, out);
            mtpscript_string_append_cstr(out, ";\n");
            break;
        case MTPSCRIPT_STMT_VAR_DECL:
            mtpscript_string_append_cstr(out, "  let ");
            mtpscript_string_append_cstr(out, mtpscript_string_cstr(stmt->data.var_decl.name));
            mtpscript_string_append_cstr(out, " = ");
            codegen_expression(stmt->data.var_decl.initializer, out);
            mtpscript_string_append_cstr(out, ";\n");
            break;
        case MTPSCRIPT_STMT_EXPRESSION_STMT:
            mtpscript_string_append_cstr(out, "  ");
            codegen_expression(stmt->data.expression_stmt.expression, out);
            mtpscript_string_append_cstr(out, ";\n");
            break;
    }
}

static void codegen_declaration(mtpscript_declaration_t *decl, mtpscript_string_t *out) {
    if (decl->kind == MTPSCRIPT_DECL_IMPORT) {
        // Generate import statement for vendored module
        mtpscript_string_append_cstr(out, "// Import ");
        mtpscript_string_append_cstr(out, mtpscript_string_cstr(decl->data.import.module_name));
        mtpscript_string_append_cstr(out, " from ");
        mtpscript_string_append_cstr(out, mtpscript_string_cstr(decl->data.import.git_url));
        mtpscript_string_append_cstr(out, "#");
        mtpscript_string_append_cstr(out, mtpscript_string_cstr(decl->data.import.git_hash));
        if (decl->data.import.tag) {
            mtpscript_string_append_cstr(out, " as ");
            mtpscript_string_append_cstr(out, mtpscript_string_cstr(decl->data.import.tag));
        }
        mtpscript_string_append_cstr(out, "\n");

        // Generate import statements for symbols
        if (decl->data.import.imports && decl->data.import.imports->size > 0) {
            mtpscript_string_append_cstr(out, "// Importing: ");
            for (size_t i = 0; i < decl->data.import.imports->size; i++) {
                if (i > 0) mtpscript_string_append_cstr(out, ", ");
                mtpscript_string_append_cstr(out, mtpscript_string_cstr(mtpscript_vector_get(decl->data.import.imports, i)));
            }
            mtpscript_string_append_cstr(out, "\n");
        }
        mtpscript_string_append_cstr(out, "\n");
    } else if (decl->kind == MTPSCRIPT_DECL_API) {
        // Generate API route handler
        mtpscript_string_append_cstr(out, "// API ");
        mtpscript_string_append_cstr(out, mtpscript_string_cstr(decl->data.api.method));
        mtpscript_string_append_cstr(out, " ");
        mtpscript_string_append_cstr(out, mtpscript_string_cstr(decl->data.api.path));
        mtpscript_string_append_cstr(out, "\n");

        if (decl->data.api.handler) {
            // Generate the function
            mtpscript_string_append_cstr(out, "function ");
            mtpscript_string_append_cstr(out, mtpscript_string_cstr(decl->data.api.handler->name));
            mtpscript_string_append_cstr(out, "(");
            for (size_t i = 0; i < decl->data.api.handler->params->size; i++) {
                mtpscript_param_t *param = mtpscript_vector_get(decl->data.api.handler->params, i);
                mtpscript_string_append_cstr(out, mtpscript_string_cstr(param->name));
                if (i < decl->data.api.handler->params->size - 1) mtpscript_string_append_cstr(out, ", ");
            }
            mtpscript_string_append_cstr(out, ") {\n");
            for (size_t i = 0; i < decl->data.api.handler->body->size; i++) {
                codegen_statement(mtpscript_vector_get(decl->data.api.handler->body, i), out);
            }
            mtpscript_string_append_cstr(out, "}\n\n");
        }
    } else if (decl->kind == MTPSCRIPT_DECL_FUNCTION) {
        mtpscript_string_append_cstr(out, "function ");
        mtpscript_string_append_cstr(out, mtpscript_string_cstr(decl->data.function.name));
        mtpscript_string_append_cstr(out, "(");
        for (size_t i = 0; i < decl->data.function.params->size; i++) {
            mtpscript_param_t *param = mtpscript_vector_get(decl->data.function.params, i);
            mtpscript_string_append_cstr(out, mtpscript_string_cstr(param->name));
            if (i < decl->data.function.params->size - 1) mtpscript_string_append_cstr(out, ", ");
        }
        mtpscript_string_append_cstr(out, ") {\n");
        for (size_t i = 0; i < decl->data.function.body->size; i++) {
            codegen_statement(mtpscript_vector_get(decl->data.function.body, i), out);
        }
        mtpscript_string_append_cstr(out, "}\n\n");
    }
}

mtpscript_error_t *mtpscript_codegen_program(mtpscript_program_t *program, mtpscript_string_t **output_out) {
    mtpscript_string_t *out = mtpscript_string_new();
    *output_out = out;

    mtpscript_string_append_cstr(out, "// Generated by MTPScript Compiler\n\n");
    for (size_t i = 0; i < program->declarations->size; i++) {
        codegen_declaration(mtpscript_vector_get(program->declarations, i), out);
    }
    return NULL;
}
