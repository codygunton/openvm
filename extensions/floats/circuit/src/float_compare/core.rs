use openvm_circuit::arch::*;
use openvm_circuit::system::memory::online::TracingMemory;
use openvm_instructions::instruction::Instruction;
use openvm_stark_backend::p3_field::PrimeField32;

/// FloatCompareExecutor handles FEQ.S, FLT.S, FLE.S by calling external handler
/// These instructions read from float registers and write boolean results to integer registers
#[derive(Clone, Copy)]
pub struct FloatCompareExecutor;

impl FloatCompareExecutor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FloatCompareExecutor {
    fn default() -> Self {
        Self::new()
    }
}

// Stub implementation - float operations call external handler, so PreflightExecutor is not used
impl<F, RA> PreflightExecutor<F, RA> for FloatCompareExecutor
where
    F: PrimeField32,
    RA: Arena,
{
    fn get_opcode_name(&self, opcode: usize) -> String {
        match opcode {
            0x30F => "FCOMPARE".to_string(),
            _ => format!("UnknownFloatCompare(0x{:x})", opcode),
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
