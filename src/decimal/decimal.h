/**
 * MTPScript Decimal Type
 * Specification §3.4
 *
 * Copyright (c) 2025 My Tech Passport Inc.
 * Author: Ryan Wong
 */

#ifndef MTPSCRIPT_DECIMAL_H
#define MTPSCRIPT_DECIMAL_H

#include "mtpscript.h"

typedef struct {
    int64_t value;
    int32_t scale;
} mtpscript_decimal_t;

mtpscript_decimal_t mtpscript_decimal_from_string(const char *str);
mtpscript_string_t *mtpscript_decimal_to_string(mtpscript_decimal_t d);

// Decimal serialization (§23) - shortest canonical form, no -0, NaN, or Infinity
mtpscript_string_t *mtpscript_decimal_to_json(mtpscript_decimal_t d);
mtpscript_string_t *mtpscript_decimal_to_cbor(mtpscript_decimal_t d);

mtpscript_decimal_t mtpscript_decimal_add(mtpscript_decimal_t a, mtpscript_decimal_t b);
mtpscript_decimal_t mtpscript_decimal_sub(mtpscript_decimal_t a, mtpscript_decimal_t b);
mtpscript_decimal_t mtpscript_decimal_mul(mtpscript_decimal_t a, mtpscript_decimal_t b);
mtpscript_decimal_t mtpscript_decimal_div(mtpscript_decimal_t a, mtpscript_decimal_t b);
int mtpscript_decimal_cmp(mtpscript_decimal_t a, mtpscript_decimal_t b);

#endif // MTPSCRIPT_DECIMAL_H
