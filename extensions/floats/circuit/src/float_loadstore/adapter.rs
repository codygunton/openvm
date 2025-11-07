use std::borrow::Borrow;

use openvm_circuit::arch::{
    AdapterAirContext, BasicAdapterInterface, ExecutionBridge, ExecutionState, MinimalInstruction,
    VmAdapterAir,
};
use openvm_circuit_primitives_derive::AlignedBorrow;
use openvm_stark_backend::{
    interaction::InteractionBuilder, p3_air::BaseAir, p3_field::{Field, FieldAlgebra},
};

/// Adapter columns for FloatLoadStore
#[repr(C)]
#[derive(AlignedBorrow, Debug, Clone)]
pub struct FloatLoadStoreAdapterCols<T> {
    pub from_state: ExecutionState<T>,
}

/// Adapter AIR for FloatLoadStore (FLW/FSW)
/// Minimal adapter - load/store operations handle memory internally
#[derive(Clone, Copy, Debug, derive_new::new)]
pub struct FloatLoadStoreAdapterAir {
    pub(crate) execution_bridge: ExecutionBridge,
}

impl<F: Field> BaseAir<F> for FloatLoadStoreAdapterAir {
    fn width(&self) -> usize {
        FloatLoadStoreAdapterCols::<F>::width()
    }
}

impl<AB: InteractionBuilder> VmAdapterAir<AB> for FloatLoadStoreAdapterAir {
    type Interface = BasicAdapterInterface<AB::Expr, MinimalInstruction<AB::Expr>, 2, 1, 4, 4>;

    fn eval(
        &self,
        builder: &mut AB,
        local: &[AB::Var],
        ctx: AdapterAirContext<AB::Expr, Self::Interface>,
    ) {
        let cols: &FloatLoadStoreAdapterCols<AB::Var> = (*local).borrow();

        // Operands: [base_addr, float_value]
        let operands = [
            ctx.reads[0][0].clone(),  // Base address from rs1
            ctx.reads[0][1].clone(),
            ctx.reads[0][2].clone(),
            ctx.reads[0][3].clone(),
            ctx.reads[1][0].clone(),  // Float value
            ctx.reads[1][1].clone(),
            ctx.reads[1][2].clone(),
            ctx.reads[1][3].clone(),
            ctx.writes[0][0].clone(), // Float value write
            ctx.writes[0][1].clone(),
            ctx.writes[0][2].clone(),
            ctx.writes[0][3].clone(),
        ];

        self.execution_bridge
            .execute_and_increment_pc(
                ctx.instruction.opcode,
                operands,
                cols.from_state,
                AB::F::ONE,
            )
            .eval(builder, ctx.instruction.is_valid);
    }

    fn get_from_pc(&self, local: &[AB::Var]) -> AB::Var {
        let cols: &FloatLoadStoreAdapterCols<AB::Var> = (*local).borrow();
        cols.from_state.pc
    }
}
