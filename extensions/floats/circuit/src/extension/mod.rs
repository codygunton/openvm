// Minimal extension integration for float support
//
// Unlike other extensions (keccak, ecc), floats don't add custom chips.
// The float library is compiled to rv32im and executes using existing chips.
//
// This extension mainly serves to document the memory layout used by the
// transpiler and guest library. All float execution happens through rv32im
// chips, with the transpiler intercepting float instructions.

use openvm_floats_guest::{FREG_CSR, FREG_FIRST, FREG_INST};

#[derive(Clone, Debug, Default)]
pub struct FloatsExtension {
    // ========================================================================
    // MEMORY ADDRESS CONFIGURATION - IMPORTED FROM GUEST LIBRARY
    // ========================================================================
    // NO COUPLING: These addresses are imported from openvm-floats-guest,
    // which is the single source of truth matching Zisk naming.
    // ========================================================================
    /// Base address for float registers f0-f31 (256 bytes)
    pub freg_first: u32,

    /// Address where transpiler writes instructions
    pub freg_inst: u32,

    /// Address for fcsr control/status register
    pub freg_csr: u32,

    /// Base address where compiled lib-float code is loaded
    /// TODO: Should match riscv_float_execute() symbol address
    pub float_lib_base: u32,
}

impl FloatsExtension {
    pub fn new() -> Self {
        Self {
            // Import addresses from guest library (single source of truth)
            freg_first: FREG_FIRST,
            freg_inst: FREG_INST,
            freg_csr: FREG_CSR,

            // Library load address (not part of Zisk constants)
            float_lib_base: 0x87F00000,
        }
    }
}

// Note: FloatsExtension does NOT implement VmCircuitExtension, VmExecutionExtension,
// or VmProverExtension because it doesn't add custom chips. Float instructions are
// intercepted by FloatsTranspilerExtension and executed via the software library
// using existing rv32im chips.
