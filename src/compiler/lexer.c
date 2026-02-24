/**
 * MTPScript Lexer Implementation
 * Specification §4.1
 *
 * Copyright (c) 2025 My Tech Passport Inc.
 * Author: Ryan Wong
 */

#include "lexer.h"
#include <string.h>
#include <ctype.h>
#include <stdio.h>

mtpscript_lexer_t *mtpscript_lexer_new(const char *source, const char *filename) {
    mtpscript_lexer_t *lexer = MTPSCRIPT_MALLOC(sizeof(mtpscript_lexer_t));
    lexer->source = source;
    lexer->length = strlen(source);
    lexer->position = 0;
    lexer->line = 1;
    lexer->column = 1;
    lexer->filename = filename;
    return lexer;
}

void mtpscript_lexer_free(mtpscript_lexer_t *lexer) {
    if (lexer) MTPSCRIPT_FREE(lexer);
}

void mtpscript_token_free(mtpscript_token_t *token) {
    if (token) {
        if (token->lexeme) mtpscript_string_free(token->lexeme);
        MTPSCRIPT_FREE(token);
    }
}

static char peek(mtpscript_lexer_t *lexer) {
    if (lexer->position >= lexer->length) return '\0';
    return lexer->source[lexer->position];
}

static char advance(mtpscript_lexer_t *lexer) {
    char c = peek(lexer);
    if (c != '\0') {
        lexer->position++;
        if (c == '\n') {
            lexer->line++;
            lexer->column = 1;
        } else {
            lexer->column++;
        }
    }
    return c;
}

static void skip_whitespace(mtpscript_lexer_t *lexer) {
    while (isspace(peek(lexer))) {
        advance(lexer);
    }
}

static mtpscript_token_t *create_token(mtpscript_lexer_t *lexer, mtpscript_token_type_t type, const char *lexeme) {
    mtpscript_token_t *token = MTPSCRIPT_MALLOC(sizeof(mtpscript_token_t));
    token->type = type;
    token->lexeme = mtpscript_string_from_cstr(lexeme);
    token->location.line = lexer->line;
    token->location.column = lexer->column;
    token->location.file = lexer->filename;
    return token;
}

