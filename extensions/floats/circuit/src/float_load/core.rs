use openvm_circuit::arch::*;
use openvm_circuit::system::memory::online::TracingMemory;
use openvm_instructions::instruction::Instruction;
use openvm_stark_backend::p3_field::PrimeField32;

/// FloatLoadExecutor handles FLW (float load word) instructions
#[derive(Clone, Copy)]
pub struct FloatLoadExecutor;

impl FloatLoadExecutor {
    pub fn new() -> Self {
        Self
    }
}

// Stub implementation - PreflightExecutor is required by VmConfig but not used for execution
impl<F, RA> PreflightExecutor<F, RA> for FloatLoadExecutor
where
    F: PrimeField32,
    RA: Arena,
{
    fn get_opcode_name(&self, _opcode: usize) -> String {
        "FLW".to_string()
    }

    fn execute(
        &self,
        _state: VmStateMut<F, TracingMemory, RA>,
        _instruction: &Instruction<F>,
    ) -> Result<(), ExecutionError> {
        panic!("Float operations should use Executor trait, not PreflightExecutor")
    }
}
