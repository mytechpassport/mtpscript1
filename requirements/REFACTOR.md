# MTPScript Project Restructuring Plan

## Executive Summary

The current MTPScript codebase suffers from organizational issues that make development, maintenance, and onboarding difficult. This document proposes a comprehensive restructuring to improve code organization, build processes, and developer experience.

## Current Issues

### 1. Mixed Build Artifacts and Source Code
- Object files (`.o`), dependency files (`.d`) scattered throughout source directories
- Build artifacts committed to version control
- No clear separation between source and build outputs

### 2. Inconsistent Directory Structure
- Some source files in root directory, others in `src/`
- Related functionality spread across multiple locations
- No clear architectural boundaries

### 3. Poor Separation of Concerns
- Runtime engine files mixed with compiler files
- Test code scattered across multiple directories
- Documentation and requirements intermixed with code

### 4. Complex Build System
- Makefile references files in inconsistent locations
- Build dependencies hard to track
- Cross-compilation logic scattered throughout

## Proposed New Structure

```
/mtpscript/
├── docs/                    # 📚 All documentation centralized
│   ├── requirements/        # Development requirements and phases
│   ├── compliance/          # SOC2, ISO27001, PCI-DSS compliance docs
│   ├── api/                 # API documentation and specifications
│   └── marketing/           # Marketing materials and website
├── core/                    # 🔧 Core runtime engine (MicroQuickJS)
│   ├── runtime/             # VM runtime and execution engine
│   ├── stdlib/              # Standard library bindings
│   ├── effects/             # Effect system implementation
│   ├── crypto/              # Cryptographic operations
│   ├── http/                # HTTP client/server functionality
│   ├── db/                  # Database connectivity
│   └── utils/               # Core utilities (dtoa, libm, cutils)
├── compiler/                # ⚙️ MTPScript compiler pipeline
│   ├── frontend/            # Lexing, parsing, AST construction
│   ├── analysis/            # Type checking, effects analysis
│   ├── backend/             # Code generation, bytecode compilation
│   ├── tools/               # Utilities (migration, OpenAPI generation)
│   └── cli/                 # Command-line interface and commands
├── extensions/              # 🛠️ IDE support
│   ├── vscode/              # Visual Studio Code extension
│   └── cursor/              # Cursor IDE extension
├── build/                   # 🏗️ Build system and infrastructure
│   ├── docker/              # Containerization and reproducible builds
│   ├── scripts/             # Build and development scripts
│   └── ci/                  # CI/CD configuration and workflows
├── runtime/                 # 🚀 Runtime deployment and hosting
│   ├── host/                # Host adapters (AWS Lambda, npm bridge)
│   ├── snapshot/            # Snapshot creation and management
│   └── gas/                 # Gas metering and cost tables
├── tests/                   # 🧪 Test suites and fixtures
│   ├── unit/                # Unit tests for individual components
│   ├── integration/         # Integration tests for full pipeline
│   ├── fixtures/            # Test data and sample programs
│   ├── migration/           # Migration test batches
│   └── executables/         # Test executables and harnesses
├── pkg/                     # 📦 Additional packages and modules
│   ├── decimal/             # Decimal arithmetic implementation
│   ├── lsp/                 # Language server protocol support
│   └── readline/            # REPL and interactive components
└── tools/                   # 🔨 Development and utility tools
    ├── bench/               # Benchmarking tools
    └── dev/                 # Development utilities
```

## Migration Strategy

### Phase 1: Directory Structure Creation
Create new directory structure while maintaining old structure for compatibility.

### Phase 2: File Migration
Move files to appropriate new locations with updated import/include paths.

### Phase 3: Build System Updates
Update Makefile and build scripts to reference new locations.

### Phase 4: Documentation Updates
Update all documentation to reflect new structure.

### Phase 5: Cleanup
Remove old directory structure and update CI/CD.

## Validation and Testing Strategy

### Pre-Refactor Baseline Establishment

**Before any file moves**, establish comprehensive baselines:

```bash
# 1. Record current build outputs
make clean && make all
./validate_build.sh --baseline > build_baseline.json

# 2. Run full test suite and record results
make test
./validate_tests.sh --baseline > test_baseline.json

# 3. Capture performance metrics
./benchmark.sh --comprehensive > perf_baseline.json

# 4. Generate and store binary hashes for comparison
find . -name "*.o" -o -name "mtpjs" -o -name "mtpsc" | xargs sha256sum > binary_hashes_baseline.txt
```

### During-Refactor Validation Checks

**For each file move or batch of moves**, perform these checks:

#### 1. **Build Validation**
```bash
# After each significant move, verify builds still work
make clean && make all

# Compare build artifacts with baseline
./validate_build.sh --compare build_baseline.json

# Check for undefined symbols or linking errors
nm build/out/*.o | grep " U " | wc -l  # Should be 0
```

