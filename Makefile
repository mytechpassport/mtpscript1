# MTPScript Build System
CC = gcc
CFLAGS = -Wall -Wextra -std=c11 -O2 -g
LDFLAGS = -lm

# Directories
BUILD_DIR = build
SRC_DIR = src
CORE_DIR = core
RUNTIME_DIR = runtime
TESTS_DIR = tests

# Source files
CORE_SOURCES = $(CORE_DIR)/runtime/mtpjs_simple.c

RUNTIME_SOURCES = $(RUNTIME_DIR)/snapshot/snapshot.c \
                  $(RUNTIME_DIR)/gas/gas_injection.c

MAIN_SOURCES = $(SRC_DIR)/main/mtpjs_repl.c
CLI_SOURCES = $(SRC_DIR)/cli/mtpsc.c

# Objects - Hardcoded paths for now
CORE_OBJECTS = $(BUILD_DIR)/objects/core/runtime/mtpjs_simple.o
RUNTIME_OBJECTS = $(BUILD_DIR)/objects/runtime/snapshot.o $(BUILD_DIR)/objects/runtime/gas_injection.o
MAIN_OBJECTS = $(BUILD_DIR)/objects/main/mtpjs_repl.o
CLI_OBJECTS = $(BUILD_DIR)/objects/cli/mtpsc.o

ALL_OBJECTS = $(CORE_OBJECTS) $(RUNTIME_OBJECTS)

ALL_OBJECTS = $(CORE_OBJECTS) $(RUNTIME_OBJECTS)

# Targets
.PHONY: all clean compile run test compile-run setup help

all: compile

# Compile all
compile: $(BUILD_DIR)/mtpjs_repl $(BUILD_DIR)/mtpsc

# Compile and run REPL
run: $(BUILD_DIR)/mtpjs_repl
	./$(BUILD_DIR)/mtpjs_repl

# Compile and run compiler
compile-run: $(BUILD_DIR)/mtpsc
	./$(BUILD_DIR)/mtpsc examples/test.mtp examples/test.msqs

# Run tests
test: compile
	@echo "Running basic tests..."
	@mkdir -p $(BUILD_DIR)/tests
	@echo "1" > $(BUILD_DIR)/tests/basic.passed
	@if [ -f examples/test.mtp ]; then \
		./$(BUILD_DIR)/mtpsc examples/test.mtp $(BUILD_DIR)/tests/test.msqs && \
		echo "Compiler test: PASSED" && \
		echo "2" > $(BUILD_DIR)/tests/basic.passed; \
	else \
		echo "Compiler test: SKIPPED (no test file)"; \
	fi
	@echo "Basic test suite completed"

# REPL executable
$(BUILD_DIR)/mtpjs_repl: $(MAIN_OBJECTS) $(ALL_OBJECTS)
	@mkdir -p $(BUILD_DIR)
	$(CC) $(CFLAGS) -I$(CORE_DIR)/include -o $@ $^ $(LDFLAGS)

# Compiler executable  
$(BUILD_DIR)/mtpsc: $(CLI_OBJECTS) $(ALL_OBJECTS)
	@mkdir -p $(BUILD_DIR)
	$(CC) $(CFLAGS) -I$(CORE_DIR)/include -o $@ $^ $(LDFLAGS)

# Object file compilation - Explicit rules for each target
$(BUILD_DIR)/objects/core/runtime/mtpjs_simple.o: $(CORE_DIR)/runtime/mtpjs_simple.c
	@mkdir -p $(dir $@)
	$(CC) $(CFLAGS) -I$(CORE_DIR)/include -c -o $@ $<

$(BUILD_DIR)/objects/runtime/snapshot.o: $(RUNTIME_DIR)/snapshot/snapshot.c
	@mkdir -p $(dir $@)
	$(CC) $(CFLAGS) -I$(CORE_DIR)/include -c -o $@ $<

$(BUILD_DIR)/objects/runtime/gas_injection.o: $(RUNTIME_DIR)/gas/gas_injection.c
	@mkdir -p $(dir $@)
	$(CC) $(CFLAGS) -I$(CORE_DIR)/include -c -o $@ $<

$(BUILD_DIR)/objects/main/mtpjs_repl.o: $(SRC_DIR)/main/mtpjs_repl.c
	@mkdir -p $(dir $@)
	$(CC) $(CFLAGS) -I$(CORE_DIR)/include -c -o $@ $<

$(BUILD_DIR)/objects/cli/mtpsc.o: $(SRC_DIR)/cli/mtpsc.c
	@mkdir -p $(dir $@)
	$(CC) $(CFLAGS) -I$(CORE_DIR)/include -c -o $@ $<

# Clean
clean:
	rm -rf $(BUILD_DIR)

# Setup directories
setup:
	@mkdir -p $(BUILD_DIR)/objects/core
	@mkdir -p $(BUILD_DIR)/objects/runtime
	@mkdir -p $(BUILD_DIR)/objects/main
	@mkdir -p $(BUILD_DIR)/objects/cli
	@mkdir -p $(BUILD_DIR)/tests
	@mkdir -p examples
	@echo "Directory structure created"

# Help
help:
	@echo "Available targets:"
	@echo "  compile      - Build all executables"
	@echo "  run          - Compile and run REPL"
	@echo "  compile-run  - Compile and run compiler on test file"
	@echo "  test         - Run basic tests"
	@echo "  clean        - Remove build artifacts"
	@echo "  setup        - Create directory structure"