/**
 * MTPScript Standard Library Implementation
 * Specification §8.0
 *
 * Copyright (c) 2025 My Tech Passport Inc.
 * Author: Ryan Wong
 */

#include "runtime.h"
#include <stdio.h>
#include <string.h>
#include <openssl/sha.h>
#include <openssl/ecdsa.h>
#include <openssl/ec.h>
#include <openssl/bn.h>
#include <openssl/obj_mac.h>

mtpscript_error_response_t *mtpscript_error_response_new(const char *error_type, const char *message) {
    mtpscript_error_response_t *error = MTPSCRIPT_MALLOC(sizeof(mtpscript_error_response_t));
    error->error_type = mtpscript_string_from_cstr(error_type);
    error->message = mtpscript_string_from_cstr(message);
    error->details = mtpscript_hash_new();
    return error;
}

void mtpscript_error_response_free(mtpscript_error_response_t *error) {
    if (error) {
        if (error->error_type) mtpscript_string_free(error->error_type);
        if (error->message) mtpscript_string_free(error->message);
        if (error->details) mtpscript_hash_free(error->details);
        MTPSCRIPT_FREE(error);
    }
}

mtpscript_string_t *mtpscript_error_response_to_json(mtpscript_error_response_t *error) {
    mtpscript_string_t *json = mtpscript_string_new();
    mtpscript_string_append_cstr(json, "{\"error\":\"");
    mtpscript_string_append(json, mtpscript_string_cstr(error->error_type), mtpscript_string_cstr(error->error_type) ? strlen(mtpscript_string_cstr(error->error_type)) : 0);
    mtpscript_string_append_cstr(json, "\",\"message\":\"");
    mtpscript_string_append(json, mtpscript_string_cstr(error->message), mtpscript_string_cstr(error->message) ? strlen(mtpscript_string_cstr(error->message)) : 0);
    mtpscript_string_append_cstr(json, "\"}");
    return json;
}

mtpscript_error_response_t *mtpscript_gas_exhausted_error(uint64_t gas_limit, uint64_t gas_used) {
    mtpscript_error_response_t *error = mtpscript_error_response_new("GasExhausted", "Computation gas limit exceeded");
    // Add gas limit and gas used to details
    char limit_str[32], used_str[32];
    sprintf(limit_str, "%llu", gas_limit);
    sprintf(used_str, "%llu", gas_used);
    mtpscript_hash_set(error->details, "gasLimit", mtpscript_string_from_cstr(limit_str));
    mtpscript_hash_set(error->details, "gasUsed", mtpscript_string_from_cstr(used_str));
    return error;
}

// Basic JSON serialization (RFC 8785 canonical)
mtpscript_string_t *mtpscript_json_serialize_int(int64_t value) {
    char buf[32];
    sprintf(buf, "%lld", value);
    return mtpscript_string_from_cstr(buf);
}

mtpscript_string_t *mtpscript_json_serialize_string(const char *value) {
    mtpscript_string_t *json = mtpscript_string_new();
    mtpscript_string_append_cstr(json, "\"");
    // Simple escaping - for full RFC 8785 compliance, more escaping would be needed
    for (const char *p = value; *p; p++) {
        if (*p == '"' || *p == '\\') {
            mtpscript_string_append_cstr(json, "\\");
        }
        mtpscript_string_append(json, p, 1);
    }
    mtpscript_string_append_cstr(json, "\"");
    return json;
}

mtpscript_string_t *mtpscript_json_serialize_bool(bool value) {
    return mtpscript_string_from_cstr(value ? "true" : "false");
}

mtpscript_string_t *mtpscript_json_serialize_null(void) {
    return mtpscript_string_from_cstr("null");
}

