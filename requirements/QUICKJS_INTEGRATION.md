# MTPScript Task Breakdown: QuickJS Integration

## Phase R.0: QuickJS Integration & Rename to MTPJS (Week 1)

### Task R.0.1: Clone QuickJS Source Code
**Objective**: Download QuickJS source code into our project structure
**Specification Reference**: TECHSPECV5.md lines 346-347 (MTPJS patches, double-path fix)

**Files to Create**: 
- `vendor/quickjs/` (temporary directory)
- `scripts/clone_quickjs.sh`

**Implementation Steps**:
```bash
#!/bin/bash
# scripts/clone_quickjs.sh
set -e

echo "Cloning QuickJS into vendor/quickjs..."

# Clone QuickJS repository
mkdir -p vendor
cd vendor
git clone https://github.com/bellard/quickjs.git
cd quickjs

# Checkout specific known-good version for stability
git checkout 5bf826d5ebaf6193c74945b24170339c2e5cc5ea

echo "QuickJS cloned to vendor/quickjs"
```

**Test to Create**: `tests/unit/quickjs_clone_test.c`
```c
int test_quickjs_clone_success() {
    // Verify QuickJS was cloned to correct location
    assert(dir_exists("vendor/quickjs"));
    assert(file_exists("vendor/quickjs/quickjs.c"));
    assert(file_exists("vendor/quickjs/quickjs.h"));
    assert(file_exists("vendor/quickjs/Makefile"));
    
    return PASS;
}
```

---

### Task R.0.2: Create MTPJS Directory Structure
**Objective**: Create the directory structure for MTPJS based on FOLDER_STRUCTURE.md
**Specification Reference**: FOLDER_STRUCTURE.md lines 41-50 (core directory structure)

**Files to Create**: Directory structure
**Build Output**: Directory structure in build/

**Implementation Steps**:
```bash
#!/bin/bash
# scripts/create_mtpjs_structure.sh
set -e

echo "Creating MTPJS directory structure..."

# Core runtime directories
mkdir -p core/runtime
mkdir -p core/utils
mkdir -p core/regex
mkdir -p core/stdlib
mkdir -p core/include
mkdir -p core/atom
mkdir -p core/opcode

# Source directories
mkdir -p src/main
mkdir -p src/cli

# Build directories
mkdir -p build/objects/core/runtime
mkdir -p build/objects/core/utils
mkdir -p build/objects/core/regex
mkdir -p build/objects/core/stdlib
mkdir -p build/objects/src/main
mkdir -p build/objects/src/cli
mkdir -p build/artifacts
mkdir -p build/generated

echo "MTPJS directory structure created"
```

**Test to Create**: `tests/unit/structure_test.c`
```c
int test_mtpjs_directory_structure() {
    // Verify core directories exist
    assert(dir_exists("core/runtime"));
    assert(dir_exists("core/utils"));
    assert(dir_exists("core/regex"));
    assert(dir_exists("core/stdlib"));
    assert(dir_exists("core/include"));
    
    // Verify source directories exist
    assert(dir_exists("src/main"));
    assert(dir_exists("src/cli"));
    
    // Verify build directories exist
    assert(dir_exists("build/objects/core/runtime"));
    assert(dir_exists("build/artifacts"));
    
    return PASS;
}
```

---

### Task R.0.3: Copy QuickJS Core Files to MTPJS Structure
**Objective**: Copy QuickJS source files to appropriate MTPJS directories
**Specification Reference**: FOLDER_STRUCTURE.md lines 158-187 (file placement guidelines)

**Files to Create**: File copies from vendor/quickjs/ to core/

**Implementation Pseudocode**:
```bash
#!/bin/bash
# scripts/copy_quickjs_files.sh
set -e

echo "Copying QuickJS files to MTPJS structure..."

# Core runtime files
cp vendor/quickjs/quickjs.c core/runtime/mtpjs.c
cp vendor/quickjs/quickjs.h core/include/mtpjs.h

# Utility files
cp vendor/quickjs/cutils.c core/utils/mtp_cutils.c
cp vendor/quickjs/cutils.h core/utils/mtp_cutils.h
cp vendor/quickjs/dtoa.c core/utils/mtp_dtoa.c
cp vendor/quickjs/dtoa.h core/utils/mtp_dtoa.h

# Regular expression files
cp vendor/quickjs/libregexp.c core/regex/mtp_libregexp.c
cp vendor/quickjs/libregexp.h core/regex/mtp_libregexp.h
cp vendor/quickjs/libunicode.c core/regex/mtp_libunicode.c
cp vendor/quickjs/libunicode.h core/regex/mtp_libunicode.h

# Standard library files
cp vendor/quickjs/quickjs-libc.c core/stdlib/mtpjs-libc.c
cp vendor/quickjs/quickjs-libc.h core/stdlib/mtpjs-libc.h

# Atom and opcode files
cp vendor/quickjs/quickjs-atom.h core/atom/mtpjs-atom.h
cp vendor/quickjs/quickjs-opcode.h core/opcode/mtpjs-opcode.h

# Main application files
cp vendor/quickjs/qjs.c src/main/mtpjs_repl.c
cp vendor/quickjs/qjsc.c src/cli/mtpsc.c

echo "QuickJS files copied to MTPJS structure"
```

