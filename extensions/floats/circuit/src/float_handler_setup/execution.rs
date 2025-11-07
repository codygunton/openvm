use std::borrow::BorrowMut;
use openvm_circuit::arch::*;
use openvm_circuit::system::memory::MemoryAuxColsFactory;
use openvm_stark_backend::p3_field::PrimeField32;

use super::core::{FloatHandlerSetupCoreRecord, FloatHandlerSetupCoreCols};

#[derive(Clone, Debug)]
pub struct FloatHandlerSetupFiller;

impl FloatHandlerSetupFiller {
    pub fn new() -> Self {
        Self
    }
}

impl<F: PrimeField32> TraceFiller<F> for FloatHandlerSetupFiller {
    fn fill_trace_row(&self, _mem_helper: &MemoryAuxColsFactory<F>, row_slice: &mut [F]) {
        // SAFETY: row_slice is guaranteed to contain valid FloatHandlerSetupCoreRecord
        // Skip adapter columns (ExecutionState = 2 fields) to get to core
        let adapter_width = 2;
        let mut core_row = &mut row_slice[adapter_width..];
        let record: &FloatHandlerSetupCoreRecord = unsafe {
            get_record_from_slice(&mut core_row, ())
        };
        let cols: &mut FloatHandlerSetupCoreCols<F> = core_row.borrow_mut();

        // Fill all columns from record - convert each field appropriately
        cols.instruction_encoding = F::from_canonical_u32(record.instruction_encoding);
        cols.handler_addr = F::from_canonical_u32(record.handler_addr);
        // Compute handler_addr_aligned by clearing the LSB (divide by 2, then multiply by 2)
        cols.handler_addr_aligned = F::from_canonical_u32(record.handler_addr & !1);
        // Copy all 31 saved registers
        for i in 0..31 {
            cols.saved_registers[i] = F::from_canonical_u32(record.saved_registers[i]);
        }
    }
}
