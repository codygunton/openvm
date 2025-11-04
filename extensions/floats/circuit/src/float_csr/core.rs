use openvm_circuit::arch::*;
use openvm_circuit::system::memory::online::TracingMemory;
use openvm_instructions::instruction::Instruction;
use openvm_stark_backend::p3_field::PrimeField32;

/// FloatCsrExecutor handles FCSR (Floating-Point Control/Status Register) access
/// Implements FRCSR (read FCSR) and FSCSR (write FCSR) instructions
#[derive(Clone, Copy)]
pub struct FloatCsrExecutor;

impl FloatCsrExecutor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FloatCsrExecutor {
    fn default() -> Self {
        Self::new()
    }
}

// Stub implementation - FCSR operations are handled by Executor trait
impl<F, RA> PreflightExecutor<F, RA> for FloatCsrExecutor
where
    F: PrimeField32,
    RA: Arena,
{
    fn get_opcode_name(&self, opcode: usize) -> String {
        match opcode {
            0x313 => "FCSR".to_string(),
            _ => format!("UnknownFloatCsr(0x{:x})", opcode),
        }
    }

    fn execute(
        &self,
        _state: VmStateMut<F, TracingMemory, RA>,
        _instruction: &Instruction<F>,
    ) -> Result<(), ExecutionError> {
        // FCSR operations use Executor trait, not PreflightExecutor
        panic!("FCSR operations should use Executor trait, not PreflightExecutor")
    }
}
