/**
 * MTPScript Decimal Type Implementation
 * Specification §3.4
 *
 * Copyright (c) 2025 My Tech Passport Inc.
 * Author: Ryan Wong
 */

#include "decimal.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>

mtpscript_decimal_t mtpscript_decimal_from_string(const char *str) {
    mtpscript_decimal_t d = {0, 0};
    const char *dot = strchr(str, '.');
    if (dot) {
        d.scale = strlen(dot + 1);
        char buf[64];
        size_t len = dot - str;
        memcpy(buf, str, len);
        strcpy(buf + len, dot + 1);
        d.value = atoll(buf);
    } else {
        d.value = atoll(str);
        d.scale = 0;
    }
    return d;
}

mtpscript_string_t *mtpscript_decimal_to_string(mtpscript_decimal_t d) {
    char buf[64];
    if (d.scale == 0) {
        sprintf(buf, "%lld", d.value);
    } else {
        long long integer_part = d.value / (long long)pow(10, d.scale);
        long long fractional_part = llabs(d.value % (long long)pow(10, d.scale));

        // Remove trailing zeros from fractional part
        int actual_scale = d.scale;
        long long temp_fractional = fractional_part;
        while (actual_scale > 0 && temp_fractional % 10 == 0) {
            temp_fractional /= 10;
            actual_scale--;
        }

        if (actual_scale == 0) {
            // No fractional part left
            sprintf(buf, "%lld", integer_part);
        } else {
            sprintf(buf, "%lld.%0*lld", integer_part, actual_scale, temp_fractional);
        }
    }
    return mtpscript_string_from_cstr(buf);
}

mtpscript_decimal_t mtpscript_decimal_add(mtpscript_decimal_t a, mtpscript_decimal_t b) {
    while (a.scale < b.scale) { a.value *= 10; a.scale++; }
    while (b.scale < a.scale) { b.value *= 10; b.scale++; }
    mtpscript_decimal_t res = {a.value + b.value, a.scale};
    return res;
}

mtpscript_decimal_t mtpscript_decimal_sub(mtpscript_decimal_t a, mtpscript_decimal_t b) {
    while (a.scale < b.scale) { a.value *= 10; a.scale++; }
    while (b.scale < a.scale) { b.value *= 10; b.scale++; }
    mtpscript_decimal_t res = {a.value - b.value, a.scale};
    return res;
}

mtpscript_decimal_t mtpscript_decimal_mul(mtpscript_decimal_t a, mtpscript_decimal_t b) {
    mtpscript_decimal_t res = {a.value * b.value, a.scale + b.scale};
    return res;
}

mtpscript_decimal_t mtpscript_decimal_div(mtpscript_decimal_t a, mtpscript_decimal_t b) {
    // Basic implementation: increase precision of numerator before dividing
    int precision_increase = 8;
    long long numerator = a.value * (long long)pow(10, precision_increase);
    mtpscript_decimal_t res = {numerator / b.value, a.scale + precision_increase - b.scale};
    return res;
}

int mtpscript_decimal_cmp(mtpscript_decimal_t a, mtpscript_decimal_t b) {
    // Normalize scales by bringing both decimals to the same scale
    mtpscript_decimal_t a_norm = a;
    mtpscript_decimal_t b_norm = b;

    while (a_norm.scale < b_norm.scale) {
        a_norm.value *= 10;
        a_norm.scale++;
    }
    while (b_norm.scale < a_norm.scale) {
        b_norm.value *= 10;
        b_norm.scale++;
    }

    if (a_norm.value < b_norm.value) return -1;
    if (a_norm.value > b_norm.value) return 1;
    return 0;
}

// Decimal serialization (§23) - shortest canonical form, no -0, NaN, or Infinity
mtpscript_string_t *mtpscript_decimal_to_json(mtpscript_decimal_t d) {
    // Remove trailing zeros from fractional part to get shortest form
    int64_t value = d.value;
    int32_t scale = d.scale;

    // Handle zero specially (no -0)
    if (value == 0) {
        return mtpscript_string_from_cstr("0");
    }

    // Remove trailing zeros from scale
    while (scale > 0 && value % 10 == 0) {
        value /= 10;
        scale--;
    }

    char buf[64];
    if (scale == 0) {
        sprintf(buf, "%lld", value);
    } else {
        // Calculate integer and fractional parts
        int64_t int_part = value;
        int64_t frac_part = 0;

        // Handle negative scale by multiplying
        if (scale < 0) {
            // This shouldn't happen in normal usage, but handle it
            int_part = value * (int64_t)pow(10, -scale);
            frac_part = 0;
            scale = 0;
        } else {
            // Split into integer and fractional parts
            int64_t divisor = (int64_t)pow(10, scale);
            int_part = value / divisor;
            frac_part = value % divisor;

            // Remove trailing zeros from fractional part
            while (frac_part > 0 && frac_part % 10 == 0) {
                frac_part /= 10;
                scale--;
            }
        }

        if (scale == 0) {
            sprintf(buf, "%lld", int_part);
        } else {
            sprintf(buf, "%lld.%0*lld", int_part, scale, frac_part);
        }
    }

    return mtpscript_string_from_cstr(buf);
}

mtpscript_string_t *mtpscript_decimal_to_cbor(mtpscript_decimal_t d) {
    // For CBOR, we serialize as a string in shortest canonical form
    mtpscript_string_t *json_str = mtpscript_decimal_to_json(d);

    // Create CBOR text string
    mtpscript_string_t *cbor = mtpscript_string_new();
    size_t len = json_str->length;
    const char *data = mtpscript_string_cstr(json_str);

    if (len <= 23) {
        // Small text string: major type 3, length in low 5 bits
        uint8_t header = 0x60 | (uint8_t)len;
        mtpscript_string_append(cbor, (char*)&header, 1);
    } else {
        // 8-byte length text string
        uint8_t header = 0x7B; // major type 3, additional info 27
        mtpscript_string_append(cbor, (char*)&header, 1);
        for (int i = 7; i >= 0; i--) {
            uint8_t byte = (len >> (i * 8)) & 0xFF;
            mtpscript_string_append(cbor, (char*)&byte, 1);
        }
    }

    mtpscript_string_append(cbor, data, len);
    mtpscript_string_free(json_str);

    return cbor;
}
