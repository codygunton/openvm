use std::{borrow::BorrowMut, sync::Arc};

use openvm_circuit::arch::{
    testing::{TestBuilder, TestChipHarness, VmChipTestBuilder},
    Arena, ExecutionBridge, MemoryConfig, PreflightExecutor,
};
use openvm_circuit::system::memory::{
    offline_checker::MemoryBridge, SharedMemoryHelper,
};
use openvm_circuit_primitives::var_range::VariableRangeCheckerChip;
use openvm_instructions::{instruction::Instruction, LocalOpcode};
use openvm_floats_transpiler::FloatOpcode;
use openvm_stark_backend::{
    p3_air::BaseAir,
    p3_field::{FieldAlgebra, PrimeField32},
    p3_matrix::{
        dense::{DenseMatrix, RowMajorMatrix},
        Matrix,
    },
    utils::disable_debug_builder,
    verifier::VerificationError,
};
use openvm_stark_sdk::{p3_baby_bear::BabyBear, utils::create_seeded_rng};
use rand::{rngs::StdRng, Rng};

use super::*;
use crate::{
    constants::*,
    float_load::FloatLoadExecutor,
    float_loadstore::{
        FloatLoadStoreAdapterAir, FloatLoadStoreAir, FloatLoadStoreChip,
        FloatLoadStoreCoreCols, FloatLoadStoreFiller,
    },
    float_store::FloatStoreExecutor,
};

const MAX_INS_CAPACITY: usize = 128;
const IMM_BITS: usize = 16;

type F = BabyBear;

/// Helper function to create a harness for FLW (load) operations
fn create_load_harness(
    memory_bridge: MemoryBridge,
    execution_bridge: ExecutionBridge,
    range_checker_chip: Arc<VariableRangeCheckerChip>,
    memory_helper: SharedMemoryHelper<F>,
) -> (
    FloatLoadStoreAir,
    FloatLoadExecutor,
    FloatLoadStoreChip<F>,
) {
    let air = FloatLoadStoreAir::new(
        FloatLoadStoreAdapterAir::new(execution_bridge),
        FloatLoadStoreCoreAir::new(FloatOpcode::FLW as usize),
    );
    let executor = FloatLoadExecutor::new();
    let chip = FloatLoadStoreChip::new(FloatLoadStoreFiller::new(), memory_helper);
    (air, executor, chip)
}

/// Helper function to create a harness for FSW (store) operations
fn create_store_harness(
    memory_bridge: MemoryBridge,
    execution_bridge: ExecutionBridge,
    range_checker_chip: Arc<VariableRangeCheckerChip>,
    memory_helper: SharedMemoryHelper<F>,
) -> (
    FloatLoadStoreAir,
    FloatStoreExecutor,
    FloatLoadStoreChip<F>,
) {
    let air = FloatLoadStoreAir::new(
        FloatLoadStoreAdapterAir::new(execution_bridge),
        FloatLoadStoreCoreAir::new(FloatOpcode::FSW as usize),
    );
    let executor = FloatStoreExecutor::new();
    let chip = FloatLoadStoreChip::new(FloatLoadStoreFiller::new(), memory_helper);
    (air, executor, chip)
}

/// Helper to convert u32 register index to address (multiply by 4)
fn reg_to_addr(reg: u8) -> u32 {
    (reg as u32) * 4
}

