use openvm_circuit::arch::*;
use openvm_circuit::system::memory::online::TracingMemory;
use openvm_instructions::instruction::Instruction;
use openvm_stark_backend::p3_field::PrimeField32;

/// FloatConvertExecutor handles float<->int conversions (FCVT.W.S, FCVT.WU.S, FCVT.S.W, FCVT.S.WU)
#[derive(Clone, Copy)]
pub struct FloatConvertExecutor;

impl FloatConvertExecutor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FloatConvertExecutor {
    fn default() -> Self {
        Self::new()
    }
}

// Stub implementation - float operations call external handler, so PreflightExecutor is not used
impl<F, RA> PreflightExecutor<F, RA> for FloatConvertExecutor
where
    F: PrimeField32,
    RA: Arena,
{
    fn get_opcode_name(&self, opcode: usize) -> String {
        match opcode {
            0x30D => "FCVT.W/WU.S".to_string(),
            0x30E => "FCVT.S.W/WU".to_string(),
            _ => format!("UnknownFloatConvert(0x{:x})", opcode),
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
