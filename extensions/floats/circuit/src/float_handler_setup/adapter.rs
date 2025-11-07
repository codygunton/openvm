use std::borrow::Borrow;

use openvm_circuit::arch::{
    AdapterAirContext, BasicAdapterInterface, ExecutionBridge, ExecutionState, MinimalInstruction,
    VmAdapterAir,
};
use openvm_circuit_primitives_derive::AlignedBorrow;
use openvm_stark_backend::{
    interaction::InteractionBuilder, p3_air::BaseAir, p3_field::{Field, FieldAlgebra},
};

/// Adapter columns for FloatHandlerSetup
#[repr(C)]
#[derive(AlignedBorrow, Debug, Clone)]
pub struct FloatHandlerSetupAdapterCols<T> {
    pub from_state: ExecutionState<T>,
}

/// Adapter AIR for FloatHandlerSetup
/// Minimal adapter - setup operation handles saving registers and PC jump internally
#[derive(Clone, Copy, Debug, derive_new::new)]
pub struct FloatHandlerSetupAdapterAir {
    pub(crate) execution_bridge: ExecutionBridge,
}

impl<F: Field> BaseAir<F> for FloatHandlerSetupAdapterAir {
    fn width(&self) -> usize {
        FloatHandlerSetupAdapterCols::<F>::width()
    }
}

impl<AB: InteractionBuilder> VmAdapterAir<AB> for FloatHandlerSetupAdapterAir {
    type Interface = BasicAdapterInterface<AB::Expr, MinimalInstruction<AB::Expr>, 0, 0, 0, 0>;

    fn eval(
        &self,
        builder: &mut AB,
        local: &[AB::Var],
        ctx: AdapterAirContext<AB::Expr, Self::Interface>,
    ) {
        let cols: &FloatHandlerSetupAdapterCols<AB::Var> = (*local).borrow();

        // The core handles:
        // - Saving 31 integer registers
        // - Writing instruction encoding
        // - PC jump to handler address
        // Build to_state with custom PC
        let to_state = ExecutionState {
            pc: ctx.to_pc.unwrap(),
            timestamp: cols.from_state.timestamp + AB::F::ONE,
        };

        let operands: [AB::Expr; 0] = [];
        self.execution_bridge
            .execute(
                ctx.instruction.opcode,
                operands,
                cols.from_state,
                to_state,
            )
            .eval(builder, ctx.instruction.is_valid);
    }

    fn get_from_pc(&self, local: &[AB::Var]) -> AB::Var {
        let cols: &FloatHandlerSetupAdapterCols<AB::Var> = (*local).borrow();
        cols.from_state.pc
    }
}
