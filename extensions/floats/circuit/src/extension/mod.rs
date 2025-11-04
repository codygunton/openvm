use derive_more::From;
use openvm_circuit::arch::*;
use openvm_circuit_derive::{AnyEnum, Executor, MeteredExecutor, PreflightExecutor};
use openvm_stark_backend::{
    config::StarkGenericConfig,
    p3_field::PrimeField32,
};
use openvm_instructions::LocalOpcode;
use openvm_floats_transpiler::FloatOpcode;
use serde::{Deserialize, Serialize};

use crate::float_load::FloatLoadExecutor;
use crate::float_store::FloatStoreExecutor;
use crate::float_csr::FloatCsrExecutor;
use crate::float_return::FloatReturnExecutor;
use crate::handler_executor::{
    FloatHandlerExecutor, FmaOp, AluOp, ConvertOp, CompareOp, MoveOp, ClassOp
};

#[derive(Clone, From, AnyEnum, Executor, MeteredExecutor, PreflightExecutor)]
pub enum Rv32FExecutor {
    FloatLoad(FloatLoadExecutor),
    FloatStore(FloatStoreExecutor),
    FloatAlu(FloatHandlerExecutor<AluOp>),
    FloatFma(FloatHandlerExecutor<FmaOp>),
    FloatConvert(FloatHandlerExecutor<ConvertOp>),
    FloatCompare(FloatHandlerExecutor<CompareOp>),
    FloatMove(FloatHandlerExecutor<MoveOp>),
    FloatClass(FloatHandlerExecutor<ClassOp>),
    FloatCsr(FloatCsrExecutor),
    FloatReturn(FloatReturnExecutor),
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Rv32F;

impl<F: PrimeField32> VmExecutionExtension<F> for Rv32F {
    type Executor = Rv32FExecutor;

    fn extend_execution(
        &self,
        inventory: &mut ExecutorInventoryBuilder<F, Rv32FExecutor>,
    ) -> Result<(), ExecutorInventoryError> {
        // Register FLW executor
        inventory.add_executor(
            FloatLoadExecutor::new(),
            [FloatOpcode::FLW.global_opcode()],
        )?;

        // Register FSW executor
        inventory.add_executor(
            FloatStoreExecutor::new(),
            [FloatOpcode::FSW.global_opcode()],
        )?;

        // Register float ALU executor for all arithmetic ops
        inventory.add_executor(
            FloatHandlerExecutor::<AluOp>::new(),
            [
                FloatOpcode::FADD.global_opcode(),
                FloatOpcode::FSUB.global_opcode(),
                FloatOpcode::FMUL.global_opcode(),
                FloatOpcode::FDIV.global_opcode(),
                FloatOpcode::FSQRT.global_opcode(),
                FloatOpcode::FMINMAX.global_opcode(),
                FloatOpcode::FSGNJ.global_opcode(),
            ],
        )?;

        // Register float FMA executor for fused multiply-add ops
        inventory.add_executor(
            FloatHandlerExecutor::<FmaOp>::new(),
            [
                FloatOpcode::FMADD.global_opcode(),
                FloatOpcode::FMSUB.global_opcode(),
                FloatOpcode::FNMSUB.global_opcode(),
                FloatOpcode::FNMADD.global_opcode(),
            ],
        )?;

        // Register float class executor for FCLASS.S
        inventory.add_executor(
            FloatHandlerExecutor::<ClassOp>::new(),
            [FloatOpcode::FCLASS.global_opcode()],
        )?;

        // Register float convert executor for float <-> int conversions
        inventory.add_executor(
            FloatHandlerExecutor::<ConvertOp>::new(),
            [
                FloatOpcode::FCVTWS.global_opcode(),  // FCVT.W.S, FCVT.WU.S
                FloatOpcode::FCVTSW.global_opcode(),  // FCVT.S.W, FCVT.S.WU
            ],
        )?;

        // Register float compare executor for comparisons
        inventory.add_executor(
            FloatHandlerExecutor::<CompareOp>::new(),
            [FloatOpcode::FCMP.global_opcode()],  // FEQ.S, FLT.S, FLE.S
        )?;

        // Register float move executor for register moves
        inventory.add_executor(
            FloatHandlerExecutor::<MoveOp>::new(),
            [
                FloatOpcode::FMVXW.global_opcode(),  // FMV.X.W (float -> int)
                FloatOpcode::FMVWX.global_opcode(),  // FMV.W.X (int -> float)
            ],
        )?;

        // Register float CSR executor for FCSR access
        inventory.add_executor(
            FloatCsrExecutor::new(),
            [FloatOpcode::FCSR.global_opcode()],  // FRCSR, FSCSR
        )?;

        // Register float return executor for handler return with register restoration
        inventory.add_executor(
            FloatReturnExecutor::new(),
            [FloatOpcode::FLOAT_RETURN.global_opcode()],
        )?;

        Ok(())
    }
}

// Stub implementation for VmCircuitExtension
// AIR circuits are not implemented yet - this is for execution only
impl<SC: StarkGenericConfig> VmCircuitExtension<SC> for Rv32F {
    fn extend_circuit(&self, _inventory: &mut AirInventory<SC>) -> Result<(), AirInventoryError> {
        // No AIR circuits to register yet - execution-only implementation
        // Future work: Add FLW, FSW, and float ALU AIR constraints
        Ok(())
    }
}