/// Test helper for FLW (float load word)
/// Loads a float value from memory into a float register
#[allow(clippy::too_many_arguments)]
fn set_and_execute_load<RA: Arena, E: PreflightExecutor<F, RA>>(
    tester: &mut impl TestBuilder<F>,
    executor: &mut E,
    arena: &mut RA,
    rng: &mut StdRng,
    rd: Option<u8>,
    rs1: Option<u8>,
    imm: Option<i32>,
    mem_data: Option<u32>,
) {
    // Generate random values if not provided
    let rd = rd.unwrap_or_else(|| rng.gen_range(0..32));
    let rs1 = rs1.unwrap_or_else(|| rng.gen_range(1..32)); // Avoid x0
    let imm_lower = if let Some(imm) = imm {
        (imm as u32) & 0xFFFF
    } else {
        // Generate 4-byte aligned immediate
        rng.gen_range(0..(1 << (IMM_BITS - 2))) << 2
    };
    let imm_sign = if let Some(imm) = imm {
        if imm < 0 { 1 } else { 0 }
    } else {
        rng.gen_range(0..2)
    };
    let imm_ext = imm_lower.wrapping_add(imm_sign * 0xffff0000);

    // Generate base address and compute memory address
    // Use 1 << 26 (64 MB) to stay well within 512 MB limit even with offsets
    let base_addr: u32 = rng.gen_range(0..(1 << 26)) << 2; // 4-byte aligned
    let mem_addr = base_addr.wrapping_add(imm_ext); // Should be aligned if both base and imm are aligned

    // Generate float data if not provided
    let float_data = mem_data.unwrap_or_else(|| rng.gen());

    // Setup: write base address to rs1 register
    let base_bytes = base_addr.to_le_bytes();
    tester.write(RV32_REGISTER_AS as usize, reg_to_addr(rs1) as usize, base_bytes.map(F::from_canonical_u8));

    // Setup: write float value to memory at computed address
    let float_bytes = float_data.to_le_bytes();
    tester.write(FLOAT_MEM_AS as usize, mem_addr as usize, float_bytes.map(F::from_canonical_u8));

    // Execute FLW instruction
    tester.execute(
        executor,
        arena,
        &Instruction::from_usize(
            FloatOpcode::FLW.global_opcode(),
            [
                rd as usize,
                reg_to_addr(rs1) as usize, // rs1 * 4 for register addressing
                imm_lower as usize,
                0,
                0,
                0,
                imm_sign as usize,
            ],
        ),
    );

    // Verify: float register should now contain the loaded value
    let loaded_bytes: [F; 4] = tester.read(FLOAT_MEM_AS as usize, float_reg_addr(rd) as usize);
    let loaded_value = u32::from_le_bytes(loaded_bytes.map(|x| x.as_canonical_u32() as u8));
    assert_eq!(loaded_value, float_data, "FLW: loaded value mismatch");
}

/// Test helper for FSW (float store word)
/// Stores a float value from a float register to memory
#[allow(clippy::too_many_arguments)]
fn set_and_execute_store<RA: Arena, E: PreflightExecutor<F, RA>>(
    tester: &mut impl TestBuilder<F>,
    executor: &mut E,
    arena: &mut RA,
    rng: &mut StdRng,
    rs1: Option<u8>,
    rs2: Option<u8>,
    imm: Option<i32>,
    float_data: Option<u32>,
) {
    // Generate random values if not provided
    let rs1 = rs1.unwrap_or_else(|| rng.gen_range(1..32)); // Avoid x0
    let rs2 = rs2.unwrap_or_else(|| rng.gen_range(0..32));
    let imm_lower = if let Some(imm) = imm {
        (imm as u32) & 0xFFFF
    } else {
        // Generate 4-byte aligned immediate
        rng.gen_range(0..(1 << (IMM_BITS - 2))) << 2
    };
    let imm_sign = if let Some(imm) = imm {
        if imm < 0 { 1 } else { 0 }
    } else {
        rng.gen_range(0..2)
    };
    let imm_ext = imm_lower.wrapping_add(imm_sign * 0xffff0000);

    // Generate base address and compute memory address
    // Use 1 << 26 (64 MB) to stay well within 512 MB limit even with offsets
    let base_addr: u32 = rng.gen_range(0..(1 << 26)) << 2; // 4-byte aligned
    let mem_addr = base_addr.wrapping_add(imm_ext); // Should be aligned if both base and imm are aligned

    // Generate float data if not provided
    let float_value = float_data.unwrap_or_else(|| rng.gen());

    // Setup: write base address to rs1 register
    let base_bytes = base_addr.to_le_bytes();
    tester.write(RV32_REGISTER_AS as usize, reg_to_addr(rs1) as usize, base_bytes.map(F::from_canonical_u8));

    // Setup: write float value to float register rs2
    let float_bytes = float_value.to_le_bytes();
    tester.write(FLOAT_MEM_AS as usize, float_reg_addr(rs2) as usize, float_bytes.map(F::from_canonical_u8));

    // Execute FSW instruction
    tester.execute(
        executor,
        arena,
        &Instruction::from_usize(
            FloatOpcode::FSW.global_opcode(),
            [
                rs2 as usize,
                reg_to_addr(rs1) as usize, // rs1 * 4 for register addressing
                imm_lower as usize,
                0,
                0,
                0,
                imm_sign as usize,
            ],
        ),
    );

    // Verify: memory should now contain the stored value
    let stored_bytes: [F; 4] = tester.read(FLOAT_MEM_AS as usize, mem_addr as usize);
    let stored_value = u32::from_le_bytes(stored_bytes.map(|x| x.as_canonical_u32() as u8));
    assert_eq!(stored_value, float_value, "FSW: stored value mismatch");
}

