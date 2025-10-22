use openvm_circuit::arch::*;
use openvm_circuit::system::memory::online::TracingMemory;
use openvm_instructions::instruction::Instruction;
use openvm_stark_backend::p3_field::PrimeField32;

/// FloatFmaExecutor handles FMADD.S, FMSUB.S, FNMSUB.S, FNMADD.S by calling external handler
#[derive(Clone, Copy)]
pub struct FloatFmaExecutor;

impl FloatFmaExecutor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FloatFmaExecutor {
    fn default() -> Self {
        Self::new()
    }
}

// Stub implementation - float operations call external handler, so PreflightExecutor is not used
impl<F, RA> PreflightExecutor<F, RA> for FloatFmaExecutor
where
    F: PrimeField32,
    RA: Arena,
{
    fn get_opcode_name(&self, opcode: usize) -> String {
        match opcode {
            0x309 => "FMADD.S".to_string(),
            0x30A => "FMSUB.S".to_string(),
            0x30B => "FNMSUB.S".to_string(),
            0x30C => "FNMADD.S".to_string(),
            _ => format!("UnknownFloatFma(0x{:x})", opcode),
        }
    }

    fn execute(
        &self,
        _state: VmStateMut<F, TracingMemory, RA>,
        _instruction: &Instruction<F>,
    ) -> Result<(), ExecutionError> {
        // Float operations are handled by external handler via JALR
        // This PreflightExecutor path should not be used
        panic!("Float FMA operations should use Executor trait, not PreflightExecutor")
    }
}
