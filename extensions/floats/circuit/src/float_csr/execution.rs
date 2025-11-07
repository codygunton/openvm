use std::borrow::{Borrow, BorrowMut};
use std::mem::size_of;

use openvm_circuit::arch::*;
use openvm_circuit::system::memory::online::GuestMemory;
use openvm_circuit::system::memory::MemoryAuxColsFactory;
use openvm_circuit_primitives_derive::AlignedBytesBorrow;
use openvm_instructions::{instruction::Instruction, program::DEFAULT_PC_STEP};
use openvm_stark_backend::p3_field::PrimeField32;

use crate::constants::*;

use super::core::FloatCsrExecutor;

#[derive(AlignedBytesBorrow, Clone)]
#[repr(C)]
pub struct FloatCsrPreCompute {
    rd: u8,       // Destination register (integer register x0-x31)
    rs1: u8,      // Source register (integer register x0-x31) or immediate value
    op_type: u8,  // CSR operation type: 0=RW, 1=RS, 2=RC, 3=RWI, 4=RSI, 5=RCI
    csr_addr: u8, // CSR address: 0x001=fflags, 0x002=frm, 0x003=fcsr
}

impl FloatCsrExecutor {
    /// Return true if enabled.
    fn pre_compute_impl<F: PrimeField32>(
        &self,
        _pc: u32,
        inst: &Instruction<F>,
        data: &mut FloatCsrPreCompute,
    ) -> Result<bool, StaticProgramError> {
        *data = FloatCsrPreCompute {
            rd: inst.a.as_canonical_u32() as u8,
            rs1: inst.b.as_canonical_u32() as u8,
            op_type: inst.c.as_canonical_u32() as u8,
            csr_addr: inst.d.as_canonical_u32() as u8,
        };
        Ok(true)
    }
}

macro_rules! dispatch {
    ($execute_impl:ident, $enabled:ident) => {
        if $enabled {
            Ok($execute_impl::<_, _, true>)
        } else {
            Ok($execute_impl::<_, _, false>)
        }
    };
}

impl<F> Executor<F> for FloatCsrExecutor
where
    F: PrimeField32,
{
    #[inline(always)]
    fn pre_compute_size(&self) -> usize {
        size_of::<FloatCsrPreCompute>()
    }

    #[inline(always)]
    fn pre_compute<Ctx: ExecutionCtxTrait>(
        &self,
        pc: u32,
        inst: &Instruction<F>,
        data: &mut [u8],
    ) -> Result<ExecuteFunc<F, Ctx>, StaticProgramError> {
        let data: &mut FloatCsrPreCompute = data.borrow_mut();
        let enabled = self.pre_compute_impl(pc, inst, data)?;
        dispatch!(execute_e1_handler, enabled)
    }
}

impl<F> MeteredExecutor<F> for FloatCsrExecutor
where
    F: PrimeField32,
{
    fn metered_pre_compute_size(&self) -> usize {
        size_of::<E2PreCompute<FloatCsrPreCompute>>()
    }

    fn metered_pre_compute<Ctx>(
        &self,
        chip_idx: usize,
        pc: u32,
        inst: &Instruction<F>,
        data: &mut [u8],
    ) -> Result<ExecuteFunc<F, Ctx>, StaticProgramError>
    where
        Ctx: MeteredExecutionCtxTrait,
    {
        let data: &mut E2PreCompute<FloatCsrPreCompute> = data.borrow_mut();
        data.chip_idx = chip_idx as u32;
        let enabled = self.pre_compute_impl(pc, inst, &mut data.data)?;
        dispatch!(execute_e2_handler, enabled)
    }
}

