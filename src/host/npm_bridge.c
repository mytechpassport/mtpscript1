/**
 * MTPScript NPM Bridging Implementation
 * Specification §21
 *
 * Copyright (c) 2025 My Tech Passport Inc.
 * Author: Ryan Wong
 */

#include "npm_bridge.h"
#include <dirent.h>
#include <sys/stat.h>
#include <unistd.h>
#include <stdio.h>
#include <string.h>
#include "../stdlib/runtime.h"

// Audit manifest implementation
mtpscript_audit_manifest_t *mtpscript_audit_manifest_new(void) {
    mtpscript_audit_manifest_t *manifest = MTPSCRIPT_MALLOC(sizeof(mtpscript_audit_manifest_t));
    manifest->manifest_version = mtpscript_string_from_cstr("1.0");
    manifest->entries = mtpscript_vector_new();
    manifest->signature = NULL;
    return manifest;
}

void mtpscript_audit_manifest_free(mtpscript_audit_manifest_t *manifest) {
    if (manifest) {
        mtpscript_string_free(manifest->manifest_version);
        for (size_t i = 0; i < manifest->entries->size; i++) {
            mtpscript_audit_entry_t *entry = mtpscript_vector_get(manifest->entries, i);
            mtpscript_string_free(entry->filename);
            mtpscript_string_free(entry->content_hash);
            mtpscript_string_free(entry->package_name);
            mtpscript_string_free(entry->version);
            mtpscript_string_free(entry->permissions);
            MTPSCRIPT_FREE(entry);
        }
        mtpscript_vector_free(manifest->entries);
        if (manifest->signature) mtpscript_string_free(manifest->signature);
        MTPSCRIPT_FREE(manifest);
    }
}

mtpscript_error_t *mtpscript_scan_unsafe_adapters(const char *unsafe_dir,
                                               mtpscript_audit_manifest_t *manifest) {
    DIR *dir = opendir(unsafe_dir);
    if (!dir) {
        mtpscript_error_t *error = MTPSCRIPT_MALLOC(sizeof(mtpscript_error_t));
        error->message = mtpscript_string_from_cstr("Failed to open unsafe adapters directory");
        error->location = (mtpscript_location_t){0, 0, "npm_bridge"};
        return error;
    }

    struct dirent *entry;
    while ((entry = readdir(dir)) != NULL) {
        // Only process .js files
        if (strstr(entry->d_name, ".js") == NULL) continue;

        // Construct full path
        char filepath[1024];
        snprintf(filepath, sizeof(filepath), "%s/%s", unsafe_dir, entry->d_name);

        // Read file content
        FILE *file = fopen(filepath, "rb");
        if (!file) continue;

        fseek(file, 0, SEEK_END);
        long file_size = ftell(file);
        fseek(file, 0, SEEK_SET);

        char *content = MTPSCRIPT_MALLOC(file_size + 1);
        size_t bytes_read = fread(content, 1, file_size, file);
        content[bytes_read] = '\0';
        fclose(file);

        // Compute content hash
        uint8_t hash[32];
        mtpscript_sha256(content, bytes_read, hash);

        char hash_str[65];
        for (int i = 0; i < 32; i++) {
            sprintf(hash_str + (i * 2), "%02x", hash[i]);
        }
        hash_str[64] = '\0';

        // Create audit entry
        mtpscript_audit_entry_t *audit_entry = MTPSCRIPT_MALLOC(sizeof(mtpscript_audit_entry_t));
        audit_entry->filename = mtpscript_string_from_cstr(entry->d_name);
        audit_entry->content_hash = mtpscript_string_from_cstr(hash_str);
        audit_entry->file_size = (uint64_t)file_size;

        // Extract package info from content (simplified parsing)
        audit_entry->package_name = mtpscript_string_from_cstr("unknown");
        audit_entry->version = mtpscript_string_from_cstr("1.0.0");
        audit_entry->permissions = mtpscript_string_from_cstr("network,filesystem");

        mtpscript_vector_push(manifest->entries, audit_entry);

        MTPSCRIPT_FREE(content);
    }

    closedir(dir);
    return NULL;
}

