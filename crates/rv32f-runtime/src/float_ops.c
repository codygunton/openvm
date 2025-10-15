/*
 * DEPRECATED: This file is NO LONGER USED in the memory-mapped implementation
 *
 * This file was part of the original dispatch table approach where each floating-point
 * operation had a separate function called via JALR through a function pointer table.
 *
 * In the NEW memory-mapped approach, all float operations are handled inline within
 * a single handler function in handler.c. The handler:
 * - Reads the original RISC-V instruction from memory-mapped register FREG_INST
 * - Decodes the instruction using a switch statement
 * - Executes the operation inline with all logic contained in handler.c
 * - Is self-contained with all 24 F extension operations
 *
 * This file is preserved as a reference for the operation implementations.
 * DO NOT compile this file - it is excluded from build.rs
 *
 * See: handler.c for the current implementation
 * See: ai_plans/rv32f-memory-mapped-approach.md for the architectural design
 */

#if 0  // ENTIRE FILE COMMENTED OUT - NOT COMPILED

// F extension floating-point operation wrappers for RV32IMF
// Adds single-precision floating-point to OpenVM's RV32IM base
// Each function is a thin wrapper around LLVM compiler-rt soft-float implementations
// with FCSR (rounding mode + exception flags) support

#include <stdint.h>
#include "fcsr.h"

// External compiler-rt soft-float functions
extern float __addsf3(float, float);
extern float __subsf3(float, float);
extern float __mulsf3(float, float);
extern float __divsf3(float, float);
extern float __sqrtsf2(float);
extern int __eqsf2(float, float);
extern int __ltsf2(float, float);
extern int __lesf2(float, float);
extern int __fixsfsi(float);
extern unsigned int __fixunssfsi(float);
extern float __floatsisf(int);
extern float __floatunsisf(unsigned int);

// Float register file (memory-mapped within VM's 512MB address space)
// Changed from 0xC0000000 which was outside VM memory bounds
#define FREG_BASE 0x18000000  // 384MB - safe within 512MB limit
#define freg(n) (*(volatile uint32_t *)(FREG_BASE + ((n) * 4)))

// Integer register file (memory-mapped at 0xA0000000)
// Each register is 4 bytes, but OpenVM uses 16-byte aligned slots
#define XREG_BASE 0xA0000000
#define xreg(n) (*(volatile uint32_t *)(XREG_BASE + ((n) * 16)))

// FCSR register (at 0xC0000080)
#define FCSR_ADDR 0xC0000080
#define fcsr (*(volatile uint32_t *)FCSR_ADDR)

// Helper to read float register as float
static inline float read_freg_f32(uint32_t idx) {
    uint32_t bits = freg(idx);
    return *(float *)&bits;
}

// Helper to write float register from float
static inline void write_freg_f32(uint32_t idx, float value) {
    uint32_t bits = *(uint32_t *)&value;
    freg(idx) = bits;
}

// Arithmetic operations

void _openvm_fadd_s(uint32_t rs1, uint32_t rs2, uint32_t rd, uint32_t rm) {
    float a = read_freg_f32(rs1);
    float b = read_freg_f32(rs2);

    // Resolve dynamic rounding mode and set hardware rounding
    uint32_t actual_rm = resolve_rm(rm);
    set_rounding_mode(actual_rm);

    // Perform operation
    float result = __addsf3(a, b);

    // Accumulate exception flags
    accumulate_exceptions();

    write_freg_f32(rd, result);
}

void _openvm_fsub_s(uint32_t rs1, uint32_t rs2, uint32_t rd, uint32_t rm) {
    float a = read_freg_f32(rs1);
    float b = read_freg_f32(rs2);

    // Resolve dynamic rounding mode and set hardware rounding
    uint32_t actual_rm = resolve_rm(rm);
    set_rounding_mode(actual_rm);

    // Perform operation
    float result = __subsf3(a, b);

    // Accumulate exception flags
    accumulate_exceptions();

    write_freg_f32(rd, result);
}

void _openvm_fmul_s(uint32_t rs1, uint32_t rs2, uint32_t rd, uint32_t rm) {
    float a = read_freg_f32(rs1);
    float b = read_freg_f32(rs2);

    // Resolve dynamic rounding mode and set hardware rounding
    uint32_t actual_rm = resolve_rm(rm);
    set_rounding_mode(actual_rm);

    // Perform operation
    float result = __mulsf3(a, b);

    // Accumulate exception flags
    accumulate_exceptions();

    write_freg_f32(rd, result);
}

