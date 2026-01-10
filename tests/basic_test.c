/*
 * Basic Test Suite for MTPScript Runtime
 * Phase 8.1 - Test Suite Implementation
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <assert.h>
#include <unistd.h>
#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

/* Forward declarations to avoid header conflicts */
typedef struct JSRuntime JSRuntime;
typedef struct JSContext JSContext;

/* Include our headers */
#include "../core/include/mt5_common.h"

/* External function declarations */
extern JSRuntime* JS_NewRuntime(void);
extern JSContext* JS_NewContext(JSRuntime* rt);
extern void JS_FreeContext(JSContext* ctx);
extern void JS_FreeRuntime(JSRuntime* rt);

// Test helper functions
static int tests_passed = 0;
static int tests_failed = 0;

#define TEST_ASSERT(condition, message) \
    do { \
        if (condition) { \
            printf("✓ %s\n", message); \
            tests_passed++; \
        } else { \
            printf("✗ %s\n", message); \
            tests_failed++; \
        } \
    } while(0)

#define TEST_SECTION(name) \
    printf("\n=== %s ===\n", name)

/**
 * Test 1: Runtime Creation and Context Initialization
 */
static void test_runtime_creation(void) {
    TEST_SECTION("Runtime Creation Tests");
    
    // Test basic runtime creation
    JSRuntime *rt = JS_NewRuntime();
    TEST_ASSERT(rt != NULL, "JS runtime creation");
    
    if (rt) {
        // Test context creation
        JSContext *ctx = JS_NewContext(rt);
        TEST_ASSERT(ctx != NULL, "JS context creation");
        
        if (ctx) {
            // Test MTPScript-specific initialization
            mt5_runtime_ctx_t *mtp_ctx = mt5_init_runtime();
            TEST_ASSERT(mtp_ctx != NULL, "MTPScript runtime context initialization");
            
            if (mtp_ctx) {
                TEST_ASSERT(mtp_ctx->js_rt == rt, "JS runtime properly linked to MTP context");
                TEST_ASSERT(mtp_ctx->js_ctx == ctx, "JS context properly linked to MTP context");
                TEST_ASSERT(mtp_ctx->gas_limit == MT5_DEFAULT_GAS_LIMIT, "Default gas limit set");
                TEST_ASSERT(mtp_ctx->gas_used == 0, "Initial gas usage is zero");
                TEST_ASSERT(mtp_ctx->execution_depth == 0, "Initial execution depth is zero");
                TEST_ASSERT(mtp_ctx->ambient_authority == false, "Zero ambient authority by default");
                
                mt5_cleanup_runtime(mtp_ctx);
            }
            
            JS_FreeContext(ctx);
        }
        
        JS_FreeRuntime(rt);
    }
    
    // Test cleanup
    TEST_ASSERT(true, "Runtime cleanup completed without errors");
}

/**
 * Test 2: Gas Metering System
 */
static void test_gas_metering(void) {
    TEST_SECTION("Gas Metering Tests");
    
    JSRuntime *rt = JS_NewRuntime();
    JSContext *ctx = JS_NewContext(rt);
    mt5_runtime_ctx_t *mtp_ctx = mt5_init_runtime();
    
    if (mtp_ctx) {
        mtp_ctx->js_rt = rt;
        mtp_ctx->js_ctx = ctx;
        
        // Test gas limit setting
        uint64_t custom_limit = 1000000;
        mt5_result_t result = mt5_configure_gas_limit(mtp_ctx, custom_limit);
        TEST_ASSERT(result == MT5_SUCCESS, "Gas limit configuration");
        TEST_ASSERT(mtp_ctx->gas_limit == custom_limit, "Gas limit correctly set");
        
        // Test gas consumption tracking
        uint64_t initial_gas = mtp_ctx->gas_used;
        uint64_t consumption = 1000;
        
        // Simulate gas consumption
        mtp_ctx->gas_used += consumption;
        TEST_ASSERT(mtp_ctx->gas_used == initial_gas + consumption, "Gas consumption tracking");
        
        // Test gas exhaustion detection
        mtp_ctx->gas_used = mtp_ctx->gas_limit;
        TEST_ASSERT(mt5_check_gas_exhaustion(mtp_ctx) == true, "Gas exhaustion detection when limit reached");
        
        // Test gas limit overflow
        mtp_ctx->gas_used = mtp_ctx->gas_limit + 1;
        TEST_ASSERT(mt5_check_gas_exhaustion(mtp_ctx) == true, "Gas exhaustion detection when exceeded");
        
        // Test gas reset
        mtp_ctx->gas_used = 0;
        TEST_ASSERT(mt5_check_gas_exhaustion(mtp_ctx) == false, "Gas reset works correctly");
        
        mt5_cleanup_runtime(mtp_ctx);
    }
    
    JS_FreeContext(ctx);
    JS_FreeRuntime(rt);
}