mtpscript_error_t *mtpscript_generate_audit_manifest(mtpscript_audit_manifest_t *manifest,
                                                  const char *output_file) {
    FILE *file = fopen(output_file, "w");
    if (!file) {
        mtpscript_error_t *error = MTPSCRIPT_MALLOC(sizeof(mtpscript_error_t));
        error->message = mtpscript_string_from_cstr("Failed to create audit manifest file");
        error->location = (mtpscript_location_t){0, 0, "npm_bridge"};
        return error;
    }

    mtpscript_string_t *json = mtpscript_audit_manifest_to_json(manifest);
    fprintf(file, "%s\n", mtpscript_string_cstr(json));
    mtpscript_string_free(json);

    fclose(file);
    return NULL;
}

mtpscript_error_t *mtpscript_verify_audit_manifest(const char *manifest_file,
                                                const char *public_key) {
    // In a real implementation, this would:
    // 1. Load the manifest file
    // 2. Verify the signature using the provided public key
    // 3. Check that all referenced files still have matching hashes

    mtpscript_error_t *error = MTPSCRIPT_MALLOC(sizeof(mtpscript_error_t));
    error->message = mtpscript_string_from_cstr("Audit manifest verification not fully implemented - requires signature verification");
    error->location = (mtpscript_location_t){0, 0, "npm_bridge"};
    return error;
}

mtpscript_string_t *mtpscript_audit_manifest_to_json(const mtpscript_audit_manifest_t *manifest) {
    mtpscript_string_t *json = mtpscript_string_new();

    mtpscript_string_append_cstr(json, "{\n");
    mtpscript_string_append_cstr(json, "  \"manifestVersion\": \"");
    mtpscript_string_append_cstr(json, mtpscript_string_cstr(manifest->manifest_version));
    mtpscript_string_append_cstr(json, "\",\n");
    mtpscript_string_append_cstr(json, "  \"entries\": [\n");

    for (size_t i = 0; i < manifest->entries->size; i++) {
        if (i > 0) mtpscript_string_append_cstr(json, ",\n");
        mtpscript_audit_entry_t *entry = mtpscript_vector_get(manifest->entries, i);

        mtpscript_string_append_cstr(json, "    {\n");
        mtpscript_string_append_cstr(json, "      \"filename\": \"");
        mtpscript_string_append_cstr(json, mtpscript_string_cstr(entry->filename));
        mtpscript_string_append_cstr(json, "\",\n");
        mtpscript_string_append_cstr(json, "      \"contentHash\": \"");
        mtpscript_string_append_cstr(json, mtpscript_string_cstr(entry->content_hash));
        mtpscript_string_append_cstr(json, "\",\n");
        mtpscript_string_append_cstr(json, "      \"packageName\": \"");
        mtpscript_string_append_cstr(json, mtpscript_string_cstr(entry->package_name));
        mtpscript_string_append_cstr(json, "\",\n");
        mtpscript_string_append_cstr(json, "      \"version\": \"");
        mtpscript_string_append_cstr(json, mtpscript_string_cstr(entry->version));
        mtpscript_string_append_cstr(json, "\",\n");
        mtpscript_string_append_cstr(json, "      \"fileSize\": ");
        char size_str[32];
        sprintf(size_str, "%llu", entry->file_size);
        mtpscript_string_append_cstr(json, size_str);
        mtpscript_string_append_cstr(json, ",\n");
        mtpscript_string_append_cstr(json, "      \"permissions\": \"");
        mtpscript_string_append_cstr(json, mtpscript_string_cstr(entry->permissions));
        mtpscript_string_append_cstr(json, "\"\n");
        mtpscript_string_append_cstr(json, "    }");
    }

    mtpscript_string_append_cstr(json, "\n  ]");

    if (manifest->signature) {
        mtpscript_string_append_cstr(json, ",\n  \"signature\": \"");
        mtpscript_string_append_cstr(json, mtpscript_string_cstr(manifest->signature));
        mtpscript_string_append_cstr(json, "\"");
    }

    mtpscript_string_append_cstr(json, "\n}\n");

    return json;
}