**Test to Create**: `tests/unit/file_copy_test.c`
```c
int test_quickjs_files_copied() {
    // Verify core runtime files were copied
    assert(file_exists("core/runtime/mtpjs.c"));
    assert(file_exists("core/include/mtpjs.h"));
    
    // Verify utility files were copied
    assert(file_exists("core/utils/mtp_cutils.c"));
    assert(file_exists("core/utils/mtp_dtoa.c"));
    
    // Verify regex files were copied
    assert(file_exists("core/regex/mtp_libregexp.c"));
    assert(file_exists("core/regex/mtp_libunicode.c"));
    
    // Verify stdlib files were copied
    assert(file_exists("core/stdlib/mtpjs-libc.c"));
    
    // Verify main files were copied
    assert(file_exists("src/main/mtpjs_repl.c"));
    assert(file_exists("src/cli/mtpsc.c"));
    
    return PASS;
}
```

---

### Task R.0.4: Systematic Rename QuickJS to MTPJS
**Objective**: Replace all references to "quickjs" with "mtpjs" in copied files
**Specification Reference**: TECHSPECV5.md lines 9-17 (JavaScript as execution encoding, MTPJS runtime)

**Files to Modify**: All .c and .h files in core/ and src/
**Build Output**: Renamed files in build/objects/

**Implementation Pseudocode**:
```bash
#!/bin/bash
# scripts/rename_to_mtpjs.sh
set -e

echo "Renaming QuickJS references to MTPJS..."

# Function to rename in a single file
rename_in_file() {
    local file="$1"
    echo "Processing $file..."
    
    # Create backup
    cp "$file" "$file.bak"
    
    # Systematic replacements
    sed -i.tmp \
        -e 's/quickjs/mtpjs/g' \
        -e 's/QuickJS/MTPJS/g' \
        -e 's/QUICKJS/MTPJS/g' \
        -e 's/qjs/mjs/g' \
        -e 's/QJS/MJS/g' \
        -e 's/libquickjs/libmtpjs/g' \
        -e 's/QUICKJS_VERSION/MTPJS_VERSION/g' \
        "$file"
    
    # Remove backup and temp files
    rm "$file.bak" "$file.tmp"
}

# Rename in all .c and .h files
find core/ src/ -name "*.c" -o -name "*.h" | while read file; do
    rename_in_file "$file"
done

# Special case: rename filenames themselves
mv core/quickjs-libc.c core/mtpjs-libc.c 2>/dev/null || true
mv core/quickjs-libc.h core/mtpjs-libc.h 2>/dev/null || true

echo "Renaming complete"
```

**Specific Renamings Required**:
```c
// In core/include/mtpjs.h:
#ifndef QUICKJS_H → #ifndef MTPJS_H
#define QUICKJS_H → #define MTPJS_H
typedef struct JSRuntime → typedef struct MTPJSRuntime
typedef struct JSContext → typedef struct MTPJSContext

// In core/runtime/mtpjs.c:
JS_NewRuntime → MTPJS_NewRuntime
JS_NewContext → MTPJS_NewContext
JS_Eval → MTPJS_Eval
JS_FreeRuntime → MTPJS_FreeRuntime
JS_FreeContext → MTPJS_FreeContext

// In core/utils/mtp_cutils.h:
js_malloc → mtpjs_malloc
js_free → mtpjs_free
js_realloc → mtpjs_realloc
```

**Test to Create**: `tests/unit/rename_test.c`
```c
int test_quickjs_references_renamed() {
    // Verify no quickjs references remain in headers
    FILE *header = fopen("core/include/mtpjs.h", "r");
    char content[4096];
    fread(content, 1, sizeof(content), header);
    fclose(header);
    
    assert(strstr(content, "quickjs") == NULL);
    assert(strstr(content, "QuickJS") == NULL);
    assert(strstr(content, "QUICKJS") == NULL);
    assert(strstr(content, "MTPJS_H") != NULL);
    
    // Verify no quickjs references remain in source
    FILE *source = fopen("core/runtime/mtpjs.c", "r");
    fread(content, 1, sizeof(content), source);
    fclose(source);
    
    assert(strstr(content, "MTPJS_NewRuntime") != NULL);
    assert(strstr(content, "MTPJS_NewContext") != NULL);
    assert(strstr(content, "quickjs") == NULL);
    
    return PASS;
}
```

