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

#[derive(Clone, From, AnyEnum, Executor, MeteredExecutor, PreflightExecutor)]
pub enum Rv32FExecutor {
    FloatLoad(FloatLoadExecutor),
    FloatStore(FloatStoreExecutor),
    FloatAlu(FloatAluExecutor),
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
            ],
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
