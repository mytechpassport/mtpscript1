#!/bin/bash
set -e

# Include paths
INCLUDES="-I. -Icore/runtime -Icore/stdlib -Icore/crypto -Icore/db -Icore/http -Icore/effects -Icore/utils -Isrc/compiler -Isrc/decimal -Isrc/snapshot -Isrc/stdlib -Isrc/effects -Isrc/host -Isrc/cli -Isrc/lsp -Ibuild/generated -Ibuild/templates"
CFLAGS="-Wall -g -D_GNU_SOURCE -DMTPSCRIPT_DETERMINISTIC -fno-math-errno -fno-trapping-math -I/usr/local/opt/openssl@1.1/include $(mysql_config --include) $INCLUDES"
LIBS="-lm -L/usr/local/opt/openssl@1.1/lib -lcrypto $(mysql_config --libs) -lcurl"

echo "🔨 Building MTPScript compiler objects..."

# Build all required objects except CLI main
SOURCES="src/compiler/mtpscript.c src/compiler/ast.c src/compiler/lexer.c src/compiler/parser.c src/compiler/typechecker.c src/compiler/codegen.c src/compiler/openapi.c src/compiler/module.c src/compiler/typescript_parser.c src/compiler/migration.c src/decimal/decimal.c src/snapshot/snapshot.c src/stdlib/runtime.c src/effects/effects.c src/host/lambda.c src/host/npm_bridge.c src/lsp/lsp.c"

# Core VM objects from root Makefile
CORE_OBJS="build/objects/mquickjs.o build/objects/mquickjs_crypto.o build/objects/mquickjs_effects.o build/objects/mquickjs_db.o build/objects/mquickjs_http.o build/objects/mquickjs_log.o build/objects/mquickjs_api.o build/objects/mquickjs_errors.o build/objects/dtoa.o build/objects/libm.o build/objects/cutils.o"

# Build compiler library objects
LIB_OBJS=""
for src in $SOURCES; do
    obj="build/objects/$(basename ${src%.c}.o)"
    echo "  Compiling $src..."
    gcc $CFLAGS -c -o $obj $src
    LIB_OBJS="$LIB_OBJS $obj"
done

echo "🔨 Compiling mtpsc CLI main..."
gcc $CFLAGS -c -o build/objects/mtpsc.o src/cli/mtpsc.c

echo "🔗 Linking mtpsc..."
gcc -g -o mtpsc $LIB_OBJS build/objects/mtpsc.o $CORE_OBJS $LIBS

echo "✅ mtpsc built successfully."

echo "🔨 Building Phase 2 Acceptance Test..."
gcc $CFLAGS -c -o build/objects/acceptance_test_phase_2.o tests/unit/acceptance_test_phase_2.c
gcc -g -o tests/executables/phase2_acceptance_test $LIB_OBJS $CORE_OBJS build/objects/acceptance_test_phase_2.o $LIBS

echo "✅ tests/executables/phase2_acceptance_test built successfully."