void _openvm_fdiv_s(uint32_t rs1, uint32_t rs2, uint32_t rd, uint32_t rm) {
    float a = read_freg_f32(rs1);
    float b = read_freg_f32(rs2);

    // Resolve dynamic rounding mode and set hardware rounding
    uint32_t actual_rm = resolve_rm(rm);
    set_rounding_mode(actual_rm);

    // Perform operation
    float result = __divsf3(a, b);

    // Accumulate exception flags
    accumulate_exceptions();

    write_freg_f32(rd, result);
}

void _openvm_fsqrt_s(uint32_t rs1, uint32_t rs2, uint32_t rd, uint32_t rm) {
    float a = read_freg_f32(rs1);

    // Resolve dynamic rounding mode and set hardware rounding
    uint32_t actual_rm = resolve_rm(rm);
    set_rounding_mode(actual_rm);

    // Perform operation
    float result = __sqrtsf2(a);

    // Accumulate exception flags
    accumulate_exceptions();

    write_freg_f32(rd, result);
}

void _openvm_fmin_s(uint32_t rs1, uint32_t rs2, uint32_t rd, uint32_t rm) {
    float a = read_freg_f32(rs1);
    float b = read_freg_f32(rs2);

    // Resolve dynamic rounding mode and set hardware rounding
    uint32_t actual_rm = resolve_rm(rm);
    set_rounding_mode(actual_rm);

    // Handle NaN: if one is NaN, return the other; if both NaN, return canonical
    float result;
    if (__builtin_isnan(a) && __builtin_isnan(b)) {
        result = canonicalize_nan_f32(a);
    } else if (__builtin_isnan(a)) {
        result = b;
    } else if (__builtin_isnan(b)) {
        result = a;
    } else {
        result = (a < b) ? a : b;
    }

    // Accumulate exception flags
    accumulate_exceptions();

    write_freg_f32(rd, result);
}

void _openvm_fmax_s(uint32_t rs1, uint32_t rs2, uint32_t rd, uint32_t rm) {
    float a = read_freg_f32(rs1);
    float b = read_freg_f32(rs2);

    // Resolve dynamic rounding mode and set hardware rounding
    uint32_t actual_rm = resolve_rm(rm);
    set_rounding_mode(actual_rm);

    // Handle NaN: if one is NaN, return the other; if both NaN, return canonical
    float result;
    if (__builtin_isnan(a) && __builtin_isnan(b)) {
        result = canonicalize_nan_f32(a);
    } else if (__builtin_isnan(a)) {
        result = b;
    } else if (__builtin_isnan(b)) {
        result = a;
    } else {
        result = (a > b) ? a : b;
    }

    // Accumulate exception flags
    accumulate_exceptions();

    write_freg_f32(rd, result);
}

// Fused multiply-add operations
// Note: compiler-rt doesn't have fma, so we use separate mul+add (less precise)

void _openvm_fmadd_s(uint32_t rs1, uint32_t rs2, uint32_t rd, uint32_t rm) {
    // rd = (rs1 * rs2) + rs3, but rs3 is passed as rd initial value
    float a = read_freg_f32(rs1);
    float b = read_freg_f32(rs2);
    float c = read_freg_f32(rd);

    // Resolve dynamic rounding mode and set hardware rounding
    uint32_t actual_rm = resolve_rm(rm);
    set_rounding_mode(actual_rm);

    // Perform operation: (a * b) + c
    // Note: This uses separate mul+add which is less precise than true FMA
    // For full RISC-V compliance, would need hardware FMA or software implementation
    float result = __addsf3(__mulsf3(a, b), c);

    // Accumulate exception flags
    accumulate_exceptions();

    write_freg_f32(rd, result);
}

void _openvm_fmsub_s(uint32_t rs1, uint32_t rs2, uint32_t rd, uint32_t rm) {
    // rd = (rs1 * rs2) - rs3
    float a = read_freg_f32(rs1);
    float b = read_freg_f32(rs2);
    float c = read_freg_f32(rd);

    // Resolve dynamic rounding mode and set hardware rounding
    uint32_t actual_rm = resolve_rm(rm);
    set_rounding_mode(actual_rm);

    // Perform operation: (a * b) - c
    float result = __subsf3(__mulsf3(a, b), c);

    // Accumulate exception flags
    accumulate_exceptions();

    write_freg_f32(rd, result);
}

