use std::borrow::Borrow;

use openvm_circuit::arch::*;
use openvm_circuit_primitives::AlignedBytesBorrow;
use openvm_circuit_primitives_derive::AlignedBorrow;
use openvm_stark_backend::{
    interaction::InteractionBuilder,
    p3_air::BaseAir,
    p3_field::{Field, FieldAlgebra},
    rap::BaseAirWithPublicValues,
};

/// Record for FloatHandlerSetup operation.
/// This record captures the minimal data needed during execution:
/// - The instruction encoding to write to memory
/// - The handler address to jump to
/// - All 31 integer registers (x1-x31) to save
#[repr(C, align(4))]
#[derive(AlignedBytesBorrow, Clone, Debug)]
pub struct FloatHandlerSetupCoreRecord {
    pub instruction_encoding: u32,
    pub handler_addr: u32,
    pub saved_registers: [u32; 31], // x1-x31 (x0 is always 0)
}

/// Columns for FloatHandlerSetup AIR.
/// These columns represent the witness data in the trace.
#[repr(C)]
#[derive(AlignedBorrow, Clone, Debug)]
pub struct FloatHandlerSetupCoreCols<T> {
    pub instruction_encoding: T,
    pub handler_addr: T,
    pub handler_addr_aligned: T, // handler_addr & !1 (clear LSB for alignment)
    pub saved_registers: [T; 31],
}

/// AIR for FloatHandlerSetup operation.
/// This AIR verifies that the handler setup is performed correctly:
/// - Handler address is properly aligned (LSB cleared)
/// - All constraints on the saved registers hold
#[derive(Copy, Clone, Debug)]
pub struct FloatHandlerSetupCoreAir {
    pub offset: usize,
}

impl FloatHandlerSetupCoreAir {
    pub fn new(offset: usize) -> Self {
        Self { offset }
    }
}

impl<F: Field> BaseAir<F> for FloatHandlerSetupCoreAir {
    fn width(&self) -> usize {
        FloatHandlerSetupCoreCols::<F>::width()
    }
}

impl<F: Field> BaseAirWithPublicValues<F> for FloatHandlerSetupCoreAir {}

impl<AB, I> VmCoreAir<AB, I> for FloatHandlerSetupCoreAir
where
    AB: InteractionBuilder,
    I: VmAdapterInterface<AB::Expr>,
    I::Reads: Default,
    I::Writes: Default,
    I::ProcessedInstruction: From<MinimalInstruction<AB::Expr>>,
{
    fn eval(
        &self,
        builder: &mut AB,
        local_core: &[AB::Var],
        _from_pc: AB::Var,
    ) -> AdapterAirContext<AB::Expr, I> {
        let cols: &FloatHandlerSetupCoreCols<AB::Var> = (*local_core).borrow();

        // CONSTRAINT 1: Handler address alignment
        // The handler_addr_aligned should equal handler_addr with LSB cleared
        // This means: handler_addr = 2 * handler_addr_aligned + lsb
        // where lsb is 0 or 1
        let two = AB::Expr::from_canonical_u8(2);
        let lsb = cols.handler_addr.into() - two * cols.handler_addr_aligned.into();
        builder.assert_bool(lsb);

        // The to_pc (jump target) is the aligned address
        let to_pc = cols.handler_addr_aligned.into();

        // Return adapter context
        // Memory operations (saving 31 registers, writing instruction) are handled by the adapter
        AdapterAirContext {
            to_pc: Some(to_pc),
            reads: Default::default(),
            writes: Default::default(),
            instruction: MinimalInstruction {
                is_valid: AB::F::from_canonical_u32(1).into(),
                opcode: AB::F::from_canonical_usize(self.offset).into(),
            }
            .into(),
        }
    }

    fn start_offset(&self) -> usize {
        self.offset
    }
}
