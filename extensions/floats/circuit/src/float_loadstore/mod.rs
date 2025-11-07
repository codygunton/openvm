//! FloatLoadStore AIR module
//!
//! Provides AIR circuits for FLW (float load word) and FSW (float store word) instructions.
//! This module combines both load and store operations into a unified AIR similar to RV32IM's LoadStore.

pub mod adapter;
pub mod core;
pub mod execution;

use openvm_circuit::arch::{VmAirWrapper, VmChipWrapper};

// Re-export main types
pub use adapter::FloatLoadStoreAdapterAir;
pub use core::{FloatLoadStoreCoreAir, FloatLoadStoreCoreCols, FloatLoadStoreCoreRecord};
pub use execution::FloatLoadStoreFiller;

/// Complete FloatLoadStore AIR (adapter + core)
/// This AIR handles both FLW (load) and FSW (store) operations
pub type FloatLoadStoreAir = VmAirWrapper<FloatLoadStoreAdapterAir, FloatLoadStoreCoreAir>;

/// Complete FloatLoadStore Chip (filler wrapper)
pub type FloatLoadStoreChip<F> = VmChipWrapper<F, FloatLoadStoreFiller>;
