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
use crate::float_alu::FloatAluExecutor;
use crate::float_fma::FloatFmaExecutor;
use crate::float_convert::FloatConvertExecutor;
use crate::float_compare::FloatCompareExecutor;
use crate::float_move::FloatMoveExecutor;
use crate::float_class::FloatClassExecutor;
use crate::float_csr::FloatCsrExecutor;

#[derive(Clone, From, AnyEnum, Executor, MeteredExecutor, PreflightExecutor)]
pub enum Rv32FExecutor {
    FloatLoad(FloatLoadExecutor),
    FloatStore(FloatStoreExecutor),
    FloatAlu(FloatAluExecutor),
    FloatFma(FloatFmaExecutor),
    FloatConvert(FloatConvertExecutor),
    FloatCompare(FloatCompareExecutor),
    FloatMove(FloatMoveExecutor),
    FloatClass(FloatClassExecutor),
    FloatCsr(FloatCsrExecutor),
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
            FloatAluExecutor::new(),
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
            FloatFmaExecutor::new(),
            [
                FloatOpcode::FMADD.global_opcode(),
                FloatOpcode::FMSUB.global_opcode(),
                FloatOpcode::FNMSUB.global_opcode(),
                FloatOpcode::FNMADD.global_opcode(),
            ],
        )?;

        // Register float class executor for FCLASS.S
        inventory.add_executor(
            FloatClassExecutor::new(),
            [FloatOpcode::FCLASS.global_opcode()],
        )?;

        // Register float convert executor for float <-> int conversions
        inventory.add_executor(
            FloatConvertExecutor::new(),
            [
                FloatOpcode::FCVTWS.global_opcode(),  // FCVT.W.S, FCVT.WU.S
                FloatOpcode::FCVTSW.global_opcode(),  // FCVT.S.W, FCVT.S.WU
            ],
        )?;

        // Register float compare executor for comparisons
        inventory.add_executor(
            FloatCompareExecutor::new(),
            [FloatOpcode::FCMP.global_opcode()],  // FEQ.S, FLT.S, FLE.S
        )?;

        // Register float move executor for register moves
        inventory.add_executor(
            FloatMoveExecutor::new(),
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