#### 2. **Header Dependency Validation**
```bash
# Verify all includes resolve correctly
find . -name "*.c" -exec gcc -M {} \; | grep -i "error\|not found" || echo "All headers resolved"

# Check for circular dependencies
./analyze_dependencies.py --check-circular
```

#### 3. **ABI Compatibility Check**
```bash
# For shared libraries or public APIs
# Compare symbol tables before/after moves
nm old_build/libmtpscript.a > old_symbols.txt
nm new_build/libmtpscript.a > new_symbols.txt
diff old_symbols.txt new_symbols.txt || echo "ABI changes detected - review required"
```

### Component-Specific Validation

#### **Core Runtime (MicroQuickJS)**
```bash
# Test VM isolation and determinism
./test_vm_isolation.sh

# Validate bytecode compatibility
./test_bytecode_compatibility.sh

# Check gas metering accuracy
./validate_gas_costs.sh gas-v5.1.csv
```

#### **Compiler Pipeline**
```bash
# Test each compilation stage
./test_lexer.sh && ./test_parser.sh && ./test_typechecker.sh

# Validate code generation determinism
./test_codegen_determinism.sh

# Test OpenAPI generation
./test_openapi_generation.sh
```

#### **Host Adapters**
```bash
# Test Lambda cold start performance
./benchmark_lambda_coldstart.sh

# Validate npm bridge security
./test_npm_bridge_sandbox.sh

# Check effect injection
./test_effect_injection.sh
```

### Post-Move Integration Testing

**After each major component relocation:**

```bash
# 1. Full compilation test
make clean && make all && make test

# 2. Cross-compilation validation
make CONFIG_ARM32=y clean all
make CONFIG_WIN32=y CONFIG_X86_32=y clean all

# 3. Integration test suite
./run_integration_tests.sh --comprehensive

# 4. Performance regression check
./benchmark.sh --compare perf_baseline.json

# 5. Determinism verification
./test_determinism.sh --full-suite
```

### Automated Validation Scripts

Create these validation tools in `build/scripts/validate/`:

#### **`validate_build.sh`**
```bash
#!/bin/bash
# Validates build artifacts and compares with baseline

MODE=$1
BASELINE_FILE=$2

if [ "$MODE" = "--baseline" ]; then
    echo "Creating build baseline..."
    # Capture build metadata, file sizes, symbols, etc.
elif [ "$MODE" = "--compare" ]; then
    echo "Comparing with baseline..."
    # Compare current build with baseline
fi
```

#### **`validate_tests.sh`**
```bash
#!/bin/bash
# Comprehensive test validation and reporting

# Run all test suites
# Compare results with baseline
# Generate detailed failure analysis
# Check test coverage metrics
```

#### **`validate_dependencies.sh`**
```bash
#!/bin/bash
# Static analysis of header dependencies

# Find all #include statements
# Build dependency graph
# Detect cycles and missing dependencies
# Suggest optimal include order
```

### Continuous Integration Validation

**CI Pipeline Checks:**
```yaml
# In build/ci/pipeline.yml
stages:
  - validate
  - build
  - test
  - performance
  - deploy

validate:
  - script:
    - ./build/scripts/validate/validate_structure.sh
    - ./build/scripts/validate/validate_dependencies.sh
    - ./build/scripts/validate/validate_build.sh --compare

build:
  - script:
    - make clean all
    - ./build/scripts/validate/validate_artifacts.sh

test:
  - script:
    - make test
    - ./build/scripts/validate/validate_tests.sh --comprehensive

performance:
  - script:
    - ./benchmark.sh --regression-check
```

### Rollback and Recovery Procedures

**If validation fails:**

```bash
# 1. Immediate rollback
git checkout HEAD~1  # Or specific commit
make clean && make all

# 2. Bisect to find problematic change
git bisect start HEAD <last_good_commit>

# 3. Analyze failure
./diagnose_failure.sh --failed-component $COMPONENT

# 4. Apply fix or adjust migration plan
# 5. Re-validate before proceeding
```

### Success Criteria Checklist

**For each migrated component:**
- [ ] Builds without warnings/errors
- [ ] All unit tests pass
- [ ] Integration tests pass
- [ ] Performance within 5% of baseline
- [ ] Binary compatibility maintained (if applicable)
- [ ] Documentation updated
- [ ] CI/CD pipelines updated

**For complete migration:**
- [ ] All components migrated
- [ ] Full test suite passes
- [ ] Performance benchmarks meet or exceed baseline
- [ ] Reproducible builds verified
- [ ] Cross-platform compatibility confirmed
- [ ] Documentation complete and accurate

### Monitoring and Alerting

**Set up monitoring for:**
- Build time regressions (>10% increase)
- Test failure rates
- Performance degradation
- Code coverage drops
- Dependency issues

