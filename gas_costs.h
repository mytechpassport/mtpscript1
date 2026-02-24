/*
 * MTPScript Gas Cost Definitions
 * Based on Annex A of MTPScript V5.1 specification
 */

#ifndef GAS_COSTS_H
#define GAS_COSTS_H

#include <stdint.h>

/* Base costs */
#define GAS_COST_BASE 1
#define GAS_COST_CALL 5
#define GAS_COST_ALLOC_BYTE 1
#define GAS_COST_ALLOC_VALUE 3
#define GAS_COST_PROPERTY_ACCESS 3
#define GAS_COST_STRING_ACCESS 2

/* Math operations */
#define GAS_COST_MATH_BASIC 2
#define GAS_COST_MATH_COMPLEX 10

/* String operations */
#define GAS_COST_STR_CONCAT 3

/* Array operations */
#define GAS_COST_ARRAY_ACCESS 2

/* JSON operations */
#define GAS_COST_JSON_PARSE 20
#define GAS_COST_JSON_STRINGIFY 15

/* Effect system */
#define GAS_COST_EFFECT_REGISTER 20
#define GAS_COST_EFFECT_CALL 100

/* Crypto operations */
#define GAS_COST_CRYPTO_HASH 50
#define GAS_COST_CRYPTO_SIGN 200
#define GAS_COST_CRYPTO_VERIFY 150

/* Error handling */
#define GAS_COST_THROW_ERROR 10

/* Get gas cost for an opcode - Basic Annex A implementation */
/* Simplified implementation that provides the framework for detailed costing */
uint32_t get_opcode_gas_cost(uint32_t opcode);

#endif /* GAS_COSTS_H */