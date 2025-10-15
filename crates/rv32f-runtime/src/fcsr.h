#ifndef OPENVM_FCSR_H
#define OPENVM_FCSR_H

#include <stdint.h>

// Rounding mode resolution
// Resolves dynamic rounding mode (rm=7) to actual FRM value
uint32_t resolve_rm(uint32_t inst_rm);

// Set hardware rounding mode
// Maps RISC-V rounding modes to C99 fenv rounding modes
void set_rounding_mode(uint32_t rm);

// Accumulate hardware exception flags into FFLAGS
// Reads hardware FP flags, converts to RISC-V format, ORs into FFLAGS
void accumulate_exceptions();

// Manual flag setting (for edge cases not caught by hardware)
void set_flag_nv();  // Invalid Operation
void set_flag_dz();  // Divide by Zero
void set_flag_of();  // Overflow
void set_flag_uf();  // Underflow
void set_flag_nx();  // Inexact

// NaN handling utilities
float canonicalize_nan_f32(float f);
float handle_nan_binary(float a, float b);
float handle_nan_ternary(float a, float b, float c);

#endif // OPENVM_FCSR_H
