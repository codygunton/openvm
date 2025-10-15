// Unified float operation handler for RV32F memory-mapped approach
// This handler provides a single entry point for all floating-point operations
// Instruction is written to FREG_INST, then handler is called to decode and execute

#include <stdint.h>

// LLVM compiler-rt soft-float functions
extern float __addsf3(float, float);
extern float __subsf3(float, float);
extern float __mulsf3(float, float);
extern float __divsf3(float, float);
extern int __eqsf2(float, float);
extern int __ltsf2(float, float);
extern int __lesf2(float, float);
extern int __fixsfsi(float);
extern unsigned int __fixunssfsi(float);
extern float __floatsisf(int);
extern float __floatunsisf(unsigned int);

// Newton-Raphson sqrt approximation (libgcc doesn't have __sqrtsf2)
static float sqrtf_approx(float x) {
    if (x == 0.0f || x != x) return x;  // Handle 0 and NaN
    if (x < 0.0f) return 0.0f / 0.0f;   // Return NaN for negative

    float guess = x;
    for (int i = 0; i < 10; i++) {
        guess = __mulsf3(__addsf3(guess, __divsf3(x, guess)), 0.5f);
    }
    return guess;
}

// Memory-mapped register definitions
#define FLOAT_REG_BASE 0x18000000  // Base address for float registers f0-f31
#define FCSR_ADDR      0x18000080  // FCSR register address (after 32 float regs × 4 bytes)
#define FREG_INST_ADDR 0x18000088  // Instruction register address

// Memory-mapped register access macros
#define freg(n) (*(volatile uint32_t *)(FLOAT_REG_BASE + ((n) * 4)))
#define fcsr (*(volatile uint32_t *)FCSR_ADDR)
#define freg_inst (*(volatile uint32_t *)FREG_INST_ADDR)

// Integer register access (TODO: OpenVM integer register interface)
#define XREG_BASE 0x00000000
#define xreg(n) (*(volatile uint32_t *)(XREG_BASE + ((n) * 4)))

// Inline helpers for float register access
static inline float read_freg_f32(uint32_t idx) {
    volatile uint32_t *reg_ptr = (volatile uint32_t *)(FLOAT_REG_BASE + (idx * 4));
    uint32_t bits = *reg_ptr;
    return *(float *)&bits;
}

static inline void write_freg_f32(uint32_t idx, float value) {
    uint32_t bits = *(uint32_t *)&value;
    volatile uint32_t *reg_ptr = (volatile uint32_t *)(FLOAT_REG_BASE + (idx * 4));
    *reg_ptr = bits;
}

// Forward declarations of helper functions
static void handle_fadd_s(uint32_t rs1, uint32_t rs2, uint32_t rd, uint32_t rm);
static void handle_fsub_s(uint32_t rs1, uint32_t rs2, uint32_t rd, uint32_t rm);
static void handle_fmul_s(uint32_t rs1, uint32_t rs2, uint32_t rd, uint32_t rm);
static void handle_fdiv_s(uint32_t rs1, uint32_t rs2, uint32_t rd, uint32_t rm);
static void handle_fsqrt_s(uint32_t rs1, uint32_t rd, uint32_t rm);
static void handle_fmin_s(uint32_t rs1, uint32_t rs2, uint32_t rd);
static void handle_fmax_s(uint32_t rs1, uint32_t rs2, uint32_t rd);
static void handle_fmadd_s(uint32_t rs1, uint32_t rs2, uint32_t rs3, uint32_t rd, uint32_t rm);
static void handle_fmsub_s(uint32_t rs1, uint32_t rs2, uint32_t rs3, uint32_t rd, uint32_t rm);
static void handle_fnmadd_s(uint32_t rs1, uint32_t rs2, uint32_t rs3, uint32_t rd, uint32_t rm);
static void handle_fnmsub_s(uint32_t rs1, uint32_t rs2, uint32_t rs3, uint32_t rd, uint32_t rm);
static void handle_fsgnj_s(uint32_t rs1, uint32_t rs2, uint32_t rd);
static void handle_fsgnjn_s(uint32_t rs1, uint32_t rs2, uint32_t rd);
static void handle_fsgnjx_s(uint32_t rs1, uint32_t rs2, uint32_t rd);
static uint32_t handle_feq_s(uint32_t rs1, uint32_t rs2);
static uint32_t handle_flt_s(uint32_t rs1, uint32_t rs2);
static uint32_t handle_fle_s(uint32_t rs1, uint32_t rs2);
static uint32_t handle_fcvt_w_s(uint32_t rs1, uint32_t rm);
static uint32_t handle_fcvt_wu_s(uint32_t rs1, uint32_t rm);
static void handle_fcvt_s_w(uint32_t rs1, uint32_t rd, uint32_t rm);
static void handle_fcvt_s_wu(uint32_t rs1, uint32_t rd, uint32_t rm);
static uint32_t handle_fmv_x_w(uint32_t rs1);
static void handle_fmv_w_x(uint32_t rs1, uint32_t rd);
static uint32_t handle_fclass_s(uint32_t rs1);