// Basic CBOR serialization (RFC 7049 §3.9 deterministic)
// Returns binary CBOR data as a string
mtpscript_string_t *mtpscript_cbor_serialize_int(int64_t value) {
    mtpscript_string_t *cbor = mtpscript_string_new();

    if (value >= 0) {
        if (value <= 23) {
            // Small positive integer: major type 0, value in low 5 bits
            uint8_t byte = 0x00 | (uint8_t)value;
            mtpscript_string_append(cbor, (char*)&byte, 1);
        } else if (value <= 255) {
            // 1-byte positive integer
            uint8_t header = 0x18; // major type 0, additional info 24
            mtpscript_string_append(cbor, (char*)&header, 1);
            uint8_t val = (uint8_t)value;
            mtpscript_string_append(cbor, (char*)&val, 1);
        } else if (value <= 65535) {
            // 2-byte positive integer
            uint8_t header = 0x19; // major type 0, additional info 25
            mtpscript_string_append(cbor, (char*)&header, 1);
            uint16_t val = (uint16_t)value;
            // Big-endian
            uint8_t bytes[2] = {(uint8_t)(val >> 8), (uint8_t)val};
            mtpscript_string_append(cbor, (char*)bytes, 2);
        } else {
            // 8-byte positive integer (simplified)
            uint8_t header = 0x1B; // major type 0, additional info 27
            mtpscript_string_append(cbor, (char*)&header, 1);
            // Big-endian 64-bit
            for (int i = 7; i >= 0; i--) {
                uint8_t byte = (value >> (i * 8)) & 0xFF;
                mtpscript_string_append(cbor, (char*)&byte, 1);
            }
        }
    } else {
        // Negative integers (simplified for basic implementation)
        uint64_t abs_val = (uint64_t)(-value - 1);
        uint8_t header = 0x20 | 0x1B; // major type 1, additional info 27
        mtpscript_string_append(cbor, (char*)&header, 1);
        for (int i = 7; i >= 0; i--) {
            uint8_t byte = (abs_val >> (i * 8)) & 0xFF;
            mtpscript_string_append(cbor, (char*)&byte, 1);
        }
    }

    return cbor;
}

mtpscript_string_t *mtpscript_cbor_serialize_string(const char *value) {
    mtpscript_string_t *cbor = mtpscript_string_new();
    size_t len = strlen(value);

    if (len <= 23) {
        // Small text string: major type 3, length in low 5 bits
        uint8_t header = 0x60 | (uint8_t)len;
        mtpscript_string_append(cbor, (char*)&header, 1);
    } else if (len <= 255) {
        // 1-byte length text string
        uint8_t header = 0x78; // major type 3, additional info 24
        mtpscript_string_append(cbor, (char*)&header, 1);
        uint8_t len_byte = (uint8_t)len;
        mtpscript_string_append(cbor, (char*)&len_byte, 1);
    } else {
        // 8-byte length text string (simplified)
        uint8_t header = 0x7B; // major type 3, additional info 27
        mtpscript_string_append(cbor, (char*)&header, 1);
        for (int i = 7; i >= 0; i--) {
            uint8_t byte = (len >> (i * 8)) & 0xFF;
            mtpscript_string_append(cbor, (char*)&byte, 1);
        }
    }

    // Add the string data
    mtpscript_string_append_cstr(cbor, value);
    return cbor;
}

mtpscript_string_t *mtpscript_cbor_serialize_bool(bool value) {
    mtpscript_string_t *cbor = mtpscript_string_new();
    uint8_t byte = value ? 0xF5 : 0xF4; // true/false
    mtpscript_string_append(cbor, (char*)&byte, 1);
    return cbor;
}

mtpscript_string_t *mtpscript_cbor_serialize_null(void) {
    mtpscript_string_t *cbor = mtpscript_string_new();
    uint8_t byte = 0xF6; // null
    mtpscript_string_append(cbor, (char*)&byte, 1);
    return cbor;
}

// FNV-1a 64-bit hashing implementation
#define FNV1A_64_OFFSET 0xcbf29ce484222325ULL
#define FNV1A_64_PRIME 0x100000001b3ULL

uint64_t mtpscript_fnv1a_64(const void *data, size_t length) {
    uint64_t hash = FNV1A_64_OFFSET;
    const uint8_t *bytes = (const uint8_t *)data;

    for (size_t i = 0; i < length; i++) {
        hash ^= bytes[i];
        hash *= FNV1A_64_PRIME;
    }

    return hash;
}

