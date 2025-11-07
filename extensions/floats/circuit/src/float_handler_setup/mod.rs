//! FloatHandlerSetup AIR and executor.
//!
//! This module implements the AIR for float handler setup operations.
//! The handler setup performs the following operations:
//! 1. Saves 31 integer registers (x1-x31) to backup memory
//! 2. Writes the instruction encoding to memory for the handler to read
//! 3. Reads the handler entry point address
//! 4. Jumps to the handler address (aligned)
//!
//! This AIR is shared by 6 different handler types:
//! - AluOp (FADD, FSUB, FMUL, FDIV, FSQRT, FMINMAX, FSGNJ)
//! - FmaOp (FMADD, FMSUB, FNMSUB, FNMADD)
//! - ClassOp (FCLASS)
//! - ConvertOp (FCVT)
//! - CompareOp (FEQ, FLT, FLE)
//! - MoveOp (FMV.X.W, FMV.W.X)
//!
//! Each handler type is registered as a separate AIR instance with different
//! opcode offsets, but they all use the same AIR implementation.

pub mod adapter;
pub mod core;
pub mod execution;

use openvm_circuit::arch::{VmAirWrapper, VmChipWrapper};

pub use adapter::FloatHandlerSetupAdapterAir;
pub use core::{FloatHandlerSetupCoreAir, FloatHandlerSetupCoreRecord};
pub use execution::FloatHandlerSetupFiller;

// Type alias for the complete FloatHandlerSetup AIR (adapter + core)
pub type FloatHandlerSetupAir = VmAirWrapper<FloatHandlerSetupAdapterAir, FloatHandlerSetupCoreAir>;

// Type alias for the complete FloatHandlerSetup Chip (filler wrapper)
pub type FloatHandlerSetupChip<F> = VmChipWrapper<F, FloatHandlerSetupFiller>;