///////////////////////////////////////////////////////////////////////////////////////
/// POSITIVE TESTS
///
/// Randomly generate computations and execute, ensuring that the generated trace
/// passes all constraints.
///////////////////////////////////////////////////////////////////////////////////////

#[test]
fn rand_flw_test() {
    let mut rng = create_seeded_rng();
    let mut mem_config = MemoryConfig::default();
    mem_config.addr_spaces[RV32_REGISTER_AS as usize].num_cells = 1 << 29;
    mem_config.addr_spaces[FLOAT_MEM_AS as usize].num_cells = 1 << 29;

    let mut tester = VmChipTestBuilder::volatile(mem_config);

    let (air, executor, chip) = create_load_harness(
        tester.memory_bridge(),
        tester.execution_bridge(),
        tester.range_checker(),
        tester.memory_helper(),
    );

    let mut harness = TestChipHarness::with_capacity(executor, air, chip, MAX_INS_CAPACITY);

    // Execute 100 random FLW operations
    for _ in 0..100 {
        set_and_execute_load(
            &mut tester,
            &mut harness.executor,
            &mut harness.arena,
            &mut rng,
            None,
            None,
            None,
            None,
        );
    }

    let tester = tester.build().load(harness).finalize();
    tester.simple_test().expect("FLW test failed");
}

#[test]
fn rand_fsw_test() {
    let mut rng = create_seeded_rng();
    let mut mem_config = MemoryConfig::default();
    mem_config.addr_spaces[RV32_REGISTER_AS as usize].num_cells = 1 << 29;
    mem_config.addr_spaces[FLOAT_MEM_AS as usize].num_cells = 1 << 29;

    let mut tester = VmChipTestBuilder::volatile(mem_config);

    let (air, executor, chip) = create_store_harness(
        tester.memory_bridge(),
        tester.execution_bridge(),
        tester.range_checker(),
        tester.memory_helper(),
    );

    let mut harness = TestChipHarness::with_capacity(executor, air, chip, MAX_INS_CAPACITY);

    // Execute 100 random FSW operations
    for _ in 0..100 {
        set_and_execute_store(
            &mut tester,
            &mut harness.executor,
            &mut harness.arena,
            &mut rng,
            None,
            None,
            None,
            None,
        );
    }

    let tester = tester.build().load(harness).finalize();
    tester.simple_test().expect("FSW test failed");
}

//////////////////////////////////////////////////////////////////////////////////////
// NEGATIVE TESTS
//
// Given a fake trace of a single operation, setup a chip and run the test. We replace
// part of the trace and check that the chip throws the expected error.
//////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Default, PartialEq)]
struct LoadStorePrankValues {
    is_load: Option<bool>,
    addr_overflow: Option<bool>,
    mem_addr: Option<u32>,
    imm_is_negative: Option<bool>,
}

