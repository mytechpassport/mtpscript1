/**
 * MTPScript Parser Implementation
 * Specification §4.2
 *
 * Copyright (c) 2025 My Tech Passport Inc.
 * Author: Ryan Wong
 */

#include "parser.h"
#include <string.h>
#include <stdio.h>

mtpscript_parser_t *mtpscript_parser_new(mtpscript_vector_t *tokens) {
    mtpscript_parser_t *parser = MTPSCRIPT_MALLOC(sizeof(mtpscript_parser_t));
    parser->tokens = tokens;
    parser->position = 0;
    return parser;
}

void mtpscript_parser_free(mtpscript_parser_t *parser) {
    if (parser) MTPSCRIPT_FREE(parser);
}

static mtpscript_token_t *peek_token(mtpscript_parser_t *parser) {
    return mtpscript_vector_get(parser->tokens, parser->position);
}

static mtpscript_token_t *advance_token(mtpscript_parser_t *parser) {
    mtpscript_token_t *token = peek_token(parser);
    if (token->type != MTPSCRIPT_TOKEN_EOF) {
        parser->position++;
    }
    return token;
}

static bool check_token(mtpscript_parser_t *parser, mtpscript_token_type_t type) {
    return peek_token(parser)->type == type;
}

static bool match_token(mtpscript_parser_t *parser, mtpscript_token_type_t type) {
    if (check_token(parser, type)) {
        advance_token(parser);
        return true;
    }
    return false;
}

static mtpscript_type_t *parse_type(mtpscript_parser_t *parser) {
    mtpscript_token_t *token = advance_token(parser);
    mtpscript_type_t *type;

    if (token->type == MTPSCRIPT_TOKEN_TYPE_NUMBER || strcmp(mtpscript_string_cstr(token->lexeme), "Int") == 0) {
        type = mtpscript_type_new(MTPSCRIPT_TYPE_INT);
    } else if (token->type == MTPSCRIPT_TOKEN_TYPE_STRING || strcmp(mtpscript_string_cstr(token->lexeme), "String") == 0) {
        type = mtpscript_type_new(MTPSCRIPT_TYPE_STRING);
    } else if (token->type == MTPSCRIPT_TOKEN_TYPE_BOOLEAN || strcmp(mtpscript_string_cstr(token->lexeme), "Bool") == 0) {
        type = mtpscript_type_new(MTPSCRIPT_TYPE_BOOL);
    } else if (strcmp(mtpscript_string_cstr(token->lexeme), "Decimal") == 0) {
        type = mtpscript_type_new(MTPSCRIPT_TYPE_DECIMAL);
    } else if (strcmp(mtpscript_string_cstr(token->lexeme), "Option") == 0) {
        type = mtpscript_type_new(MTPSCRIPT_TYPE_OPTION);
        match_token(parser, MTPSCRIPT_TOKEN_LANGLE);
        type->inner = parse_type(parser);
        match_token(parser, MTPSCRIPT_TOKEN_RANGLE);
    } else if (strcmp(mtpscript_string_cstr(token->lexeme), "Result") == 0) {
        type = mtpscript_type_new(MTPSCRIPT_TYPE_RESULT);
        match_token(parser, MTPSCRIPT_TOKEN_LANGLE);
        type->inner = parse_type(parser);  // T (success type)
        match_token(parser, MTPSCRIPT_TOKEN_COMMA);
        type->error = parse_type(parser);  // E (error type)
        match_token(parser, MTPSCRIPT_TOKEN_RANGLE);
    } else {
        type = mtpscript_type_new(MTPSCRIPT_TYPE_CUSTOM);
        type->name = mtpscript_string_from_cstr(mtpscript_string_cstr(token->lexeme));
    }
    return type;
}

