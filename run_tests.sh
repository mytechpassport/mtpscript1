#!/bin/bash

# Test runner for MTPScript fixture tests

DATASET_DIR="tests/fixture/dataset"
RESULT_DIR="tests/fixture/result"
MTP_BINARY="./target/release/mtp"

PASSED=0
FAILED=0
TOTAL=0

echo "Running MTPScript fixture tests..."
echo "=================================="

for mtp_file in "$DATASET_DIR"/*.mtp; do
    if [[ -f "$mtp_file" ]]; then
        basename=$(basename "$mtp_file" .mtp)
        expected_file="$RESULT_DIR/$basename.json"

        if [[ ! -f "$expected_file" ]]; then
            echo "SKIP: $basename (no expected result file)"
            continue
        fi

        TOTAL=$((TOTAL + 1))

        # Run the test
        output=$("$MTP_BINARY" execute "$mtp_file" 2>&1 | grep "Execution result:" | sed 's/Execution result: //')

        # Read expected
        expected=$(cat "$expected_file")

        if [[ "$output" == "$expected" ]]; then
            echo "PASS: $basename"
            PASSED=$((PASSED + 1))
        else
            echo "FAIL: $basename"
            echo "  Expected: $expected"
            echo "  Got:      $output"
            FAILED=$((FAILED + 1))
        fi
    fi
done

echo "=================================="
echo "Results: $PASSED passed, $FAILED failed, $TOTAL total"
if [[ $FAILED -eq 0 ]]; then
    echo "All tests passed!"
else
    echo "$FAILED tests failed."
fi