uint64_t mtpscript_fnv1a_64_string(const char *str) {
    return mtpscript_fnv1a_64(str, strlen(str));
}

// SHA-256 implementation
void mtpscript_sha256(const void *data, size_t length, uint8_t output[MTPSCRIPT_SHA256_DIGEST_SIZE]) {
    SHA256(data, length, output);
}

// ECDSA-P256 signature verification
bool mtpscript_ecdsa_verify(const void *data, size_t data_len,
                          const uint8_t signature[64],
                          const mtpscript_ecdsa_public_key_t *pubkey) {
    int ret = 0;
    EC_KEY *ec_key = NULL;
    EC_GROUP *group = NULL;
    BIGNUM *x = NULL;
    BIGNUM *y = NULL;
    unsigned char hash[SHA256_DIGEST_LENGTH];
    ECDSA_SIG *ecdsa_sig = NULL;
    BIGNUM *r = NULL;
    BIGNUM *s = NULL;

    /* Basic validation */
    if (!data || !signature || !pubkey || data_len == 0) {
        return false;
    }

    /* Create EC key for P-256 curve */
    group = EC_GROUP_new_by_curve_name(NID_X9_62_prime256v1);
    if (!group) goto cleanup;

    ec_key = EC_KEY_new();
    if (!ec_key) goto cleanup;

    if (!EC_KEY_set_group(ec_key, group)) goto cleanup;

    /* Set public key from provided coordinates */
    x = BN_bin2bn(pubkey->x, 32, NULL);
    y = BN_bin2bn(pubkey->y, 32, NULL);
    if (!x || !y) goto cleanup;

    if (!EC_KEY_set_public_key_affine_coordinates(ec_key, x, y)) goto cleanup;

    /* Hash the data with SHA-256 */
    SHA256(data, data_len, hash);

    /* Parse signature - ECDSA-P256 uses 64 bytes: r(32) + s(32) */
    r = BN_bin2bn(signature, 32, NULL);
    s = BN_bin2bn(signature + 32, 32, NULL);
    if (!r || !s) goto cleanup;

    ecdsa_sig = ECDSA_SIG_new();
    if (!ecdsa_sig) goto cleanup;

    if (!ECDSA_SIG_set0(ecdsa_sig, r, s)) goto cleanup;
    r = s = NULL; /* Ownership transferred */

    /* Verify the signature */
    ret = ECDSA_do_verify(hash, SHA256_DIGEST_LENGTH, ecdsa_sig, ec_key);

cleanup:
    if (ecdsa_sig) ECDSA_SIG_free(ecdsa_sig);
    if (r) BN_free(r);
    if (s) BN_free(s);
    if (x) BN_free(x);
    if (y) BN_free(y);
    if (ec_key) EC_KEY_free(ec_key);
    if (group) EC_GROUP_free(group);

    return ret == 1;
}

// First-class JSON ADT implementation with JsonNull constraint (§9)

// JSON ADT constructors (JsonNull cannot be constructed directly)
mtpscript_json_t *mtpscript_json_new_bool(bool value) {
    mtpscript_json_t *json = MTPSCRIPT_MALLOC(sizeof(mtpscript_json_t));
    json->kind = MTPSCRIPT_JSON_BOOL;
    json->data.bool_val = value;
    return json;
}

mtpscript_json_t *mtpscript_json_new_int(int64_t value) {
    mtpscript_json_t *json = MTPSCRIPT_MALLOC(sizeof(mtpscript_json_t));
    json->kind = MTPSCRIPT_JSON_INT;
    json->data.int_val = value;
    return json;
}

mtpscript_json_t *mtpscript_json_new_string(const char *value) {
    mtpscript_json_t *json = MTPSCRIPT_MALLOC(sizeof(mtpscript_json_t));
    json->kind = MTPSCRIPT_JSON_STRING;
    json->data.string_val = mtpscript_string_from_cstr(value);
    return json;
}

