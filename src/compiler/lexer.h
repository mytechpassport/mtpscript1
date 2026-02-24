/**
 * MTPScript Lexer
 * Specification §4.1
 *
 * Copyright (c) 2025 My Tech Passport Inc.
 * Author: Ryan Wong
 */

#ifndef MTPSCRIPT_LEXER_H
#define MTPSCRIPT_LEXER_H

#include "mtpscript.h"

typedef enum {
    MTPSCRIPT_TOKEN_EOF,
    MTPSCRIPT_TOKEN_IDENTIFIER,
    MTPSCRIPT_TOKEN_INT,
    MTPSCRIPT_TOKEN_STRING,
    MTPSCRIPT_TOKEN_DECIMAL,
    MTPSCRIPT_TOKEN_BOOL,

    // Keywords
    MTPSCRIPT_TOKEN_FUNCTION,
    MTPSCRIPT_TOKEN_API,
    MTPSCRIPT_TOKEN_USES,
    MTPSCRIPT_TOKEN_LET,
    MTPSCRIPT_TOKEN_CONST,
    MTPSCRIPT_TOKEN_RETURN,
    MTPSCRIPT_TOKEN_IF,
    MTPSCRIPT_TOKEN_ELSE,
    MTPSCRIPT_TOKEN_MATCH,
    MTPSCRIPT_TOKEN_AWAIT,
    MTPSCRIPT_TOKEN_IMPORT,
    MTPSCRIPT_TOKEN_FROM,
    MTPSCRIPT_TOKEN_AS,
    MTPSCRIPT_TOKEN_SERVE,
    MTPSCRIPT_TOKEN_TYPE_NUMBER,
    MTPSCRIPT_TOKEN_TYPE_STRING,
    MTPSCRIPT_TOKEN_TYPE_BOOLEAN,

    // Punctuation
    MTPSCRIPT_TOKEN_LPAREN,
    MTPSCRIPT_TOKEN_RPAREN,
    MTPSCRIPT_TOKEN_LBRACE,
    MTPSCRIPT_TOKEN_RBRACE,
    MTPSCRIPT_TOKEN_LBRACKET,
    MTPSCRIPT_TOKEN_RBRACKET,
    MTPSCRIPT_TOKEN_COMMA,
    MTPSCRIPT_TOKEN_COLON,
    MTPSCRIPT_TOKEN_ARROW,
    MTPSCRIPT_TOKEN_EQUALS,
    MTPSCRIPT_TOKEN_PLUS,
    MTPSCRIPT_TOKEN_MINUS,
    MTPSCRIPT_TOKEN_STAR,
    MTPSCRIPT_TOKEN_SLASH,
    MTPSCRIPT_TOKEN_PIPE,  // |> pipeline operator
    MTPSCRIPT_TOKEN_LANGLE,  // <
    MTPSCRIPT_TOKEN_RANGLE,  // >
    MTPSCRIPT_TOKEN_DOT,     // .

    // HTTP Methods (for API decl)
    MTPSCRIPT_TOKEN_GET,
    MTPSCRIPT_TOKEN_POST,
    MTPSCRIPT_TOKEN_PUT,
    MTPSCRIPT_TOKEN_DELETE,
    MTPSCRIPT_TOKEN_PATCH
} mtpscript_token_type_t;

typedef struct {
    mtpscript_token_type_t type;
    mtpscript_string_t *lexeme;
    mtpscript_location_t location;
} mtpscript_token_t;

typedef struct {
    const char *source;
    size_t length;
    size_t position;
    int line;
    int column;
    const char *filename;
} mtpscript_lexer_t;

mtpscript_lexer_t *mtpscript_lexer_new(const char *source, const char *filename);
void mtpscript_lexer_free(mtpscript_lexer_t *lexer);
mtpscript_error_t *mtpscript_lexer_tokenize(mtpscript_lexer_t *lexer, mtpscript_vector_t **tokens);

void mtpscript_token_free(mtpscript_token_t *token);

#endif // MTPSCRIPT_LEXER_H
