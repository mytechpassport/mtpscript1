# Random Test Results

## Test Suite: tests/testingset/

Run date: 2026-01-20

### Summary
- **Total tests**: 20
- **Passing**: 1
- **Failing**: 19

### Passing Tests

| Test | Result |
|------|--------|
| 03_nested_objects.mtp | PASS - Nested record types and JSON objects work correctly |

### Failing Tests (Updated after fixes)

| Test | Error | Category |
|------|-------|----------|
| 01_async_basic.mtp | `TypeError("Undefined variable: httpGet")` | Missing stdlib |
| 02_db_sqlite.mtp | `TypeError("Undefined variable: DbWrite")` | Missing stdlib |
| 04_recursion_loops.mtp | `TypeError("Arithmetic operations require numbers")` | Type system - `List<T>.length` not recognized as number |
| 05_decimal_money.mtp | `ParserError("Expected string literal")` | Parser - `Decimal.fromString()` namespace syntax |
| 06_pattern_matching.mtp | `TypeError("Undefined variable: Red")` | ADT constructors not available as values |
| 07_pipeline_operator.mtp | `ParserError("Expected ':' after parameter name")` | Parser - pipeline lambda syntax |
| 08_json_operations.mtp | `TypeError("Dot access only on records")` | Type system - `Json.parse()` namespace call |
| 09_gas_metering.mtp | `TypeError("Undefined variable: Some")` | ADT constructors not available as values |
| 10_option_result.mtp | `ParserError("Expected primary expression, found RBrace")` | Parser - Option/Result syntax |
| 11_hash_cbor.mtp | `TypeError("Undefined variable: fnv1a32")` | Missing stdlib |
| 12_http_effects.mtp | `TypeError("Undefined variable: HttpOut")` | Missing stdlib effect |
| 13_closures_lambdas.mtp | `ParserError("Expected identifier")` | Parser - closure capture syntax |
| 14_list_map_ops.mtp | `TypeError("Cannot infer type of empty array")` | Type system - empty array inference |
| 15_api_system.mtp | `ParserError("Expected '}' after API body")` | Parser - API body syntax |
| 16_number_overflow.mtp | `TypeError("Undefined variable: checked_add")` | Missing stdlib |
| 17_structural_equality.mtp | `ParserError("Expected primary expression, found Type")` | Parser - type expressions in values |
| 18_log_effects.mtp | `ParserError("Expected '}' after API body")` | Parser - Log effect syntax |
| 19_error_handling.mtp | `ParserError("Expected '}' after block expression")` | Parser - error handling syntax |
| 20_comprehensive.mtp | `ParserError("Expected '}' after API body")` | Parser - complex test file |

### Issues Fixed During This Session

1. **Lexer: Comment support (UTF-8)**
   - Added `//` single-line and `/* */` multi-line comment parsing
   - Fixed UTF-8 byte offset tracking for multi-byte characters (like `§`)
   - File: `mtpscript-core/src/lexer/scanner.rs`

2. **Parser: Multiline type definitions**
   - Made commas optional between record type fields
   - Allows newline-separated fields in type definitions
   - File: `mtpscript-core/src/parser/mod.rs`

3. **Parser: Named local functions**
   - Added support for `function name(params) { body }` inside expression contexts
   - Desugars to `const name = function(params) { body }; rest`
   - File: `mtpscript-core/src/parser/mod.rs`

4. **Lexer/Parser: i64::MIN parsing**
   - Special handling for `9223372036854775808` (only valid as `-9223372036854775808`)
   - Lexer stores as i64::MIN, parser absorbs the minus sign correctly
   - Files: `mtpscript-core/src/lexer/scanner.rs`, `mtpscript-core/src/parser/mod.rs`

5. **Codegen: Lambda return statements**
   - Lambda bodies now generate `return expr;` instead of just `expr`
   - File: `mtpscript-core/src/compiler/codegen.rs`

6. **JS Parser: Function expressions**
   - Added support for parsing `function(params) { body }` in expression context
   - Handles both anonymous and named function expressions
   - File: `mtpscript-core/src/runtime/js_parser.rs`

7. **Interpreter: Anonymous function support**
   - Anonymous functions now get unique internal names for body lookup
   - Switched to AST-based execution with fallback to string-based
   - File: `mtpscript-core/src/runtime/interpreter.rs`

8. **Parser: Match expression codegen**
   - Match expressions now generate IIFE-wrapped code for use in expression context
   - File: `mtpscript-core/src/compiler/pattern.rs`

9. **Parser: Match arm body parsing**
   - Added `parse_match_arm_body` function for proper match arm parsing
   - Handles both simple expressions and block expressions in match arms
   - Made commas optional between match arms
   - File: `mtpscript-core/src/parser/mod.rs`

### Remaining Issues to Address

#### ADT Constructor Values
ADT type definitions like `type Color = Red | Green | Blue` create type-level constructs but don't automatically make the constructors (`Red`, `Green`, `Blue`) available as values. This needs:
- Automatic creation of constructor values when ADT types are defined
- Proper handling of variant constructors with data like `Some(x)`, `Ok(v)`, `Err(e)`

#### Missing Standard Library Functions
The following functions/effects need implementation:
- `httpGet` - Async HTTP GET
- `DbWrite`, `DbRead` - Database effects
- `fnv1a32`, `sha256` - Hash functions
- `HttpOut` - HTTP effect
- `checked_add`, `checked_sub`, `checked_mul`, `checked_div` - Overflow-checked arithmetic
- `Decimal.fromString`, `Decimal.add`, etc. - Fixed-point decimal arithmetic
- `Json.parse`, `Json.stringify` - JSON operations
- `Log` effect

#### Parser Enhancements Needed
1. **Namespace.method syntax**: `Decimal.fromString("123")`, `Json.parse("{}")`
2. **Pipeline operator with lambdas**: `value |> (x) => transform(x)`
3. **Generic type syntax in expressions**: `List<number>`, `Option<T>`
4. **Empty array type inference**: Allow `[]` with context-based inference

#### Type System Enhancements
1. **Generic types**: Proper support for `List<T>`, `Option<T>`, `Result<T, E>`
2. **Method resolution**: `.length`, `.push()`, etc. on built-in types
3. **Namespace types**: `Decimal`, `Json` as namespaces with static methods

### Test Expectations vs Actual

For HTTP test (12_http_effects.mtp), the expected response from `https://jsonplaceholder.typicode.com/todos/1`:
```json
{
  "userId": 1,
  "id": 1,
  "title": "delectus aut autem",
  "completed": false
}
```

Expected answer in `tests/testinganswerset/12_http_effects.json`:
```json
{"method":"GET","response":{"userId":1,"id":1,"title":"delectus aut autem","completed":false}}
```
