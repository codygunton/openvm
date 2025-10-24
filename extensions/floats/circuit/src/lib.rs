//! The RISC-V F extension
//!
//! **Note**: This extension does NOT implement custom ZK circuits.
//! Instead, it uses software emulation - the portable float library
//! (lib-float from Zisk) implements IEEE 754 operations using
//! standard rv32im instructions.
//!

mod constants;
mod float_load;
mod float_store;
mod float_alu;
mod float_fma;
mod float_convert;
mod float_compare;
mod float_move;
mod float_class;
mod float_csr;
pub mod extension;

pub use constants::*;
pub use extension::{Rv32F, Rv32FExecutor};
