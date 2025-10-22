use openvm_circuit::arch::*;
use openvm_circuit::system::memory::online::TracingMemory;
use openvm_instructions::instruction::Instruction;
use openvm_stark_backend::p3_field::PrimeField32;

/// FloatAluExecutor handles FADD, FSUB, FMUL, FDIV by calling external handler
#[derive(Clone, Copy)]
pub struct FloatAluExecutor;

impl FloatAluExecutor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FloatAluExecutor {
    fn default() -> Self {
        Self::new()
    }
}

// Stub implementation - float operations call external handler, so PreflightExecutor is not used
impl<F, RA> PreflightExecutor<F, RA> for FloatAluExecutor
where
    F: PrimeField32,
    RA: Arena,
{
    fn get_opcode_name(&self, opcode: usize) -> String {
        match opcode {
            0x302 => "FADD".to_string(),
            0x303 => "FSUB".to_string(),
            0x304 => "FMUL".to_string(),
            0x305 => "FDIV".to_string(),
            _ => format!("UnknownFloatAlu(0x{:x})", opcode),
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
