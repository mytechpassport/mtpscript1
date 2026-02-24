#!/bin/bash

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# Get the project root (parent of scripts directory)
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
# Path to mtpsc binary
MTPSC="$PROJECT_ROOT/mtpsc"
# Path to fixtures
FIXTURES_DIR="$PROJECT_ROOT/tests/fixtures"
echo "MTPScript Complex Model Test Suite"
echo "=================================="

# Test that the complex fixture compiles
echo ""
echo "Testing complex fixture compilation..."
if $MTPSC compile $FIXTURES_DIR/complex_model.mtp > /dev/null 2>&1; then
    echo "✅ Complex fixture compiled successfully"
    FIXTURE_COMPILE=true
else
    echo "❌ Complex fixture compilation failed"
    FIXTURE_COMPILE=false
fi

# Test that the complex fixture passes type checking
echo ""
echo "Testing complex fixture type checking..."
if $MTPSC check $FIXTURES_DIR/complex_model.mtp > /dev/null 2>&1; then
    echo "✅ Complex fixture type check passed"
    FIXTURE_CHECK=true
else
    echo "❌ Complex fixture type check failed"
    FIXTURE_CHECK=false
fi

# Test primitive types
echo ""
echo "Testing primitive types handling..."
cat > primitive_test.mtp << 'EOF'
function testPrimitives(): {
    num: number,
    dec: Decimal,
    str: string,
    bool: boolean
} {
    return {
        num: 42,
        dec: 3.14,
        str: "hello",
        bool: true
    }
}
EOF

if $MTPSC compile primitive_test.mtp > primitive_output.js 2>&1; then
    # Check that output contains expected content
    if grep -q "testPrimitives" primitive_output.js && grep -q "num" primitive_output.js && grep -q "dec" primitive_output.js && grep -q "str" primitive_output.js && grep -q "bool" primitive_output.js; then
        echo "✅ Primitive types handled correctly with expected output"
        PRIMITIVES=true
    else
        echo "❌ Primitive types test failed - missing expected content in compiled output"
        PRIMITIVES=false
    fi
else
    echo "❌ Primitive types test failed - compilation error"
    PRIMITIVES=false
fi
rm -f primitive_test.mtp primitive_output.js

# Test nested structures
echo ""
echo "Testing nested structures..."
cat > nested_test.mtp << 'EOF'
function testNested(): {
    outer: {
        inner: {
            value: number
        }
    }
} {
    return {
        outer: {
            inner: {
                value: 123
            }
        }
    }
}
EOF

if $MTPSC compile nested_test.mtp > nested_output.js 2>&1; then
    if grep -q "testNested" nested_output.js && grep -q "outer" nested_output.js && grep -q "inner" nested_output.js && grep -q "value" nested_output.js; then
        echo "✅ Nested structures work correctly with expected output"
        NESTED=true
    else
        echo "❌ Nested structures test failed - missing expected content in compiled output"
        NESTED=false
    fi
else
    echo "❌ Nested structures test failed - compilation error"
    NESTED=false
fi
rm -f nested_test.mtp nested_output.js

# Test array types
echo ""
echo "Testing array types..."
cat > array_test.mtp << 'EOF'
function testArrays(): {
    numbers: [number],
    strings: [string]
} {
    return {
        numbers: [1, 2, 3],
        strings: ["a", "b", "c"]
    }
}
EOF

if $MTPSC compile array_test.mtp > array_output.js 2>&1; then
    if grep -q "testArrays" array_output.js && grep -q "numbers" array_output.js && grep -q "strings" array_output.js; then
        echo "✅ Array types work correctly with expected output"
        ARRAYS=true
    else
        echo "❌ Array types test failed - missing expected content in compiled output"
        ARRAYS=false
    fi
else
    echo "❌ Array types test failed - compilation error"
    ARRAYS=false
fi
rm -f array_test.mtp array_output.js

# Test API endpoints (using regular function since API endpoints don't produce executable code)
echo ""
echo "Testing API endpoints..."
cat > api_test.mtp << 'EOF'
function getData(): { id: number, success: boolean } {
    return { id: 123, success: true }
}
EOF

if $MTPSC compile api_test.mtp > api_output.js 2>&1; then
    if grep -q "getData" api_output.js && grep -q "id" api_output.js && grep -q "success" api_output.js; then
        echo "✅ API endpoints work correctly with expected output"
        API=true
    else
        echo "❌ API endpoints test failed - missing expected content in compiled output"
        API=false
    fi
else
    echo "❌ API endpoints test failed - compilation error"
    API=false
fi
rm -f api_test.mtp api_output.js

# Summary
echo ""
echo "Complex Model Test Summary:"
echo "=========================="
echo "Fixture Compilation: $( [ "$FIXTURE_COMPILE" = true ] && echo "PASS" || echo "FAIL" )"
echo "Fixture Type Check:  $( [ "$FIXTURE_CHECK" = true ] && echo "PASS" || echo "FAIL" )"
echo "Primitive Types:     $( [ "$PRIMITIVES" = true ] && echo "PASS" || echo "FAIL" )"
echo "Nested Structures:   $( [ "$NESTED" = true ] && echo "PASS" || echo "FAIL" )"
echo "Array Types:         $( [ "$ARRAYS" = true ] && echo "PASS" || echo "FAIL" )"
echo "API Endpoints:       $( [ "$API" = true ] && echo "PASS" || echo "FAIL" )"

echo ""
if [ "$FIXTURE_COMPILE" = true ] && [ "$FIXTURE_CHECK" = true ] && [ "$PRIMITIVES" = true ] && [ "$NESTED" = true ] && [ "$ARRAYS" = true ] && [ "$API" = true ]; then
    echo "🎉 All complex model tests PASSED!"
    exit 0
else
    echo "💥 Some complex model tests FAILED!"
    exit 1
fi
