use openvm_circuit::arch::*;
use openvm_circuit::system::memory::online::TracingMemory;
use openvm_instructions::instruction::Instruction;
use openvm_stark_backend::p3_field::PrimeField32;

/// FloatReturnExecutor handles the custom FLOAT_RETURN instruction
/// This instruction restores caller-saved registers after returning from the float handler
#[derive(Clone, Copy)]
pub struct FloatReturnExecutor;

impl FloatReturnExecutor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FloatReturnExecutor {
    fn default() -> Self {
        Self::new()
    }
}

// Stub implementation - FLOAT_RETURN operations are handled by Executor trait
impl<F, RA> PreflightExecutor<F, RA> for FloatReturnExecutor
where
    F: PrimeField32,
    RA: Arena,
{
    fn get_opcode_name(&self, opcode: usize) -> String {
        match opcode {
            0x314 => "FLOAT_RETURN".to_string(),
            _ => format!("UnknownFloatReturn(0x{:x})", opcode),
        }
    }

    fn execute(
        &self,
        _state: VmStateMut<F, TracingMemory, RA>,
        _instruction: &Instruction<F>,
    ) -> Result<(), ExecutionError> {
        // FLOAT_RETURN operations use Executor trait, not PreflightExecutor
        panic!("FLOAT_RETURN operations should use Executor trait, not PreflightExecutor")
    }
}