mtpscript_error_t *mtpscript_lexer_tokenize(mtpscript_lexer_t *lexer, mtpscript_vector_t **tokens_out) {
    mtpscript_vector_t *tokens = mtpscript_vector_new();
    *tokens_out = tokens;

    while (peek(lexer) != '\0') {
        skip_whitespace(lexer);
        char c = peek(lexer);
        if (c == '\0') break;

        if (isalpha(c) || c == '_') {
            mtpscript_string_t *buf = mtpscript_string_new();
            while (isalnum(peek(lexer)) || peek(lexer) == '_') {
                char ch = advance(lexer);
                mtpscript_string_append(buf, &ch, 1);
            }
            const char *lexeme = mtpscript_string_cstr(buf);
            mtpscript_token_type_t type = MTPSCRIPT_TOKEN_IDENTIFIER;
            if (strcmp(lexeme, "function") == 0) type = MTPSCRIPT_TOKEN_FUNCTION;
            else if (strcmp(lexeme, "fn") == 0) type = MTPSCRIPT_TOKEN_FUNCTION; // Support short form
            else if (strcmp(lexeme, "api") == 0) type = MTPSCRIPT_TOKEN_API;
            else if (strcmp(lexeme, "uses") == 0) type = MTPSCRIPT_TOKEN_USES;
            else if (strcmp(lexeme, "let") == 0) type = MTPSCRIPT_TOKEN_LET;
            else if (strcmp(lexeme, "const") == 0) type = MTPSCRIPT_TOKEN_CONST;
            else if (strcmp(lexeme, "return") == 0) type = MTPSCRIPT_TOKEN_RETURN;
            else if (strcmp(lexeme, "if") == 0) type = MTPSCRIPT_TOKEN_IF;
            else if (strcmp(lexeme, "else") == 0) type = MTPSCRIPT_TOKEN_ELSE;
            else if (strcmp(lexeme, "import") == 0) type = MTPSCRIPT_TOKEN_IMPORT;
            else if (strcmp(lexeme, "from") == 0) type = MTPSCRIPT_TOKEN_FROM;
            else if (strcmp(lexeme, "as") == 0) type = MTPSCRIPT_TOKEN_AS;
            else if (strcmp(lexeme, "serve") == 0) type = MTPSCRIPT_TOKEN_SERVE;
            else if (strcmp(lexeme, "true") == 0) type = MTPSCRIPT_TOKEN_BOOL;
            else if (strcmp(lexeme, "false") == 0) type = MTPSCRIPT_TOKEN_BOOL;
            else if (strcmp(lexeme, "number") == 0) type = MTPSCRIPT_TOKEN_TYPE_NUMBER;
            else if (strcmp(lexeme, "string") == 0) type = MTPSCRIPT_TOKEN_TYPE_STRING;
            else if (strcmp(lexeme, "boolean") == 0) type = MTPSCRIPT_TOKEN_TYPE_BOOLEAN;
            else if (strcmp(lexeme, "GET") == 0) type = MTPSCRIPT_TOKEN_GET;
            else if (strcmp(lexeme, "await") == 0) type = MTPSCRIPT_TOKEN_AWAIT;
            else if (strcmp(lexeme, "POST") == 0) type = MTPSCRIPT_TOKEN_POST;

            mtpscript_vector_push(tokens, create_token(lexer, type, lexeme));
            mtpscript_string_free(buf);
        } else if (isdigit(c)) {
            mtpscript_string_t *buf = mtpscript_string_new();
            bool is_decimal = false;
            while (isdigit(peek(lexer)) || peek(lexer) == '.') {
                if (peek(lexer) == '.') is_decimal = true;
                char ch = advance(lexer);
                mtpscript_string_append(buf, &ch, 1);
            }
            mtpscript_vector_push(tokens, create_token(lexer, is_decimal ? MTPSCRIPT_TOKEN_DECIMAL : MTPSCRIPT_TOKEN_INT, mtpscript_string_cstr(buf)));
            mtpscript_string_free(buf);
        } else if (c == '"') {
            // String literal
            advance(lexer); // consume opening quote
            mtpscript_string_t *buf = mtpscript_string_new();
            while (peek(lexer) != '"' && peek(lexer) != '\0') {
                char ch = advance(lexer);
                mtpscript_string_append(buf, &ch, 1);
            }
            if (peek(lexer) == '"') {
                advance(lexer); // consume closing quote
            }
            mtpscript_vector_push(tokens, create_token(lexer, MTPSCRIPT_TOKEN_STRING, mtpscript_string_cstr(buf)));
            mtpscript_string_free(buf);
        } else {
            advance(lexer);
            char lexeme[2] = {c, '\0'};
            mtpscript_token_type_t type;
            switch (c) {
                case '(': type = MTPSCRIPT_TOKEN_LPAREN; break;
                case ')': type = MTPSCRIPT_TOKEN_RPAREN; break;
                case '{': type = MTPSCRIPT_TOKEN_LBRACE; break;
                case '}': type = MTPSCRIPT_TOKEN_RBRACE; break;
                case '[': type = MTPSCRIPT_TOKEN_LBRACKET; break;
                case ']': type = MTPSCRIPT_TOKEN_RBRACKET; break;
                case '+': type = MTPSCRIPT_TOKEN_PLUS; break;
                case '-':
                    if (peek(lexer) == '>') {
                        advance(lexer);
                        mtpscript_vector_push(tokens, create_token(lexer, MTPSCRIPT_TOKEN_ARROW, "->"));
                        continue;
                    }
                    type = MTPSCRIPT_TOKEN_MINUS; break;
                case '|':
                    if (peek(lexer) == '>') {
                        advance(lexer);
                        mtpscript_vector_push(tokens, create_token(lexer, MTPSCRIPT_TOKEN_PIPE, "|>"));
                        continue;
                    }
                    // Handle single | if needed, for now skip
                    continue;
                case '*': type = MTPSCRIPT_TOKEN_STAR; break;
                case '/': type = MTPSCRIPT_TOKEN_SLASH; break;
                case '=': type = MTPSCRIPT_TOKEN_EQUALS; break;
                case ':': type = MTPSCRIPT_TOKEN_COLON; break;
                case ',': type = MTPSCRIPT_TOKEN_COMMA; break;
                case '<': type = MTPSCRIPT_TOKEN_LANGLE; break;
                case '>': type = MTPSCRIPT_TOKEN_RANGLE; break;
                case '.': type = MTPSCRIPT_TOKEN_DOT; break;
                default:
                    {
                        mtpscript_error_t *error = MTPSCRIPT_MALLOC(sizeof(mtpscript_error_t));
                        char msg[256];
                        sprintf(msg, "Unexpected character: '%c'", c);
                        error->message = mtpscript_string_from_cstr(msg);
                        error->location.line = lexer->line;
                        error->location.column = lexer->column;
                        error->location.file = lexer->filename;
                        return error;
                    }
            }
            mtpscript_vector_push(tokens, create_token(lexer, type, lexeme));
        }
    }
    mtpscript_vector_push(tokens, create_token(lexer, MTPSCRIPT_TOKEN_EOF, ""));
    return NULL;
}
