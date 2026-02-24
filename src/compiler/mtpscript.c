/**
 * MTPScript Core Utilities Implementation
 * Specification
 *
 * Copyright (c) 2025 My Tech Passport Inc.
 * Author: Ryan Wong
 */

#include "mtpscript.h"
#include <string.h>
#include <stdio.h>

void mtpscript_error_free(mtpscript_error_t *error) {
    if (error) {
        if (error->message) mtpscript_string_free(error->message);
        MTPSCRIPT_FREE(error);
    }
}

mtpscript_string_t *mtpscript_string_new(void) {
    mtpscript_string_t *str = MTPSCRIPT_MALLOC(sizeof(mtpscript_string_t));
    str->length = 0;
    str->capacity = 16;
    str->data = MTPSCRIPT_MALLOC(str->capacity);
    str->data[0] = '\0';
    return str;
}

mtpscript_string_t *mtpscript_string_from_cstr(const char *cstr) {
    mtpscript_string_t *str = mtpscript_string_new();
    if (cstr) {
        mtpscript_string_append_cstr(str, cstr);
    }
    return str;
}

void mtpscript_string_free(mtpscript_string_t *str) {
    if (str) {
        if (str->data) MTPSCRIPT_FREE(str->data);
        MTPSCRIPT_FREE(str);
    }
}

void mtpscript_string_append(mtpscript_string_t *str, const char *data, size_t len) {
    if (str->length + len + 1 > str->capacity) {
        while (str->length + len + 1 > str->capacity) {
            str->capacity *= 2;
        }
        str->data = MTPSCRIPT_REALLOC(str->data, str->capacity);
    }
    memcpy(str->data + str->length, data, len);
    str->length += len;
    str->data[str->length] = '\0';
}

void mtpscript_string_append_cstr(mtpscript_string_t *str, const char *data) {
    mtpscript_string_append(str, data, strlen(data));
}

const char *mtpscript_string_cstr(mtpscript_string_t *str) {
    return str->data;
}

mtpscript_vector_t *mtpscript_vector_new(void) {
    mtpscript_vector_t *vec = MTPSCRIPT_MALLOC(sizeof(mtpscript_vector_t));
    vec->size = 0;
    vec->capacity = 8;
    vec->items = MTPSCRIPT_MALLOC(sizeof(void *) * vec->capacity);
    return vec;
}

void mtpscript_vector_free(mtpscript_vector_t *vec) {
    if (vec) {
        if (vec->items) MTPSCRIPT_FREE(vec->items);
        MTPSCRIPT_FREE(vec);
    }
}

void mtpscript_vector_push(mtpscript_vector_t *vec, void *item) {
    if (vec->size + 1 > vec->capacity) {
        vec->capacity *= 2;
        vec->items = MTPSCRIPT_REALLOC(vec->items, sizeof(void *) * vec->capacity);
    }
    vec->items[vec->size++] = item;
}

void *mtpscript_vector_get(mtpscript_vector_t *vec, size_t index) {
    if (index < vec->size) {
        return vec->items[index];
    }
    return NULL;
}

mtpscript_hash_t *mtpscript_hash_new(void) {
    mtpscript_hash_t *hash = MTPSCRIPT_MALLOC(sizeof(mtpscript_hash_t));
    hash->size = 0;
    hash->capacity = 16;
    hash->entries = MTPSCRIPT_MALLOC(sizeof(mtpscript_hash_entry_t) * hash->capacity);
    return hash;
}

void mtpscript_hash_free(mtpscript_hash_t *hash) {
    if (hash) {
        if (hash->entries) MTPSCRIPT_FREE(hash->entries);
        MTPSCRIPT_FREE(hash);
    }
}

void mtpscript_hash_set(mtpscript_hash_t *hash, const char *key, void *value) {
    // Simplified hash table implementation (linear search for now)
    for (size_t i = 0; i < hash->size; i++) {
        if (strcmp(hash->entries[i].key, key) == 0) {
            hash->entries[i].value = value;
            return;
        }
    }
    if (hash->size + 1 > hash->capacity) {
        hash->capacity *= 2;
        hash->entries = MTPSCRIPT_REALLOC(hash->entries, sizeof(mtpscript_hash_entry_t) * hash->capacity);
    }
    hash->entries[hash->size].key = strdup(key);
    hash->entries[hash->size].value = value;
    hash->size++;
}

void *mtpscript_hash_get(mtpscript_hash_t *hash, const char *key) {
    for (size_t i = 0; i < hash->size; i++) {
        if (strcmp(hash->entries[i].key, key) == 0) {
            return hash->entries[i].value;
        }
    }
    return NULL;
}

// Hash iteration implementation
mtpscript_hash_iterator_t *mtpscript_hash_iterator_new(mtpscript_hash_t *hash) {
    mtpscript_hash_iterator_t *iter = MTPSCRIPT_MALLOC(sizeof(mtpscript_hash_iterator_t));
    iter->hash = hash;
    iter->index = 0;
    return iter;
}

void mtpscript_hash_iterator_free(mtpscript_hash_iterator_t *iter) {
    MTPSCRIPT_FREE(iter);
}

bool mtpscript_hash_iterator_next(mtpscript_hash_iterator_t *iter) {
    while (iter->index < iter->hash->size) {
        if (iter->hash->entries[iter->index].key != NULL) {
            return true;
        }
        iter->index++;
    }
    return false;
}

const char *mtpscript_hash_iterator_key(mtpscript_hash_iterator_t *iter) {
    if (iter->index < iter->hash->size) {
        return iter->hash->entries[iter->index].key;
    }
    return NULL;
}

void *mtpscript_hash_iterator_value(mtpscript_hash_iterator_t *iter) {
    if (iter->index < iter->hash->size) {
        void *value = iter->hash->entries[iter->index].value;
        iter->index++; // Advance to next entry
        return value;
    }
    return NULL;
}

// Source mapping utilities
mtpscript_string_t *mtpscript_location_to_string(mtpscript_location_t location) {
    mtpscript_string_t *str = mtpscript_string_new();
    mtpscript_string_append_cstr(str, location.file ? location.file : "<unknown>");
    mtpscript_string_append_cstr(str, ":");
    char buf[32];
    sprintf(buf, "%d", location.line);
    mtpscript_string_append_cstr(str, buf);
    mtpscript_string_append_cstr(str, ":");
    sprintf(buf, "%d", location.column);
    mtpscript_string_append_cstr(str, buf);
    return str;
}

mtpscript_string_t *mtpscript_format_error_with_location(mtpscript_error_t *error) {
    mtpscript_string_t *str = mtpscript_string_new();
    mtpscript_string_t *loc_str = mtpscript_location_to_string(error->location);
    mtpscript_string_append_cstr(str, "Error at ");
    mtpscript_string_append(str, mtpscript_string_cstr(loc_str), strlen(mtpscript_string_cstr(loc_str)));
    mtpscript_string_append_cstr(str, ": ");
    if (error->message) {
        mtpscript_string_append(str, mtpscript_string_cstr(error->message), strlen(mtpscript_string_cstr(error->message)));
    }
    mtpscript_string_free(loc_str);
    return str;
}
