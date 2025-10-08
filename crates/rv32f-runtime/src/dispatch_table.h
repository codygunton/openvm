#ifndef OPENVM_RV32F_DISPATCH_TABLE_H
#define OPENVM_RV32F_DISPATCH_TABLE_H

#include <stdint.h>

// Function pointer table for F extension operations (RV32IMF)
// Adds single-precision floating-point to OpenVM's RV32IM base
// This table is placed at a fixed address (0xE0000000) via linker script
// The transpiler emits indirect calls through this table

// Function pointer type for float operations
// All operations use the same signature for simplicity:
// - a0: rs1 (source register 1 index)
// - a1: rs2 (source register 2 index)
// - a2: rd (destination register index)
// - a3: rm (rounding mode)
typedef void (*float_op_fn)(uint32_t, uint32_t, uint32_t, uint32_t);

// Dispatch table indices (must match transpiler)
#define FLOAT_OP_FADD_S    0
#define FLOAT_OP_FSUB_S    1
#define FLOAT_OP_FMUL_S    2
#define FLOAT_OP_FDIV_S    3
#define FLOAT_OP_FSQRT_S   4
#define FLOAT_OP_FMIN_S    5
#define FLOAT_OP_FMAX_S    6
#define FLOAT_OP_FMADD_S   7
#define FLOAT_OP_FMSUB_S   8
#define FLOAT_OP_FNMADD_S  9
#define FLOAT_OP_FNMSUB_S  10
#define FLOAT_OP_FSGNJ_S   11
#define FLOAT_OP_FSGNJN_S  12
#define FLOAT_OP_FSGNJX_S  13
#define FLOAT_OP_FEQ_S     14
#define FLOAT_OP_FLT_S     15
#define FLOAT_OP_FLE_S     16
#define FLOAT_OP_FCVT_W_S  17
#define FLOAT_OP_FCVT_WU_S 18
#define FLOAT_OP_FCVT_S_W  19
#define FLOAT_OP_FCVT_S_WU 20
#define FLOAT_OP_FMV_X_W   21
#define FLOAT_OP_FMV_W_X   22
#define FLOAT_OP_FCLASS_S  23

#define FLOAT_OP_TABLE_SIZE 32

// The dispatch table (defined in dispatch_table.c)
extern const float_op_fn openvm_float_dispatch_table[FLOAT_OP_TABLE_SIZE];

// Fixed address where table will be placed (must match linker script and transpiler)
// Must be below OpenVM's MEM_SIZE limit of 0x20000000 (512MB)
#define FLOAT_DISPATCH_TABLE_ADDR 0x10000000

#endif // OPENVM_RV32F_DISPATCH_TABLE_H
