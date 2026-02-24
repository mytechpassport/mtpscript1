/**
 * MTPScript NPM Bridging System
 * Specification §21
 *
 * Copyright (c) 2025 My Tech Passport Inc.
 * Author: Ryan Wong
 */

#ifndef MTPSCRIPT_NPM_BRIDGE_H
#define MTPSCRIPT_NPM_BRIDGE_H

#include "../compiler/mtpscript.h"

// Audit manifest entry for unsafe adapter
typedef struct {
    mtpscript_string_t *filename;
    mtpscript_string_t *content_hash;  // SHA-256 hash of file content
    mtpscript_string_t *package_name;  // npm package name
    mtpscript_string_t *version;       // package version
    uint64_t file_size;
    mtpscript_string_t *permissions;   // allowed host permissions
} mtpscript_audit_entry_t;

// Audit manifest
typedef struct {
    mtpscript_string_t *manifest_version;
    mtpscript_vector_t *entries;  // vector of mtpscript_audit_entry_t
    mtpscript_string_t *signature; // signature of the manifest
} mtpscript_audit_manifest_t;

// NPM bridging functions
mtpscript_audit_manifest_t *mtpscript_audit_manifest_new(void);
void mtpscript_audit_manifest_free(mtpscript_audit_manifest_t *manifest);

mtpscript_error_t *mtpscript_scan_unsafe_adapters(const char *unsafe_dir,
                                               mtpscript_audit_manifest_t *manifest);

mtpscript_error_t *mtpscript_generate_audit_manifest(mtpscript_audit_manifest_t *manifest,
                                                  const char *output_file);

mtpscript_error_t *mtpscript_verify_audit_manifest(const char *manifest_file,
                                                const char *public_key);

mtpscript_string_t *mtpscript_audit_manifest_to_json(const mtpscript_audit_manifest_t *manifest);

#endif // MTPSCRIPT_NPM_BRIDGE_H