---

### Task R.0.5: Create MTPJS Build System
**Objective**: Create Makefile to build MTPJS from copied files
**Specification Reference**: FOLDER_STRUCTURE.md lines 158-187 (build configuration)

**Files to Create**: `Makefile`, `build/mtpjs.rules.mk`
**Build Output**: `build/artifacts/libmtpjs.a`, `build/artifacts/mtpjs_repl`, `build/artifacts/mtpsc`

**Implementation Pseudocode**:
```makefile
# Makefile (root level)
CC = gcc
CFLAGS = -O2 -Wall -Wextra -std=c11 -fPIC
LDFLAGS = -lm -ldl -lpthread

# Directories
OBJDIR = build/objects
BUILD = build/artifacts

# MTPJS Core Objects
MTPJS_CORE_OBJS = \
    $(OBJDIR)/core/runtime/mtpjs.o \
    $(OBJDIR)/core/utils/mtp_cutils.o \
    $(OBJDIR)/core/utils/mtp_dtoa.o \
    $(OBJDIR)/core/regex/mtp_libregexp.o \
    $(OBJDIR)/core/regex/mtp_libunicode.o \
    $(OBJDIR)/core/stdlib/mtpjs-libc.o

# Main program objects
MTPJS_MAIN_OBJS = \
    $(OBJDIR)/src/main/mtpjs_repl.o \
    $(MTPJS_CORE_OBJS)

# Compiler objects
MTPSC_OBJS = \
    $(OBJDIR)/src/cli/mtpsc.o \
    $(MTPJS_CORE_OBJS)

# Include build rules
include build/mtpjs.rules.mk

# Default target
all: $(BUILD)/libmtpjs.a $(BUILD)/mtpjs_repl $(BUILD)/mtpsc

# Clean
clean:
	rm -rf build

.PHONY: all clean
```

```makefile
# build/mtpjs.rules.mk
# MTPJS build rules

# Include paths
INCLUDES = -Icore/include -Icore/utils -Icore/regex -Icore/stdlib

# Compilation rule
$(OBJDIR)/%.o: %.c
	@mkdir -p $(dir $@)
	$(CC) $(CFLAGS) $(INCLUDES) -DMTPSCRIPT_BUILD -c -o $@ $<

# Static library
$(BUILD)/libmtpjs.a: $(MTPJS_CORE_OBJS)
	@mkdir -p $(BUILD)
	ar rcs $@ $^

# REPL executable
$(BUILD)/mtpjs_repl: $(MTPJS_MAIN_OBJS)
	@mkdir -p $(BUILD)
	$(CC) $(CFLAGS) -o $@ $^ $(LDFLAGS)

# Compiler executable
$(BUILD)/mtpsc: $(MTPSC_OBJS)
	@mkdir -p $(BUILD)
	$(CC) $(CFLAGS) -o $@ $^ $(LDFLAGS)
```

**Test to Create**: `tests/unit/build_test.c`
```c
int test_mtpjs_build_system() {
    // Test that make produces correct outputs
    int result = system("make clean && make");
    assert(result == 0);
    
    // Verify static library was created
    assert(file_exists("build/artifacts/libmtpjs.a"));
    
    // Verify executables were created
    assert(file_exists("build/artifacts/mtpjs_repl"));
    assert(file_exists("build/artifacts/mtpsc"));
    
    // Test basic functionality
    result = system("echo '1+1' | build/artifacts/mtpjs_repl");
    assert(result == 0);
    
    return PASS;
}
```

---

### Task R.0.6: Apply Initial MTPScript Security Patches
**Objective**: Apply basic security patches to MTPJS runtime
**Specification Reference**: TECHSPECV5.md lines 11-15 (zero ambient authority), lines 346-347 (forbidden JS)

**Files to Modify**: `core/runtime/mtpjs.c`, `core/include/mtpjs.h`
**Build Output**: Patched MTPJS with basic security constraints

