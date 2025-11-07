use std::borrow::Borrow;

use openvm_circuit::arch::{
    AdapterAirContext, BasicAdapterInterface, ExecutionBridge, ExecutionState, ImmInstruction,
    VmAdapterAir,
};
use openvm_circuit_primitives_derive::AlignedBorrow;
use openvm_stark_backend::{
    interaction::InteractionBuilder, p3_air::BaseAir, p3_field::{Field, FieldAlgebra},
};

/// Adapter columns for FloatCsr operations
/// Minimal adapter - just execution state tracking
#[repr(C)]
#[derive(AlignedBorrow, Debug, Clone)]
pub struct FloatCsrAdapterCols<T> {
    pub from_state: ExecutionState<T>,
}

/// Adapter AIR for FloatCsr operations
/// Handles execution bridge for CSR operations
#[derive(Clone, Copy, Debug, derive_new::new)]
pub struct FloatCsrAdapterAir {
    pub(crate) execution_bridge: ExecutionBridge,
}

impl<F: Field> BaseAir<F> for FloatCsrAdapterAir {
    fn width(&self) -> usize {
        FloatCsrAdapterCols::<F>::width()
    }
}

impl<AB: InteractionBuilder> VmAdapterAir<AB> for FloatCsrAdapterAir {
    type Interface = BasicAdapterInterface<AB::Expr, ImmInstruction<AB::Expr>, 1, 2, 1, 1>;

    fn eval(
        &self,
        builder: &mut AB,
        local: &[AB::Var],
        ctx: AdapterAirContext<AB::Expr, Self::Interface>,
    ) {
        let cols: &FloatCsrAdapterCols<AB::Var> = (*local).borrow();

        // Execute instruction with PC increment
        // Operands: [read_value, write_to_rd, write_to_csr]
        let operands = [
            ctx.reads[0][0].clone(),   // rs1/zimm value
            ctx.writes[0][0].clone(),  // rd value (old CSR)
            ctx.writes[1][0].clone(),  // CSR write value
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
        let cols: &FloatCsrAdapterCols<AB::Var> = (*local).borrow();
        cols.from_state.pc
    }
}
