# MTPScript Syntax Discrepancies

This document tracks discrepancies between MTPScript syntax (as implemented and documented) and the goal of being mapped 1-to-1 with JavaScript/TypeScript syntax.

## Task List

- [x] **Rename `func` to `function`**: The implementation uses `func`, while the goal is 1-to-1 mapping with JS `function`.
- [x] **Unify `fn` documentation**: `README.md` uses `fn` for function declarations, which matches neither the implementation (`func`) nor JS (`function`).
- [x] **Type Keyword Mapping**: Rename/Alias primitive types:
  - [x] `Int` -> `number`
  - [x] `String` -> `string`
  - [x] `Bool` -> `boolean`
  - [x] `Decimal` -> `Decimal` (kept as is per user request)
- [x] **If-Statement Parentheses**: Require parentheses for `if` conditions to match JS: `if (condition) { ... }`. (Implemented in documentation and test fixtures; parser now correctly handles updated fixtures)
- [x] **Return Type Syntax**: Standardize on `: Type` (TypeScript style) instead of `-> Type` (currently in README and lexer).
- [x] **Variable Declarations**: Support `const` in addition to `let` (mapping all to immutable state).
- [ ] **Control Flow**: Re-evaluate `match` vs JS `switch` statement.
- [x] **Pipeline Operator**: Document that `|>` is a non-standard JS extension.
- [x] **API/Effect Keywords**: Document `api`, `uses`, and `effect` as language extensions not present in JS.

## Current Discrepancy Matrix

| Feature | implementation (`.c`) | documentation (`README.md`) | Goal (JavaScript/TypeScript) |
|---------|-----------------------|-----------------------------|------------------------------|
| Function Keyword | `func` | `fn` | `function` |
| Variable Keyword | `let` | `let` | `const`, `let` |
| Int Type | `Int` | `Int` | `number` |
| String Type | `String` | `String` | `string` |
| Bool Type | `Bool` | `Bool` | `boolean` |
| Return Type | `:` (parser) | `->` (docs/lexer) | `:` |
| If Condition | `if condition` | `if condition` | `if (condition)` |
| Match Expression | `match` | `match` | `switch` |

## Documentation Locations to Fix

### 1. `README.md` [COMPLETED]
- Update all `fn` examples to `function`.
- Update type names from `Int`, `String`, `Bool` to `number`, `string`, `boolean`.
- Add parentheses to `if` conditions.
- Update `->` return type arrows to `:`.
- Update `let` to `const` where appropriate (since all are immutable).

### 2. `requirements/TECHSPECV5.md` [COMPLETED]
- Section 3 (Syntax & Grammar): Update grammar rules for `function`, `if` parentheses, and type keywords.
- Section 4.1 (Primitive Types): Update type names to match JS.
- Section 7-a (Async Effect): Update surface syntax examples.
- Section 8 (API System): Update `uses` and `api` examples.

### 3. `requirements/HOWTOCONVERTTYPESCRIPT.md` [COMPLETED]
- Update the "Manual Conversion Steps" to show that mapping is now 1-to-1, reducing the need for many transforms.
- Update the type mapping table.

### 4. `requirements/HOWTOGUIDE.md` [COMPLETED]
- Update all code examples to use the new JS-aligned syntax.

