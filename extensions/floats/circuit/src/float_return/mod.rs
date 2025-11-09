mod adapter;
mod core;
mod execution;
#[cfg(test)]
mod tests;

use openvm_circuit::arch::{VmAirWrapper, VmChipWrapper};

pub use adapter::FloatHandlerReturnAdapterAir;
pub use core::{
    FloatHandlerReturnCoreAir, FloatHandlerReturnCoreCols, FloatHandlerReturnCoreRecord,
    FloatHandlerReturnFiller, FloatReturnExecutor,
};

/// Complete FloatHandlerReturn AIR (adapter + core)
pub type FloatHandlerReturnAir = VmAirWrapper<FloatHandlerReturnAdapterAir, FloatHandlerReturnCoreAir>;

/// Complete FloatHandlerReturn Chip (filler wrapper)
pub type FloatHandlerReturnChip<F> = VmChipWrapper<F, FloatHandlerReturnFiller>;
