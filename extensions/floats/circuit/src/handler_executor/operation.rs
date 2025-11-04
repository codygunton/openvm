use openvm_circuit::arch::*;
use openvm_circuit::system::memory::online::GuestMemory;
use openvm_instructions::instruction::Instruction;
use openvm_stark_backend::p3_field::PrimeField32;

/// Trait for float operations that call the native handler.
/// Each operation type implements this to provide instruction-specific logic.
pub trait FloatOperation: 'static {
    /// PreCompute struct type for this operation.
    /// Must be AlignedBytesBorrow + Clone + repr(C).
    type PreCompute: Clone;

    /// Extract instruction fields into PreCompute struct.
    /// Returns true if instruction is enabled (should execute).
    fn extract_fields<F: PrimeField32>(
        inst: &Instruction<F>,
        data: &mut Self::PreCompute,
    ) -> Result<bool, StaticProgramError>;

    /// Reconstruct the 32-bit RISC-V instruction encoding.
    fn reconstruct_riscv_instruction(data: &Self::PreCompute) -> u32;

    /// Prepare operation before calling handler (e.g., copy int→float values).
    /// Default: no preparation needed.
    #[allow(unused_variables)]
    unsafe fn prepare_for_handler<F: PrimeField32, CTX: ExecutionCtxTrait>(
        data: &Self::PreCompute,
        exec_state: &mut VmExecState<F, GuestMemory, CTX>,
    ) {
        // Default: no preparation
    }
}