#[inline(always)]
unsafe fn execute_e12_impl<F: PrimeField32, CTX: ExecutionCtxTrait, const ENABLED: bool>(
    pre_compute: &FloatCsrPreCompute,
    instret: &mut u64,
    pc: &mut u32,
    exec_state: &mut VmExecState<F, GuestMemory, CTX>,
) {
    if !ENABLED {
        *pc += DEFAULT_PC_STEP;
        *instret += 1;
        return;
    }

    // Read current FCSR value from memory
    let fcsr_bytes = exec_state.vm_read::<u8, 4>(FLOAT_MEM_AS, FLOAT_CSR_FCSR);
    let fcsr_full = u32::from_le_bytes(fcsr_bytes);

    // Extract the appropriate field based on CSR address:
    // 0x001 (fflags): bits [4:0] - exception flags
    // 0x002 (frm):    bits [7:5] - rounding mode
    // 0x003 (fcsr):   bits [7:0] - full register
    let fcsr_old = match pre_compute.csr_addr {
        0x001 => fcsr_full & 0x1F,        // fflags: bits [4:0]
        0x002 => (fcsr_full >> 5) & 0x07, // frm: bits [7:5] shifted to [2:0]
        0x003 => fcsr_full & 0xFF,        // fcsr: bits [7:0]
        _ => {
            *pc += DEFAULT_PC_STEP;
            *instret += 1;
            return;
        }
    };

    // Process CSR operation based on op_type
    // op_type: 0=CSRRW, 1=CSRRS, 2=CSRRC, 3=CSRRWI, 4=CSRRSI, 5=CSRRCI
    let (fcsr_new, write_csr) = match pre_compute.op_type {
        0 => {
            // CSRRW: Read/Write - t=CSR; CSR=rs1; rd=t
            let rs1_bytes =
                exec_state.vm_read::<u8, 4>(RV32_REGISTER_AS, pre_compute.rs1 as u32 * 4);
            let rs1_val = u32::from_le_bytes(rs1_bytes);
            (rs1_val, true)
        }
        1 => {
            // CSRRS: Read and Set - t=CSR; CSR=t|rs1; rd=t
            // If rs1=x0, this is FRCSR (read-only, no write)
            if pre_compute.rs1 == 0 {
                (fcsr_old, false)
            } else {
                let rs1_bytes =
                    exec_state.vm_read::<u8, 4>(RV32_REGISTER_AS, pre_compute.rs1 as u32 * 4);
                let rs1_val = u32::from_le_bytes(rs1_bytes);
                (fcsr_old | rs1_val, true)
            }
        }
        2 => {
            // CSRRC: Read and Clear - t=CSR; CSR=t&~rs1; rd=t
            if pre_compute.rs1 == 0 {
                (fcsr_old, false)
            } else {
                let rs1_bytes =
                    exec_state.vm_read::<u8, 4>(RV32_REGISTER_AS, pre_compute.rs1 as u32 * 4);
                let rs1_val = u32::from_le_bytes(rs1_bytes);
                (fcsr_old & !rs1_val, true)
            }
        }
        3 => {
            // CSRRWI: Read/Write Immediate - t=CSR; CSR=zimm; rd=t
            (pre_compute.rs1 as u32, true)
        }
        4 => {
            // CSRRSI: Read and Set Immediate - t=CSR; CSR=t|zimm; rd=t
            if pre_compute.rs1 == 0 {
                (fcsr_old, false)
            } else {
                (fcsr_old | (pre_compute.rs1 as u32), true)
            }
        }
        5 => {
            // CSRRCI: Read and Clear Immediate - t=CSR; CSR=t&~zimm; rd=t
            if pre_compute.rs1 == 0 {
                (fcsr_old, false)
            } else {
                (fcsr_old & !(pre_compute.rs1 as u32), true)
            }
        }
        _ => {
            // Invalid op_type - should not happen
            (fcsr_old, false)
        }
    };

    // Write new FCSR value if needed
    if write_csr {
        // Merge the new value back into the full FCSR based on CSR address
        let fcsr_final = match pre_compute.csr_addr {
            0x001 => {
                // fflags: update bits [4:0], preserve bits [7:5]
                (fcsr_full & 0xFFFFFFE0) | (fcsr_new & 0x1F)
            }
            0x002 => {
                // frm: update bits [7:5], preserve bits [4:0]
                // fcsr_new contains the 3-bit rounding mode in bits [2:0]
                (fcsr_full & 0xFFFFFF1F) | ((fcsr_new & 0x07) << 5)
            }
            0x003 => {
                // fcsr: update bits [7:0]
                (fcsr_full & 0xFFFFFF00) | (fcsr_new & 0xFF)
            }
            _ => fcsr_full, // Should not happen, already validated above
        };

        exec_state.vm_write(FLOAT_MEM_AS, FLOAT_CSR_FCSR, &fcsr_final.to_le_bytes());
    }

    // Write old FCSR value to rd (unless rd=x0)
    if pre_compute.rd != 0 {
        exec_state.vm_write(
            RV32_REGISTER_AS,
            pre_compute.rd as u32 * 4,
            &fcsr_old.to_le_bytes(),
        );
    }

    *pc += DEFAULT_PC_STEP;
    *instret += 1;
}

#[create_handler]
#[inline(always)]
unsafe fn execute_e1_impl<F: PrimeField32, CTX: ExecutionCtxTrait, const ENABLED: bool>(
    pre_compute: &[u8],
    instret: &mut u64,
    pc: &mut u32,
    _instret_end: u64,
    exec_state: &mut VmExecState<F, GuestMemory, CTX>,
) {
    let pre_compute: &FloatCsrPreCompute = pre_compute.borrow();
    execute_e12_impl::<F, CTX, ENABLED>(pre_compute, instret, pc, exec_state);
}

#[create_handler]
#[inline(always)]
unsafe fn execute_e2_impl<F: PrimeField32, CTX: MeteredExecutionCtxTrait, const ENABLED: bool>(
    pre_compute: &[u8],
    instret: &mut u64,
    pc: &mut u32,
    _arg: u64,
    exec_state: &mut VmExecState<F, GuestMemory, CTX>,
) {
    let pre_compute: &E2PreCompute<FloatCsrPreCompute> = pre_compute.borrow();
    exec_state
        .ctx
        .on_height_change(pre_compute.chip_idx as usize, 1);
    execute_e12_impl::<F, CTX, ENABLED>(&pre_compute.data, instret, pc, exec_state);
}

use super::core::{FloatCsrCoreRecord, FloatCsrCoreCols};

#[derive(Clone, Debug)]
pub struct FloatCsrFiller;

impl FloatCsrFiller {
    pub fn new() -> Self {
        Self
    }
}

impl<F: PrimeField32> TraceFiller<F> for FloatCsrFiller {
    fn fill_trace_row(&self, _mem_helper: &MemoryAuxColsFactory<F>, row_slice: &mut [F]) {
        // SAFETY: row_slice is guaranteed to contain valid FloatCsrCoreRecord
        // Skip adapter columns (ExecutionState = 2 fields) to get to core
        let adapter_width = 2;
        let mut core_row = &mut row_slice[adapter_width..];
        let record: &FloatCsrCoreRecord = unsafe {
            get_record_from_slice(&mut core_row, ())
        };
        let cols: &mut FloatCsrCoreCols<F> = core_row.borrow_mut();

        // Fill all columns from record
        cols.csr_addr = F::from_canonical_u32(record.csr_addr);
        cols.csr_value = F::from_canonical_u32(record.csr_value);
        cols.write_value = F::from_canonical_u32(record.write_value);
        cols.is_write = F::from_bool(record.is_write);
    }
}