**Automated alerts:**
```bash
# Daily validation cron job
0 2 * * * /path/to/mtpscript/build/scripts/validate/daily_check.sh
```

## Practical Migration Workflow

### Developer Checklist for File Moves

**Before starting any moves:**
```bash
# 1. Create feature branch
git checkout -b refactor/component-migration

# 2. Run baseline validation
./build/scripts/validate/establish_baseline.sh

# 3. Ensure clean working directory
git status --porcelain | wc -l  # Should be 0
```

**For each file or small batch of files:**
```bash
# 1. Plan the move
echo "Moving: $SOURCE_FILES -> $DESTINATION_DIR"
echo "Updated includes needed: $HEADERS_TO_UPDATE"

# 2. Move files with git mv (preserves history)
git mv $SOURCE_FILES $DESTINATION_DIR/

# 3. Update all include paths
./build/scripts/migrate/update_includes.sh $HEADERS_TO_UPDATE

# 4. Update Makefile references
./build/scripts/migrate/update_makefile.sh $MOVED_FILES

# 5. Run immediate validation
make clean && make all
if [ $? -ne 0 ]; then
    echo "Build failed! Rolling back..."
    git checkout HEAD~1
    exit 1
fi

# 6. Run component-specific tests
./build/scripts/validate/test_component.sh $COMPONENT_NAME

# 7. Commit with descriptive message
git add .
git commit -m "refactor: Move $COMPONENT_NAME to $DESTINATION_DIR

- Moved $SOURCE_FILES to $DESTINATION_DIR/
- Updated include paths in affected files
- Updated Makefile references
- Verified build and basic functionality"
```

**After each major component migration:**
```bash
# 1. Full test suite
make test && ./run_integration_tests.sh

# 2. Performance check
./benchmark.sh --quick-check

# 3. Cross-platform validation
make CONFIG_ARM32=y clean all
make CONFIG_WIN32=y clean all

# 4. Create checkpoint commit
git commit -m "refactor: Complete $COMPONENT_NAME migration

- All files moved to new structure
- Build system updated
- Tests passing
- Performance validated
- Cross-platform compatibility confirmed"
```

### Migration Automation Scripts

Create these helper scripts in `build/scripts/migrate/`:

#### **`update_includes.sh`**
```bash
#!/bin/bash
# Automatically update #include paths after file moves

for header in "$@"; do
    old_path=$(find_old_location "$header")
    new_path=$(find_new_location "$header")

    # Update all references
    sed -i "s|#include \"$old_path\"|#include \"$new_path\"|g" $(find . -name "*.c" -o -name "*.h")
done
```

#### **`update_makefile.sh`**
```bash
#!/bin/bash
# Update Makefile references to moved files

for file in "$@"; do
    old_path=$file
    new_path=$(map_to_new_location "$file")

    # Update source file lists in Makefile
    sed -i "s|$old_path|$new_path|g" Makefile
done
```

#### **`validate_component.sh`**
```bash
#!/bin/bash
# Component-specific validation after moves

COMPONENT=$1

case $COMPONENT in
    "runtime")
        echo "Validating runtime component..."
        make clean && make mtpjs
        ./test_vm_basic.sh
        ./test_bytecode.sh
        ;;
    "compiler")
        echo "Validating compiler component..."
        make clean && make mtpsc
        ./test_compilation.sh
        ./test_typecheck.sh
        ;;
    "effects")
        echo "Validating effects system..."
        ./test_effects.sh
        ./test_determinism.sh
        ;;
    *)
        echo "Unknown component: $COMPONENT"
        exit 1
        ;;
esac
```

### Risk Assessment Matrix

| Risk Level | Risk Description | Mitigation Strategy | Validation Required |
|------------|------------------|-------------------|-------------------|
| **Critical** | Build completely fails | Immediate rollback + manual fix | Full build + all tests |
| **High** | Performance regression >20% | Performance profiling + optimization | Benchmark comparison |
| **Medium** | Cross-platform build fails | Platform-specific fixes | Multi-platform CI |
| **Low** | Include path warnings | Automated path updates | Build warnings check |
| **Low** | Test timeout increases | Test optimization | Timeout monitoring |

### Communication Plan

**During Migration:**
- Daily status updates in team chat
- Weekly progress reports
- Immediate notification of any build failures
- Code review required for all migration commits

**Post-Migration:**
- Migration completion announcement
- Updated contributor documentation
- Training sessions for new structure
- Updated CI/CD documentation

## Detailed File Mappings

### Core Runtime Engine → `core/`

#### Current → New
```
mquickjs*.c,h        → core/runtime/
mquickjs_api.*       → core/stdlib/
mquickjs_crypto.*    → core/crypto/
mquickjs_db.*        → core/db/
mquickjs_http.*      → core/http/
mquickjs_log.*       → core/effects/
mquickjs_errors.*    → core/runtime/
cutils.*             → core/utils/
dtoa.*               → core/utils/
libm.*               → core/utils/
list.h               → core/utils/
gas_costs.h          → runtime/gas/
```

