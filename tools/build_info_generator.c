#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <openssl/sha.h>
#include "src/stdlib/runtime.h"

int main(int argc, char *argv[]) {
    if (argc != 3) {
        fprintf(stderr, "Usage: %s <source_hash> <output_json_file>\n", argv[0]);
        return 1;
    }

    const char *source_hash = argv[1];
    const char *output_file = argv[2];

    // Initialize runtime for build info functions
    mtpscript_error_t *err = mtpscript_stdlib_init(NULL);
    if (err) {
        fprintf(stderr, "Failed to initialize stdlib: %s\n", err->message ? err->message->data : "unknown error");
        mtpscript_error_free(err);
        return 1;
    }

    // Create build info
    mtpscript_build_info_t *build_info = mtpscript_build_info_create(source_hash, "mtpscript-v5.1");
    if (!build_info) {
        fprintf(stderr, "Failed to create build info\n");
        return 1;
    }

    // Create a deterministic ECDSA key for reproducible builds
    // In production, this would be a real signing key
    mtpscript_ecdsa_public_key_t dummy_key;
    memset(&dummy_key, 0, sizeof(dummy_key));

    // Sign the build info
    err = mtpscript_build_info_sign(build_info, &dummy_key);
    if (err) {
        fprintf(stderr, "Failed to sign build info: %s\n", err->message ? err->message->data : "unknown error");
        mtpscript_error_free(err);
        mtpscript_build_info_free(build_info);
        return 1;
    }

    // Convert to JSON and write to file
    mtpscript_string_t *json = mtpscript_build_info_to_json(build_info);
    if (!json) {
        fprintf(stderr, "Failed to convert build info to JSON\n");
        mtpscript_build_info_free(build_info);
        return 1;
    }

    // Write to output file
    FILE *fp = fopen(output_file, "w");
    if (!fp) {
        fprintf(stderr, "Failed to open output file: %s\n", output_file);
        mtpscript_string_free(json);
        mtpscript_build_info_free(build_info);
        return 1;
    }

    fprintf(fp, "%s\n", json->data);
    fclose(fp);

    printf("Build info written to %s\n", output_file);

    // Cleanup
    mtpscript_string_free(json);
    mtpscript_build_info_free(build_info);

    return 0;
}