mtpscript_json_t *mtpscript_json_new_array(void) {
    mtpscript_json_t *json = MTPSCRIPT_MALLOC(sizeof(mtpscript_json_t));
    json->kind = MTPSCRIPT_JSON_ARRAY;
    json->data.array_val = mtpscript_vector_new();
    return json;
}

mtpscript_json_t *mtpscript_json_new_object(void) {
    mtpscript_json_t *json = MTPSCRIPT_MALLOC(sizeof(mtpscript_json_t));
    json->kind = MTPSCRIPT_JSON_OBJECT;
    json->data.object_val = mtpscript_hash_new();
    return json;
}

// JSON ADT accessors
bool mtpscript_json_is_null(const mtpscript_json_t *json) {
    return json->kind == MTPSCRIPT_JSON_NULL;
}

bool mtpscript_json_as_bool(const mtpscript_json_t *json) {
    return json->kind == MTPSCRIPT_JSON_BOOL ? json->data.bool_val : false;
}

int64_t mtpscript_json_as_int(const mtpscript_json_t *json) {
    return json->kind == MTPSCRIPT_JSON_INT ? json->data.int_val : 0;
}

const char *mtpscript_json_as_string(const mtpscript_json_t *json) {
    return json->kind == MTPSCRIPT_JSON_STRING ? mtpscript_string_cstr(json->data.string_val) : NULL;
}

mtpscript_vector_t *mtpscript_json_as_array(const mtpscript_json_t *json) {
    return json->kind == MTPSCRIPT_JSON_ARRAY ? json->data.array_val : NULL;
}

mtpscript_hash_t *mtpscript_json_as_object(const mtpscript_json_t *json) {
    return json->kind == MTPSCRIPT_JSON_OBJECT ? json->data.object_val : NULL;
}

// JSON ADT mutators
void mtpscript_json_array_push(mtpscript_json_t *array, mtpscript_json_t *value) {
    if (array->kind == MTPSCRIPT_JSON_ARRAY) {
        mtpscript_vector_push(array->data.array_val, value);
    }
}

void mtpscript_json_object_set(mtpscript_json_t *object, const char *key, mtpscript_json_t *value) {
    if (object->kind == MTPSCRIPT_JSON_OBJECT) {
        mtpscript_hash_set(object->data.object_val, key, value);
    }
}

