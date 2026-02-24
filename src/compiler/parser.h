/**
 * MTPScript Parser
 * Specification §4.2
 *
 * Copyright (c) 2025 My Tech Passport Inc.
 * Author: Ryan Wong
 */

#ifndef MTPSCRIPT_PARSER_H
#define MTPSCRIPT_PARSER_H

#include "lexer.h"
#include "ast.h"

typedef struct {
    mtpscript_vector_t *tokens;
    size_t position;
} mtpscript_parser_t;

mtpscript_parser_t *mtpscript_parser_new(mtpscript_vector_t *tokens);
void mtpscript_parser_free(mtpscript_parser_t *parser);
mtpscript_error_t *mtpscript_parser_parse(mtpscript_parser_t *parser, mtpscript_program_t **program);

#endif // MTPSCRIPT_PARSER_H
