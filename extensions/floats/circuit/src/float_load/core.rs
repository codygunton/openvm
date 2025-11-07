use openvm_circuit::arch::*;
use openvm_circuit::system::memory::online::TracingMemory;
use openvm_instructions::{instruction::Instruction, program::DEFAULT_PC_STEP, riscv::RV32_REGISTER_NUM_LIMBS};
use openvm_stark_backend::p3_field::PrimeField32;

use crate::constants::*;
use crate::float_loadstore::FloatLoadStoreCoreRecord;

/// FloatLoadExecutor handles FLW (float load word) instructions
#[derive(Clone, Copy)]
pub struct FloatLoadExecutor;

impl FloatLoadExecutor {
    pub fn new() -> Self {
        Self
    }
}

// PreflightExecutor implementation for traced execution (E3)
impl<F, RA> PreflightExecutor<F, RA> for FloatLoadExecutor
where
    F: PrimeField32,
    for<'buf> RA: RecordArena<'buf, MultiRowLayout<EmptyMultiRowMetadata>, &'buf mut FloatLoadStoreCoreRecord>,
{
    fn get_opcode_name(&self, _opcode: usize) -> String {
        "FLW".to_string()
    }

    fn execute(
        &self,
        state: VmStateMut<F, TracingMemory, RA>,
        instruction: &Instruction<F>,
    ) -> Result<(), ExecutionError> {
        unsafe {
            // Allocate record for trace generation
            let core_record = state.ctx.alloc(MultiRowLayout::new(EmptyMultiRowMetadata::new()));

            // Extract instruction fields (same as in execution::pre_compute_impl)
            let rd = instruction.a.as_canonical_u32() as u8;
            let rs1 = instruction.b.as_canonical_u32() as u8;  // This is rs1 * 4
            let imm_lower = instruction.c.as_canonical_u32();
            let imm_sign = instruction.g.as_canonical_u32();
            let imm = (imm_lower + imm_sign * 0xffff0000) as i32;

            // 1. Read base address from rs1 register
            let (_, base_bytes) = state.memory.read::<u8, 4, 4>(RV32_REGISTER_AS, rs1 as u32);
            let base_addr = u32::from_le_bytes(base_bytes);

            // 2. Calculate effective address
            let mem_addr = base_addr.wrapping_add(imm as u32);

            // 3. Load word from heap memory
            let (_, word_bytes) = state.memory.read::<u8, 4, 4>(FLOAT_MEM_AS, mem_addr);
            let float_value = u32::from_le_bytes(word_bytes);

            // 4. Write to float register memory
            let float_addr = float_reg_addr(rd);
        state.memory.write::<u8, 4, 4>(FLOAT_MEM_AS, float_addr, word_bytes);

            // 5. Fill the record with minimal data needed for trace generation
            core_record.base_addr = base_addr;
            core_record.imm = imm as i16;  // Truncate to i16 for record
            core_record.float_value = float_value;
            core_record.is_load = true;

            // 6. Update PC
            *state.pc = state.pc.wrapping_add(DEFAULT_PC_STEP);

            Ok(())
            }
    }
}
