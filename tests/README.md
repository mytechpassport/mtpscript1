# MTPScript Test Suite

This directory contains the complete test suite for MTPScript, organized into clear categories for maintainability and execution.

## Directory Structure

```
tests/
├── unit/              # C unit tests for compiler/runtime components
├── integration/       # JavaScript integration tests (run with mtpjs)
├── executables/       # Compiled test binaries
└── fixtures/          # Test fixture files (.mtp source files)
```

## Test Categories

### Unit Tests (`tests/unit/`)
C-language unit tests that test individual components:

- `phase0_regression_test.c` - **Main Phase 0 regression test** (19 tests)
  - Comprehensive verification of all Phase 0 requirements
  - Tests snapshot isolation, deterministic execution, gas limits, etc.
- `acceptance_tests.c` - Acceptance criteria tests
- `test.c` - Core utility tests (strings, vectors, decimals)

### Integration Tests (`tests/integration/`)
JavaScript test files executed by the `mtpjs` runtime:

- `test_builtin.js` - Built-in function tests
- `test_closure.js` - Closure functionality tests
- `test_language.js` - Language feature tests
- `test_loop.js` - Loop construct tests
- `test_rect.js` - C API integration tests (Rectangle class)
- `mandelbrot.js` - Performance/benchmark tests
- `microbench.js` - Microbenchmark tests

### Test Executables (`tests/executables/`)
Compiled test binaries:

- `phase0_regression_test` - Main Phase 0 regression test executable
- `mtpsc_test` - MTPScript compiler unit tests

### Test Fixtures (`tests/fixtures/`)
MTPScript source files used for testing:

- Various `.mtp` files testing different language features
- Compiler test cases and examples

## Running Tests

### Run All Tests
```bash
make test
```

### Run Specific Test Categories

#### Phase 0 Regression Tests (Main)
```bash
make phase0_regression_test
./phase0_regression_test
```

#### Unit Tests Only
```bash
make mtpsc_test
./tests/executables/mtpsc_test
```

#### Integration Tests Only
```bash
./mtpjs tests/integration/test_builtin.js
./mtpjs tests/integration/test_closure.js
# ... etc
```

### Manual Test Execution

#### Individual JavaScript Tests
```bash
./mtpjs tests/integration/test_rect.js
```

#### Individual C Tests
```bash
./tests/executables/phase0_regression_test
./tests/executables/mtpsc_test
```

## Test Coverage

### Phase 0 Requirements (100% Verified)
- ✅ **1. Snapshot-Based Execution Model** - VM snapshots, signing, isolation
- ✅ **2. Deterministic Seed Injection** - SHA-256 seeds, runtime determinism
- ✅ **3. Runtime Gas Limit Injection** - Gas contracts, exhaustion semantics
- ✅ **4. IEEE-754 Decimal Arithmetic** - Decimal types, operations, serialization
- ✅ **5. Engine Hardening & Security** - Forbidden features, memory constraints
- ✅ **6. Canonical JSON Output** - Deterministic serialization, response hashing

### Phase 1 Integration Features
- ✅ **Compiler Pipeline** - Lexer → Parser → Typechecker → Codegen
- ✅ **JSON ADT** - First-class JSON with JsonNull constraint
- ✅ **JavaScript Code Generation** - Pipeline operators, async desugaring
- ✅ **CLI Tools** - mtpsc check command

## Adding New Tests

### Adding Unit Tests
1. Add C test file to `tests/unit/`
2. Update `Makefile` TEST_PROGS and object dependencies
3. Add test execution to `make test` target

### Adding Integration Tests
1. Add JavaScript test file to `tests/integration/`
2. Update `make test` target to include the new test

### Adding Test Fixtures
1. Add `.mtp` files to `tests/fixtures/`
2. Reference them in appropriate test files

## Test Results

**Current Status: 19/19 tests passing**

```
===========================================
MTPScript Phase 0 Regression Tests: 19/19 passed
===========================================
✓ ALL PHASE 0 REQUIREMENTS VERIFIED
✓ PHASE 1 INTEGRATION TESTS PASSING
✓ READY FOR PRODUCTION
```

## Continuous Integration

The test suite is designed to be run automatically:

- `make test` - Runs complete test suite
- Individual test executables can be run separately
- All tests must pass for successful builds

---

**Maintainer Notes:**
- Keep test organization clean and categorized
- Update this README when adding new test categories
- All Phase 0 requirements must remain 100% tested