// Simple JSON parser (only place where JsonNull can be created)
mtpscript_json_t *mtpscript_json_parse(const char *json_str, mtpscript_error_t **error) {
    // Skip whitespace
    while (*json_str && (*json_str == ' ' || *json_str == '\t' || *json_str == '\n' || *json_str == '\r')) {
        json_str++;
    }

    if (*json_str == '\0') {
        mtpscript_error_t *err = MTPSCRIPT_MALLOC(sizeof(mtpscript_error_t));
        err->message = mtpscript_string_from_cstr("Empty JSON string");
        err->location = (mtpscript_location_t){0, 0, "json_parse"};
        *error = err;
        return NULL;
    }

    switch (*json_str) {
        case 'n': // null
            if (strncmp(json_str, "null", 4) == 0) {
                mtpscript_json_t *json = MTPSCRIPT_MALLOC(sizeof(mtpscript_json_t));
                json->kind = MTPSCRIPT_JSON_NULL;
                return json;
            }
            break;
        case 't': // true
            if (strncmp(json_str, "true", 4) == 0) {
                return mtpscript_json_new_bool(true);
            }
            break;
        case 'f': // false
            if (strncmp(json_str, "false", 5) == 0) {
                return mtpscript_json_new_bool(false);
            }
            break;
        case '"': {
            // Parse string (simplified)
            const char *start = json_str + 1;
            const char *end = start;
            while (*end && *end != '"') {
                if (*end == '\\') end++; // Skip escaped chars
                if (*end) end++;
            }
            if (*end == '"') {
                size_t len = end - start;
                char *str = MTPSCRIPT_MALLOC(len + 1);
                memcpy(str, start, len);
                str[len] = '\0';
                mtpscript_json_t *json = mtpscript_json_new_string(str);
                MTPSCRIPT_FREE(str);
                return json;
            }
            break;
        }
        case '[': {
            // Parse array (simplified)
            mtpscript_json_t *array = mtpscript_json_new_array();
            json_str++; // Skip '['
            while (*json_str && *json_str != ']') {
                mtpscript_json_t *item = mtpscript_json_parse(json_str, error);
                if (!item) {
                    mtpscript_json_free(array);
                    return NULL;
                }
                mtpscript_json_array_push(array, item);
                // Skip to next item
                while (*json_str && *json_str != ',' && *json_str != ']') json_str++;
                if (*json_str == ',') json_str++;
            }
            return array;
        }
        case '{': {
            // Parse object (simplified)
            mtpscript_json_t *object = mtpscript_json_new_object();
            json_str++; // Skip '{'
            while (*json_str && *json_str != '}') {
                // Skip whitespace and expect string key
                while (*json_str && (*json_str == ' ' || *json_str == '\t' || *json_str == '\n' || *json_str == '\r')) json_str++;
                if (*json_str != '"') break;

                // Parse key (simplified)
                const char *key_start = json_str + 1;
                const char *key_end = key_start;
                while (*key_end && *key_end != '"') {
                    if (*key_end == '\\') key_end++;
                    if (*key_end) key_end++;
                }
                size_t key_len = key_end - key_start;
                char *key = MTPSCRIPT_MALLOC(key_len + 1);
                memcpy(key, key_start, key_len);
                key[key_len] = '\0';

                // Skip to colon
                json_str = key_end + 1;
                while (*json_str && *json_str != ':') json_str++;
                if (*json_str == ':') json_str++;

                // Parse value
                mtpscript_json_t *value = mtpscript_json_parse(json_str, error);
                if (!value) {
                    MTPSCRIPT_FREE(key);
                    mtpscript_json_free(object);
                    return NULL;
                }

                mtpscript_json_object_set(object, key, value);
                MTPSCRIPT_FREE(key);

                // Skip to next item
                while (*json_str && *json_str != ',' && *json_str != '}') json_str++;
                if (*json_str == ',') json_str++;
            }
            return object;
        }
        default: {
            // Parse number (simplified)
            char *endptr;
            long long val = strtoll(json_str, &endptr, 10);
            if (endptr != json_str) {
                return mtpscript_json_new_int(val);
            }
            break;
        }
    }

    *error = MTPSCRIPT_MALLOC(sizeof(mtpscript_error_t));
    (*error)->message = mtpscript_string_from_cstr("Invalid JSON");
    (*error)->location = (mtpscript_location_t){0, 0, "json_parse"};
    return NULL;
}

