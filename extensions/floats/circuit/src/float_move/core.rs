use openvm_circuit::arch::*;
use openvm_circuit::system::memory::online::TracingMemory;
use openvm_instructions::instruction::Instruction;
use openvm_stark_backend::p3_field::PrimeField32;

/// FloatMoveExecutor handles FMV.X.W and FMV.W.X by calling external handler
#[derive(Clone, Copy)]
pub struct FloatMoveExecutor;

impl FloatMoveExecutor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FloatMoveExecutor {
    fn default() -> Self {
        Self::new()
    }
}

// Stub implementation - float operations call external handler, so PreflightExecutor is not used
impl<F, RA> PreflightExecutor<F, RA> for FloatMoveExecutor
where
    F: PrimeField32,
    RA: Arena,
{
    fn get_opcode_name(&self, opcode: usize) -> String {
        match opcode {
            0x310 => "FMV.X.W".to_string(),
            0x311 => "FMV.W.X".to_string(),
            _ => format!("UnknownFloatMove(0x{:x})", opcode),
        }
    }

    fn execute(
        &self,
        _state: VmStateMut<F, TracingMemory, RA>,
        _instruction: &Instruction<F>,
    ) -> Result<(), ExecutionError> {
        // Float operations are handled by external handler via JALR
        // This PreflightExecutor path should not be used
        panic!("Float operations should use Executor trait, not PreflightExecutor")
    }
}