// Main handler function - entry point for all float operations
// Placed at fixed address 0x10000000 via linker script section
__attribute__((section(".float_handler")))
void _openvm_float(void) {
    // Read instruction from memory-mapped register
    uint32_t inst = freg_inst;

    // Decode instruction fields
    uint32_t opcode = inst & 0x7F;
    uint32_t rd = (inst >> 7) & 0x1F;
    uint32_t funct3 = (inst >> 12) & 0x7;
    uint32_t rs1 = (inst >> 15) & 0x1F;
    uint32_t rs2 = (inst >> 20) & 0x1F;
    uint32_t funct7 = (inst >> 25) & 0x7F;

    uint32_t result;

    switch (opcode) {
        case 0x53: // OP-FP (most float operations)
            switch (funct7) {
                case 0x00: handle_fadd_s(rs1, rs2, rd, funct3); break;
                case 0x04: handle_fsub_s(rs1, rs2, rd, funct3); break;
                case 0x08: handle_fmul_s(rs1, rs2, rd, funct3); break;
                case 0x0C: handle_fdiv_s(rs1, rs2, rd, funct3); break;
                case 0x2C: handle_fsqrt_s(rs1, rd, funct3); break;

                case 0x10: // FSGNJ.S, FSGNJN.S, FSGNJX.S
                    switch (funct3) {
                        case 0x0: handle_fsgnj_s(rs1, rs2, rd); break;
                        case 0x1: handle_fsgnjn_s(rs1, rs2, rd); break;
                        case 0x2: handle_fsgnjx_s(rs1, rs2, rd); break;
                    }
                    break;

                case 0x14: // FMIN.S, FMAX.S
                    switch (funct3) {
                        case 0x0: handle_fmin_s(rs1, rs2, rd); break;
                        case 0x1: handle_fmax_s(rs1, rs2, rd); break;
                    }
                    break;

                case 0x50: // FLE.S, FLT.S, FEQ.S
                    switch (funct3) {
                        case 0x0:
                            result = handle_fle_s(rs1, rs2);
                            xreg(rd) = result; // TODO: Proper integer register write
                            break;
                        case 0x1:
                            result = handle_flt_s(rs1, rs2);
                            xreg(rd) = result; // TODO: Proper integer register write
                            break;
                        case 0x2:
                            result = handle_feq_s(rs1, rs2);
                            xreg(rd) = result; // TODO: Proper integer register write
                            break;
                    }
                    break;

                case 0x60: // FCVT.W.S, FCVT.WU.S
                    switch (rs2) {
                        case 0x0:
                            result = handle_fcvt_w_s(rs1, funct3);
                            xreg(rd) = result; // TODO: Proper integer register write
                            break;
                        case 0x1:
                            result = handle_fcvt_wu_s(rs1, funct3);
                            xreg(rd) = result; // TODO: Proper integer register write
                            break;
                    }
                    break;

                case 0x68: // FCVT.S.W, FCVT.S.WU
                    switch (rs2) {
                        case 0x0: handle_fcvt_s_w(rs1, rd, funct3); break;
                        case 0x1: handle_fcvt_s_wu(rs1, rd, funct3); break;
                    }
                    break;

                case 0x70: // FMV.X.W, FCLASS.S
                    switch (funct3) {
                        case 0x0:
                            result = handle_fmv_x_w(rs1);
                            xreg(rd) = result; // TODO: Proper integer register write
                            break;
                        case 0x1:
                            result = handle_fclass_s(rs1);
                            xreg(rd) = result; // TODO: Proper integer register write
                            break;
                    }
                    break;

                case 0x78: // FMV.W.X
                    handle_fmv_w_x(rs1, rd);
                    break;
            }
            break;

        case 0x43: // FMADD.S
            {
                uint32_t rs3 = (inst >> 27) & 0x1F;
                handle_fmadd_s(rs1, rs2, rs3, rd, funct3);
            }
            break;

        case 0x47: // FMSUB.S
            {
                uint32_t rs3 = (inst >> 27) & 0x1F;
                handle_fmsub_s(rs1, rs2, rs3, rd, funct3);
            }
            break;

        case 0x4B: // FNMSUB.S
            {
                uint32_t rs3 = (inst >> 27) & 0x1F;
                handle_fnmsub_s(rs1, rs2, rs3, rd, funct3);
            }
            break;

        case 0x4F: // FNMADD.S
            {
                uint32_t rs3 = (inst >> 27) & 0x1F;
                handle_fnmadd_s(rs1, rs2, rs3, rd, funct3);
            }
            break;
    }

    // Return to caller via the return address set by JAL
    return;
}