// JSON serialization (RFC 8785 canonical)
mtpscript_string_t *mtpscript_json_serialize(const mtpscript_json_t *json) {
    mtpscript_string_t *result = mtpscript_string_new();

    switch (json->kind) {
        case MTPSCRIPT_JSON_NULL:
            mtpscript_string_append_cstr(result, "null");
            break;
        case MTPSCRIPT_JSON_BOOL:
            mtpscript_string_append_cstr(result, json->data.bool_val ? "true" : "false");
            break;
        case MTPSCRIPT_JSON_INT:
            {
                char buf[32];
                sprintf(buf, "%lld", json->data.int_val);
                mtpscript_string_append_cstr(result, buf);
            }
            break;
        case MTPSCRIPT_JSON_STRING:
            mtpscript_string_append_cstr(result, "\"");
            // Simple escaping
            const char *str = mtpscript_string_cstr(json->data.string_val);
            for (size_t i = 0; str[i]; i++) {
                if (str[i] == '"' || str[i] == '\\') {
                    mtpscript_string_append_cstr(result, "\\");
                }
                mtpscript_string_append(result, &str[i], 1);
            }
            mtpscript_string_append_cstr(result, "\"");
            break;
        case MTPSCRIPT_JSON_ARRAY:
            mtpscript_string_append_cstr(result, "[");
            for (size_t i = 0; i < json->data.array_val->size; i++) {
                if (i > 0) mtpscript_string_append_cstr(result, ",");
                mtpscript_json_t *item = mtpscript_vector_get(json->data.array_val, i);
                mtpscript_string_t *item_str = mtpscript_json_serialize(item);
                mtpscript_string_append(result, mtpscript_string_cstr(item_str), item_str->length);
                mtpscript_string_free(item_str);
            }
            mtpscript_string_append_cstr(result, "]");
            break;
        case MTPSCRIPT_JSON_OBJECT:
            mtpscript_string_append_cstr(result, "{");
            mtpscript_hash_iterator_t *iter = mtpscript_hash_iterator_new(json->data.object_val);
            bool first = true;
            while (mtpscript_hash_iterator_next(iter)) {
                if (!first) mtpscript_string_append_cstr(result, ",");
                first = false;

                mtpscript_string_append_cstr(result, "\"");
                mtpscript_string_append_cstr(result, mtpscript_hash_iterator_key(iter));
                mtpscript_string_append_cstr(result, "\":");

                mtpscript_json_t *value = mtpscript_hash_iterator_value(iter);
                mtpscript_string_t *value_str = mtpscript_json_serialize(value);
                mtpscript_string_append(result, mtpscript_string_cstr(value_str), value_str->length);
                mtpscript_string_free(value_str);
            }
            mtpscript_hash_iterator_free(iter);
            mtpscript_string_append_cstr(result, "}");
            break;
    }

    return result;
}

// JSON cleanup
void mtpscript_json_free(mtpscript_json_t *json) {
    if (!json) return;

    switch (json->kind) {
        case MTPSCRIPT_JSON_STRING:
            if (json->data.string_val) mtpscript_string_free(json->data.string_val);
            break;
        case MTPSCRIPT_JSON_ARRAY:
            if (json->data.array_val) {
                for (size_t i = 0; i < json->data.array_val->size; i++) {
                    mtpscript_json_t *item = mtpscript_vector_get(json->data.array_val, i);
                    mtpscript_json_free(item);
                }
                mtpscript_vector_free(json->data.array_val);
            }
            break;
        case MTPSCRIPT_JSON_OBJECT:
            if (json->data.object_val) {
                mtpscript_hash_iterator_t *iter = mtpscript_hash_iterator_new(json->data.object_val);
                while (mtpscript_hash_iterator_next(iter)) {
                    mtpscript_json_t *value = mtpscript_hash_iterator_value(iter);
                    mtpscript_json_free(value);
                }
                mtpscript_hash_iterator_free(iter);
                mtpscript_hash_free(json->data.object_val);
            }
            break;
        default:
            break;
    }

    MTPSCRIPT_FREE(json);
}

// Deterministic seed generation (§0-b)
// SHA-256(Req_Id || Acc_Id || Ver || "mtpscript-v5.1" || SnapHash || GasLimit_ASCII)
void mtpscript_generate_deterministic_seed(const char *req_id, const char *acc_id,
                                         const char *version, const uint8_t *snap_hash,
                                         uint64_t gas_limit, uint8_t seed_out[MTPSCRIPT_SEED_SIZE]) {
    // Create concatenated input: Req_Id || Acc_Id || Ver || "mtpscript-v5.1" || SnapHash || GasLimit_ASCII
    mtpscript_string_t *input = mtpscript_string_new();

    // Append Req_Id
    mtpscript_string_append_cstr(input, req_id);

    // Append Acc_Id
    mtpscript_string_append_cstr(input, acc_id);

    // Append Ver
    mtpscript_string_append_cstr(input, version);

    // Append fixed string "mtpscript-v5.1"
    mtpscript_string_append_cstr(input, "mtpscript-v5.1");

    // Append SnapHash (32 bytes)
    mtpscript_string_append(input, (const char*)snap_hash, 32);

    // Append GasLimit_ASCII (no leading zeros)
    char gas_limit_str[32];
    sprintf(gas_limit_str, "%llu", gas_limit);
    mtpscript_string_append_cstr(input, gas_limit_str);

    // Compute SHA-256 hash
    mtpscript_sha256(mtpscript_string_cstr(input), input->length, seed_out);

    mtpscript_string_free(input);
}

