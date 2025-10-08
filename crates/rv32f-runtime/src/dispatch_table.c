// Dispatch table for F extension operations (RV32IMF)
// Adds single-precision floating-point to OpenVM's RV32IM base
// This table is placed at a fixed address via linker script

#include "dispatch_table.h"

// Forward declarations of all float operations
void _openvm_fadd_s(uint32_t, uint32_t, uint32_t, uint32_t);
void _openvm_fsub_s(uint32_t, uint32_t, uint32_t, uint32_t);
void _openvm_fmul_s(uint32_t, uint32_t, uint32_t, uint32_t);
void _openvm_fdiv_s(uint32_t, uint32_t, uint32_t, uint32_t);
void _openvm_fsqrt_s(uint32_t, uint32_t, uint32_t, uint32_t);
void _openvm_fmin_s(uint32_t, uint32_t, uint32_t, uint32_t);
void _openvm_fmax_s(uint32_t, uint32_t, uint32_t, uint32_t);
void _openvm_fmadd_s(uint32_t, uint32_t, uint32_t, uint32_t);
void _openvm_fmsub_s(uint32_t, uint32_t, uint32_t, uint32_t);
void _openvm_fnmadd_s(uint32_t, uint32_t, uint32_t, uint32_t);
void _openvm_fnmsub_s(uint32_t, uint32_t, uint32_t, uint32_t);
void _openvm_fsgnj_s(uint32_t, uint32_t, uint32_t, uint32_t);
void _openvm_fsgnjn_s(uint32_t, uint32_t, uint32_t, uint32_t);
void _openvm_fsgnjx_s(uint32_t, uint32_t, uint32_t, uint32_t);
void _openvm_feq_s(uint32_t, uint32_t, uint32_t, uint32_t);
void _openvm_flt_s(uint32_t, uint32_t, uint32_t, uint32_t);
void _openvm_fle_s(uint32_t, uint32_t, uint32_t, uint32_t);
void _openvm_fcvt_w_s(uint32_t, uint32_t, uint32_t, uint32_t);
void _openvm_fcvt_wu_s(uint32_t, uint32_t, uint32_t, uint32_t);
void _openvm_fcvt_s_w(uint32_t, uint32_t, uint32_t, uint32_t);
void _openvm_fcvt_s_wu(uint32_t, uint32_t, uint32_t, uint32_t);
void _openvm_fmv_x_w(uint32_t, uint32_t, uint32_t, uint32_t);
void _openvm_fmv_w_x(uint32_t, uint32_t, uint32_t, uint32_t);
void _openvm_fclass_s(uint32_t, uint32_t, uint32_t, uint32_t);

// The dispatch table
// This is placed at 0xE0000000 via the linker script
__attribute__((section(".float_dispatch_table")))
const float_op_fn openvm_float_dispatch_table[FLOAT_OP_TABLE_SIZE] = {
    [FLOAT_OP_FADD_S]    = _openvm_fadd_s,
    [FLOAT_OP_FSUB_S]    = _openvm_fsub_s,
    [FLOAT_OP_FMUL_S]    = _openvm_fmul_s,
    [FLOAT_OP_FDIV_S]    = _openvm_fdiv_s,
    [FLOAT_OP_FSQRT_S]   = _openvm_fsqrt_s,
    [FLOAT_OP_FMIN_S]    = _openvm_fmin_s,
    [FLOAT_OP_FMAX_S]    = _openvm_fmax_s,
    [FLOAT_OP_FMADD_S]   = _openvm_fmadd_s,
    [FLOAT_OP_FMSUB_S]   = _openvm_fmsub_s,
    [FLOAT_OP_FNMADD_S]  = _openvm_fnmadd_s,
    [FLOAT_OP_FNMSUB_S]  = _openvm_fnmsub_s,
    [FLOAT_OP_FSGNJ_S]   = _openvm_fsgnj_s,
    [FLOAT_OP_FSGNJN_S]  = _openvm_fsgnjn_s,
    [FLOAT_OP_FSGNJX_S]  = _openvm_fsgnjx_s,
    [FLOAT_OP_FEQ_S]     = _openvm_feq_s,
    [FLOAT_OP_FLT_S]     = _openvm_flt_s,
    [FLOAT_OP_FLE_S]     = _openvm_fle_s,
    [FLOAT_OP_FCVT_W_S]  = _openvm_fcvt_w_s,
    [FLOAT_OP_FCVT_WU_S] = _openvm_fcvt_wu_s,
    [FLOAT_OP_FCVT_S_W]  = _openvm_fcvt_s_w,
    [FLOAT_OP_FCVT_S_WU] = _openvm_fcvt_s_wu,
    [FLOAT_OP_FMV_X_W]   = _openvm_fmv_x_w,
    [FLOAT_OP_FMV_W_X]   = _openvm_fmv_w_x,
    [FLOAT_OP_FCLASS_S]  = _openvm_fclass_s,
    // Remaining entries are NULL
};
