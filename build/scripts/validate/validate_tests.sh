#!/bin/bash
# MTPScript Test Validation Script
# Usage: ./validate_tests.sh [--baseline|--compare baseline.json|--quick]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

cd "$PROJECT_ROOT"

MODE=$1
BASELINE_FILE=$2

if [ "$MODE" = "--baseline" ]; then
    echo "Creating test baseline..."

    # Run full test suite
    echo "Running comprehensive test suite..."
    START_TIME=$(date +%s)

    # Capture test output
    {
        echo "=== Test Results $(date -Iseconds) ==="
        echo ""

        # Run make test
        echo "Running 'make test'..."
        make test 2>&1
        MAKE_TEST_EXIT=$?

        echo ""
        echo "=== Integration Tests ==="

        # Run integration tests if they exist
        if [ -f "tests/integration/test_closure.js" ]; then
            echo "Running integration tests..."
            ./mtpjs tests/integration/test_closure.js || echo "test_closure.js failed"
            ./mtpjs tests/integration/test_language.js || echo "test_language.js failed"
            ./mtpjs tests/integration/test_loop.js || echo "test_loop.js failed"
            ./mtpjs tests/integration/test_builtin.js || echo "test_builtin.js failed"
        fi

        # Test bytecode generation
        if [ -f "test_builtin.bin" ]; then
            echo "Testing bytecode generation..."
            ./mtpjs -b test_builtin.bin || echo "bytecode test failed"
        fi

        # Test example program
        if [ -f "example" ]; then
            echo "Testing example program..."
            ./example tests/integration/test_rect.js || echo "example test failed"
        fi

        # Test compiler acceptance tests
        if [ -f "mtpsc_acceptance" ]; then
            echo "Running compiler acceptance tests..."
            ./mtpsc_acceptance || echo "acceptance test failed"
        fi

    } > test_output.log 2>&1

    END_TIME=$(date +%s)
    TEST_TIME=$((END_TIME - START_TIME))

    # Analyze results
    TOTAL_TESTS=$(grep -c "test_" test_output.log || echo "0")
    PASSED_TESTS=$(grep -c "PASS\|OK\|SUCCESS" test_output.log || echo "0")
    FAILED_TESTS=$(grep -c "FAIL\|ERROR\|fail" test_output.log || echo "0")

    # Create baseline JSON
    cat > test_baseline.json << EOF
{
  "timestamp": "$(date -Iseconds)",
  "test_duration_seconds": $TEST_TIME,
  "results": {
    "total_tests": $TOTAL_TESTS,
    "passed": $PASSED_TESTS,
    "failed": $FAILED_TESTS,
    "make_test_exit_code": $MAKE_TEST_EXIT
  },
  "coverage": {
    "lines_covered": $(gcov -b src/compiler/*.c 2>/dev/null | grep -A 4 "File '.*'" | grep "Lines executed" | sed 's/.*:\([0-9.]*\)%.*/\1/' | head -1 || echo "0"),
    "functions_covered": $(gcov -b src/compiler/*.c 2>/dev/null | grep -A 4 "File '.*'" | grep "Functions executed" | sed 's/.*:\([0-9.]*\)%.*/\1/' | head -1 || echo "0")
  },
  "performance": {
    "binary_size_kb": $(ls -la mtpjs mtpsc 2>/dev/null | awk '{sum += $5} END {print int(sum/1024)}' || echo "0")
  }
}
EOF

    echo "Test baseline created: test_baseline.json"
    echo "Tests run: $TOTAL_TESTS, Passed: $PASSED_TESTS, Failed: $FAILED_TESTS"
    echo "Test duration: ${TEST_TIME}s"

elif [ "$MODE" = "--compare" ] && [ -n "$BASELINE_FILE" ]; then
    echo "Comparing current tests with baseline..."

    if [ ! -f "$BASELINE_FILE" ]; then
        echo "Error: Baseline file $BASELINE_FILE not found"
        exit 1
    fi

    # Run tests
    echo "Running test suite..."
    START_TIME=$(date +%s)

    # Quick test run (subset for during-refactor validation)
    make test > test_current.log 2>&1
    TEST_EXIT=$?

    END_TIME=$(date +%s)
    TEST_TIME=$((END_TIME - START_TIME))

    # Compare key metrics
    BASELINE_PASSED=$(jq -r '.results.passed' "$BASELINE_FILE")
    BASELINE_FAILED=$(jq -r '.results.failed' "$BASELINE_FILE")

    # Count current results from log
    CURRENT_PASSED=$(grep -c "PASS\|OK\|SUCCESS" test_current.log || echo "0")
    CURRENT_FAILED=$(grep -c "FAIL\|ERROR\|fail" test_current.log || echo "0")

    echo "=== Test Comparison ==="
    echo "Baseline - Passed: $BASELINE_PASSED, Failed: $BASELINE_FAILED"
    echo "Current  - Passed: $CURRENT_PASSED, Failed: $CURRENT_FAILED"

    if [ "$TEST_EXIT" -eq 0 ]; then
        echo "✅ Test suite completed successfully"
    else
        echo "❌ Test suite failed with exit code $TEST_EXIT"
        echo "=== Test Failures ==="
        grep -A 5 -B 5 "FAIL\|ERROR\|fail" test_current.log || echo "No specific failures found"
        exit 1
    fi

    # Performance check
    BASELINE_SIZE=$(jq -r '.performance.binary_size_kb' "$BASELINE_FILE")
    CURRENT_SIZE=$(ls -la mtpjs mtpsc 2>/dev/null | awk '{sum += $5} END {print int(sum/1024)}' || echo "0")

    SIZE_DIFF=$((CURRENT_SIZE - BASELINE_SIZE))
    SIZE_CHANGE_PERCENT=$((SIZE_DIFF * 100 / BASELINE_SIZE)) 2>/dev/null || SIZE_CHANGE_PERCENT=0

    echo ""
    echo "=== Performance Check ==="
    echo "Binary size: ${CURRENT_SIZE}KB (baseline: ${BASELINE_SIZE}KB, change: ${SIZE_CHANGE_PERCENT}%)"

    if [ "${SIZE_CHANGE_PERCENT#-}" -gt 20 ]; then
        echo "⚠️  Significant binary size change detected"
    else
        echo "✅ Binary size within acceptable range"
    fi

elif [ "$MODE" = "--quick" ]; then
    echo "Running quick validation tests..."

    # Just check that basic compilation works
    echo "Testing basic compilation..."
    ./mtpsc --help > /dev/null 2>&1 && echo "✅ Compiler help works" || echo "❌ Compiler help failed"

    echo "Testing basic runtime..."
    echo "console.log('test')" | ./mtpjs - > /dev/null 2>&1 && echo "✅ Runtime basic execution works" || echo "❌ Runtime basic execution failed"

    # Check for critical functionality
    if [ -f "tests/fixtures/simple_func.mtp" ]; then
        echo "Testing simple function compilation..."
        ./mtpsc compile tests/fixtures/simple_func.mtp > /dev/null 2>&1 && echo "✅ Simple compilation works" || echo "❌ Simple compilation failed"
    fi

else
    echo "Usage: $0 --baseline | --compare baseline.json | --quick"
    echo ""
    echo "Examples:"
    echo "  $0 --baseline > test_baseline.json    # Create baseline"
    echo "  $0 --compare test_baseline.json       # Compare with baseline"
    echo "  $0 --quick                             # Quick validation during refactor"
    exit 1
fi