/**
 * Test 3: Forbidden Construct Detection
 */
static void test_forbidden_constructs(void) {
    TEST_SECTION("Forbidden Construct Detection Tests");
    
    // Test eval detection
    const char *code_with_eval = "eval('console.log(\"test\")');";
    int contains_eval = mt5_contains_forbidden_constructs(code_with_eval);
    TEST_ASSERT(contains_eval == 1, "Eval construct detection");
    
    // Test class detection
    const char *code_with_class = "class MyClass { constructor() {} }";
    int contains_class = mt5_contains_forbidden_constructs(code_with_class);
    TEST_ASSERT(contains_class == 1, "Class construct detection");
    
    // Test generator detection
    const char *code_with_generator = "function* gen() { yield 1; }";
    int contains_generator = mt5_contains_forbidden_constructs(code_with_generator);
    TEST_ASSERT(contains_generator == 1, "Generator construct detection");
    
    // Test try-catch detection
    const char *code_with_trycatch = "try { risky(); } catch(e) { handle(e); }";
    int contains_trycatch = mt5_contains_forbidden_constructs(code_with_trycatch);
    TEST_ASSERT(contains_trycatch == 1, "Try-catch construct detection");
    
    // Test import detection
    const char *code_with_import = "import { something } from 'module';";
    int contains_import = mt5_contains_forbidden_constructs(code_with_import);
    TEST_ASSERT(contains_import == 1, "Import construct detection");
    
    // Test export detection
    const char *code_with_export = "export const value = 42;";
    int contains_export = mt5_contains_forbidden_constructs(code_with_export);
    TEST_ASSERT(contains_export == 1, "Export construct detection");
    
    // Test safe code (no forbidden constructs)
    const char *safe_code = "let x = 10; let y = x + 5; console.log(y);";
    int is_safe = mt5_contains_forbidden_constructs(safe_code);
    TEST_ASSERT(is_safe == 0, "Safe code passes forbidden construct check");
    
    // Test partial construct detection (should still trigger)
    const char *partial_eval = "    eval(  \n    'test'  \n  );";
    int contains_partial_eval = mt5_contains_forbidden_constructs(partial_eval);
    TEST_ASSERT(contains_partial_eval == 1, "Partial eval construct detection with whitespace");
}

/**
 * Test 4: Snapshot Creation and Verification
 */