**Implementation Pseudocode**:
```c
// core/include/mtpjs.h - Add MTPScript security definitions
#define MTPJS_SECURITY_PATCHES 1
#define MTPJS_FORBIDDEN_EVAL 1
#define MTPJS_FORBIDDEN_CLASS 1
#define MTPJS_FORBIDDEN_THIS 1
#define MTPJS_FORBIDDEN_GLOBAL_MUTATION 1

// Security flags
typedef struct {
    uint32_t forbidden_features;
    bool deterministic_mode;
    bool per_request_isolation;
} mtpjs_config_t;

// Security functions
int mtpjs_configure_security(MTPJSContext *ctx, mtpjs_config_t *config);
int mtpjs_check_forbidden_constructs(const char *code, size_t len);
```

```c
// core/runtime/mtpjs.c - Add security enforcement
static const char* forbidden_constructs[] = {
    "eval", "class", "this", "try", "catch", "for", "while", "do"
};

int mtpjs_check_forbidden_constructs(const char *code, size_t len) {
    for (int i = 0; i < sizeof(forbidden_constructs)/sizeof(forbidden_constructs[0]); i++) {
        if (strstr(code, forbidden_constructs[i]) != NULL) {
            return MTPJS_ERROR_FORBIDDEN_CONSTRUCT;
        }
    }
    return MTPJS_SUCCESS;
}

// Override eval function
static MTPJSValue mtpjs_eval_forbidden(MTPJSContext *ctx, 
                                        MTPJSValueConst this_val,
                                        int argc, MTPJSValueConst *argv) {
    return MTPJS_ThrowTypeError(ctx, "eval() is forbidden in MTPScript");
}

// Override class creation
static MTPJSValue mtpjs_class_forbidden(MTPJSContext *ctx,
                                       MTPJSValueConst this_val,
                                       int argc, MTPJSValueConst *argv) {
    return MTPJS_ThrowTypeError(ctx, "class is forbidden in MTPScript");
}
```

**Test to Create**: `tests/unit/security_test.c`
```c
int test_forbidden_constructs_blocked() {
    MTPJSRuntime *rt = MTPJS_NewRuntime();
    MTPJSContext *ctx = MTPJS_NewContext(rt);
    
    // Configure security
    mtpjs_config_t config = {
        .forbidden_features = MTPJS_FORBIDDEN_EVAL | MTPJS_FORBIDDEN_CLASS,
        .deterministic_mode = true,
        .per_request_isolation = true
    };
    mtpjs_configure_security(ctx, &config);
    
    // Test eval is blocked
    MTPJSValue result = MTPJS_Eval(ctx, "eval('1+1')", -1, "<test>", 0);
    assert(MTPJS_IsException(result));
    
    // Test class is blocked
    result = MTPJS_Eval(ctx, "class X {}", -1, "<test>", 0);
    assert(MTPJS_IsException(result));
    
    MTPJS_FreeContext(ctx);
    MTPJS_FreeRuntime(rt);
    return PASS;
}
```

---

## Phase R.0 Success Criteria

### Technical Success:
1. **QuickJS Source Integration**: All relevant QuickJS files copied to MTPJS structure
2. **Systematic Renaming**: All references to "quickjs" replaced with "mtpjs"
3. **Build System**: Makefile successfully builds MTPJS library and executables
4. **Basic Security**: Forbidden JS constructs (eval, class, this) are blocked
5. **Directory Structure**: Complies with FOLDER_STRUCTURE.md requirements

### Validation Success:
1. **Unit Tests Pass**: All R.0.x test cases pass
2. **Build Success**: `make` completes without errors
3. **Basic Execution**: MTPJS REPL can execute simple JavaScript expressions
4. **Security Enforcement**: Forbidden constructs throw appropriate errors
5. **File Organization**: All files in correct locations per specification

### Next Phase Readiness:
1. **Runtime Foundation**: MTPJS core ready for snapshot system integration
2. **Security Framework**: Basic patches in place for advanced security features
3. **Build Infrastructure**: Ready for gas metering and effect system integration
4. **Test Infrastructure**: Unit test framework established for runtime testing

---

## Implementation Commands

### Execute Phase R.0:
```bash
# 1. Clone QuickJS
./scripts/clone_quickjs.sh

# 2. Create directory structure
./scripts/create_mtpjs_structure.sh

# 3. Copy files
./scripts/copy_quickjs_files.sh

# 4. Rename references
./scripts/rename_to_mtpjs.sh

# 5. Test build
make clean && make

# 6. Run tests
./tests/unit/test_runner
```

This phase establishes the foundation for the entire MTPScript runtime by integrating QuickJS into our project structure and applying initial security constraints. All subsequent runtime features (gas metering, snapshot system, effect system) will build upon this MTPJS foundation.