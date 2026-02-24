/**
 * MTPScript OpenAPI Generator Implementation
 * Specification §7.0
 *
 * Copyright (c) 2025 My Tech Passport Inc.
 * Author: Ryan Wong
 */

#include "openapi.h"
#include <string.h>
#include <stdio.h>

/* Helper function to generate deterministic type schema */
static void generate_type_schema(mtpscript_type_t *type, mtpscript_string_t *out, mtpscript_hash_t *schemas) {
    switch (type->kind) {
        case MTPSCRIPT_TYPE_INT:
            mtpscript_string_append_cstr(out, "{\"type\": \"integer\", \"format\": \"int64\"}");
            break;
        case MTPSCRIPT_TYPE_STRING:
            mtpscript_string_append_cstr(out, "{\"type\": \"string\"}");
            break;
        case MTPSCRIPT_TYPE_BOOL:
            mtpscript_string_append_cstr(out, "{\"type\": \"boolean\"}");
            break;
        case MTPSCRIPT_TYPE_DECIMAL:
            mtpscript_string_append_cstr(out, "{\"type\": \"string\", \"format\": \"decimal\"}");
            break;
        case MTPSCRIPT_TYPE_OPTION:
            mtpscript_string_append_cstr(out, "{\"oneOf\": [");
            generate_type_schema(type->inner, out, schemas);
            mtpscript_string_append_cstr(out, ", {\"type\": \"null\"}]}");
            break;
        case MTPSCRIPT_TYPE_RESULT:
            mtpscript_string_append_cstr(out, "{\"oneOf\": [");
            generate_type_schema(type->value, out, schemas);
            mtpscript_string_append_cstr(out, ", ");
            generate_type_schema(type->error, out, schemas);
            mtpscript_string_append_cstr(out, "]}");
            break;
        case MTPSCRIPT_TYPE_LIST:
            mtpscript_string_append_cstr(out, "{\"type\": \"array\", \"items\": ");
            generate_type_schema(type->inner, out, schemas);
            mtpscript_string_append_cstr(out, "}");
            break;
        case MTPSCRIPT_TYPE_MAP:
            mtpscript_string_append_cstr(out, "{\"type\": \"object\", \"additionalProperties\": ");
            generate_type_schema(type->value, out, schemas);
            mtpscript_string_append_cstr(out, "}");
            break;
        case MTPSCRIPT_TYPE_CUSTOM:
            {
                char ref_name[256];
                sprintf(ref_name, "#/components/schemas/%s", mtpscript_string_cstr(type->name));
                mtpscript_string_append_cstr(out, "{\"$ref\": \"");
                mtpscript_string_append_cstr(out, ref_name);
                mtpscript_string_append_cstr(out, "\"}");

                /* Add to schemas if not already present */
                if (!mtpscript_hash_get(schemas, mtpscript_string_cstr(type->name))) {
                    mtpscript_hash_set(schemas, mtpscript_string_cstr(type->name), (void*)1);
                }
            }
            break;
    }
}

/* Helper function to generate parameter schema */
static void generate_parameter_schema(mtpscript_param_t *param, mtpscript_string_t *out, mtpscript_hash_t *schemas) {
    mtpscript_string_append_cstr(out, "        {\n");
    mtpscript_string_append_cstr(out, "          \"name\": \"");
    mtpscript_string_append_cstr(out, mtpscript_string_cstr(param->name));
    mtpscript_string_append_cstr(out, "\",\n");
    mtpscript_string_append_cstr(out, "          \"in\": \"query\",\n");
    mtpscript_string_append_cstr(out, "          \"required\": true,\n");
    mtpscript_string_append_cstr(out, "          \"schema\": ");
    generate_type_schema(param->type, out, schemas);
    mtpscript_string_append_cstr(out, "\n        }");
}

