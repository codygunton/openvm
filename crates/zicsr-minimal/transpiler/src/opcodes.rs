use openvm_instructions::LocalOpcode;
use openvm_instructions_derive::LocalOpcode;
use strum::{EnumIter, FromRepr};

/// Opcodes for minimal Zicsr extension (FCSR registers only)
/// 
/// This enum defines the opcodes for CSR operations that are supported
/// for floating-point control and status registers (fflags, frm, fcsr).
#[derive(LocalOpcode, EnumIter, FromRepr, Clone, Copy, Debug, PartialEq, Eq)]
#[opcode_offset = 0x2C0]  // Reserve 16 opcodes before float (0x2C0-0x2CF)
#[repr(usize)]
pub enum CsrOpcode {
    /// CSRRW - Atomic Read/Write CSR
    CSRRW,
    /// CSRRS - Atomic Read and Set Bits in CSR
    CSRRS,
    /// CSRRC - Atomic Read and Clear Bits in CSR
    CSRRC,
    /// CSRRWI - Atomic Read/Write CSR (Immediate)
    CSRRWI,
    /// CSRRSI - Atomic Read and Set Bits in CSR (Immediate)
    CSRRSI,
    /// CSRRCI - Atomic Read and Clear Bits in CSR (Immediate)
    CSRRCI,
}