static mtpscript_expression_t *parse_primary_expression(mtpscript_parser_t *parser) {
    mtpscript_token_t *token;

    // Check for await
    if (match_token(parser, MTPSCRIPT_TOKEN_AWAIT)) {
        mtpscript_expression_t *await_expr = mtpscript_expression_new(MTPSCRIPT_EXPR_AWAIT_EXPR);
        await_expr->data.await.expression = parse_primary_expression(parser);
        return await_expr;
    }

    token = advance_token(parser);
    mtpscript_expression_t *expr;
    if (token->type == MTPSCRIPT_TOKEN_INT) {
        expr = mtpscript_expression_new(MTPSCRIPT_EXPR_INT_LITERAL);
        expr->data.int_val = atoll(mtpscript_string_cstr(token->lexeme));
    } else if (token->type == MTPSCRIPT_TOKEN_DECIMAL) {
        expr = mtpscript_expression_new(MTPSCRIPT_EXPR_DECIMAL_LITERAL);
        expr->data.decimal_val = mtpscript_string_from_cstr(mtpscript_string_cstr(token->lexeme));
    } else if (token->type == MTPSCRIPT_TOKEN_BOOL) {
        expr = mtpscript_expression_new(MTPSCRIPT_EXPR_BOOL_LITERAL);
        expr->data.bool_val = strcmp(mtpscript_string_cstr(token->lexeme), "true") == 0;
    } else if (token->type == MTPSCRIPT_TOKEN_IDENTIFIER) {
        expr = mtpscript_expression_new(MTPSCRIPT_EXPR_VARIABLE);
        expr->data.variable.name = mtpscript_string_from_cstr(mtpscript_string_cstr(token->lexeme));
    } else {
        // Fallback
        expr = mtpscript_expression_new(MTPSCRIPT_EXPR_INT_LITERAL);
    }
    return expr;
}

static mtpscript_expression_t *parse_expression(mtpscript_parser_t *parser) {
    mtpscript_expression_t *expr = parse_primary_expression(parser);

    // Handle binary operators
    if (check_token(parser, MTPSCRIPT_TOKEN_STAR) ||
        check_token(parser, MTPSCRIPT_TOKEN_PLUS) ||
        check_token(parser, MTPSCRIPT_TOKEN_MINUS) ||
        check_token(parser, MTPSCRIPT_TOKEN_SLASH)) {
        mtpscript_token_t *op_token = advance_token(parser);
        mtpscript_expression_t *binary_expr = mtpscript_expression_new(MTPSCRIPT_EXPR_BINARY_EXPR);
        binary_expr->data.binary.left = expr;
        binary_expr->data.binary.op = mtpscript_string_cstr(op_token->lexeme);
        binary_expr->data.binary.right = parse_expression(parser);
        expr = binary_expr;
    }

    // Handle pipeline operators (left-associative)
    while (match_token(parser, MTPSCRIPT_TOKEN_PIPE)) {
        mtpscript_expression_t *pipe_expr = mtpscript_expression_new(MTPSCRIPT_EXPR_PIPE_EXPR);
        pipe_expr->data.pipe.left = expr;
        pipe_expr->data.pipe.right = parse_primary_expression(parser);
        expr = pipe_expr;
    }

    return expr;
}

static mtpscript_statement_t *parse_statement(mtpscript_parser_t *parser) {
    if (match_token(parser, MTPSCRIPT_TOKEN_RETURN)) {
        mtpscript_statement_t *stmt = mtpscript_statement_new(MTPSCRIPT_STMT_RETURN_STMT);
        stmt->data.return_stmt.expression = parse_expression(parser);
        return stmt;
    } else if (match_token(parser, MTPSCRIPT_TOKEN_LET) || match_token(parser, MTPSCRIPT_TOKEN_CONST)) {
        mtpscript_statement_t *stmt = mtpscript_statement_new(MTPSCRIPT_STMT_VAR_DECL);
        mtpscript_token_t *name_token = advance_token(parser);
        stmt->data.var_decl.name = mtpscript_string_from_cstr(mtpscript_string_cstr(name_token->lexeme));

        // Optional type annotation
        if (match_token(parser, MTPSCRIPT_TOKEN_COLON)) {
            stmt->data.var_decl.type = parse_type(parser);
        } else {
            stmt->data.var_decl.type = NULL;
        }

        match_token(parser, MTPSCRIPT_TOKEN_EQUALS);
        stmt->data.var_decl.initializer = parse_expression(parser);
        return stmt;
    }
    // Fallback
    mtpscript_statement_t *stmt = mtpscript_statement_new(MTPSCRIPT_STMT_EXPRESSION_STMT);
    stmt->data.expression_stmt.expression = parse_expression(parser);
    return stmt;
}

