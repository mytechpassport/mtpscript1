#ifndef MTPSCRIPT_GAS_COSTS_H
#define MTPSCRIPT_GAS_COSTS_H

#include <stdint.h>
#include <stddef.h>

typedef enum {
    OP_LOAD_CONST = 1,
    OP_GET_GLOBAL = 2,
    OP_SET_GLOBAL = 3,
    OP_BINARY_OP = 4,
    OP_UNARY_OP = 5,
    OP_FUNCTION_CALL = 100,
    OP_RETURN = 6,
    OP_TAIL_CALL = 0,
    OP_DB_READ = 1000,
    OP_DB_WRITE = 2000,
    OP_HTTP_OUT = 2000,
    OP_LOG = 50,
    OP_ASYNC_AWAIT = 500,
} mtp_opcode_t;

typedef struct {
    mtp_opcode_t opcode;
    uint32_t gas_cost;
} gas_cost_entry_t;

static const gas_cost_entry_t gas_costs[] = {
    {OP_LOAD_CONST, 1},
    {OP_GET_GLOBAL, 2},
    {OP_SET_GLOBAL, 3},
    {OP_BINARY_OP, 4},
    {OP_UNARY_OP, 5},
    {OP_FUNCTION_CALL, 100},
    {OP_RETURN, 6},
    {OP_TAIL_CALL, 0},
    {OP_DB_READ, 1000},
    {OP_DB_WRITE, 2000},
    {OP_HTTP_OUT, 2000},
    {OP_LOG, 50},
    {OP_ASYNC_AWAIT, 500},
};

static inline uint32_t get_gas_cost(mtp_opcode_t opcode) {
    const size_t num_entries = sizeof(gas_costs) / sizeof(gas_cost_entry_t);
    for (size_t i = 0; i < num_entries; i++) {
        if (gas_costs[i].opcode == opcode) {
            return gas_costs[i].gas_cost;
        }
    }
    return 0; // Default for unknown opcodes
}

#endif // MTPSCRIPT_GAS_COSTS_H