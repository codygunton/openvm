// FCSR (Floating-Point Control and Status Register) Implementation
// Provides rounding mode control and exception flag management for RV32F
//
// FCSR Layout (32-bit):
// - Bits [7:5]: FRM (Floating-point Rounding Mode)
// - Bits [4:0]: FFLAGS (Floating-point exception Flags)
//
// Rounding Modes (FRM):
//   0 (RNE): Round to Nearest, ties to Even
//   1 (RTZ): Round towards Zero
//   2 (RDN): Round Down (towards -∞)
//   3 (RUP): Round Up (towards +∞)
//   4 (RMM): Round to Nearest, ties to Max Magnitude
//   7 (DYN): Dynamic (use frm CSR value)
//
// Exception Flags (FFLAGS):
//   Bit 0 (NV): Invalid Operation
//   Bit 1 (DZ): Divide by Zero
//   Bit 2 (OF): Overflow
//   Bit 3 (UF): Underflow
//   Bit 4 (NX): Inexact

#include <stdint.h>
#include <fenv.h>

// FCSR register address (memory-mapped)
#define FCSR_ADDR 0xC0000080
#define fcsr (*(volatile uint32_t *)FCSR_ADDR)

// FRM register address (upper 3 bits of FCSR)
#define FRM_ADDR 0xC0000084
#define frm (*(volatile uint32_t *)FRM_ADDR)

// FFLAGS register address (lower 5 bits of FCSR)
#define FFLAGS_ADDR 0xC0000088
#define fflags (*(volatile uint32_t *)FFLAGS_ADDR)

// Rounding mode values
#define RM_RNE 0  // Round to Nearest, ties to Even
#define RM_RTZ 1  // Round towards Zero
#define RM_RDN 2  // Round Down (towards -∞)
#define RM_RUP 3  // Round Up (towards +∞)
#define RM_RMM 4  // Round to Nearest, ties to Max Magnitude
#define RM_DYN 7  // Dynamic (use FRM register)

// Exception flag bits
#define FLAG_NV 0x01  // Invalid Operation
#define FLAG_DZ 0x02  // Divide by Zero
#define FLAG_OF 0x04  // Overflow
#define FLAG_UF 0x08  // Underflow
#define FLAG_NX 0x10  // Inexact

// FCSR bit positions
#define FCSR_FFLAGS_SHIFT 0
#define FCSR_FRM_SHIFT 5
#define FCSR_FFLAGS_MASK 0x1F
#define FCSR_FRM_MASK 0x07

// Read the current rounding mode from FRM register
static inline uint32_t get_frm() {
    return (fcsr >> FCSR_FRM_SHIFT) & FCSR_FRM_MASK;
}

// Read the current exception flags from FFLAGS register
static inline uint32_t get_fflags() {
    return fcsr & FCSR_FFLAGS_MASK;
}

// Set exception flags (OR-based accumulation)
static inline void set_fflags(uint32_t flags) {
    uint32_t current = fcsr;
    // OR the new flags into the existing flags (accumulation)
    uint32_t new_flags = (current & FCSR_FFLAGS_MASK) | (flags & FCSR_FFLAGS_MASK);
    // Update FCSR with new flags, preserving FRM
    fcsr = (current & ~FCSR_FFLAGS_MASK) | new_flags;
}

// Clear all exception flags
static inline void clear_fflags() {
    fcsr = fcsr & ~FCSR_FFLAGS_MASK;
}

// Resolve dynamic rounding mode (rm=7)
// Returns the actual rounding mode to use for an operation
uint32_t resolve_rm(uint32_t inst_rm) {
    // If instruction specifies dynamic rounding (7), use FRM register
    if (inst_rm == RM_DYN) {
        return get_frm();
    }
    // Otherwise use the instruction-encoded rounding mode
    return inst_rm;
}

// Set hardware rounding mode using fenv.h
// Maps RISC-V rounding modes to C99 fenv rounding modes
void set_rounding_mode(uint32_t rm) {
    int fe_rm;
    switch (rm) {
        case RM_RNE:  // Round to Nearest, ties to Even
            fe_rm = FE_TONEAREST;
            break;
        case RM_RTZ:  // Round towards Zero
            fe_rm = FE_TOWARDZERO;
            break;
        case RM_RDN:  // Round Down (towards -∞)
            fe_rm = FE_DOWNWARD;
            break;
        case RM_RUP:  // Round Up (towards +∞)
            fe_rm = FE_UPWARD;
            break;
        case RM_RMM:  // Round to Nearest, ties to Max Magnitude
            // Note: RMM is not directly supported by C99 fenv
            // We use FE_TONEAREST as approximation
            // For full compliance, would need custom rounding implementation
            fe_rm = FE_TONEAREST;
            break;
        default:
            // Invalid rounding mode, use default (RNE)
            fe_rm = FE_TONEAREST;
            break;
    }
    fesetround(fe_rm);
}