// Host adapter contract validation (§13.2)
mtpscript_error_t *mtpscript_validate_gas_limit(uint64_t gas_limit) {
    if (gas_limit == 0 || gas_limit > MTPSCRIPT_MAX_GAS_LIMIT) {
        mtpscript_error_t *error = MTPSCRIPT_MALLOC(sizeof(mtpscript_error_t));
        error->message = mtpscript_string_from_cstr("Invalid gas limit: must be 1-2B");
        error->location = (mtpscript_location_t){0, 0, "gas_validation"};
        return error;
    }
    return NULL;
}

mtpscript_error_t *mtpscript_inject_gas_limit(const char *js_code, uint64_t gas_limit, mtpscript_string_t **output) {
    // Validate gas limit first
    mtpscript_error_t *validation_error = mtpscript_validate_gas_limit(gas_limit);
    if (validation_error) {
        return validation_error;
    }

    // Inject gas limit as a global constant before static initialization
    mtpscript_string_t *result = mtpscript_string_new();
    char gas_limit_str[32];
    sprintf(gas_limit_str, "%llu", gas_limit);

    mtpscript_string_append_cstr(result, "// Injected gas limit for host adapter contract\n");
    mtpscript_string_append_cstr(result, "const MTP_GAS_LIMIT = ");
    mtpscript_string_append_cstr(result, gas_limit_str);
    mtpscript_string_append_cstr(result, ";\n\n");
    mtpscript_string_append_cstr(result, js_code);

    *output = result;
    return NULL;
}

// Memory protection (§22) - secure memory wipe and zero cross-request state
void mtpscript_secure_memory_wipe(void *ptr, size_t size) {
    if (!ptr || size == 0) return;

    // Multiple overwrites for security (Gutmann method simplified)
    volatile unsigned char *p = (volatile unsigned char *)ptr;

    // Pass 1: 0xFF
    for (size_t i = 0; i < size; i++) {
        p[i] = 0xFF;
    }

    // Pass 2: 0x00
    for (size_t i = 0; i < size; i++) {
        p[i] = 0x00;
    }

    // Pass 3: 0xFF again
    for (size_t i = 0; i < size; i++) {
        p[i] = 0xFF;
    }

    // Final pass: random-like pattern
    for (size_t i = 0; i < size; i++) {
        p[i] = (unsigned char)(i % 256);
    }

    // Final zero for good measure
    memset(ptr, 0, size);
}

void mtpscript_zero_cross_request_state(void) {
    // This function would be called between requests to ensure
    // no sensitive data persists across request boundaries

    // In a real implementation, this would:
    // 1. Clear all global variables
    // 2. Reset all caches and internal state
    // 3. Wipe sensitive memory regions
    // 4. Reset cryptographic contexts

    // For now, this is a placeholder that would be integrated
    // with the host runtime to ensure clean state between requests

    // Example: Clear any global state that might persist
    // (This would be specific to the host environment)

    // Secure wipe of any sensitive static buffers would go here
}

// Reproducible builds (§18) - build info generation and signing
mtpscript_build_info_t *mtpscript_build_info_create(const char *source_hash, const char *compiler_version) {
    mtpscript_build_info_t *build_info = MTPSCRIPT_MALLOC(sizeof(mtpscript_build_info_t));

    // Generate unique build ID (simplified - in real impl, use UUID)
    char build_id[64];
    sprintf(build_id, "build-%lx", (unsigned long)time(NULL));
    build_info->build_id = strdup(build_id);

    // Current timestamp
    time_t now = time(NULL);
    build_info->timestamp = strdup(ctime(&now));
    // Remove trailing newline
    build_info->timestamp[strlen(build_info->timestamp) - 1] = '\0';

    build_info->source_hash = strdup(source_hash);
    build_info->compiler_version = strdup(compiler_version);
    build_info->build_environment = strdup("mtpscript-v5.1");

    memset(build_info->signature, 0, sizeof(build_info->signature));

    return build_info;
}