// ============================================================================
// Helper function implementations
// ============================================================================

// Arithmetic operations

static void handle_fadd_s(uint32_t rs1, uint32_t rs2, uint32_t rd, uint32_t rm) {
    float a = read_freg_f32(rs1);
    float b = read_freg_f32(rs2);
    float result = __addsf3(a, b);
    write_freg_f32(rd, result);
}

static void handle_fsub_s(uint32_t rs1, uint32_t rs2, uint32_t rd, uint32_t rm) {
    float a = read_freg_f32(rs1);
    float b = read_freg_f32(rs2);
    float result = __subsf3(a, b);
    write_freg_f32(rd, result);
}

static void handle_fmul_s(uint32_t rs1, uint32_t rs2, uint32_t rd, uint32_t rm) {
    float a = read_freg_f32(rs1);
    float b = read_freg_f32(rs2);
    float result = __mulsf3(a, b);
    write_freg_f32(rd, result);
}

static void handle_fdiv_s(uint32_t rs1, uint32_t rs2, uint32_t rd, uint32_t rm) {
    float a = read_freg_f32(rs1);
    float b = read_freg_f32(rs2);
    float result = __divsf3(a, b);
    write_freg_f32(rd, result);
}

static void handle_fsqrt_s(uint32_t rs1, uint32_t rd, uint32_t rm) {
    float a = read_freg_f32(rs1);
    float result = sqrtf_approx(a);
    write_freg_f32(rd, result);
}

static void handle_fmin_s(uint32_t rs1, uint32_t rs2, uint32_t rd) {
    float a = read_freg_f32(rs1);
    float b = read_freg_f32(rs2);
    // TODO: Handle NaN cases per RISC-V spec
    float result = (a < b) ? a : b;
    write_freg_f32(rd, result);
}

static void handle_fmax_s(uint32_t rs1, uint32_t rs2, uint32_t rd) {
    float a = read_freg_f32(rs1);
    float b = read_freg_f32(rs2);
    // TODO: Handle NaN cases per RISC-V spec
    float result = (a > b) ? a : b;
    write_freg_f32(rd, result);
}

// Fused multiply-add operations

static void handle_fmadd_s(uint32_t rs1, uint32_t rs2, uint32_t rs3, uint32_t rd, uint32_t rm) {
    float a = read_freg_f32(rs1);
    float b = read_freg_f32(rs2);
    float c = read_freg_f32(rs3);
    // rd = (rs1 * rs2) + rs3
    float result = __addsf3(__mulsf3(a, b), c);
    write_freg_f32(rd, result);
}

static void handle_fmsub_s(uint32_t rs1, uint32_t rs2, uint32_t rs3, uint32_t rd, uint32_t rm) {
    float a = read_freg_f32(rs1);
    float b = read_freg_f32(rs2);
    float c = read_freg_f32(rs3);
    // rd = (rs1 * rs2) - rs3
    float result = __subsf3(__mulsf3(a, b), c);
    write_freg_f32(rd, result);
}

static void handle_fnmadd_s(uint32_t rs1, uint32_t rs2, uint32_t rs3, uint32_t rd, uint32_t rm) {
    float a = read_freg_f32(rs1);
    float b = read_freg_f32(rs2);
    float c = read_freg_f32(rs3);
    // rd = -(rs1 * rs2) - rs3
    float result = -__addsf3(__mulsf3(a, b), c);
    write_freg_f32(rd, result);
}

static void handle_fnmsub_s(uint32_t rs1, uint32_t rs2, uint32_t rs3, uint32_t rd, uint32_t rm) {
    float a = read_freg_f32(rs1);
    float b = read_freg_f32(rs2);
    float c = read_freg_f32(rs3);
    // rd = -(rs1 * rs2) + rs3
    float result = __subsf3(c, __mulsf3(a, b));
    write_freg_f32(rd, result);
}

// Sign injection operations

