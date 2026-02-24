/**
 * TypeScript AST Parser for MTPScript Migration Tool
 *
 * Parses TypeScript source code into an AST that can be transformed
 * to MTPScript syntax. Handles basic TypeScript constructs needed
 * for mechanical migration.
 */

#ifndef MTPSCRIPT_TYPESCRIPT_PARSER_H
#define MTPSCRIPT_TYPESCRIPT_PARSER_H

#include <stdbool.h>
#include "../compiler/ast.h"  // Reuse existing AST structures where possible

// Forward declarations
typedef struct mtpscript_typescript_parser_t mtpscript_typescript_parser_t;
typedef struct mtpscript_ts_node_t mtpscript_ts_node_t;
typedef struct mtpscript_ts_program_t mtpscript_ts_program_t;

// TypeScript AST Node Types
typedef enum {
    TS_NODE_PROGRAM,
    TS_NODE_INTERFACE_DECL,
    TS_NODE_CLASS_DECL,
    TS_NODE_FUNCTION_DECL,
    TS_NODE_VARIABLE_DECL,
    TS_NODE_TYPE_ALIAS,
    TS_NODE_ENUM_DECL,
    TS_NODE_IMPORT_DECL,
    TS_NODE_EXPORT_DECL,
    TS_NODE_PROPERTY,
    TS_NODE_METHOD,
    TS_NODE_PARAMETER,
    TS_NODE_TYPE_REF,
    TS_NODE_LITERAL,
    TS_NODE_BLOCK,
    TS_NODE_EXPRESSION_STMT,
    TS_NODE_RETURN_STMT,
    TS_NODE_IF_STMT,
    TS_NODE_FOR_STMT,
    TS_NODE_WHILE_STMT,
    TS_NODE_TRY_STMT,
    TS_NODE_THROW_STMT
} mtpscript_ts_node_type_t;

// TypeScript Type Reference
typedef struct {
    char *name;
    mtpscript_vector_t *type_args;  // Vector of mtpscript_ts_type_ref_t
    bool is_array;
    bool is_optional;
} mtpscript_ts_type_ref_t;

// TypeScript Property
typedef struct {
    char *name;
    mtpscript_ts_type_ref_t *type;
    bool readonly;
    bool optional;
} mtpscript_ts_property_t;

// TypeScript Method
typedef struct {
    char *name;
    mtpscript_vector_t *parameters;  // Vector of mtpscript_ts_parameter_t
    mtpscript_ts_type_ref_t *return_type;
    bool is_static;
} mtpscript_ts_method_t;

// TypeScript Parameter
typedef struct {
    char *name;
    mtpscript_ts_type_ref_t *type;
    bool optional;
} mtpscript_ts_parameter_t;

// TypeScript Interface Declaration
typedef struct {
    char *name;
    mtpscript_vector_t *properties;  // Vector of mtpscript_ts_property_t
    mtpscript_vector_t *methods;     // Vector of mtpscript_ts_method_t
    mtpscript_vector_t *extends;     // Vector of char* (interface names)
} mtpscript_ts_interface_decl_t;

// TypeScript Class Declaration
typedef struct {
    char *name;
    mtpscript_vector_t *properties;  // Vector of mtpscript_ts_property_t
    mtpscript_vector_t *methods;     // Vector of mtpscript_ts_method_t
    mtpscript_vector_t *implements;  // Vector of char* (interface names)
    char *extends;                   // Single class name
} mtpscript_ts_class_decl_t;

// TypeScript Function Declaration
typedef struct {
    char *name;
    mtpscript_vector_t *parameters;  // Vector of mtpscript_ts_parameter_t
    mtpscript_ts_type_ref_t *return_type;
    bool is_async;
    bool is_export;
} mtpscript_ts_function_decl_t;

// TypeScript Import Declaration
typedef struct {
    mtpscript_vector_t *imports;  // Vector of char* (imported names)
    char *from;                   // Module path
    bool is_default;
    char *default_name;
} mtpscript_ts_import_decl_t;

// Generic TypeScript AST Node
struct mtpscript_ts_node_t {
    mtpscript_ts_node_type_t type;
    union {
        mtpscript_ts_program_t *program;
        mtpscript_ts_interface_decl_t *interface_decl;
        mtpscript_ts_class_decl_t *class_decl;
        mtpscript_ts_function_decl_t *function_decl;
        mtpscript_ts_import_decl_t *import_decl;
        mtpscript_ts_property_t *property;
        mtpscript_ts_method_t *method;
        mtpscript_ts_parameter_t *parameter;
        mtpscript_ts_type_ref_t *type_ref;
        char *literal;
    } data;
    int line;
    int column;
};

// TypeScript Program (root node)
struct mtpscript_ts_program_t {
    mtpscript_vector_t *declarations;  // Vector of mtpscript_ts_node_t
};

// Parser structure
struct mtpscript_typescript_parser_t {
    const char *source;
    size_t source_len;
    size_t position;
    int line;
    int column;
    mtpscript_ts_program_t *program;
};

// Parser API
mtpscript_typescript_parser_t *mtpscript_typescript_parser_new(const char *source);
void mtpscript_typescript_parser_free(mtpscript_typescript_parser_t *parser);

mtpscript_ts_program_t *mtpscript_typescript_parse(mtpscript_typescript_parser_t *parser);

// Node constructors and destructors
mtpscript_ts_node_t *mtpscript_ts_node_new(mtpscript_ts_node_type_t type);
void mtpscript_ts_node_free(mtpscript_ts_node_t *node);

mtpscript_ts_type_ref_t *mtpscript_ts_type_ref_new(const char *name);
void mtpscript_ts_type_ref_free(mtpscript_ts_type_ref_t *type_ref);

mtpscript_ts_property_t *mtpscript_ts_property_new(const char *name, mtpscript_ts_type_ref_t *type);
void mtpscript_ts_property_free(mtpscript_ts_property_t *property);

mtpscript_ts_parameter_t *mtpscript_ts_parameter_new(const char *name, mtpscript_ts_type_ref_t *type);
void mtpscript_ts_parameter_free(mtpscript_ts_parameter_t *parameter);

// Migration API
char *mtpscript_typescript_node_to_mtpscript(mtpscript_ts_node_t *node);
char *mtpscript_typescript_program_to_mtpscript(mtpscript_ts_program_t *program);

// Utility functions
bool mtpscript_typescript_is_keyword(const char *str);
bool mtpscript_typescript_is_builtin_type(const char *str);
char *mtpscript_typescript_type_to_mtpscript(const char *ts_type);

#endif // MTPSCRIPT_TYPESCRIPT_PARSER_H
