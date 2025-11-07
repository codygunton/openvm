use derive_more::From;
use openvm_circuit::arch::*;
use openvm_circuit::system::SystemPort;
use openvm_circuit_derive::{AnyEnum, Executor, MeteredExecutor, PreflightExecutor};
use openvm_stark_backend::{
    config::{StarkGenericConfig, Val},
    engine::StarkEngine,
    p3_field::PrimeField32,
    prover::cpu::CpuBackend,
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

impl<SC: StarkGenericConfig> VmCircuitExtension<SC> for Rv32F {
    fn extend_circuit(&self, inventory: &mut AirInventory<SC>) -> Result<(), AirInventoryError> {
        use crate::float_loadstore::{FloatLoadStoreAir, FloatLoadStoreAdapterAir, FloatLoadStoreCoreAir};
        use crate::float_handler_setup::{FloatHandlerSetupAir, FloatHandlerSetupAdapterAir, FloatHandlerSetupCoreAir};
        use crate::float_csr::{FloatCsrAir, FloatCsrAdapterAir, FloatCsrCoreAir};
        use crate::float_return::{FloatHandlerReturnAir, FloatHandlerReturnAdapterAir, FloatHandlerReturnCoreAir};

        let SystemPort {
            execution_bus,
            program_bus,
            ..
        } = inventory.system().port();
        let exec_bridge = ExecutionBridge::new(execution_bus, program_bus);

        // Register FloatLoadStore AIR for FLW (Load)
        let float_load = FloatLoadStoreAir::new(
            FloatLoadStoreAdapterAir::new(exec_bridge),
            FloatLoadStoreCoreAir::new(FloatOpcode::FLW as usize),
        );
        inventory.add_air(float_load);

        // Register FloatLoadStore AIR for FSW (Store)
        let float_store = FloatLoadStoreAir::new(
            FloatLoadStoreAdapterAir::new(exec_bridge),
            FloatLoadStoreCoreAir::new(FloatOpcode::FSW as usize),
        );
        inventory.add_air(float_store);

        // Register FloatHandlerSetup AIR for ALU operations (FADD, FSUB, FMUL, FDIV, FSQRT, FMINMAX, FSGNJ)
        let float_alu = FloatHandlerSetupAir::new(
            FloatHandlerSetupAdapterAir::new(exec_bridge),
            FloatHandlerSetupCoreAir::new(FloatOpcode::FADD as usize),
        );
        inventory.add_air(float_alu);

        // Register FloatHandlerSetup AIR for FMA operations (FMADD, FMSUB, FNMSUB, FNMADD)
        let float_fma = FloatHandlerSetupAir::new(
            FloatHandlerSetupAdapterAir::new(exec_bridge),
            FloatHandlerSetupCoreAir::new(FloatOpcode::FMADD as usize),
        );
        inventory.add_air(float_fma);

        // Register FloatHandlerSetup AIR for Convert operations (FCVT.W.S, FCVT.WU.S, FCVT.S.W, FCVT.S.WU)
        let float_convert = FloatHandlerSetupAir::new(
            FloatHandlerSetupAdapterAir::new(exec_bridge),
            FloatHandlerSetupCoreAir::new(FloatOpcode::FCVTWS as usize),
        );
        inventory.add_air(float_convert);

        // Register FloatHandlerSetup AIR for Compare operations (FEQ.S, FLT.S, FLE.S)
        let float_compare = FloatHandlerSetupAir::new(
            FloatHandlerSetupAdapterAir::new(exec_bridge),
            FloatHandlerSetupCoreAir::new(FloatOpcode::FCMP as usize),
        );
        inventory.add_air(float_compare);

        // Register FloatHandlerSetup AIR for Move operations (FMV.X.W, FMV.W.X)
        let float_move = FloatHandlerSetupAir::new(
            FloatHandlerSetupAdapterAir::new(exec_bridge),
            FloatHandlerSetupCoreAir::new(FloatOpcode::FMVXW as usize),
        );
        inventory.add_air(float_move);

        // Register FloatHandlerSetup AIR for Class operation (FCLASS.S)
        let float_class = FloatHandlerSetupAir::new(
            FloatHandlerSetupAdapterAir::new(exec_bridge),
            FloatHandlerSetupCoreAir::new(FloatOpcode::FCLASS as usize),
        );
        inventory.add_air(float_class);

        // Register FloatCsr AIR for CSR operations (FRCSR, FSCSR)
        let float_csr = FloatCsrAir::new(
            FloatCsrAdapterAir::new(exec_bridge),
            FloatCsrCoreAir::new(FloatOpcode::FCSR as usize),
        );
        inventory.add_air(float_csr);

        // Register FloatHandlerReturn AIR for handler return (register restoration)
        let float_return = FloatHandlerReturnAir::new(
            FloatHandlerReturnAdapterAir::new(exec_bridge),
            FloatHandlerReturnCoreAir::new(FloatOpcode::FLOAT_RETURN as usize),
        );
        inventory.add_air(float_return);

        Ok(())
    }
}

// CPU backend prover extension for Rv32F
#[derive(Clone, Copy, Debug, Default)]
pub struct Rv32FCpuProverExt;

impl<E, SC, RA> VmProverExtension<E, RA, Rv32F> for Rv32FCpuProverExt
where
    SC: StarkGenericConfig,
    E: StarkEngine<SC = SC, PB = CpuBackend<SC>>,
    RA: RowMajorMatrixArena<Val<SC>>,
    Val<SC>: PrimeField32,
{
    fn extend_prover(
        &self,
        _: &Rv32F,
        inventory: &mut ChipInventory<SC, RA, E::PB>,
    ) -> Result<(), ChipInventoryError> {
        use crate::float_loadstore::{FloatLoadStoreAir, FloatLoadStoreFiller, FloatLoadStoreChip};
        use crate::float_handler_setup::{FloatHandlerSetupAir, FloatHandlerSetupFiller, FloatHandlerSetupChip};
        use crate::float_csr::{FloatCsrAir, FloatCsrFiller, FloatCsrChip};
        use crate::float_return::{FloatHandlerReturnAir, FloatHandlerReturnFiller, FloatHandlerReturnChip};
        use openvm_circuit::system::memory::SharedMemoryHelper;

        let range_checker = inventory.range_checker()?.clone();
        let timestamp_max_bits = inventory.timestamp_max_bits();
        let mem_helper = SharedMemoryHelper::new(range_checker, timestamp_max_bits);

        // Register FloatLoad chip
        inventory.next_air::<FloatLoadStoreAir>()?;
        let float_load_chip = FloatLoadStoreChip::new(FloatLoadStoreFiller::new(), mem_helper.clone());
        inventory.add_executor_chip(float_load_chip);

        // Register FloatStore chip
        inventory.next_air::<FloatLoadStoreAir>()?;
        let float_store_chip = FloatLoadStoreChip::new(FloatLoadStoreFiller::new(), mem_helper.clone());
        inventory.add_executor_chip(float_store_chip);

        // Register FloatAlu chip (handler setup for ALU ops)
        inventory.next_air::<FloatHandlerSetupAir>()?;
        let float_alu_chip = FloatHandlerSetupChip::new(FloatHandlerSetupFiller::new(), mem_helper.clone());
        inventory.add_executor_chip(float_alu_chip);

        // Register FloatFma chip (handler setup for FMA ops)
        inventory.next_air::<FloatHandlerSetupAir>()?;
        let float_fma_chip = FloatHandlerSetupChip::new(FloatHandlerSetupFiller::new(), mem_helper.clone());
        inventory.add_executor_chip(float_fma_chip);

        // Register FloatConvert chip (handler setup for convert ops)
        inventory.next_air::<FloatHandlerSetupAir>()?;
        let float_convert_chip = FloatHandlerSetupChip::new(FloatHandlerSetupFiller::new(), mem_helper.clone());
        inventory.add_executor_chip(float_convert_chip);

        // Register FloatCompare chip (handler setup for compare ops)
        inventory.next_air::<FloatHandlerSetupAir>()?;
        let float_compare_chip = FloatHandlerSetupChip::new(FloatHandlerSetupFiller::new(), mem_helper.clone());
        inventory.add_executor_chip(float_compare_chip);

        // Register FloatMove chip (handler setup for move ops)
        inventory.next_air::<FloatHandlerSetupAir>()?;
        let float_move_chip = FloatHandlerSetupChip::new(FloatHandlerSetupFiller::new(), mem_helper.clone());
        inventory.add_executor_chip(float_move_chip);

        // Register FloatClass chip (handler setup for class op)
        inventory.next_air::<FloatHandlerSetupAir>()?;
        let float_class_chip = FloatHandlerSetupChip::new(FloatHandlerSetupFiller::new(), mem_helper.clone());
        inventory.add_executor_chip(float_class_chip);

        // Register FloatCsr chip
        inventory.next_air::<FloatCsrAir>()?;
        let float_csr_chip = FloatCsrChip::new(FloatCsrFiller::new(), mem_helper.clone());
        inventory.add_executor_chip(float_csr_chip);

        // Register FloatReturn chip
        inventory.next_air::<FloatHandlerReturnAir>()?;
        let float_return_chip = FloatHandlerReturnChip::new(FloatHandlerReturnFiller::new(), mem_helper.clone());
        inventory.add_executor_chip(float_return_chip);

        Ok(())
    }
}
