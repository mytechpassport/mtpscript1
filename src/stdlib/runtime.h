/**
 * MTPScript Standard Library
 * Specification §8.0
 *
 * Copyright (c) 2025 My Tech Passport Inc.
 * Author: Ryan Wong
 */

#ifndef MTPSCRIPT_RUNTIME_H
#define MTPSCRIPT_RUNTIME_H

#include "../compiler/mtpscript.h"

// Option type
typedef struct {
    bool has_value;
    void *value;
} mtpscript_option_t;

// Result type
typedef struct {
    bool is_ok;
    void *value;
    void *error;
} mtpscript_result_t;

// Error system for deterministic error shapes
typedef struct {
    mtpscript_string_t *error_type;
    mtpscript_string_t *message;
    mtpscript_hash_t *details; // Additional error details
} mtpscript_error_response_t;

mtpscript_error_response_t *mtpscript_error_response_new(const char *error_type, const char *message);
void mtpscript_error_response_free(mtpscript_error_response_t *error);
mtpscript_string_t *mtpscript_error_response_to_json(mtpscript_error_response_t *error);

// Gas exhaustion error as specified in §79
mtpscript_error_response_t *mtpscript_gas_exhausted_error(uint64_t gas_limit, uint64_t gas_used);

// Basic JSON serialization (RFC 8785 canonical)
mtpscript_string_t *mtpscript_json_serialize_int(int64_t value);
mtpscript_string_t *mtpscript_json_serialize_string(const char *value);
mtpscript_string_t *mtpscript_json_serialize_bool(bool value);
mtpscript_string_t *mtpscript_json_serialize_null(void);

// Basic CBOR serialization (RFC 7049 §3.9 deterministic)
mtpscript_string_t *mtpscript_cbor_serialize_int(int64_t value);
mtpscript_string_t *mtpscript_cbor_serialize_string(const char *value);
mtpscript_string_t *mtpscript_cbor_serialize_bool(bool value);
mtpscript_string_t *mtpscript_cbor_serialize_null(void);

// First-class JSON ADT with JsonNull constraint (§9)
typedef enum {
    MTPSCRIPT_JSON_NULL,    // Only inhabited through parsing, no literals allowed
    MTPSCRIPT_JSON_BOOL,
    MTPSCRIPT_JSON_INT,
    MTPSCRIPT_JSON_STRING,
    MTPSCRIPT_JSON_ARRAY,
    MTPSCRIPT_JSON_OBJECT
} mtpscript_json_kind_t;

typedef struct mtpscript_json_t {
    mtpscript_json_kind_t kind;
    union {
        bool bool_val;
        int64_t int_val;
        mtpscript_string_t *string_val;
        mtpscript_vector_t *array_val;  // vector of mtpscript_json_t*
        mtpscript_hash_t *object_val;   // hash of string -> mtpscript_json_t*
    } data;
} mtpscript_json_t;

// JSON ADT constructors (JsonNull cannot be constructed directly)
mtpscript_json_t *mtpscript_json_new_bool(bool value);
mtpscript_json_t *mtpscript_json_new_int(int64_t value);
mtpscript_json_t *mtpscript_json_new_string(const char *value);
mtpscript_json_t *mtpscript_json_new_array(void);
mtpscript_json_t *mtpscript_json_new_object(void);

// JSON ADT accessors
bool mtpscript_json_is_null(const mtpscript_json_t *json);
bool mtpscript_json_as_bool(const mtpscript_json_t *json);
int64_t mtpscript_json_as_int(const mtpscript_json_t *json);
const char *mtpscript_json_as_string(const mtpscript_json_t *json);
mtpscript_vector_t *mtpscript_json_as_array(const mtpscript_json_t *json);
mtpscript_hash_t *mtpscript_json_as_object(const mtpscript_json_t *json);

// JSON ADT mutators
void mtpscript_json_array_push(mtpscript_json_t *array, mtpscript_json_t *value);
void mtpscript_json_object_set(mtpscript_json_t *object, const char *key, mtpscript_json_t *value);

// JSON parsing (only place where JsonNull can be created)
mtpscript_json_t *mtpscript_json_parse(const char *json_str, mtpscript_error_t **error);

// JSON serialization
mtpscript_string_t *mtpscript_json_serialize(const mtpscript_json_t *json);

// JSON cleanup
void mtpscript_json_free(mtpscript_json_t *json);

// Hashing and crypto primitives
uint64_t mtpscript_fnv1a_64(const void *data, size_t length);
uint64_t mtpscript_fnv1a_64_string(const char *str);

// SHA-256 hash (32 bytes)
#define MTPSCRIPT_SHA256_DIGEST_SIZE 32
void mtpscript_sha256(const void *data, size_t length, uint8_t output[MTPSCRIPT_SHA256_DIGEST_SIZE]);

// ECDSA-P256 signature verification
typedef struct {
    uint8_t x[32];
    uint8_t y[32];
} mtpscript_ecdsa_public_key_t;

bool mtpscript_ecdsa_verify(const void *data, size_t data_len,
                          const uint8_t signature[64],
                          const mtpscript_ecdsa_public_key_t *pubkey);

// Deterministic seed generation (§0-b)
#define MTPSCRIPT_SEED_SIZE 32
void mtpscript_generate_deterministic_seed(const char *req_id, const char *acc_id,
                                         const char *version, const uint8_t *snap_hash,
                                         uint64_t gas_limit, uint8_t seed_out[MTPSCRIPT_SEED_SIZE]);

// Host adapter contract validation (§13.2)
#define MTPSCRIPT_MAX_GAS_LIMIT 2000000000ULL // 2B gas limit
mtpscript_error_t *mtpscript_validate_gas_limit(uint64_t gas_limit);
mtpscript_error_t *mtpscript_inject_gas_limit(const char *js_code, uint64_t gas_limit, mtpscript_string_t **output);

// Memory protection (§22)
void mtpscript_secure_memory_wipe(void *ptr, size_t size);
void mtpscript_zero_cross_request_state(void);

// Reproducible builds (§18)
typedef struct {
    char *build_id;
    char *timestamp;
    char *source_hash;
    char *compiler_version;
    char *build_environment;
    uint8_t signature[64];
} mtpscript_build_info_t;

mtpscript_build_info_t *mtpscript_build_info_create(const char *source_hash, const char *compiler_version);
void mtpscript_build_info_free(mtpscript_build_info_t *build_info);
mtpscript_error_t *mtpscript_build_info_sign(mtpscript_build_info_t *build_info, const mtpscript_ecdsa_public_key_t *key);
mtpscript_string_t *mtpscript_build_info_to_json(const mtpscript_build_info_t *build_info);

// Initialize the standard library in a JS context
mtpscript_error_t *mtpscript_stdlib_init(void *js_context);

#endif // MTPSCRIPT_STDLIB_H