### Compiler → `compiler/`

#### Current → New
```
src/compiler/*.c,h   → compiler/frontend/ (lexer, parser, ast)
                     → compiler/analysis/ (typechecker)
                     → compiler/backend/ (codegen, bytecode)
                     → compiler/tools/ (openapi, migration, typescript_parser)
src/cli/*.c          → compiler/cli/
```

### Runtime Components → `runtime/`

#### Current → New
```
src/host/*.c,h       → runtime/host/
src/snapshot/*.c,h   → runtime/snapshot/
src/effects/*.c,h    → core/effects/
gas-v5.1.csv         → runtime/gas/
```

### Tests → `tests/`

#### Current → New
```
tests/*              → tests/integration/
src/test/*           → tests/unit/
test_migration_batch/ → tests/migration/
```

### Documentation → `docs/`

#### Current → New
```
requirements/*       → docs/requirements/
compliance/*         → docs/compliance/
README.md            → docs/README.md (main project README)
marketing/*          → docs/marketing/
```

### Extensions → `extensions/`

#### Current → New
```
extensions/*         → extensions/*/ (already well organized)
```

### Build System → `build/`

#### Current → New
```
Dockerfile           → build/docker/
ci.yml.txt           → build/ci/
generate_build_info.sh → build/scripts/
build_info_generator.c → build/scripts/
```

### Packages → `pkg/`

#### Current → New
```
src/decimal/*.c,h    → pkg/decimal/
src/lsp/*.c,h        → pkg/lsp/
readline*.*          → pkg/readline/
```

## Updated Build System

### New Makefile Structure
```
build/Makefile.core     # Core runtime build rules
build/Makefile.compiler # Compiler build rules
build/Makefile.test     # Test build rules
Makefile               # Main orchestrating makefile
```

### Build Output Isolation
- All build artifacts (`.o`, `.d`, executables) go to `build/out/`
- Clean separation between source and build products
- Support for multiple build configurations

## Benefits of New Structure

### 1. **Clear Architectural Boundaries**
- Runtime engine separated from compiler
- Host adapters isolated from core logic
- Build system components centralized

### 2. **Improved Developer Experience**
- Related code co-located
- Easier navigation and understanding
- Consistent naming conventions

### 3. **Better Build Process**
- Isolated build artifacts
- Cleaner dependency tracking
- Easier cross-compilation support

### 4. **Enhanced Maintainability**
- Clear ownership of components
- Easier testing and debugging
- Simplified onboarding for new developers

### 5. **Future-Proofing**
- Extensible structure for new features
- Clear patterns for adding new components
- Separation allows independent evolution of subsystems

## Implementation Timeline

### Week 1-2: Planning and Setup
- [ ] Create new directory structure
- [ ] Set up symbolic links for backward compatibility
- [ ] Update CI to build from both old and new structures

### Week 3-4: Core Runtime Migration
- [ ] Move MicroQuickJS files to `core/runtime/`
- [ ] Update all include paths
- [ ] Verify core runtime builds correctly

### Week 5-6: Compiler Migration
- [ ] Move compiler components to appropriate subdirectories
- [ ] Update build system references
- [ ] Test compilation pipeline

### Week 7-8: Supporting Components
- [ ] Move host adapters, snapshots, tests
- [ ] Update documentation and requirements
- [ ] Migrate build system components

### Week 9-10: Integration and Testing
- [ ] Update all build scripts and CI
- [ ] Comprehensive testing across all components
- [ ] Performance regression testing

### Week 11-12: Cleanup and Documentation
- [ ] Remove old directory structure
- [ ] Update all documentation
- [ ] Create migration guide for contributors

## Risk Mitigation

### 1. **Backward Compatibility**
- Maintain old structure during transition
- Gradual migration with continuous integration
- Rollback plan if issues arise

### 2. **Build System Stability**
- Parallel build support for old and new structures
- Comprehensive testing before switching
- Incremental makefile updates

### 3. **Developer Impact**
- Clear communication of changes
- Training sessions for new structure
- Migration scripts to automate file moves

## Success Metrics

- [ ] All builds pass with new structure
- [ ] No performance regressions
- [ ] Developer onboarding time reduced by 30%
- [ ] Build time improved due to better dependency tracking
- [ ] Code navigation and understanding improved

## Conclusion

This restructuring addresses fundamental organizational issues in the MTPScript codebase while providing a solid foundation for future development. The new structure improves maintainability, developer experience, and architectural clarity without compromising the project's core functionality.

The migration should be approached carefully with thorough testing at each phase to ensure stability and performance are maintained throughout the transition.