static void handle_fsgnj_s(uint32_t rs1, uint32_t rs2, uint32_t rd) {
    uint32_t a = freg(rs1);
    uint32_t b = freg(rs2);
    // Copy magnitude from rs1, sign from rs2
    uint32_t result = (a & 0x7FFFFFFF) | (b & 0x80000000);
    freg(rd) = result;
}

static void handle_fsgnjn_s(uint32_t rs1, uint32_t rs2, uint32_t rd) {
    uint32_t a = freg(rs1);
    uint32_t b = freg(rs2);
    // Copy magnitude from rs1, negated sign from rs2
    uint32_t result = (a & 0x7FFFFFFF) | (~b & 0x80000000);
    freg(rd) = result;
}

static void handle_fsgnjx_s(uint32_t rs1, uint32_t rs2, uint32_t rd) {
    uint32_t a = freg(rs1);
    uint32_t b = freg(rs2);
    // XOR signs of rs1 and rs2
    uint32_t result = a ^ (b & 0x80000000);
    freg(rd) = result;
}

// Comparison operations (return integer results)

static uint32_t handle_feq_s(uint32_t rs1, uint32_t rs2) {
    float a = read_freg_f32(rs1);
    float b = read_freg_f32(rs2);
    return (__eqsf2(a, b) == 0) ? 1 : 0;
}

static uint32_t handle_flt_s(uint32_t rs1, uint32_t rs2) {
    float a = read_freg_f32(rs1);
    float b = read_freg_f32(rs2);
    return (__ltsf2(a, b) < 0) ? 1 : 0;
}

static uint32_t handle_fle_s(uint32_t rs1, uint32_t rs2) {
    float a = read_freg_f32(rs1);
    float b = read_freg_f32(rs2);
    return (__lesf2(a, b) <= 0) ? 1 : 0;
}

// Conversion operations

static uint32_t handle_fcvt_w_s(uint32_t rs1, uint32_t rm) {
    float a = read_freg_f32(rs1);
    int32_t result = __fixsfsi(a);
    return (uint32_t)result;
}

static uint32_t handle_fcvt_wu_s(uint32_t rs1, uint32_t rm) {
    float a = read_freg_f32(rs1);
    uint32_t result = __fixunssfsi(a);
    return result;
}

static void handle_fcvt_s_w(uint32_t rs1, uint32_t rd, uint32_t rm) {
    // TODO: Read from integer register
    int32_t a = (int32_t)xreg(rs1);
    float result = __floatsisf(a);
    write_freg_f32(rd, result);
}

static void handle_fcvt_s_wu(uint32_t rs1, uint32_t rd, uint32_t rm) {
    // TODO: Read from integer register
    uint32_t a = xreg(rs1);
    float result = __floatunsisf(a);
    write_freg_f32(rd, result);
}

// Move operations

static uint32_t handle_fmv_x_w(uint32_t rs1) {
    // Move float register bits to integer register
    uint32_t bits = freg(rs1);
    return bits;
}

static void handle_fmv_w_x(uint32_t rs1, uint32_t rd) {
    // Move integer register bits to float register
    // TODO: Read from integer register
    uint32_t bits = xreg(rs1);
    freg(rd) = bits;
}

// Classification operation

static uint32_t handle_fclass_s(uint32_t rs1) {
    uint32_t bits = freg(rs1);
    uint32_t result = 0;

    // Extract sign, exponent, and mantissa
    uint32_t sign = (bits >> 31) & 1;
    uint32_t exp = (bits >> 23) & 0xFF;
    uint32_t mant = bits & 0x7FFFFF;

    // Classify according to IEEE 754
    if (exp == 0) {
        if (mant == 0) {
            // Zero
            result = sign ? (1 << 3) : (1 << 4);  // Bit 3: -0, Bit 4: +0
        } else {
            // Subnormal
            result = sign ? (1 << 2) : (1 << 5);  // Bit 2: -subnormal, Bit 5: +subnormal
        }
    } else if (exp == 0xFF) {
        if (mant == 0) {
            // Infinity
            result = sign ? (1 << 0) : (1 << 7);  // Bit 0: -inf, Bit 7: +inf
        } else {
            // NaN
            if (mant & 0x400000) {
                result = (1 << 9);  // Bit 9: quiet NaN
            } else {
                result = (1 << 8);  // Bit 8: signaling NaN
            }
        }
    } else {
        // Normal
        result = sign ? (1 << 1) : (1 << 6);  // Bit 1: -normal, Bit 6: +normal
    }

    return result;
}