// Check for hardware exception flags and accumulate them
// This reads the hardware FP exception flags and ORs them into FFLAGS
void accumulate_exceptions() {
    int hw_flags = fetestexcept(FE_ALL_EXCEPT);
    uint32_t riscv_flags = 0;

    // Map C99 fenv flags to RISC-V FFLAGS
    if (hw_flags & FE_INVALID)   riscv_flags |= FLAG_NV;
    if (hw_flags & FE_DIVBYZERO) riscv_flags |= FLAG_DZ;
    if (hw_flags & FE_OVERFLOW)  riscv_flags |= FLAG_OF;
    if (hw_flags & FE_UNDERFLOW) riscv_flags |= FLAG_UF;
    if (hw_flags & FE_INEXACT)   riscv_flags |= FLAG_NX;

    // Accumulate flags into FFLAGS (OR-based)
    set_fflags(riscv_flags);

    // Clear hardware flags for next operation
    feclearexcept(FE_ALL_EXCEPT);
}

// Manual flag setting for operations that don't raise hardware exceptions
// (e.g., NaN propagation, sign injection edge cases)
void set_flag_nv() { set_fflags(FLAG_NV); }
void set_flag_dz() { set_fflags(FLAG_DZ); }
void set_flag_of() { set_fflags(FLAG_OF); }
void set_flag_uf() { set_fflags(FLAG_UF); }
void set_flag_nx() { set_fflags(FLAG_NX); }

// Check if a float is NaN
static inline int is_nan_f32(float f) {
    uint32_t bits = *(uint32_t *)&f;
    uint32_t exp = (bits >> 23) & 0xFF;
    uint32_t frac = bits & 0x7FFFFF;
    return (exp == 0xFF) && (frac != 0);
}

// Check if a float is signaling NaN
static inline int is_snan_f32(float f) {
    uint32_t bits = *(uint32_t *)&f;
    uint32_t exp = (bits >> 23) & 0xFF;
    uint32_t frac = bits & 0x7FFFFF;
    // sNaN: exp=0xFF, frac!=0, frac[22]=0 (MSB of mantissa is 0)
    return (exp == 0xFF) && (frac != 0) && ((frac & 0x400000) == 0);
}

// Check if a float is quiet NaN
static inline int is_qnan_f32(float f) {
    uint32_t bits = *(uint32_t *)&f;
    uint32_t exp = (bits >> 23) & 0xFF;
    uint32_t frac = bits & 0x7FFFFF;
    // qNaN: exp=0xFF, frac[22]=1 (MSB of mantissa is 1)
    return (exp == 0xFF) && ((frac & 0x400000) != 0);
}

// Canonicalize NaN to RISC-V standard (0x7FC00000)
float canonicalize_nan_f32(float f) {
    if (is_nan_f32(f)) {
        // RISC-V canonical NaN: sign=0, exp=0xFF, frac=0x400000
        uint32_t canonical = 0x7FC00000;
        return *(float *)&canonical;
    }
    return f;
}

// Handle NaN propagation for binary operations
// Sets NV flag if any operand is sNaN
// Returns canonical NaN if any operand is NaN
float handle_nan_binary(float a, float b) {
    int a_is_snan = is_snan_f32(a);
    int b_is_snan = is_snan_f32(b);
    int a_is_nan = is_nan_f32(a);
    int b_is_nan = is_nan_f32(b);

    // If either operand is sNaN, set invalid flag
    if (a_is_snan || b_is_snan) {
        set_flag_nv();
    }

    // If any operand is NaN (quiet or signaling), return canonical NaN
    if (a_is_nan || b_is_nan) {
        uint32_t canonical = 0x7FC00000;
        return *(float *)&canonical;
    }

    // No NaN, return a (doesn't matter, caller will do the actual operation)
    return a;
}

// Handle NaN propagation for ternary operations (FMA)
// Sets NV flag if any operand is sNaN
// Returns canonical NaN if any operand is NaN
float handle_nan_ternary(float a, float b, float c) {
    int a_is_snan = is_snan_f32(a);
    int b_is_snan = is_snan_f32(b);
    int c_is_snan = is_snan_f32(c);
    int a_is_nan = is_nan_f32(a);
    int b_is_nan = is_nan_f32(b);
    int c_is_nan = is_nan_f32(c);

    // If any operand is sNaN, set invalid flag
    if (a_is_snan || b_is_snan || c_is_snan) {
        set_flag_nv();
    }

    // If any operand is NaN (quiet or signaling), return canonical NaN
    if (a_is_nan || b_is_nan || c_is_nan) {
        uint32_t canonical = 0x7FC00000;
        return *(float *)&canonical;
    }

    // No NaN, return a (doesn't matter, caller will do the actual operation)
    return a;
}
