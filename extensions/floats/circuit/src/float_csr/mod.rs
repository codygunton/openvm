pub mod adapter;
pub mod core;
pub mod execution;

#[cfg(test)]
mod tests;

use openvm_circuit::arch::{VmAirWrapper, VmChipWrapper};

// Re-export executor and AIR components
pub use adapter::FloatCsrAdapterAir;
pub use core::{FloatCsrCoreAir, FloatCsrCoreCols, FloatCsrCoreRecord, FloatCsrExecutor};
pub use execution::FloatCsrFiller;

/// Complete FloatCsr AIR (adapter + core)
pub type FloatCsrAir = VmAirWrapper<FloatCsrAdapterAir, FloatCsrCoreAir>;

/// Complete FloatCsr Chip (filler wrapper)
pub type FloatCsrChip<F> = VmChipWrapper<F, FloatCsrFiller>;