void _openvm_fnmadd_s(uint32_t rs1, uint32_t rs2, uint32_t rd, uint32_t rm) {
    // rd = -((rs1 * rs2) + rs3)
    float a = read_freg_f32(rs1);
    float b = read_freg_f32(rs2);
    float c = read_freg_f32(rd);

    // Resolve dynamic rounding mode and set hardware rounding
    uint32_t actual_rm = resolve_rm(rm);
    set_rounding_mode(actual_rm);

    // Perform operation: -((a * b) + c)
    float result = -__addsf3(__mulsf3(a, b), c);

    // Accumulate exception flags
    accumulate_exceptions();

    write_freg_f32(rd, result);
}

void _openvm_fnmsub_s(uint32_t rs1, uint32_t rs2, uint32_t rd, uint32_t rm) {
    // rd = -((rs1 * rs2) - rs3)
    float a = read_freg_f32(rs1);
    float b = read_freg_f32(rs2);
    float c = read_freg_f32(rd);

    // Resolve dynamic rounding mode and set hardware rounding
    uint32_t actual_rm = resolve_rm(rm);
    set_rounding_mode(actual_rm);

    // Perform operation: -((a * b) - c)
    float result = -__subsf3(__mulsf3(a, b), c);

    // Accumulate exception flags
    accumulate_exceptions();

    write_freg_f32(rd, result);
}

// Sign injection operations
// These are bitwise operations that don't set exception flags or use rounding modes

void _openvm_fsgnj_s(uint32_t rs1, uint32_t rs2, uint32_t rd, uint32_t rm) {
    uint32_t a = freg(rs1);
    uint32_t b = freg(rs2);
    uint32_t result = (a & 0x7FFFFFFF) | (b & 0x80000000);
    freg(rd) = result;
    // No FCSR interaction: sign injection doesn't set flags
}

void _openvm_fsgnjn_s(uint32_t rs1, uint32_t rs2, uint32_t rd, uint32_t rm) {
    uint32_t a = freg(rs1);
    uint32_t b = freg(rs2);
    uint32_t result = (a & 0x7FFFFFFF) | (~b & 0x80000000);
    freg(rd) = result;
    // No FCSR interaction: sign injection doesn't set flags
}

void _openvm_fsgnjx_s(uint32_t rs1, uint32_t rs2, uint32_t rd, uint32_t rm) {
    uint32_t a = freg(rs1);
    uint32_t b = freg(rs2);
    uint32_t result = a ^ (b & 0x80000000);
    freg(rd) = result;
    // No FCSR interaction: sign injection doesn't set flags
}

// Comparison operations (write to integer registers)
// Note: rd is an integer register index for these
// Set NV flag if either operand is a signaling NaN

void _openvm_feq_s(uint32_t rs1, uint32_t rs2, uint32_t rd, uint32_t rm) {
    float a = read_freg_f32(rs1);
    float b = read_freg_f32(rs2);

    // Check for signaling NaN and set NV flag
    if (__builtin_isnan(a) || __builtin_isnan(b)) {
        set_flag_nv();
    }

    // Perform comparison (NaN always compares false for equality)
    uint32_t result = (__eqsf2(a, b) == 0) ? 1 : 0;

    // Write result to integer register
    xreg(rd) = result;
}

void _openvm_flt_s(uint32_t rs1, uint32_t rs2, uint32_t rd, uint32_t rm) {
    float a = read_freg_f32(rs1);
    float b = read_freg_f32(rs2);

    // Check for signaling NaN and set NV flag
    if (__builtin_isnan(a) || __builtin_isnan(b)) {
        set_flag_nv();
    }

    // Perform comparison (NaN always compares false for less-than)
    uint32_t result = (__ltsf2(a, b) < 0) ? 1 : 0;

    // Write result to integer register
    xreg(rd) = result;
}

void _openvm_fle_s(uint32_t rs1, uint32_t rs2, uint32_t rd, uint32_t rm) {
    float a = read_freg_f32(rs1);
    float b = read_freg_f32(rs2);

    // Check for signaling NaN and set NV flag
    if (__builtin_isnan(a) || __builtin_isnan(b)) {
        set_flag_nv();
    }

    // Perform comparison (NaN always compares false for less-than-or-equal)
    uint32_t result = (__lesf2(a, b) <= 0) ? 1 : 0;

    // Write result to integer register
    xreg(rd) = result;
}

// Conversion operations
// These convert between float and integer, using rounding modes and setting flags

