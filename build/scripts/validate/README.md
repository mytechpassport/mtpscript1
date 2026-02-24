# MTPScript Validation Scripts

These scripts ensure code functionality is preserved during the project restructuring.

## Quick Start

### Before Starting Refactor
```bash
# Create baselines for current working code
./validate_build.sh --baseline > build_baseline.json
./validate_tests.sh --baseline > test_baseline.json
```

### During Refactor (After Each Move)
```bash
# Quick validation after file moves
make clean && make all
./validate_build.sh --compare build_baseline.json
./validate_tests.sh --quick
```

### After Major Component Migration
```bash
# Comprehensive validation
./validate_tests.sh --compare test_baseline.json
```

## Scripts Overview

### `validate_build.sh`
- **Purpose**: Validates build artifacts and compares with baseline
- **Checks**:
  - Executable sizes and hashes
  - Build artifacts count
  - Build warnings and errors
  - Build time comparison

### `validate_tests.sh`
- **Purpose**: Runs test suite and validates results
- **Checks**:
  - Test pass/fail counts
  - Performance regressions
  - Binary size changes
  - Coverage metrics (when available)

## Validation Workflow

### 1. Establish Baselines
```bash
# Must be run on clean, working codebase
./validate_build.sh --baseline > build_baseline_$(date +%Y%m%d).json
./validate_tests.sh --baseline > test_baseline_$(date +%Y%m%d).json
```

### 2. During Migration
For each batch of file moves:

```bash
# Move files
git mv source/file.c destination/

# Update includes and makefiles
./update_includes.sh affected_headers
./update_makefile.sh moved_files

# Validate immediately
make clean && make all
./validate_build.sh --compare build_baseline.json

# If build passes, run tests
./validate_tests.sh --quick
```

### 3. Component Validation
After moving a major component:

```bash
# Full validation
make test
./validate_tests.sh --compare test_baseline.json

# Cross-platform check
make CONFIG_ARM32=y clean all
make CONFIG_WIN32=y clean all
```

### 4. Rollback Procedure
If validation fails:

```bash
# Immediate rollback
git reset --hard HEAD~1

# Rebuild and verify
make clean && make all
./validate_build.sh --compare build_baseline.json
```

## Exit Codes and Meanings

- **0**: All checks passed
- **1**: Critical failure (build broken, tests failing)
- **2**: Warning (performance regression, size increase)

## Common Issues and Solutions

### Build Fails After Move
```bash
# Check for missing includes
grep -r "#include" destination/ | grep -v "found"

# Verify makefile paths
grep -n "source/" Makefile
```

### Tests Fail After Move
```bash
# Check test data paths
grep -r "fixtures/" tests/

# Verify executable paths in tests
grep -r "\./mtp" tests/
```

### Performance Regression
```bash
# Profile build time
time make clean all

# Check binary size
ls -la mtpjs mtpsc
```

## Integration with CI/CD

Add to your CI pipeline:

```yaml
validate:
  script:
    - ./build/scripts/validate/validate_build.sh --compare build_baseline.json
    - ./build/scripts/validate/validate_tests.sh --compare test_baseline.json

  artifacts:
    reports:
      junit: test_results.xml
    expire_in: 1 week
```

## Extending Validation

### Adding New Checks
1. Create new validation script in this directory
2. Update baseline format to include new metrics
3. Add comparison logic for new checks
4. Update this README

### Custom Component Validation
```bash
# Example: Validate compiler component
./validate_component.sh compiler

# Add to validate_tests.sh case statement
"compiler")
    ./test_lexer.sh && ./test_parser.sh && ./test_codegen.sh
    ;;
```

## Troubleshooting

### Scripts Not Found
```bash
# Ensure scripts are executable
chmod +x build/scripts/validate/*.sh

# Check PATH
pwd && ls -la build/scripts/validate/
```

### Baseline Files Missing
```bash
# Recreate baselines
./validate_build.sh --baseline > build_baseline.json
./validate_tests.sh --baseline > test_baseline.json
```

### Permission Issues
```bash
# On macOS/Linux
chmod +x build/scripts/validate/*.sh
```

For questions or issues, see the main REFACTOR.md document.
