#!/bin/bash
# MTPScript Build Validation Script
# Usage: ./validate_build.sh [--baseline|--compare baseline.json]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

cd "$PROJECT_ROOT"

MODE=$1
BASELINE_FILE=$2

if [ "$MODE" = "--baseline" ]; then
    echo "Creating build baseline..."

    # Clean build
    make clean

    # Record start time
    START_TIME=$(date +%s)

    # Full build
    make all

    # Record end time
    END_TIME=$(date +%s)
    BUILD_TIME=$((END_TIME - START_TIME))

    # Collect build artifacts info
    cat > build_baseline.json << EOF
{
  "timestamp": "$(date -Iseconds)",
  "build_time_seconds": $BUILD_TIME,
  "executables": {
EOF

    # Check executable sizes and existence
    first=true
    for exe in mtpjs mtpsc mtpsc_test; do
        if [ -f "$exe" ]; then
            size=$(stat -f%z "$exe" 2>/dev/null || stat -c%s "$exe" 2>/dev/null || echo "0")
            hash=$(sha256sum "$exe" 2>/dev/null | cut -d' ' -f1 || echo "unknown")

            if [ "$first" = true ]; then
                first=false
            else
                echo "," >> build_baseline.json
            fi

            cat >> build_baseline.json << EOF
    "$exe": {
      "exists": true,
      "size_bytes": $size,
      "sha256": "$hash"
    }
EOF
        else
            if [ "$first" = true ]; then
                first=false
            else
                echo "," >> build_baseline.json
            fi

            cat >> build_baseline.json << EOF
    "$exe": {
      "exists": false,
      "size_bytes": 0,
      "sha256": "none"
    }
EOF
        fi
    done

    # Check for build artifacts
    OBJ_COUNT=$(find . -name "*.o" | wc -l)
    DEP_COUNT=$(find . -name "*.d" | wc -l)

    cat >> build_baseline.json << EOF
  },
  "build_artifacts": {
    "object_files": $OBJ_COUNT,
    "dependency_files": $DEP_COUNT
  },
  "warnings": $(make 2>&1 | grep -c "warning:" || echo "0"),
  "errors": $(make 2>&1 | grep -c "error:" || echo "0")
}
EOF

    echo "Build baseline created: build_baseline.json"
    echo "Build time: ${BUILD_TIME}s"
    echo "Executables: $(ls -la mtpjs mtpsc 2>/dev/null | wc -l) found"

elif [ "$MODE" = "--compare" ] && [ -n "$BASELINE_FILE" ]; then
    echo "Comparing current build with baseline..."

    if [ ! -f "$BASELINE_FILE" ]; then
        echo "Error: Baseline file $BASELINE_FILE not found"
        exit 1
    fi

    # Clean and rebuild
    make clean && make all

    # Extract baseline data
    if command -v jq >/dev/null 2>&1; then
        BASELINE_TIME=$(jq -r '.build_time_seconds' "$BASELINE_FILE")
        CURRENT_TIME=$(date +%s)

        # Compare executables
        echo "=== Executable Comparison ==="
        jq -r '.executables | keys[]' "$BASELINE_FILE" | while read exe; do
            baseline_exists=$(jq -r ".executables.\"$exe\".exists" "$BASELINE_FILE")
            baseline_size=$(jq -r ".executables.\"$exe\".size_bytes" "$BASELINE_FILE")
            baseline_hash=$(jq -r ".executables.\"$exe\".sha256" "$BASELINE_FILE")

        if [ -f "$exe" ]; then
            current_size=$(stat -f%z "$exe" 2>/dev/null || stat -c%s "$exe" 2>/dev/null || echo "0")
            current_hash=$(sha256sum "$exe" 2>/dev/null | cut -d' ' -f1 || echo "unknown")

            if [ "$baseline_exists" = "true" ]; then
                size_diff=$((current_size - baseline_size))
                if [ "$current_hash" != "$baseline_hash" ]; then
                    echo "⚠️  $exe: CHANGED (size: $size_diff bytes)"
                else
                    echo "✅ $exe: UNCHANGED"
                fi
            else
                echo "🆕 $exe: NEW EXECUTABLE"
            fi
        else
            if [ "$baseline_exists" = "true" ]; then
                echo "❌ $exe: MISSING"
            else
                echo "➖ $exe: Still missing"
            fi
        fi
    done

    # Compare build artifacts
    echo ""
    echo "=== Build Artifacts ==="
    BASELINE_OBJ=$(jq -r '.build_artifacts.object_files' "$BASELINE_FILE")
    CURRENT_OBJ=$(find . -name "*.o" | wc -l)
    echo "Object files: $CURRENT_OBJ (baseline: $BASELINE_OBJ)"

    BASELINE_DEP=$(jq -r '.build_artifacts.dependency_files' "$BASELINE_FILE")
    CURRENT_DEP=$(find . -name "*.d" | wc -l)
    echo "Dependency files: $CURRENT_DEP (baseline: $BASELINE_DEP)"

    # Check for build issues
    WARNINGS=$(make 2>&1 | grep -c "warning:" || echo "0")
    ERRORS=$(make 2>&1 | grep -c "error:" || echo "0")

    echo ""
    echo "=== Build Health ==="
    if [ "$WARNINGS" -gt 0 ]; then
        echo "⚠️  Warnings: $WARNINGS"
    else
        echo "✅ No warnings"
    fi

    if [ "$ERRORS" -gt 0 ]; then
        echo "❌ Errors: $ERRORS"
        exit 1
    else
        echo "✅ No errors"
    fi

    else
        echo "⚠️  jq not available - skipping detailed comparison"
        echo "✅ Build completed successfully (basic validation)"
    fi

else
    echo "Usage: $0 --baseline | --compare baseline.json"
    echo ""
    echo "Examples:"
    echo "  $0 --baseline > build_baseline.json"
    echo "  $0 --compare build_baseline.json"
    exit 1
fi