void _openvm_fcvt_w_s(uint32_t rs1, uint32_t rs2, uint32_t rd, uint32_t rm) {
    float a = read_freg_f32(rs1);

    // Resolve dynamic rounding mode and set hardware rounding
    uint32_t actual_rm = resolve_rm(rm);
    set_rounding_mode(actual_rm);

    // Convert float to signed integer
    int32_t result = __fixsfsi(a);

    // Accumulate exception flags
    accumulate_exceptions();

    // Write result to integer register
    xreg(rd) = (uint32_t)result;
}

void _openvm_fcvt_wu_s(uint32_t rs1, uint32_t rs2, uint32_t rd, uint32_t rm) {
    float a = read_freg_f32(rs1);

    // Resolve dynamic rounding mode and set hardware rounding
    uint32_t actual_rm = resolve_rm(rm);
    set_rounding_mode(actual_rm);

    // Convert float to unsigned integer
    uint32_t result = __fixunssfsi(a);

    // Accumulate exception flags
    accumulate_exceptions();

    // Write result to integer register
    xreg(rd) = result;
}

void _openvm_fcvt_s_w(uint32_t rs1, uint32_t rs2, uint32_t rd, uint32_t rm) {
    // Read from integer register
    int32_t a = (int32_t)xreg(rs1);

    // Resolve dynamic rounding mode and set hardware rounding
    uint32_t actual_rm = resolve_rm(rm);
    set_rounding_mode(actual_rm);

    // Convert signed integer to float
    float result = __floatsisf(a);

    // Accumulate exception flags
    accumulate_exceptions();

    write_freg_f32(rd, result);
}

void _openvm_fcvt_s_wu(uint32_t rs1, uint32_t rs2, uint32_t rd, uint32_t rm) {
    // Read from integer register
    uint32_t a = xreg(rs1);

    // Resolve dynamic rounding mode and set hardware rounding
    uint32_t actual_rm = resolve_rm(rm);
    set_rounding_mode(actual_rm);

    // Convert unsigned integer to float
    float result = __floatunsisf(a);

    // Accumulate exception flags
    accumulate_exceptions();

    write_freg_f32(rd, result);
}

// Move operations
// These are bitwise moves that don't set exception flags or use rounding modes

void _openvm_fmv_x_w(uint32_t rs1, uint32_t rs2, uint32_t rd, uint32_t rm) {
    // Move bits from float register to integer register (bitcast)
    uint32_t bits = freg(rs1);
    xreg(rd) = bits;
    // No FCSR interaction: bitwise move doesn't set flags
}

void _openvm_fmv_w_x(uint32_t rs1, uint32_t rs2, uint32_t rd, uint32_t rm) {
    // Move bits from integer register to float register (bitcast)
    uint32_t bits = xreg(rs1);
    freg(rd) = bits;
    // No FCSR interaction: bitwise move doesn't set flags
}

// Classify operation
// Classifies a float into one of 10 categories, writing a 10-bit mask to integer register

void _openvm_fclass_s(uint32_t rs1, uint32_t rs2, uint32_t rd, uint32_t rm) {
    uint32_t bits = freg(rs1);
    uint32_t result = 0;

    // Extract IEEE 754 components
    uint32_t sign = (bits >> 31) & 1;
    uint32_t exp = (bits >> 23) & 0xFF;
    uint32_t frac = bits & 0x7FFFFF;

    // Classify according to RISC-V spec
    if (exp == 0xFF) {
        // Infinity or NaN
        if (frac == 0) {
            // Infinity
            result = sign ? (1 << 0) : (1 << 7);  // Bit 0: -inf, Bit 7: +inf
        } else {
            // NaN (signaling if MSB of frac is 0, quiet if MSB is 1)
            if ((frac & 0x400000) == 0) {
                result = 1 << 8;  // Bit 8: signaling NaN
            } else {
                result = 1 << 9;  // Bit 9: quiet NaN
            }
        }
    } else if (exp == 0) {
        // Zero or subnormal
        if (frac == 0) {
            // Zero
            result = sign ? (1 << 3) : (1 << 4);  // Bit 3: -0, Bit 4: +0
        } else {
            // Subnormal
            result = sign ? (1 << 2) : (1 << 5);  // Bit 2: -subnormal, Bit 5: +subnormal
        }
    } else {
        // Normal
        result = sign ? (1 << 1) : (1 << 6);  // Bit 1: -normal, Bit 6: +normal
    }

    // Write result to integer register
    xreg(rd) = result;
    // No FCSR interaction: classification doesn't set flags
}

#endif  // End of commented out code
