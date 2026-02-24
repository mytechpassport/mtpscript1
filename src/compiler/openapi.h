/**
 * MTPScript OpenAPI Generator
 * Specification §7.0
 *
 * Copyright (c) 2025 My Tech Passport Inc.
 * Author: Ryan Wong
 */

#ifndef MTPSCRIPT_OPENAPI_H
#define MTPSCRIPT_OPENAPI_H

#include "ast.h"

mtpscript_error_t *mtpscript_openapi_generate(mtpscript_program_t *program, mtpscript_string_t **output);

#endif // MTPSCRIPT_OPENAPI_H
