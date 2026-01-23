# MTPScript Test Failures - Issue Tracker

## Summary
- **Passing**: 1/20 tests (03_nested_objects.mtp)
- **Failing**: 19/20 tests

---

## Failing Tests by Category

### Category 1: Parser Errors (10 tests)

#### 1. 05_decimal_money.mtp
**Error**: `ParserError("Expected string literal")`
**Cause**: Parser doesn't support decimal literal syntax like `123.45m` or similar decimal notation.
**Fix Needed**: Add decimal literal parsing support in lexer/parser.

#### 2. 06_pattern_matching.mtp
**Error**: `ParserError("Expected '}' after block expression")`
**Cause**: Complex match arm bodies with nested blocks and local type declarations are not parsed correctly.
**Fix Needed**: Improve block expression parsing in match arms.

#### 3. 07_pipeline_operator.mtp
**Error**: `ParserError("Expected ':' after parameter name")`
**Cause**: Pipeline operator with lambda shorthand syntax (e.g., `|> map(x => x + 1)`) not supported.
**Fix Needed**: Add lambda expression parsing without explicit type annotations.

#### 4. 09_gas_metering.mtp
**Error**: `ParserError("Expected '}' after API body")`
**Cause**: Recursive local function declarations inside API handlers not parsed correctly.
**Fix Needed**: Fix local function scoping in API bodies.

#### 5. 10_option_result.mtp
**Error**: `ParserError("Expected primary expression, found RBrace")`
**Cause**: Empty block expressions `{}` or trailing syntax issues.
**Fix Needed**: Handle empty blocks and improve expression termination parsing.

#### 6. 12_http_effects.mtp
**Error**: `ParserError("Expected '}' after block expression")`
**Cause**: Complex nested expressions in API handlers.
**Fix Needed**: Same as #2 - improve block expression parsing.

#### 7. 13_closures_lambdas.mtp
**Error**: `ParserError("Expected identifier")`
**Cause**: Arrow function syntax `(x) => x + 1` or closure capture syntax not supported.
**Fix Needed**: Add arrow function expression parsing.

#### 8. 15_api_system.mtp
**Error**: `ParserError("Expected '}' after API body")`
**Cause**: Complex API handler syntax with multiple features.
**Fix Needed**: Same as #4.

#### 9. 16_number_overflow.mtp
**Error**: `ParserError("Expected '}' after block expression")`
**Cause**: Block expressions with overflow checking code.
**Fix Needed**: Same as #2.

#### 10. 17_structural_equality.mtp
**Error**: `ParserError("Expected primary expression, found Type")`
**Cause**: Local type declarations inside expressions not supported.
**Fix Needed**: Add support for local type declarations in expression contexts.

#### 11. 18_log_effects.mtp
**Error**: `ParserError("Expected '}' after API body")`
**Cause**: Same as #4.
**Fix Needed**: Same as #4.

#### 12. 19_error_handling.mtp
**Error**: `ParserError("Expected '}' after block expression")`
**Cause**: Error handling constructs with complex blocks.
**Fix Needed**: Same as #2.

#### 13. 20_comprehensive.mtp
**Error**: `ParserError("Expected '}' after API body")`
**Cause**: Comprehensive test with multiple features.
**Fix Needed**: Combination of above fixes.

---

### Category 2: Type Errors (5 tests)

#### 1. 02_db_sqlite.mtp
**Error**: `TypeError("Cannot infer type of empty array")`
**Cause**: Type checker cannot infer element type from `[]` empty array literal.
**Fix Needed**: Either require type annotation for empty arrays or infer from context.

#### 2. 04_recursion_loops.mtp
**Error**: `TypeError("Arithmetic operations require numbers")`
**Cause**: Recursive function return type inference fails, causing type mismatch.
**Fix Needed**: Improve recursive function type inference to properly unify return types.

#### 3. 08_json_operations.mtp
**Error**: `TypeError("Dot access only on records")`
**Cause**: Attempting to use dot access on JSON values which aren't typed as records.
**Fix Needed**: Add JSON type with dynamic property access, or use index syntax `obj["prop"]`.

#### 4. 11_hash_cbor.mtp
**Error**: `TypeError("Array elements must have same type")`
**Cause**: Array with mixed types (e.g., `[1, "hello", true]`) not allowed.
**Fix Needed**: Either require homogeneous arrays or add union/any type support.

#### 5. 14_list_map_ops.mtp
**Error**: `TypeError("Cannot infer type of empty array")`
**Cause**: Same as #1 in this category.
**Fix Needed**: Same as #1.

---

### Category 3: Runtime Errors (1 test)

#### 1. 01_async_basic.mtp
**Error**: `RuntimeError: ValueError("Undefined variable: Async")`
**Cause**: `Async` constructor/wrapper not defined in interpreter globals.
**Fix Needed**: Add `Async` constructor to interpreter's builtin objects.

---

## Priority Fixes

### High Priority (Would fix multiple tests)
1. **Block expression parsing** - Fixes tests 6, 12, 16, 19
2. **API body parsing with local functions** - Fixes tests 9, 15, 18, 20
3. **Empty array type inference** - Fixes tests 2, 14
4. **Arrow function/lambda parsing** - Fixes tests 7, 13

### Medium Priority
5. **Recursive function type inference** - Fixes test 4
6. **Local type declarations** - Fixes test 17
7. **Decimal literal syntax** - Fixes test 5
8. **JSON dot access** - Fixes test 8

### Low Priority
9. **Async constructor** - Fixes test 1
10. **Mixed-type arrays** - Fixes test 11

---

## Root Cause Analysis

### Parser Issues (Most Common)
The parser has difficulty with:
- Complex nested block expressions
- Local function declarations inside API handlers
- Arrow function syntax `(x) => expr`
- Local type declarations in expression context
- Empty blocks `{}`

### Type System Issues
The type checker has difficulty with:
- Empty array type inference
- Recursive function return type inference
- Dynamic property access on non-record types
- Heterogeneous array types

### Runtime Issues
The interpreter is missing:
- `Async` constructor for async operations

---

## Recommended Fix Order

1. Fix block expression parsing in parser
2. Fix local function scoping in parser
3. Add arrow function parsing
4. Fix empty array type inference
5. Fix recursive function type inference
6. Add remaining missing constructors/functions