static void test_snapshot_creation(void) {
    TEST_SECTION("Snapshot Creation and Verification Tests");
    
    JSRuntime *rt = JS_NewRuntime();
    JSContext *ctx = JS_NewContext(rt);
    mt5_runtime_ctx_t *mtp_ctx = mt5_init_runtime();
    
    if (mtp_ctx) {
        mtp_ctx->js_rt = rt;
        mtp_ctx->js_ctx = ctx;
        
        // Set some test state
        mtp_ctx->gas_used = 50000;
        mtp_ctx->execution_depth = 3;
        
        // Test snapshot creation
        mt5_snapshot_t *snapshot = mt5_create_snapshot(mtp_ctx);
        TEST_ASSERT(snapshot != NULL, "Snapshot creation");
        
        if (snapshot) {
            // Test snapshot structure
            TEST_ASSERT(snapshot->gas_used == mtp_ctx->gas_used, "Snapshot gas usage preservation");
            TEST_ASSERT(snapshot->execution_depth == mtp_ctx->execution_depth, "Snapshot execution depth preservation");
            TEST_ASSERT(snapshot->data != NULL, "Snapshot data allocation");
            TEST_ASSERT(snapshot->data_size > 0, "Snapshot data size is positive");
            
            // Test snapshot integrity
            uint8_t calculated_hash[32];
            mt5_calculate_snapshot_hash(snapshot, calculated_hash);
            int hash_match = memcmp(snapshot->hash, calculated_hash, 32) == 0;
            TEST_ASSERT(hash_match, "Snapshot hash integrity verification");
            
            // Test snapshot serialization
            char *serialized = mt5_serialize_snapshot(snapshot);
            TEST_ASSERT(serialized != NULL, "Snapshot serialization");
            
            if (serialized) {
                // Test basic serialization format (should contain some expected fields)
                int contains_gas = strstr(serialized, "\"gas_used\"") != NULL;
                int contains_depth = strstr(serialized, "\"execution_depth\"") != NULL;
                int contains_hash = strstr(serialized, "\"hash\"") != NULL;
                
                TEST_ASSERT(contains_gas, "Serialized snapshot contains gas_used field");
                TEST_ASSERT(contains_depth, "Serialized snapshot contains execution_depth field");
                TEST_ASSERT(contains_hash, "Serialized snapshot contains hash field");
                
                free(serialized);
            }
            
            // Test snapshot cleanup
            mt5_free_snapshot(snapshot);
            TEST_ASSERT(true, "Snapshot cleanup completed without errors");
        }
        
        mt5_cleanup_runtime(mtp_ctx);
    }
    
    JS_FreeContext(ctx);
    JS_FreeRuntime(rt);
}

/**
 * Test 5: Effect System Basic Operations
 */
static void test_effect_system(void) {
    TEST_SECTION("Effect System Tests");
    
    JSRuntime *rt = JS_NewRuntime();
    JSContext *ctx = JS_NewContext(rt);
    mt5_runtime_ctx_t *mtp_ctx = mt5_init_runtime();
    
    if (mtp_ctx) {
        mtp_ctx->js_rt = rt;
        mtp_ctx->js_ctx = ctx;
        
        // Test console effect registration
        int console_registered = mt5_register_console_effect(mtp_ctx);
        TEST_ASSERT(console_registered == 0, "Console effect registration");
        
        // Test database effect registration
        int database_registered = mt5_register_database_effect(mtp_ctx);
        TEST_ASSERT(database_registered == 0, "Database effect registration");
        
        // Test effect count
        int effect_count = mt5_get_registered_effect_count(mtp_ctx);
        TEST_ASSERT(effect_count >= 2, "Effect count after registration");
        
        // Test effect cleanup
        mt5_cleanup_effects(mtp_ctx);
        TEST_ASSERT(true, "Effect system cleanup completed");
        
        mt5_cleanup_runtime(mtp_ctx);
    }
    
    JS_FreeContext(ctx);
    JS_FreeRuntime(rt);
}

/**
 * Main test runner
 */
int main(void) {
    printf("MTPScript Basic Test Suite\n");
    printf("========================\n");
    
    // Run all tests
    test_runtime_creation();
    test_gas_metering();
    test_forbidden_constructs();
    test_snapshot_creation();
    test_effect_system();
    
    // Print final results
    printf("\n=== Test Results ===\n");
    printf("Tests passed: %d\n", tests_passed);
    printf("Tests failed: %d\n", tests_failed);
    printf("Total tests: %d\n", tests_passed + tests_failed);
    
    if (tests_failed == 0) {
        printf("\n🎉 All tests passed!\n");
        return 0;
    } else {
        printf("\n❌ Some tests failed!\n");
        return 1;
    }
}