fn run_negative_loadstore_test(
    is_load: bool,
    rd_or_rs2: Option<u8>,
    rs1: Option<u8>,
    imm: Option<i32>,
    prank_vals: LoadStorePrankValues,
) {
    let mut rng = create_seeded_rng();
    let mut mem_config = MemoryConfig::default();
    mem_config.addr_spaces[RV32_REGISTER_AS as usize].num_cells = 1 << 29;
    mem_config.addr_spaces[FLOAT_MEM_AS as usize].num_cells = 1 << 29;

    let mut tester = VmChipTestBuilder::volatile(mem_config);

    if is_load {
        let (air, executor, chip) = create_load_harness(
            tester.memory_bridge(),
            tester.execution_bridge(),
            tester.range_checker(),
            tester.memory_helper(),
        );

        let mut harness = TestChipHarness::with_capacity(executor, air, chip, MAX_INS_CAPACITY);

        set_and_execute_load(
            &mut tester,
            &mut harness.executor,
            &mut harness.arena,
            &mut rng,
            rd_or_rs2,
            rs1,
            imm,
            None,
        );

        let adapter_width = BaseAir::<F>::width(&harness.air.adapter);

        let modify_trace = |trace: &mut DenseMatrix<BabyBear>| {
            let mut trace_row = trace.row_slice(0).to_vec();
            let (_adapter_row, core_row) = trace_row.split_at_mut(adapter_width);
            let core_cols: &mut FloatLoadStoreCoreCols<F> = core_row.borrow_mut();

            if let Some(is_load_val) = prank_vals.is_load {
                core_cols.is_load = F::from_bool(is_load_val);
            }
            if let Some(overflow) = prank_vals.addr_overflow {
                core_cols.addr_overflow = F::from_bool(overflow);
            }
            if let Some(addr) = prank_vals.mem_addr {
                core_cols.mem_addr = F::from_canonical_u32(addr);
            }
            if let Some(is_neg) = prank_vals.imm_is_negative {
                core_cols.imm_is_negative = F::from_bool(is_neg);
            }

            *trace = RowMajorMatrix::new(trace_row, trace.width());
        };

        disable_debug_builder();
        let tester = tester
            .build()
            .load_and_prank_trace(harness, modify_trace)
            .finalize();
        tester.simple_test_with_expected_error(VerificationError::OodEvaluationMismatch);
    } else {
        let (air, executor, chip) = create_store_harness(
            tester.memory_bridge(),
            tester.execution_bridge(),
            tester.range_checker(),
            tester.memory_helper(),
        );

        let mut harness = TestChipHarness::with_capacity(executor, air, chip, MAX_INS_CAPACITY);

        set_and_execute_store(
            &mut tester,
            &mut harness.executor,
            &mut harness.arena,
            &mut rng,
            rs1,
            rd_or_rs2,
            imm,
            None,
        );

        let adapter_width = BaseAir::<F>::width(&harness.air.adapter);

        let modify_trace = |trace: &mut DenseMatrix<BabyBear>| {
            let mut trace_row = trace.row_slice(0).to_vec();
            let (_adapter_row, core_row) = trace_row.split_at_mut(adapter_width);
            let core_cols: &mut FloatLoadStoreCoreCols<F> = core_row.borrow_mut();

            if let Some(is_load_val) = prank_vals.is_load {
                core_cols.is_load = F::from_bool(is_load_val);
            }
            if let Some(overflow) = prank_vals.addr_overflow {
                core_cols.addr_overflow = F::from_bool(overflow);
            }
            if let Some(addr) = prank_vals.mem_addr {
                core_cols.mem_addr = F::from_canonical_u32(addr);
            }
            if let Some(is_neg) = prank_vals.imm_is_negative {
                core_cols.imm_is_negative = F::from_bool(is_neg);
            }

            *trace = RowMajorMatrix::new(trace_row, trace.width());
        };

        disable_debug_builder();
        let tester = tester
            .build()
            .load_and_prank_trace(harness, modify_trace)
            .finalize();
        tester.simple_test_with_expected_error(VerificationError::OodEvaluationMismatch);
    }
}

#[test]
fn test_corrupted_is_load() {
    // Test FLW with is_load flipped to false
    run_negative_loadstore_test(
        true,
        None,
        None,
        None,
        LoadStorePrankValues {
            is_load: Some(false),
            ..Default::default()
        },
    );

    // Test FSW with is_load flipped to true
    run_negative_loadstore_test(
        false,
        None,
        None,
        None,
        LoadStorePrankValues {
            is_load: Some(true),
            ..Default::default()
        },
    );
}

#[test]
fn test_invalid_address_overflow() {
    // Test with incorrectly set overflow flag
    run_negative_loadstore_test(
        true,
        Some(5),
        Some(10),
        Some(100),
        LoadStorePrankValues {
            addr_overflow: Some(true), // Force overflow when there shouldn't be
            ..Default::default()
        },
    );
}

#[test]
fn test_invalid_address_computation() {
    // Test with corrupted memory address
    run_negative_loadstore_test(
        true,
        Some(3),
        Some(7),
        Some(-200),
        LoadStorePrankValues {
            mem_addr: Some(0x12345678), // Wrong address
            ..Default::default()
        },
    );
}

#[test]
fn test_invalid_sign_extension() {
    // Test with negative immediate but wrong sign flag
    run_negative_loadstore_test(
        false,
        Some(8),
        Some(12),
        Some(-500),
        LoadStorePrankValues {
            imm_is_negative: Some(false), // Should be true for negative imm
            ..Default::default()
        },
    );
}