void mtpscript_build_info_free(mtpscript_build_info_t *build_info) {
    if (build_info) {
        free(build_info->build_id);
        free(build_info->timestamp);
        free(build_info->source_hash);
        free(build_info->compiler_version);
        free(build_info->build_environment);
        MTPSCRIPT_FREE(build_info);
    }
}

mtpscript_error_t *mtpscript_build_info_sign(mtpscript_build_info_t *build_info, const mtpscript_ecdsa_public_key_t *key) {
    // Create JSON representation for signing
    mtpscript_string_t *json = mtpscript_string_new();
    mtpscript_string_append_cstr(json, "{");
    mtpscript_string_append_cstr(json, "\"buildId\":\"");
    mtpscript_string_append_cstr(json, build_info->build_id);
    mtpscript_string_append_cstr(json, "\",\"timestamp\":\"");
    mtpscript_string_append_cstr(json, build_info->timestamp);
    mtpscript_string_append_cstr(json, "\",\"sourceHash\":\"");
    mtpscript_string_append_cstr(json, build_info->source_hash);
    mtpscript_string_append_cstr(json, "\",\"compilerVersion\":\"");
    mtpscript_string_append_cstr(json, build_info->compiler_version);
    mtpscript_string_append_cstr(json, "\",\"buildEnvironment\":\"");
    mtpscript_string_append_cstr(json, build_info->build_environment);
    mtpscript_string_append_cstr(json, "\"}");

    // In a real implementation, this would sign the JSON with a private key
    // For now, we just create a deterministic "signature" based on the content
    uint8_t hash[32];
    mtpscript_sha256(mtpscript_string_cstr(json), json->length, hash);
    memcpy(build_info->signature, hash, 32);
    memset(build_info->signature + 32, 0, 32); // Pad to 64 bytes

    mtpscript_string_free(json);
    return NULL;
}

mtpscript_string_t *mtpscript_build_info_to_json(const mtpscript_build_info_t *build_info) {
    mtpscript_string_t *json = mtpscript_string_new();

    mtpscript_string_append_cstr(json, "{\n");
    mtpscript_string_append_cstr(json, "  \"buildId\": \"");
    mtpscript_string_append_cstr(json, build_info->build_id);
    mtpscript_string_append_cstr(json, "\",\n");
    mtpscript_string_append_cstr(json, "  \"timestamp\": \"");
    mtpscript_string_append_cstr(json, build_info->timestamp);
    mtpscript_string_append_cstr(json, "\",\n");
    mtpscript_string_append_cstr(json, "  \"sourceHash\": \"");
    mtpscript_string_append_cstr(json, build_info->source_hash);
    mtpscript_string_append_cstr(json, "\",\n");
    mtpscript_string_append_cstr(json, "  \"compilerVersion\": \"");
    mtpscript_string_append_cstr(json, build_info->compiler_version);
    mtpscript_string_append_cstr(json, "\",\n");
    mtpscript_string_append_cstr(json, "  \"buildEnvironment\": \"");
    mtpscript_string_append_cstr(json, build_info->build_environment);
    mtpscript_string_append_cstr(json, "\",\n");
    mtpscript_string_append_cstr(json, "  \"signature\": \"");
    // Convert signature to hex string
    for (int i = 0; i < 64; i++) {
        char hex[3];
        sprintf(hex, "%02x", build_info->signature[i]);
        mtpscript_string_append_cstr(json, hex);
    }
    mtpscript_string_append_cstr(json, "\"\n");
    mtpscript_string_append_cstr(json, "}\n");

    return json;
}

mtpscript_error_t *mtpscript_stdlib_init(void *js_context) {
    // Evaluation of standard library JS code would go here
    (void)js_context;
    return NULL;
}
