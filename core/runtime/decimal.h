#ifndef DECIMAL_H
#define DECIMAL_H

#include <stdint.h>

// Minimal decimal type definition for core runtime
typedef struct {
    char *value;    // significand as string
    int32_t scale;  // decimal places (0-28)
} Decimal;

// Stub decimal types and functions for missing implementation
typedef struct {
    long long value;
    int32_t scale;
} mtpscript_decimal_t;

typedef struct {
    char *data;
    size_t len;
} mtpscript_string_t;

// Stub implementations
static inline mtpscript_decimal_t mtpscript_decimal_from_string(const char *str) {
    mtpscript_decimal_t d = {0, 0};
    return d;
}

static inline mtpscript_string_t *mtpscript_decimal_to_string(mtpscript_decimal_t d) {
    static mtpscript_string_t stub = {"0", 1};
    return &stub; // stub
}

static inline int mtpscript_decimal_cmp(mtpscript_decimal_t d1, mtpscript_decimal_t d2) {
    return 0; // stub - assume equal
}

static inline const char *mtpscript_string_cstr(mtpscript_string_t *str) {
    return str ? str->data : "0";
}

static inline void mtpscript_string_free(mtpscript_string_t *str) {
    // stub - do nothing
}

#endif
