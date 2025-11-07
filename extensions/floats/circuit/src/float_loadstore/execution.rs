use std::borrow::BorrowMut;
use openvm_circuit::arch::*;
use openvm_circuit::system::memory::MemoryAuxColsFactory;
use openvm_stark_backend::p3_field::PrimeField32;

use super::core::{FloatLoadStoreCoreRecord, FloatLoadStoreCoreCols};

#[derive(Clone, Debug)]
pub struct FloatLoadStoreFiller;

impl FloatLoadStoreFiller {
    pub fn new() -> Self {
        Self
    }
}

impl<F: PrimeField32> TraceFiller<F> for FloatLoadStoreFiller {
    fn fill_trace_row(&self, _mem_helper: &MemoryAuxColsFactory<F>, row_slice: &mut [F]) {
        // Skip adapter columns (ExecutionState = 2 fields) to get to core
        let adapter_width = 2;
        let mut core_row = &mut row_slice[adapter_width..];
        let record: &FloatLoadStoreCoreRecord = unsafe {
            get_record_from_slice(&mut core_row, ())
        };
        let cols: &mut FloatLoadStoreCoreCols<F> = core_row.borrow_mut();

        // Fill all columns from record
        cols.base_addr = F::from_canonical_u32(record.base_addr);
        cols.imm = F::from_canonical_u16(record.imm.unsigned_abs());
        cols.imm_is_negative = F::from_bool(record.imm < 0);

        // Compute memory address with overflow detection
        let signed_imm = if record.imm < 0 {
            (record.imm as i32) as u32
        } else {
            record.imm as u32
        };
        let (mem_addr, overflow) = record.base_addr.overflowing_add(signed_imm);
        cols.mem_addr = F::from_canonical_u32(mem_addr);
        cols.addr_overflow = F::from_bool(overflow);

        // Fill float_value (4 bytes)
        let float_bytes = record.float_value.to_le_bytes();
        for i in 0..4 {
            cols.float_value[i] = F::from_canonical_u8(float_bytes[i]);
        }

        cols.is_load = F::from_bool(record.is_load);
    }
}
