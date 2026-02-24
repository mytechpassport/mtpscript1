/**
 * TypeScript AST Parser Implementation
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>
#include <assert.h>
#include "typescript_parser.h"
#include "cutils.h"

// Forward declarations for free functions
void mtpscript_ts_interface_decl_free(mtpscript_ts_interface_decl_t *interface);
void mtpscript_ts_program_free(mtpscript_ts_program_t *program);

// Parser implementation
mtpscript_typescript_parser_t *mtpscript_typescript_parser_new(const char *source) {
    mtpscript_typescript_parser_t *parser = calloc(1, sizeof(mtpscript_typescript_parser_t));
    parser->source = source;
    parser->source_len = strlen(source);
    parser->position = 0;
    parser->line = 1;
    parser->column = 1;
    parser->program = calloc(1, sizeof(mtpscript_ts_program_t));
    parser->program->declarations = mtpscript_vector_new();
    return parser;
}

void mtpscript_typescript_parser_free(mtpscript_typescript_parser_t *parser) {
    if (parser->program) {
        mtpscript_ts_program_free(parser->program);
    }
    free(parser);
}

static char mtpscript_typescript_peek(mtpscript_typescript_parser_t *parser) {
    if (parser->position >= parser->source_len) {
        return '\0';
    }
    return parser->source[parser->position];
}

static char mtpscript_typescript_advance(mtpscript_typescript_parser_t *parser) {
    char c = mtpscript_typescript_peek(parser);
    if (c == '\n') {
        parser->line++;
        parser->column = 1;
    } else {
        parser->column++;
    }
    parser->position++;
    return c;
}

static void mtpscript_typescript_skip_whitespace(mtpscript_typescript_parser_t *parser) {
    while (parser->position < parser->source_len) {
        char c = mtpscript_typescript_peek(parser);
        if (isspace(c)) {
            mtpscript_typescript_advance(parser);
        } else if (c == '/' && parser->position + 1 < parser->source_len) {
            if (parser->source[parser->position + 1] == '/') {
                // Skip single-line comment
                while (parser->position < parser->source_len && mtpscript_typescript_peek(parser) != '\n') {
                    mtpscript_typescript_advance(parser);
                }
            } else if (parser->source[parser->position + 1] == '*') {
                // Skip multi-line comment
                mtpscript_typescript_advance(parser); // skip /
                mtpscript_typescript_advance(parser); // skip *
                while (parser->position + 1 < parser->source_len) {
                    if (mtpscript_typescript_peek(parser) == '*' && parser->source[parser->position + 1] == '/') {
                        mtpscript_typescript_advance(parser); // skip *
                        mtpscript_typescript_advance(parser); // skip /
                        break;
                    }
                    mtpscript_typescript_advance(parser);
                }
            } else {
                break;
            }
        } else {
            break;
        }
    }
}

static bool mtpscript_typescript_match(mtpscript_typescript_parser_t *parser, const char *str) {
    size_t len = strlen(str);
    if (parser->position + len > parser->source_len) {
        return false;
    }
    if (strncmp(&parser->source[parser->position], str, len) == 0) {
        for (size_t i = 0; i < len; i++) {
            mtpscript_typescript_advance(parser);
        }
        return true;
    }
    return false;
}

static char *mtpscript_typescript_parse_identifier(mtpscript_typescript_parser_t *parser) {
    size_t start = parser->position;
    if (!isalpha(mtpscript_typescript_peek(parser)) && mtpscript_typescript_peek(parser) != '_') {
        return NULL;
    }
    mtpscript_typescript_advance(parser);
    while (isalnum(mtpscript_typescript_peek(parser)) || mtpscript_typescript_peek(parser) == '_') {
        mtpscript_typescript_advance(parser);
    }
    size_t len = parser->position - start;
    char *ident = malloc(len + 1);
    memcpy(ident, &parser->source[start], len);
    ident[len] = '\0';
    return ident;
}

static mtpscript_ts_type_ref_t *mtpscript_typescript_parse_type_ref(mtpscript_typescript_parser_t *parser) {
    mtpscript_typescript_skip_whitespace(parser);

    // Handle union types (limited support)
    if (mtpscript_typescript_match(parser, "null")) {
        mtpscript_typescript_skip_whitespace(parser);
        if (mtpscript_typescript_match(parser, "|")) {
            mtpscript_typescript_skip_whitespace(parser);
            // Parse the actual type
            char *type_name = mtpscript_typescript_parse_identifier(parser);
            if (!type_name) return NULL;

            mtpscript_ts_type_ref_t *type_ref = mtpscript_ts_type_ref_new("Option");
            type_ref->type_args = mtpscript_vector_new();
            mtpscript_ts_type_ref_t *inner_type = mtpscript_ts_type_ref_new(type_name);
            mtpscript_vector_push(type_ref->type_args, inner_type);
            free(type_name);
            return type_ref;
        }
    }

    char *type_name = mtpscript_typescript_parse_identifier(parser);
    if (!type_name) return NULL;

    mtpscript_ts_type_ref_t *type_ref = mtpscript_ts_type_ref_new(type_name);
    free(type_name);

    // Handle array types
    mtpscript_typescript_skip_whitespace(parser);
    if (mtpscript_typescript_match(parser, "[")) {
        if (mtpscript_typescript_match(parser, "]")) {
            type_ref->is_array = true;
        } else {
            // Invalid array syntax
            mtpscript_ts_type_ref_free(type_ref);
            return NULL;
        }
    }

    // Handle optional types
    if (mtpscript_typescript_match(parser, "?")) {
        type_ref->is_optional = true;
    }

    return type_ref;
}

static mtpscript_ts_property_t *mtpscript_typescript_parse_property(mtpscript_typescript_parser_t *parser) {
    mtpscript_typescript_skip_whitespace(parser);

    bool readonly = false;
    if (mtpscript_typescript_match(parser, "readonly")) {
        readonly = true;
        mtpscript_typescript_skip_whitespace(parser);
    }

    char *name = mtpscript_typescript_parse_identifier(parser);
    if (!name) return NULL;

    bool optional = false;
    mtpscript_typescript_skip_whitespace(parser);
    if (mtpscript_typescript_match(parser, "?")) {
        optional = true;
    }

    mtpscript_typescript_skip_whitespace(parser);
    if (!mtpscript_typescript_match(parser, ":")) {
        free(name);
        return NULL;
    }

    mtpscript_ts_type_ref_t *type = mtpscript_typescript_parse_type_ref(parser);
    if (!type) {
        free(name);
        return NULL;
    }

    mtpscript_ts_property_t *prop = mtpscript_ts_property_new(name, type);
    prop->readonly = readonly;
    prop->optional = optional;

    free(name);
    return prop;
}

static mtpscript_ts_interface_decl_t *mtpscript_typescript_parse_interface(mtpscript_typescript_parser_t *parser) {
    mtpscript_typescript_skip_whitespace(parser);

    if (!mtpscript_typescript_match(parser, "interface")) {
        return NULL;
    }

    mtpscript_typescript_skip_whitespace(parser);
    char *name = mtpscript_typescript_parse_identifier(parser);
    if (!name) return NULL;

    mtpscript_ts_interface_decl_t *interface = calloc(1, sizeof(mtpscript_ts_interface_decl_t));
    interface->name = name;
    interface->properties = mtpscript_vector_new();
    interface->methods = mtpscript_vector_new();
    interface->extends = mtpscript_vector_new();

    mtpscript_typescript_skip_whitespace(parser);
    if (mtpscript_typescript_match(parser, "extends")) {
        // Parse extends clause (simplified)
        mtpscript_typescript_skip_whitespace(parser);
        char *extends_name = mtpscript_typescript_parse_identifier(parser);
        if (extends_name) {
            mtpscript_vector_push(interface->extends, extends_name);
        }
    }

    mtpscript_typescript_skip_whitespace(parser);
    if (!mtpscript_typescript_match(parser, "{")) {
        mtpscript_ts_interface_decl_free(interface);
        return NULL;
    }

    // Parse interface body
    while (!mtpscript_typescript_match(parser, "}")) {
        mtpscript_typescript_skip_whitespace(parser);

        if (mtpscript_typescript_peek(parser) == '\0') {
            mtpscript_ts_interface_decl_free(interface);
            return NULL;
        }

        mtpscript_ts_property_t *prop = mtpscript_typescript_parse_property(parser);
        if (prop) {
            mtpscript_vector_push(interface->properties, prop);
            mtpscript_typescript_skip_whitespace(parser);
            mtpscript_typescript_match(parser, ";"); // optional semicolon
            continue;
        }

        // Skip unrecognized declarations for now
        while (mtpscript_typescript_peek(parser) != '\n' && mtpscript_typescript_peek(parser) != '}') {
            mtpscript_typescript_advance(parser);
        }
    }

    return interface;
}

mtpscript_ts_program_t *mtpscript_typescript_parse(mtpscript_typescript_parser_t *parser) {
    while (parser->position < parser->source_len) {
        mtpscript_typescript_skip_whitespace(parser);

        if (parser->position >= parser->source_len) {
            break;
        }

        // Try to parse interface
        if (strncmp(&parser->source[parser->position], "interface", 9) == 0) {
            mtpscript_ts_interface_decl_t *interface = mtpscript_typescript_parse_interface(parser);
            if (interface) {
                mtpscript_ts_node_t *node = mtpscript_ts_node_new(TS_NODE_INTERFACE_DECL);
                node->data.interface_decl = interface;
                mtpscript_vector_push(parser->program->declarations, node);
                continue;
            }
        }

        // Skip other declarations for now
        while (parser->position < parser->source_len &&
               mtpscript_typescript_peek(parser) != '\n') {
            mtpscript_typescript_advance(parser);
        }
    }

    return parser->program;
}

// Node constructors and destructors
mtpscript_ts_node_t *mtpscript_ts_node_new(mtpscript_ts_node_type_t type) {
    mtpscript_ts_node_t *node = calloc(1, sizeof(mtpscript_ts_node_t));
    node->type = type;
    return node;
}

void mtpscript_ts_node_free(mtpscript_ts_node_t *node) {
    if (!node) return;

    switch (node->type) {
        case TS_NODE_INTERFACE_DECL:
            if (node->data.interface_decl) {
                mtpscript_ts_interface_decl_free(node->data.interface_decl);
            }
            break;
        // Add other cases as needed
        default:
            break;
    }

    free(node);
}

mtpscript_ts_type_ref_t *mtpscript_ts_type_ref_new(const char *name) {
    mtpscript_ts_type_ref_t *type_ref = calloc(1, sizeof(mtpscript_ts_type_ref_t));
    type_ref->name = strdup(name);
    type_ref->type_args = mtpscript_vector_new();
    return type_ref;
}

void mtpscript_ts_type_ref_free(mtpscript_ts_type_ref_t *type_ref) {
    if (!type_ref) return;
    free(type_ref->name);
    if (type_ref->type_args) {
        for (size_t i = 0; i < type_ref->type_args->size; i++) {
            mtpscript_ts_type_ref_free(type_ref->type_args->items[i]);
        }
        mtpscript_vector_free(type_ref->type_args);
    }
    free(type_ref);
}

mtpscript_ts_property_t *mtpscript_ts_property_new(const char *name, mtpscript_ts_type_ref_t *type) {
    mtpscript_ts_property_t *prop = calloc(1, sizeof(mtpscript_ts_property_t));
    prop->name = strdup(name);
    prop->type = type;
    return prop;
}

void mtpscript_ts_property_free(mtpscript_ts_property_t *property) {
    if (!property) return;
    free(property->name);
    if (property->type) {
        mtpscript_ts_type_ref_free(property->type);
    }
    free(property);
}

mtpscript_ts_parameter_t *mtpscript_ts_parameter_new(const char *name, mtpscript_ts_type_ref_t *type) {
    mtpscript_ts_parameter_t *param = calloc(1, sizeof(mtpscript_ts_parameter_t));
    param->name = strdup(name);
    param->type = type;
    return param;
}

void mtpscript_ts_parameter_free(mtpscript_ts_parameter_t *parameter) {
    if (!parameter) return;
    free(parameter->name);
    if (parameter->type) {
        mtpscript_ts_type_ref_free(parameter->type);
    }
    free(parameter);
}

void mtpscript_ts_interface_decl_free(mtpscript_ts_interface_decl_t *interface) {
    if (!interface) return;
    free(interface->name);

    if (interface->properties) {
        for (size_t i = 0; i < interface->properties->size; i++) {
            mtpscript_ts_property_free(interface->properties->items[i]);
        }
        mtpscript_vector_free(interface->properties);
    }

    if (interface->methods) {
        // Free methods if implemented
        mtpscript_vector_free(interface->methods);
    }

    if (interface->extends) {
        for (size_t i = 0; i < interface->extends->size; i++) {
            free(interface->extends->items[i]);
        }
        mtpscript_vector_free(interface->extends);
    }

    free(interface);
}

void mtpscript_ts_program_free(mtpscript_ts_program_t *program) {
    if (!program) return;

    if (program->declarations) {
        for (size_t i = 0; i < program->declarations->size; i++) {
            mtpscript_ts_node_free(program->declarations->items[i]);
        }
        mtpscript_vector_free(program->declarations);
    }

    free(program);
}

// Migration API
char *mtpscript_typescript_node_to_mtpscript(mtpscript_ts_node_t *node) {
    if (!node) return NULL;

    switch (node->type) {
        case TS_NODE_INTERFACE_DECL: {
            mtpscript_ts_interface_decl_t *interface = node->data.interface_decl;
            size_t buffer_size = 1024;
            char *result = malloc(buffer_size);
            size_t pos = 0;

            // Convert interface to record
            pos += snprintf(result + pos, buffer_size - pos, "record %s {\n", interface->name);

            // Convert properties
            for (size_t i = 0; i < interface->properties->size; i++) {
                mtpscript_ts_property_t *prop = interface->properties->items[i];
                char *mtpscript_type = mtpscript_typescript_type_to_mtpscript(prop->type->name);

                pos += snprintf(result + pos, buffer_size - pos, "  %s: %s",
                               prop->name, mtpscript_type);

                if (prop->optional) {
                    pos += snprintf(result + pos, buffer_size - pos, "?");
                }

                pos += snprintf(result + pos, buffer_size - pos, ",\n");

                free(mtpscript_type);
            }

            pos += snprintf(result + pos, buffer_size - pos, "}\n");
            return result;
        }
        default:
            return strdup("// Unsupported TypeScript construct\n");
    }
}

char *mtpscript_typescript_program_to_mtpscript(mtpscript_ts_program_t *program) {
    if (!program) return NULL;

    size_t total_size = 1024;
    char *result = malloc(total_size);
    size_t pos = 0;

    for (size_t i = 0; i < program->declarations->size; i++) {
        char *node_str = mtpscript_typescript_node_to_mtpscript(program->declarations->items[i]);
        size_t node_len = strlen(node_str);

        if (pos + node_len >= total_size) {
            total_size *= 2;
            result = realloc(result, total_size);
        }

        memcpy(result + pos, node_str, node_len);
        pos += node_len;
        free(node_str);
    }

    result[pos] = '\0';
    return result;
}

// Utility functions
bool mtpscript_typescript_is_keyword(const char *str) {
    const char *keywords[] = {
        "interface", "class", "function", "const", "let", "var",
        "if", "else", "for", "while", "try", "catch", "throw",
        "return", "import", "export", "enum", "type", "extends",
        "implements", "readonly", "private", "public", "protected",
        "static", "async", "await", NULL
    };

    for (int i = 0; keywords[i]; i++) {
        if (strcmp(str, keywords[i]) == 0) {
            return true;
        }
    }
    return false;
}

bool mtpscript_typescript_is_builtin_type(const char *str) {
    const char *types[] = {
        "string", "number", "boolean", "any", "void", "null", "undefined",
        "String", "Number", "Boolean", "Array", "Object", "Promise", NULL
    };

    for (int i = 0; types[i]; i++) {
        if (strcmp(str, types[i]) == 0) {
            return true;
        }
    }
    return false;
}

char *mtpscript_typescript_type_to_mtpscript(const char *ts_type) {
    if (strcmp(ts_type, "string") == 0) return strdup("String");
    if (strcmp(ts_type, "number") == 0) return strdup("Int");
    if (strcmp(ts_type, "boolean") == 0) return strdup("Bool");
    if (strcmp(ts_type, "Option") == 0) return strdup("Option");

    // Return as-is for unknown types (may be custom types)
    return strdup(ts_type);
}