/* Compare function for deterministic sorting of API declarations */
static int compare_api_decls(const void *a, const void *b) {
    const mtpscript_declaration_t *decl_a = *(const mtpscript_declaration_t **)a;
    const mtpscript_declaration_t *decl_b = *(const mtpscript_declaration_t **)b;

    /* Sort by path first, then by method */
    int path_cmp = strcmp(mtpscript_string_cstr(decl_a->data.api.path),
                         mtpscript_string_cstr(decl_b->data.api.path));
    if (path_cmp != 0) return path_cmp;

    return strcmp(mtpscript_string_cstr(decl_a->data.api.method),
                 mtpscript_string_cstr(decl_b->data.api.method));
}

mtpscript_error_t *mtpscript_openapi_generate(mtpscript_program_t *program, mtpscript_string_t **output_out) {
    mtpscript_string_t *out = mtpscript_string_new();
    *output_out = out;

    /* Collect API declarations and sort for deterministic ordering */
    mtpscript_vector_t *api_decls = mtpscript_vector_new();
    for (size_t i = 0; i < program->declarations->size; i++) {
        mtpscript_declaration_t *decl = mtpscript_vector_get(program->declarations, i);
        if (decl->kind == MTPSCRIPT_DECL_API) {
            mtpscript_vector_push(api_decls, decl);
        }
    }

    /* Sort API declarations deterministically by path then method */
    /* This ensures consistent OpenAPI output regardless of source order */
    qsort(api_decls->items, api_decls->size, sizeof(void*), compare_api_decls);

    /* Track schemas for $ref folding */
    mtpscript_hash_t *schemas = mtpscript_hash_new();

    /* Generate OpenAPI 3.0.3 spec with deterministic ordering */
    mtpscript_string_append_cstr(out, "{\n");
    mtpscript_string_append_cstr(out, "  \"openapi\": \"3.0.3\",\n");
    mtpscript_string_append_cstr(out, "  \"info\": {\n");
    mtpscript_string_append_cstr(out, "    \"title\": \"MTPScript API\",\n");
    mtpscript_string_append_cstr(out, "    \"version\": \"v5.1\",\n");
    mtpscript_string_append_cstr(out, "    \"description\": \"Deterministic smart contract API\"\n");
    mtpscript_string_append_cstr(out, "  },\n");

    /* Generate paths */
    mtpscript_string_append_cstr(out, "  \"paths\": {\n");

    for (size_t i = 0; i < api_decls->size; i++) {
        mtpscript_declaration_t *decl = mtpscript_vector_get(api_decls, i);
        mtpscript_api_decl_t *api = &decl->data.api;

        if (i > 0) mtpscript_string_append_cstr(out, ",\n");

        mtpscript_string_append_cstr(out, "    \"");
        mtpscript_string_append_cstr(out, mtpscript_string_cstr(api->path));
        mtpscript_string_append_cstr(out, "\": {\n");
        mtpscript_string_append_cstr(out, "      \"");
        mtpscript_string_append_cstr(out, mtpscript_string_cstr(api->method));
        mtpscript_string_append_cstr(out, "\": {\n");

        /* Generate parameters */
        if (api->handler->params->size > 0) {
            mtpscript_string_append_cstr(out, "        \"parameters\": [\n");
            for (size_t j = 0; j < api->handler->params->size; j++) {
                if (j > 0) mtpscript_string_append_cstr(out, ",\n");
                mtpscript_param_t *param = mtpscript_vector_get(api->handler->params, j);
                generate_parameter_schema(param, out, schemas);
            }
            mtpscript_string_append_cstr(out, "\n        ],\n");
        }

        /* Generate responses */
        mtpscript_string_append_cstr(out, "        \"responses\": {\n");
        mtpscript_string_append_cstr(out, "          \"200\": {\n");
        mtpscript_string_append_cstr(out, "            \"description\": \"Success\",\n");
        mtpscript_string_append_cstr(out, "            \"content\": {\n");
        mtpscript_string_append_cstr(out, "              \"application/json\": {\n");
        mtpscript_string_append_cstr(out, "                \"schema\": ");
        generate_type_schema(api->handler->return_type, out, schemas);
        mtpscript_string_append_cstr(out, "\n");
        mtpscript_string_append_cstr(out, "              }\n");
        mtpscript_string_append_cstr(out, "            }\n");
        mtpscript_string_append_cstr(out, "          },\n");
        mtpscript_string_append_cstr(out, "          \"400\": {\n");
        mtpscript_string_append_cstr(out, "            \"description\": \"Bad Request\",\n");
        mtpscript_string_append_cstr(out, "            \"content\": {\n");
        mtpscript_string_append_cstr(out, "              \"application/json\": {\n");
        mtpscript_string_append_cstr(out, "                \"schema\": {\n");
        mtpscript_string_append_cstr(out, "                  \"$ref\": \"#/components/schemas/ErrorResponse\"\n");
        mtpscript_string_append_cstr(out, "                }\n");
        mtpscript_string_append_cstr(out, "              }\n");
        mtpscript_string_append_cstr(out, "            }\n");
        mtpscript_string_append_cstr(out, "          }\n");
        mtpscript_string_append_cstr(out, "        }\n");
        mtpscript_string_append_cstr(out, "      }\n");
        mtpscript_string_append_cstr(out, "    }");
    }

    mtpscript_string_append_cstr(out, "\n  }");

    /* Generate components/schemas for $ref folding */
    mtpscript_hash_iterator_t *schema_iter = mtpscript_hash_iterator_new(schemas);
    bool has_schemas = mtpscript_hash_iterator_next(schema_iter);
    mtpscript_hash_iterator_free(schema_iter);

    if (has_schemas || api_decls->size > 0) {
        mtpscript_string_append_cstr(out, ",\n  \"components\": {\n");
        mtpscript_string_append_cstr(out, "    \"schemas\": {\n");

        /* Add ErrorResponse schema */
        mtpscript_string_append_cstr(out, "      \"ErrorResponse\": {\n");
        mtpscript_string_append_cstr(out, "        \"type\": \"object\",\n");
        mtpscript_string_append_cstr(out, "        \"properties\": {\n");
        mtpscript_string_append_cstr(out, "          \"error\": {\n");
        mtpscript_string_append_cstr(out, "            \"type\": \"string\"\n");
        mtpscript_string_append_cstr(out, "          },\n");
        mtpscript_string_append_cstr(out, "          \"message\": {\n");
        mtpscript_string_append_cstr(out, "            \"type\": \"string\"\n");
        mtpscript_string_append_cstr(out, "          }\n");
        mtpscript_string_append_cstr(out, "        },\n");
        mtpscript_string_append_cstr(out, "        \"required\": [\"error\", \"message\"]\n");
        mtpscript_string_append_cstr(out, "      }");

        /* Add custom type schemas */
        schema_iter = mtpscript_hash_iterator_new(schemas);
        while (mtpscript_hash_iterator_next(schema_iter)) {
            mtpscript_string_append_cstr(out, ",\n      \"");
            mtpscript_string_append_cstr(out, mtpscript_hash_iterator_key(schema_iter));
            mtpscript_string_append_cstr(out, "\": {\n");
            mtpscript_string_append_cstr(out, "        \"type\": \"object\",\n");
            mtpscript_string_append_cstr(out, "        \"description\": \"Custom MTPScript type\"\n");
            mtpscript_string_append_cstr(out, "      }");
        }
        mtpscript_hash_iterator_free(schema_iter);

        mtpscript_string_append_cstr(out, "\n    }\n");
        mtpscript_string_append_cstr(out, "  }");
    }

    mtpscript_string_append_cstr(out, "\n}\n");

    /* Cleanup */
    mtpscript_vector_free(api_decls);
    mtpscript_hash_free(schemas);

    return NULL;
}
