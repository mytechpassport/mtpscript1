#!/bin/bash

# MTPScript Build Info Generator for Reproducible Builds
# This script generates a signed build-info.json file as required by §18

set -e

echo "Generating build info for reproducible builds..."

# Calculate SHA-256 hash of all source files
echo "Calculating source hash..."
SOURCE_HASH=$(find . -name "*.c" -o -name "*.h" -o -name "Makefile" -o -name "*.md" | \
              grep -v "build_info_generator" | \
              sort | \
              xargs cat | \
              shasum -a 256 | \
              cut -d' ' -f1)

echo "Source hash: $SOURCE_HASH"

# Build the build info generator if needed
if [ ! -f "build_info_generator" ]; then
    echo "Building build info generator..."
    gcc -o build_info_generator build_info_generator.c \
        -Isrc/stdlib -Isrc/compiler -I. \
        -L/usr/local/opt/openssl@1.1/lib -lcrypto \
        src/stdlib/runtime.o \
        src/compiler/mtpscript.o \
        mquickjs.o mquickjs_crypto.o mquickjs_effects.o mquickjs_errors.o \
        dtoa.o libm.o cutils.o
fi

# Generate the signed build-info.json
echo "Generating signed build-info.json..."
./build_info_generator "$SOURCE_HASH" "build-info.json"

echo "Build info generation complete."
echo "Contents of build-info.json:"
cat build-info.json