static mtpscript_declaration_t *parse_declaration(mtpscript_parser_t *parser) {
    if (match_token(parser, MTPSCRIPT_TOKEN_IMPORT)) {
        mtpscript_declaration_t *decl = mtpscript_declaration_new(MTPSCRIPT_DECL_IMPORT);

        // Parse module name
        if (!check_token(parser, MTPSCRIPT_TOKEN_IDENTIFIER)) {
            // Error: expected module name
            return NULL;
        }
        mtpscript_token_t *module_token = advance_token(parser);
        decl->data.import.module_name = mtpscript_string_from_cstr(mtpscript_string_cstr(module_token->lexeme));

        // Parse 'from'
        if (!match_token(parser, MTPSCRIPT_TOKEN_FROM)) {
            // Error: expected 'from'
            return NULL;
        }

        // Parse git URL with hash (string literal containing URL#hash)
        if (!check_token(parser, MTPSCRIPT_TOKEN_STRING)) {
            // Error: expected git URL
            return NULL;
        }
        mtpscript_token_t *url_token = advance_token(parser);
        const char *url_with_hash = mtpscript_string_cstr(url_token->lexeme);
        const char *hash_pos = strrchr(url_with_hash, '#');
        if (!hash_pos) {
            // Error: expected #hash in URL
            return NULL;
        }

        // Split URL and hash
        size_t url_len = hash_pos - url_with_hash;
        char *url_only = MTPSCRIPT_MALLOC(url_len + 1);
        memcpy(url_only, url_with_hash, url_len);
        url_only[url_len] = '\0';

        decl->data.import.git_url = mtpscript_string_from_cstr(url_only);
        decl->data.import.git_hash = mtpscript_string_from_cstr(hash_pos + 1);
        MTPSCRIPT_FREE(url_only);

        // Optional: parse 'as' tag
        decl->data.import.tag = NULL;
        if (match_token(parser, MTPSCRIPT_TOKEN_AS)) {
            mtpscript_token_t *tag_token = advance_token(parser);
            if (tag_token->type != MTPSCRIPT_TOKEN_STRING) {
                // Error: expected tag string
                return NULL;
            }
            decl->data.import.tag = mtpscript_string_from_cstr(mtpscript_string_cstr(tag_token->lexeme));
        }

        // Parse import list
        decl->data.import.imports = mtpscript_vector_new();
        if (match_token(parser, MTPSCRIPT_TOKEN_LBRACE)) {
            while (!check_token(parser, MTPSCRIPT_TOKEN_RBRACE) && !check_token(parser, MTPSCRIPT_TOKEN_EOF)) {
                if (!check_token(parser, MTPSCRIPT_TOKEN_IDENTIFIER)) {
                    // Error: expected identifier
                    return NULL;
                }
                mtpscript_token_t *import_token = advance_token(parser);
                mtpscript_vector_push(decl->data.import.imports,
                                    mtpscript_string_from_cstr(mtpscript_string_cstr(import_token->lexeme)));
                if (!match_token(parser, MTPSCRIPT_TOKEN_COMMA)) break;
            }
            if (!match_token(parser, MTPSCRIPT_TOKEN_RBRACE)) {
                // Error: expected closing brace
                return NULL;
            }
        }

        return decl;
    } else if (match_token(parser, MTPSCRIPT_TOKEN_API)) {
        mtpscript_declaration_t *decl = mtpscript_declaration_new(MTPSCRIPT_DECL_API);

        // Parse HTTP method - should be GET, POST, etc.
        mtpscript_token_t *method_token = advance_token(parser);
        if (method_token->type < MTPSCRIPT_TOKEN_GET || method_token->type > MTPSCRIPT_TOKEN_DELETE) {
            // For now, accept any identifier as method
            decl->data.api.method = mtpscript_string_from_cstr(mtpscript_string_cstr(method_token->lexeme));
        } else {
            decl->data.api.method = mtpscript_string_from_cstr(mtpscript_string_cstr(method_token->lexeme));
        }

        // Parse path (string literal)
        mtpscript_token_t *path_token = advance_token(parser);
        if (path_token->type != MTPSCRIPT_TOKEN_STRING) {
            // Error handling
            return NULL;
        }
        decl->data.api.path = mtpscript_string_from_cstr(mtpscript_string_cstr(path_token->lexeme));

        // Parse the function that follows
        if (!match_token(parser, MTPSCRIPT_TOKEN_FUNCTION)) {
            // Error: expected 'func' after API path
            return NULL;
        }

        // Parse function name
        mtpscript_token_t *func_name = advance_token(parser);
        if (func_name->type != MTPSCRIPT_TOKEN_IDENTIFIER) {
            return NULL;
        }

        // Parse parameters
        if (!match_token(parser, MTPSCRIPT_TOKEN_LPAREN)) {
            return NULL;
        }
        mtpscript_vector_t *params = mtpscript_vector_new();
        if (!match_token(parser, MTPSCRIPT_TOKEN_RPAREN)) {
            do {
                mtpscript_param_t *param = MTPSCRIPT_MALLOC(sizeof(mtpscript_param_t));
                mtpscript_token_t *param_name = advance_token(parser);
                param->name = mtpscript_string_from_cstr(mtpscript_string_cstr(param_name->lexeme));
                if (!match_token(parser, MTPSCRIPT_TOKEN_COLON)) {
                    return NULL;
                }
                param->type = parse_type(parser);
                mtpscript_vector_push(params, param);
            } while (match_token(parser, MTPSCRIPT_TOKEN_COMMA));
            if (!match_token(parser, MTPSCRIPT_TOKEN_RPAREN)) {
                return NULL;
            }
        }

        // Parse return type
        mtpscript_type_t *return_type = NULL;
        if (match_token(parser, MTPSCRIPT_TOKEN_COLON)) {
            return_type = parse_type(parser);
        }

        // Parse effects (skip for now)
        mtpscript_vector_t *effects = mtpscript_vector_new();

        // Parse function body
        if (!match_token(parser, MTPSCRIPT_TOKEN_LBRACE)) {
            return NULL;
        }
        mtpscript_vector_t *body = mtpscript_vector_new();
        while (!check_token(parser, MTPSCRIPT_TOKEN_RBRACE) && !check_token(parser, MTPSCRIPT_TOKEN_EOF)) {
            mtpscript_statement_t *stmt = parse_statement(parser);
            if (stmt) {
                mtpscript_vector_push(body, stmt);
            }
        }
        if (!match_token(parser, MTPSCRIPT_TOKEN_RBRACE)) {
            return NULL;
        }

        // Create function declaration
        mtpscript_function_decl_t *func = MTPSCRIPT_MALLOC(sizeof(mtpscript_function_decl_t));
        func->name = mtpscript_string_from_cstr(mtpscript_string_cstr(func_name->lexeme));
        func->params = params;
        func->return_type = return_type;
        func->body = body;
        func->effects = effects;

        decl->data.api.handler = func;

        return decl;
    } else if (match_token(parser, MTPSCRIPT_TOKEN_FUNCTION)) {
        mtpscript_declaration_t *decl = mtpscript_declaration_new(MTPSCRIPT_DECL_FUNCTION);
        mtpscript_token_t *name = advance_token(parser);
        decl->data.function.name = mtpscript_string_from_cstr(mtpscript_string_cstr(name->lexeme));

        match_token(parser, MTPSCRIPT_TOKEN_LPAREN);
        decl->data.function.params = mtpscript_vector_new();
        while (!check_token(parser, MTPSCRIPT_TOKEN_RPAREN)) {
            mtpscript_param_t *param = MTPSCRIPT_MALLOC(sizeof(mtpscript_param_t));
            param->name = mtpscript_string_from_cstr(mtpscript_string_cstr(advance_token(parser)->lexeme));
            match_token(parser, MTPSCRIPT_TOKEN_COLON);
            param->type = parse_type(parser);
            mtpscript_vector_push(decl->data.function.params, param);
            if (!match_token(parser, MTPSCRIPT_TOKEN_COMMA)) break;
        }
        match_token(parser, MTPSCRIPT_TOKEN_RPAREN);

        if (match_token(parser, MTPSCRIPT_TOKEN_COLON)) {
            decl->data.function.return_type = parse_type(parser);
        }

        match_token(parser, MTPSCRIPT_TOKEN_LBRACE);
        decl->data.function.body = mtpscript_vector_new();
        while (!check_token(parser, MTPSCRIPT_TOKEN_RBRACE)) {
            mtpscript_vector_push(decl->data.function.body, parse_statement(parser));
        }
        match_token(parser, MTPSCRIPT_TOKEN_RBRACE);
        decl->data.function.effects = mtpscript_vector_new(); // Empty for now
        return decl;
    } else if (match_token(parser, MTPSCRIPT_TOKEN_SERVE)) {
        mtpscript_declaration_t *decl = mtpscript_declaration_new(MTPSCRIPT_DECL_SERVE);

        // Parse serve { port: 8080, routes: [...] }
        if (!match_token(parser, MTPSCRIPT_TOKEN_LBRACE)) {
            return NULL;
        }

        // Parse configuration object
        int port = 8080; // default
        mtpscript_string_t *host = NULL;
        mtpscript_vector_t *routes = mtpscript_vector_new();

        while (!check_token(parser, MTPSCRIPT_TOKEN_RBRACE) && !check_token(parser, MTPSCRIPT_TOKEN_EOF)) {
            mtpscript_token_t *key = advance_token(parser);
            if (!match_token(parser, MTPSCRIPT_TOKEN_COLON)) {
                return NULL;
            }

            if (strcmp(mtpscript_string_cstr(key->lexeme), "port") == 0) {
                // Parse port number
                mtpscript_token_t *port_token = advance_token(parser);
                if (port_token->type != MTPSCRIPT_TOKEN_INT) {
                    return NULL;
                }
                port = atoi(mtpscript_string_cstr(port_token->lexeme));
            } else if (strcmp(mtpscript_string_cstr(key->lexeme), "host") == 0) {
                // Parse host string
                mtpscript_token_t *host_token = advance_token(parser);
                if (host_token->type != MTPSCRIPT_TOKEN_STRING) {
                    return NULL;
                }
                host = mtpscript_string_from_cstr(mtpscript_string_cstr(host_token->lexeme));
            } else if (strcmp(mtpscript_string_cstr(key->lexeme), "routes") == 0) {
                // Parse routes array
                if (!match_token(parser, MTPSCRIPT_TOKEN_LBRACKET)) {
                    return NULL;
                }

                while (!check_token(parser, MTPSCRIPT_TOKEN_RBRACKET) && !check_token(parser, MTPSCRIPT_TOKEN_EOF)) {
                    // Parse route object { method: "GET", path: "/health", handler: health_func }
                    if (!match_token(parser, MTPSCRIPT_TOKEN_LBRACE)) {
                        return NULL;
                    }

                    mtpscript_api_decl_t *route = MTPSCRIPT_MALLOC(sizeof(mtpscript_api_decl_t));

                    while (!check_token(parser, MTPSCRIPT_TOKEN_RBRACE) && !check_token(parser, MTPSCRIPT_TOKEN_EOF)) {
                        mtpscript_token_t *route_key = advance_token(parser);
                        if (!match_token(parser, MTPSCRIPT_TOKEN_COLON)) {
                            return NULL;
                        }

                        if (strcmp(mtpscript_string_cstr(route_key->lexeme), "method") == 0) {
                            mtpscript_token_t *method_token = advance_token(parser);
                            route->method = mtpscript_string_from_cstr(mtpscript_string_cstr(method_token->lexeme));
                        } else if (strcmp(mtpscript_string_cstr(route_key->lexeme), "path") == 0) {
                            mtpscript_token_t *path_token = advance_token(parser);
                            route->path = mtpscript_string_from_cstr(mtpscript_string_cstr(path_token->lexeme));
                        } else if (strcmp(mtpscript_string_cstr(route_key->lexeme), "handler") == 0) {
                            // Parse handler function reference
                            mtpscript_token_t *handler_token = advance_token(parser);
                            // For now, just store the handler name - full parsing would require symbol resolution
                            route->handler = MTPSCRIPT_MALLOC(sizeof(mtpscript_function_decl_t));
                            route->handler->name = mtpscript_string_from_cstr(mtpscript_string_cstr(handler_token->lexeme));
                        }

                        if (!match_token(parser, MTPSCRIPT_TOKEN_COMMA)) break;
                    }

                    if (!match_token(parser, MTPSCRIPT_TOKEN_RBRACE)) {
                        return NULL;
                    }

                    mtpscript_vector_push(routes, route);
                    if (!match_token(parser, MTPSCRIPT_TOKEN_COMMA)) break;
                }

                if (!match_token(parser, MTPSCRIPT_TOKEN_RBRACKET)) {
                    return NULL;
                }
            }

            if (!match_token(parser, MTPSCRIPT_TOKEN_COMMA)) break;
        }

        if (!match_token(parser, MTPSCRIPT_TOKEN_RBRACE)) {
            return NULL;
        }

        // Create serve declaration
        mtpscript_serve_decl_t *serve = MTPSCRIPT_MALLOC(sizeof(mtpscript_serve_decl_t));
        serve->port = port;
        serve->host = host ? host : mtpscript_string_from_cstr("localhost");
        serve->routes = routes;

        decl->data.serve = *serve;

        return decl;
    }
    return NULL;
}

mtpscript_error_t *mtpscript_parser_parse(mtpscript_parser_t *parser, mtpscript_program_t **program_out) {
    mtpscript_program_t *program = mtpscript_program_new();
    *program_out = program;

    while (peek_token(parser)->type != MTPSCRIPT_TOKEN_EOF) {
        mtpscript_declaration_t *decl = parse_declaration(parser);
        if (decl) {
            mtpscript_vector_push(program->declarations, decl);
        } else {
            advance_token(parser);
        }
    }
    return NULL;
}
