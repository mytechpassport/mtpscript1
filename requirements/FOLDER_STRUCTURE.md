# MTPScript Project Folder Structure

This document outlines the organization and purpose of all directories in the MTPScript project.

## Root Level Structure

```
mtpscript/
├── build/                    # Build artifacts and generated files
├── core/                     # Core runtime engine and libraries
├── src/                      # Source code organization
├── pkg/                      # Package management and extensions
├── tools/                    # Development and build tools
├── tests/                    # Test suites and fixtures
├── scripts/                  # Build and test automation scripts
├── requirements/             # Project requirements and documentation
├── examples/                 # Example applications and demos
├── extensions/               # Editor integrations (VS Code, Cursor)
├── compliance/               # Compliance documentation
├── docs/                     # General documentation
├── marketing/                # Marketing materials
├── runtime/                  # Runtime-specific code
├── compiler/                 # Compiler-specific code
├── vendor/                   # Third-party dependencies
└── [config files]           # Root-level configuration files
```

## Detailed Directory Descriptions

### Core Directories

#### `build/`
**Purpose:** Contains all build artifacts, generated files, and cross-compilation outputs.
- `artifacts/` - Final build outputs
- `ci/` - CI/CD scripts
- `docker/` - Docker configurations
- `generated/` - Auto-generated headers and files
- `objects/` - Compiled object files (.o)
- `templates/` - Build templates

#### `core/`
**Purpose:** Core runtime engine containing the QuickJS-based runtime that executes MTPScript code.
- `crypto/` - Cryptographic operations (mquickjs_crypto.c/h)
- `db/` - Database integration (mquickjs_db.c/h)
- `effects/` - Side effect tracking (mquickjs_effects.c/h, mquickjs_log.c/h)
- `http/` - HTTP client/server (mquickjs_http.c/h)
- `runtime/` - Core VM and execution (mquickjs.c/h, runtime.c/h)
- `stdlib/` - Standard library (mquickjs_api.c/h)
- `utils/` - Utility functions (cutils.c/h, dtoa.c/h)

#### `src/`
**Purpose:** Main source code organization for the compiler and runtime.
- `cli/` - Command-line tools (mtpsc.c)
- `compiler/` - Complete compilation pipeline (lexer, parser, typechecker, codegen, etc.)
- `decimal/` - Decimal arithmetic implementation
- `effects/` - Side effect tracking system
- `host/` - Host environment integrations (Lambda, NPM bridge)
- `lsp/` - Language Server Protocol implementation
- `main/` - Main entry points (mtpjs.c, readline.c)
- `snapshot/` - Snapshot creation and management
- `stdlib/` - Standard library implementations

### Test and Quality Assurance

#### `tests/`
**Purpose:** Comprehensive test suite for quality assurance.
- `unit/` - Unit tests (C source files and executables)
- `integration/` - Integration tests and benchmarks
- `fixtures/` - Test MTPScript files and data
- `executables/` - Compiled test executables
- `migration/` - Migration testing resources

#### `scripts/`
**Purpose:** Automation scripts for building, testing, and development.
- `verify_tests.sh` - Test verification and compilation script
- `run_complex_model_test.sh` - Complex model test runner
- `generate_build_info.sh` - Build metadata generation

### Requirements and Documentation

#### `requirements/`
**Purpose:** Project requirements, specifications, and documentation.
- `PHASE1TASK.md` - Phase 1 task specifications
- `PHASE2TASK.md` - Phase 2 task specifications
- `PHASE3TASK.md` - Phase 3 task specifications
- `HOWTOCONVERTTYPESCRIPT.md` - TypeScript conversion guide
- `HOWTOMAKEPACKAGEMANAGER.md` - Package manager implementation
- `TECHSPECV5.md` - Technical specifications v5
- `FOLDER_STRUCTURE.md` - This file

### Examples and Extensions

#### `examples/`
**Purpose:** Example applications demonstrating MTPScript usage.
- `basic_commandline/` - Basic CLI application
- `basic_crud/` - CRUD operations example
- `hashing/` - Cryptographic hashing example
- `long_running_loop/` - Long-running process example
- `third_party_api_call/` - External API integration

#### `extensions/`
**Purpose:** Editor integrations for development experience.
- `vscode/` - Visual Studio Code extension
  - `src/` - Extension source code
  - `syntaxes/` - Syntax highlighting definitions
- `cursor/` - Cursor editor support
  - `src/` - Extension source code
  - `syntaxes/` - Syntax highlighting definitions

### Specialized Directories

#### `pkg/`
**Purpose:** Package management and extended functionality.
- `decimal/` - Decimal arithmetic package
- `lsp/` - Language server package
- `readline/` - Enhanced readline package

#### `tools/`
**Purpose:** Development and build tools.
- `bench/` - Benchmarking tools
- `dev/` - Development utilities

#### `runtime/`
**Purpose:** Runtime-specific implementations.
- `gas/` - Gas metering and cost calculation
- `host/` - Host environment adaptations
- `snapshot/` - Snapshot management

#### `compiler/`
**Purpose:** Compiler-specific implementations.
- `analysis/` - Static analysis tools
- `backend/` - Code generation backends
- `cli/` - Compiler command-line interface
- `frontend/` - Compiler frontend components
- `tools/` - Compiler development tools

### Documentation and Compliance

#### `compliance/`
**Purpose:** Compliance documentation for standards.
- `iso27001-compliance.md` - ISO 27001 compliance
- `pci-dss-compliance.md` - PCI DSS compliance
- `soc2-compliance.md` - SOC 2 compliance
- `sox-compliance.md` - SOX compliance

#### `docs/`
**Purpose:** General project documentation.
- `api/` - API documentation
- `compliance/` - Compliance guides
- `marketing/` - Marketing materials
- `requirements/` - Requirements documentation

#### `marketing/`
**Purpose:** Marketing and promotional materials.

## File Placement Guidelines

### Source Code Files (.c, .h)
- **Core runtime:** `core/` subdirectories
- **Compiler:** `src/compiler/`
- **CLI tools:** `src/cli/`
- **Libraries:** `src/` subdirectories
- **Headers:** Same directory as implementation or `core/` includes

### Test Files
- **Unit tests:** `tests/unit/` (.c files and executables)
- **Integration tests:** `tests/integration/`
- **Test fixtures:** `tests/fixtures/` (.mtp, .js, .ts files)
- **Test data:** `tests/` subdirectories as appropriate

### Configuration Files
- **Build:** Root level (Makefile, Dockerfile)
- **CI/CD:** `.github/workflows/`
- **Package:** Root level (mtp.lock, package files)
- **Gas costs:** Root level (gas_costs.h, gas-v5.1.csv)

### Documentation
- **Requirements:** `requirements/`
- **API docs:** `docs/api/`
- **Compliance:** `compliance/`
- **General:** `docs/`

### Scripts and Tools
- **Build scripts:** `scripts/`
- **Development tools:** `tools/`
- **CI scripts:** `.github/workflows/`

## Directory Creation Rules

1. **Never create directories directly in root** except for established top-level directories
2. **Use subdirectories** for organization within established top-level directories
3. **Follow existing patterns** - if similar functionality exists, place in same location
4. **Document new directories** in this file when created