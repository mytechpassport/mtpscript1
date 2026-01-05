/**
 * MTPScript Complex Model Test
 * Tests complex data structures with all primitive types and nested variables
 *
 * This test validates:
 * - Compilation of complex nested types
 * - All primitive types (number, string, boolean, Decimal)
 * - Complex object structures
 * - Array types with complex elements
 * - Union types (UserStatus)
 * - Effect usage (Log)
 * - API endpoints with complex return types
 *
 * Copyright (c) 2025 My Tech Passport Inc.
 * Author: Ryan Wong
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>

#define ASSERT(expr, msg) \
    if (!(expr)) { \
        printf("FAIL: %s\n", msg); \
        return false; \
    }

// Test that the complex model fixture compiles successfully
bool test_complex_model_compilation() {
    printf("Testing complex model compilation...\n");

    // Try to compile the complex model fixture
    int compile_result = system("../../mtpsc compile ../fixtures/complex_model.mtp 2>/dev/null");
    if (compile_result != 0) {
        printf("  Complex model compilation failed\n");
        return false;
    }

    printf("  Complex model compiled successfully\n");
    return true;
}

// Test that the complex model can be type-checked
bool test_complex_model_type_check() {
    printf("Testing complex model type checking...\n");

    // Try to type-check the complex model fixture
    int check_result = system("../../mtpsc check ../fixtures/complex_model.mtp 2>/dev/null");
    if (check_result != 0) {
        printf("  Complex model type check failed\n");
        return false;
    }

    printf("  Complex model type check passed\n");
    return true;
}

// Test compilation of individual functions from the complex model
bool test_individual_function_compilation() {
    printf("Testing individual function compilation...\n");

    // Test createPrimitiveShowcase function with deep nesting
    const char *primitive_test =
        "function createPrimitiveShowcase(): {\n"
        "    integer_num: number,\n"
        "    decimal_num: Decimal,\n"
        "    text: string,\n"
        "    flag: boolean,\n"
        "    nested: {\n"
        "        inner_text: string,\n"
        "        inner_number: number,\n"
        "        deep_nested: {\n"
        "            deep_text: string,\n"
        "            deep_decimal: Decimal,\n"
        "            deep_bool: boolean\n"
        "        }\n"
        "    },\n"
        "    array_of_numbers: [number],\n"
        "    array_of_strings: [string],\n"
        "    array_of_objects: [{ key: string, value: number }]\n"
        "} {\n"
        "    return {\n"
        "        integer_num: 42,\n"
        "        decimal_num: 3.14159265359,\n"
        "        text: \"Hello complex world\",\n"
        "        flag: true,\n"
        "        nested: {\n"
        "            inner_text: \"nested content\",\n"
        "            inner_number: 99,\n"
        "            deep_nested: {\n"
        "                deep_text: \"very deep content\",\n"
        "                deep_decimal: 2.71828182846,\n"
        "                deep_bool: false\n"
        "            }\n"
        "        },\n"
        "        array_of_numbers: [1, 2, 3, 4, 5],\n"
        "        array_of_strings: [\"alpha\", \"beta\", \"gamma\"],\n"
        "        array_of_objects: [\n"
        "            { key: \"first\", value: 100 },\n"
        "            { key: \"second\", value: 200 },\n"
        "            { key: \"third\", value: 300 }\n"
        "        ]\n"
        "    }\n"
        "}\n";

    FILE *f = fopen("primitive_test.mtp", "w");
    if (!f) return false;
    fprintf(f, "%s", primitive_test);
    fclose(f);

    // Compile and capture output
    int compile_result = system("../../mtpsc compile primitive_test.mtp > primitive_output.js 2>/dev/null");
    if (compile_result != 0) {
        printf("  Primitive showcase function compilation failed\n");
        remove("primitive_test.mtp");
        return false;
    }

    // Read and validate compiled output
    FILE *out_f = fopen("primitive_output.js", "r");
    if (!out_f) {
        printf("  Could not read compiled output\n");
        remove("primitive_test.mtp");
        return false;
    }

    char buffer[2048] = {0};
    size_t bytes_read = fread(buffer, 1, sizeof(buffer) - 1, out_f);
    buffer[bytes_read] = '\0';
    fclose(out_f);

    // Check that output contains expected elements including deep nesting
    bool has_function_name = strstr(buffer, "createPrimitiveShowcase") != NULL;
    bool has_integer_num = strstr(buffer, "integer_num") != NULL;
    bool has_decimal_num = strstr(buffer, "decimal_num") != NULL;
    bool has_text = strstr(buffer, "text") != NULL;
    bool has_flag = strstr(buffer, "flag") != NULL;
    bool has_nested = strstr(buffer, "nested") != NULL;
    bool has_inner_text = strstr(buffer, "inner_text") != NULL;
    bool has_inner_number = strstr(buffer, "inner_number") != NULL;
    bool has_deep_nested = strstr(buffer, "deep_nested") != NULL;
    bool has_deep_text = strstr(buffer, "deep_text") != NULL;
    bool has_deep_decimal = strstr(buffer, "deep_decimal") != NULL;
    bool has_deep_bool = strstr(buffer, "deep_bool") != NULL;
    bool has_array_of_numbers = strstr(buffer, "array_of_numbers") != NULL;
    bool has_array_of_strings = strstr(buffer, "array_of_strings") != NULL;
    bool has_array_of_objects = strstr(buffer, "array_of_objects") != NULL;

    remove("primitive_test.mtp");
    remove("primitive_output.js");

    if (!has_function_name || !has_integer_num || !has_decimal_num || !has_text ||
        !has_flag || !has_nested || !has_inner_text || !has_inner_number ||
        !has_deep_nested || !has_deep_text || !has_deep_decimal || !has_deep_bool ||
        !has_array_of_numbers || !has_array_of_strings || !has_array_of_objects) {
        printf("  Compiled output missing expected content\n");
        printf("  Expected: function name, all primitive types, nested structures, deep nesting, arrays\n");
        printf("  Found: function_name=%d, integer_num=%d, decimal_num=%d, text=%d, flag=%d\n",
               has_function_name, has_integer_num, has_decimal_num, has_text, has_flag);
        printf("        nested=%d, inner_text=%d, inner_number=%d, deep_nested=%d, deep_text=%d\n",
               has_nested, has_inner_text, has_inner_number, has_deep_nested, has_deep_text);
        printf("        deep_decimal=%d, deep_bool=%d, array_numbers=%d, array_strings=%d, array_objects=%d\n",
               has_deep_decimal, has_deep_bool, has_array_of_numbers, has_array_of_strings, has_array_of_objects);
        return false;
    }

    printf("  Primitive showcase function compiled successfully with expected output\n");
    return true;
}

// Test union type compilation and pattern matching
bool test_union_type_compilation() {
    printf("Testing union type compilation...\n");

    const char *union_test =
        "type UserStatus = | Active | Inactive | Suspended\n"
        "function process_status(status: UserStatus): string {\n"
        "    match status {\n"
        "        Active => \"User is active\"\n"
        "        Inactive => \"User is inactive\"\n"
        "        Suspended => \"User is suspended\"\n"
        "    }\n"
        "}\n";

    FILE *f = fopen("union_test.mtp", "w");
    if (!f) return false;
    fprintf(f, "%s", union_test);
    fclose(f);

    // Compile and capture output
    int compile_result = system("../../mtpsc compile union_test.mtp > union_output.js 2>/dev/null");
    if (compile_result != 0) {
        printf("  Union type compilation failed\n");
        remove("union_test.mtp");
        return false;
    }

    // Read and validate compiled output
    FILE *out_f = fopen("union_output.js", "r");
    if (!out_f) {
        printf("  Could not read compiled union output\n");
        remove("union_test.mtp");
        return false;
    }

    char buffer[2048] = {0};
    size_t bytes_read = fread(buffer, 1, sizeof(buffer) - 1, out_f);
    buffer[bytes_read] = '\0';
    fclose(out_f);

    // Check that output contains expected elements
    bool has_function_name = strstr(buffer, "process_status") != NULL;
    bool has_active_case = strstr(buffer, "Active") != NULL;
    bool has_inactive_case = strstr(buffer, "Inactive") != NULL;
    bool has_suspended_case = strstr(buffer, "Suspended") != NULL;

    remove("union_test.mtp");
    remove("union_output.js");

    if (!has_function_name || !has_active_case || !has_inactive_case || !has_suspended_case) {
        printf("  Compiled union output missing expected content\n");
        printf("  Expected: function name, Active, Inactive, Suspended cases\n");
        printf("  Found: function_name=%d, active=%d, inactive=%d, suspended=%d\n",
               has_function_name, has_active_case, has_inactive_case, has_suspended_case);
        return false;
    }

    printf("  Union type compiled successfully with expected output\n");
    return true;
}

// Test effect usage compilation
bool test_effect_usage_compilation() {
    printf("Testing effect usage compilation...\n");

    const char *effect_test =
        "uses { Log }\n"
        "function log_message(msg: string): string {\n"
        "    log(\"info\", \"Message: \" + msg)\n"
        "    return \"Logged: \" + msg\n"
        "}\n";

    FILE *f = fopen("effect_test.mtp", "w");
    if (!f) return false;
    fprintf(f, "%s", effect_test);
    fclose(f);

    // Compile and capture output
    int compile_result = system("../../mtpsc compile effect_test.mtp > effect_output.js 2>/dev/null");
    if (compile_result != 0) {
        printf("  Effect usage compilation failed\n");
        remove("effect_test.mtp");
        return false;
    }

    // Read and validate compiled output
    FILE *out_f = fopen("effect_output.js", "r");
    if (!out_f) {
        printf("  Could not read compiled effect output\n");
        remove("effect_test.mtp");
        return false;
    }

    char buffer[2048] = {0};
    size_t bytes_read = fread(buffer, 1, sizeof(buffer) - 1, out_f);
    buffer[bytes_read] = '\0';
    fclose(out_f);

    // Check that output contains expected elements
    bool has_function_name = strstr(buffer, "log_message") != NULL;
    bool has_log_call = strstr(buffer, "log") != NULL;

    remove("effect_test.mtp");
    remove("effect_output.js");

    if (!has_function_name || !has_log_call) {
        printf("  Compiled effect output missing expected content\n");
        printf("  Expected: function name, log call\n");
        printf("  Found: function_name=%d, log_call=%d\n", has_function_name, has_log_call);
        return false;
    }

    printf("  Effect usage compiled successfully with expected output\n");
    return true;
}

// Test API endpoint compilation with complex return types
bool test_api_endpoint_compilation() {
    printf("Testing API endpoint compilation...\n");

    const char *api_test =
        "type ComplexResponse = {\n"
        "    id: number,\n"
        "    data: {\n"
        "        name: string,\n"
        "        values: [number]\n"
        "    },\n"
        "    success: boolean\n"
        "}\n"
        "api GET \"/complex\" function get_complex(): ComplexResponse {\n"
        "    return {\n"
        "        id: 123,\n"
        "        data: {\n"
        "            name: \"test\",\n"
        "            values: [1, 2, 3]\n"
        "        },\n"
        "        success: true\n"
        "    }\n"
        "}\n";

    FILE *f = fopen("api_test.mtp", "w");
    if (!f) return false;
    fprintf(f, "%s", api_test);
    fclose(f);

    // Compile and capture output
    int compile_result = system("../../mtpsc compile api_test.mtp > api_output.js 2>/dev/null");
    if (compile_result != 0) {
        printf("  API endpoint compilation failed\n");
        remove("api_test.mtp");
        return false;
    }

    // Read and validate compiled output
    FILE *out_f = fopen("api_output.js", "r");
    if (!out_f) {
        printf("  Could not read compiled API output\n");
        remove("api_test.mtp");
        return false;
    }

    char buffer[2048] = {0};
    size_t bytes_read = fread(buffer, 1, sizeof(buffer) - 1, out_f);
    buffer[bytes_read] = '\0';
    fclose(out_f);

    // Check that output contains expected elements
    bool has_function_name = strstr(buffer, "get_complex") != NULL;
    bool has_api_structure = strstr(buffer, "id") != NULL && strstr(buffer, "data") != NULL;

    remove("api_test.mtp");
    remove("api_output.js");

    if (!has_function_name || !has_api_structure) {
        printf("  Compiled API output missing expected content\n");
        printf("  Expected: function name, API structure (id, data)\n");
        printf("  Found: function_name=%d, api_structure=%d\n", has_function_name, has_api_structure);
        return false;
    }

    printf("  API endpoint compiled successfully with expected output\n");
    return true;
}

// Test Decimal type usage
bool test_decimal_type_usage() {
    printf("Testing Decimal type usage...\n");

    const char *decimal_test =
        "function decimal_operations(): {\n"
        "    result: Decimal,\n"
        "    description: string\n"
        "} {\n"
        "    const pi = 3.14159265359\n"
        "    const e = 2.71828182846\n"
        "    const result = pi + e\n"
        "    return {\n"
        "        result: result,\n"
        "        description: \"Pi + E calculation\"\n"
        "    }\n"
        "}\n";

    FILE *f = fopen("decimal_test.mtp", "w");
    if (!f) return false;
    fprintf(f, "%s", decimal_test);
    fclose(f);

    // Compile and capture output
    int compile_result = system("../../mtpsc compile decimal_test.mtp > decimal_output.js 2>/dev/null");
    if (compile_result != 0) {
        printf("  Decimal type usage compilation failed\n");
        remove("decimal_test.mtp");
        return false;
    }

    // Read and validate compiled output
    FILE *out_f = fopen("decimal_output.js", "r");
    if (!out_f) {
        printf("  Could not read compiled decimal output\n");
        remove("decimal_test.mtp");
        return false;
    }

    char buffer[2048] = {0};
    size_t bytes_read = fread(buffer, 1, sizeof(buffer) - 1, out_f);
    buffer[bytes_read] = '\0';
    fclose(out_f);

    // Check that output contains expected elements
    bool has_function_name = strstr(buffer, "decimal_operations") != NULL;
    bool has_result_field = strstr(buffer, "result") != NULL;
    bool has_description_field = strstr(buffer, "description") != NULL;

    remove("decimal_test.mtp");
    remove("decimal_output.js");

    if (!has_function_name || !has_result_field || !has_description_field) {
        printf("  Compiled decimal output missing expected content\n");
        printf("  Expected: function name, result field, description field\n");
        printf("  Found: function_name=%d, result=%d, description=%d\n",
               has_function_name, has_result_field, has_description_field);
        return false;
    }

    printf("  Decimal type usage compiled successfully with expected output\n");
    return true;
}

// Test deeply nested structure compilation
bool test_deeply_nested_structure() {
    printf("Testing deeply nested structure compilation...\n");

    const char *nested_test =
        "type DeepStructure = {\n"
        "    level1: {\n"
        "        level2: {\n"
        "            level3: {\n"
        "                level4: {\n"
        "                    level5: {\n"
        "                        value: string,\n"
        "                        number: number,\n"
        "                        flag: boolean\n"
        "                    }\n"
        "                }\n"
        "            }\n"
        "        }\n"
        "    }\n"
        "}\n"
        "function create_deep_structure(): DeepStructure {\n"
        "    return {\n"
        "        level1: {\n"
        "            level2: {\n"
        "                level3: {\n"
        "                    level4: {\n"
        "                        level5: {\n"
        "                            value: \"deep value\",\n"
        "                            number: 42,\n"
        "                            flag: true\n"
        "                        }\n"
        "                    }\n"
        "                }\n"
        "            }\n"
        "        }\n"
        "    }\n"
        "}\n";

    FILE *f = fopen("nested_test.mtp", "w");
    if (!f) return false;
    fprintf(f, "%s", nested_test);
    fclose(f);

    // Compile and capture output
    int compile_result = system("../../mtpsc compile nested_test.mtp > nested_output.js 2>/dev/null");
    if (compile_result != 0) {
        printf("  Deeply nested structure compilation failed\n");
        remove("nested_test.mtp");
        return false;
    }

    // Read and validate compiled output
    FILE *out_f = fopen("nested_output.js", "r");
    if (!out_f) {
        printf("  Could not read compiled nested output\n");
        remove("nested_test.mtp");
        return false;
    }

    char buffer[2048] = {0};
    size_t bytes_read = fread(buffer, 1, sizeof(buffer) - 1, out_f);
    buffer[bytes_read] = '\0';
    fclose(out_f);

    // Check that output contains expected elements
    bool has_function_name = strstr(buffer, "create_deep_structure") != NULL;
    bool has_level1 = strstr(buffer, "level1") != NULL;
    bool has_level2 = strstr(buffer, "level2") != NULL;
    bool has_level3 = strstr(buffer, "level3") != NULL;
    bool has_value_field = strstr(buffer, "value") != NULL;

    remove("nested_test.mtp");
    remove("nested_output.js");

    if (!has_function_name || !has_level1 || !has_level2 || !has_level3 || !has_value_field) {
        printf("  Compiled nested output missing expected content\n");
        printf("  Expected: function name, level1, level2, level3, value field\n");
        printf("  Found: function_name=%d, level1=%d, level2=%d, level3=%d, value=%d\n",
               has_function_name, has_level1, has_level2, has_level3, has_value_field);
        return false;
    }

    printf("  Deeply nested structure compiled successfully with expected output\n");
    return true;
}

// Test array operations with complex types
bool test_array_operations() {
    printf("Testing array operations with complex types...\n");

    const char *array_test =
        "type Person = { name: string, age: number }\n"
        "function process_people(people: [Person]): { count: number, names: [string] } {\n"
        "    const count = length(people)\n"
        "    const names = map(people, |p| p.name)\n"
        "    return { count: count, names: names }\n"
        "}\n";

    FILE *f = fopen("array_test.mtp", "w");
    if (!f) return false;
    fprintf(f, "%s", array_test);
    fclose(f);

    // Compile and capture output
    int compile_result = system("../../mtpsc compile array_test.mtp > array_output.js 2>/dev/null");
    if (compile_result != 0) {
        printf("  Array operations compilation failed\n");
        remove("array_test.mtp");
        return false;
    }

    // Read and validate compiled output
    FILE *out_f = fopen("array_output.js", "r");
    if (!out_f) {
        printf("  Could not read compiled array output\n");
        remove("array_test.mtp");
        return false;
    }

    char buffer[2048] = {0};
    size_t bytes_read = fread(buffer, 1, sizeof(buffer) - 1, out_f);
    buffer[bytes_read] = '\0';
    fclose(out_f);

    // Check that output contains expected elements
    bool has_function_name = strstr(buffer, "process_people") != NULL;
    bool has_count_field = strstr(buffer, "count") != NULL;
    bool has_names_field = strstr(buffer, "names") != NULL;

    remove("array_test.mtp");
    remove("array_output.js");

    if (!has_function_name || !has_count_field || !has_names_field) {
        printf("  Compiled array output missing expected content\n");
        printf("  Expected: function name, count field, names field\n");
        printf("  Found: function_name=%d, count=%d, names=%d\n",
               has_function_name, has_count_field, has_names_field);
        return false;
    }

    printf("  Array operations compiled successfully with expected output\n");
    return true;
}

// Test that the complex fixture compiles with mtpsc
bool test_complex_fixture_compilation() {
    printf("Testing complex fixture compilation...\n");

    // Test that our fixture compiles
    int result = system("../../mtpsc compile ../fixtures/complex_model.mtp > /dev/null 2>&1");
    if (result != 0) {
        printf("  Complex fixture compilation failed\n");
        return false;
    }

    printf("  Complex fixture compiled successfully\n");
    return true;
}

// Test that the complex fixture passes type checking
bool test_complex_fixture_type_check() {
    printf("Testing complex fixture type checking...\n");

    // Test that our fixture passes type checking
    int result = system("../../mtpsc check ../fixtures/complex_model.mtp > /dev/null 2>&1");
    if (result != 0) {
        printf("  Complex fixture type check failed\n");
        return false;
    }

    printf("  Complex fixture type check passed\n");
    return true;
}

// Test that primitive types are handled correctly
bool test_primitive_types() {
    printf("Testing primitive types handling...\n");

    const char *primitive_test =
        "function testPrimitives(): {\n"
        "    num: number,\n"
        "    dec: Decimal,\n"
        "    str: string,\n"
        "    bool: boolean\n"
        "} {\n"
        "    return {\n"
        "        num: 42,\n"
        "        dec: 3.14,\n"
        "        str: \"hello\",\n"
        "        bool: true\n"
        "    }\n"
        "}\n";

    FILE *f = fopen("primitive_test.mtp", "w");
    if (!f) return false;
    fprintf(f, "%s", primitive_test);
    fclose(f);

    int compile_result = system("../../mtpsc primitive_test.mtp > /dev/null 2>&1");
    remove("primitive_test.mtp");

    if (compile_result != 0) {
        printf("  Primitive types test failed\n");
        return false;
    }

    printf("  Primitive types handled correctly\n");
    return true;
}

// Test that nested structures work
bool test_nested_structures() {
    printf("Testing nested structures...\n");

    const char *nested_test =
        "function testNested(): {\n"
        "    outer: {\n"
        "        inner: {\n"
        "            value: number\n"
        "        }\n"
        "    }\n"
        "} {\n"
        "    return {\n"
        "        outer: {\n"
        "            inner: {\n"
        "                value: 123\n"
        "            }\n"
        "        }\n"
        "    }\n"
        "}\n";

    FILE *f = fopen("nested_test.mtp", "w");
    if (!f) return false;
    fprintf(f, "%s", nested_test);
    fclose(f);

    int compile_result = system("../../mtpsc nested_test.mtp > /dev/null 2>&1");
    remove("nested_test.mtp");

    if (compile_result != 0) {
        printf("  Nested structures test failed\n");
        return false;
    }

    printf("  Nested structures work correctly\n");
    return true;
}

// Test that arrays work
bool test_array_types() {
    printf("Testing array types...\n");

    const char *array_test =
        "function testArrays(): {\n"
        "    numbers: [number],\n"
        "    strings: [string]\n"
        "} {\n"
        "    return {\n"
        "        numbers: [1, 2, 3],\n"
        "        strings: [\"a\", \"b\", \"c\"]\n"
        "    }\n"
        "}\n";

    FILE *f = fopen("array_test.mtp", "w");
    if (!f) return false;
    fprintf(f, "%s", array_test);
    fclose(f);

    int compile_result = system("../../mtpsc array_test.mtp > /dev/null 2>&1");
    remove("array_test.mtp");

    if (compile_result != 0) {
        printf("  Array types test failed\n");
        return false;
    }

    printf("  Array types work correctly\n");
    return true;
}

// Run all complex model tests
bool test_complex_model_all() {
    printf("Running Complex Model Tests...\n");

    bool fixture_compilation = test_complex_fixture_compilation();
    bool fixture_type_check = test_complex_fixture_type_check();
    bool primitives = test_primitive_types();
    bool nested = test_nested_structures();
    bool arrays = test_array_types();

    printf("\nComplex Model Test Results:\n");
    printf("Fixture Compilation: %s\n", fixture_compilation ? "PASS" : "FAIL");
    printf("Fixture Type Check: %s\n", fixture_type_check ? "PASS" : "FAIL");
    printf("Primitive Types: %s\n", primitives ? "PASS" : "FAIL");
    printf("Nested Structures: %s\n", nested ? "PASS" : "FAIL");
    printf("Array Types: %s\n", arrays ? "PASS" : "FAIL");

    return fixture_compilation && fixture_type_check && primitives && nested && arrays;
}

int main() {
    printf("MTPScript Complex Model Test Suite\n");
    printf("=================================\n\n");

    bool all_passed = test_complex_model_all();

    printf("\n");
    if (all_passed) {
        printf("All complex model tests PASSED! ✅\n");
        return 0;
    } else {
        printf("Some complex model tests FAILED! ❌\n");
        return 1;
    }
}
