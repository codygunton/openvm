use std::borrow::BorrowMut;

use openvm_circuit::arch::testing::{TestBuilder, TestChipHarness, VmChipTestBuilder};
use openvm_circuit::arch::{Arena, PreflightExecutor};
use openvm_floats_transpiler::FloatOpcode;
use openvm_instructions::{instruction::Instruction, riscv::RV32_REGISTER_AS, LocalOpcode};
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
    float_return::{
        FloatHandlerReturnAdapterAir, FloatHandlerReturnAir, FloatHandlerReturnChip,
        FloatHandlerReturnCoreCols, FloatReturnExecutor,
    },
};

const MAX_INS_CAPACITY: usize = 128;
type F = BabyBear;
type Harness =
    TestChipHarness<F, FloatReturnExecutor, FloatHandlerReturnAir, FloatHandlerReturnChip<F>>;

fn create_harness(tester: &mut VmChipTestBuilder<F>) -> Harness {
    let air = FloatHandlerReturnAir::new(
        FloatHandlerReturnAdapterAir::new(tester.execution_bridge()),
        FloatHandlerReturnCoreAir::new(FloatOpcode::FLOAT_RETURN as usize),
    );
    let executor = FloatReturnExecutor::new();
    let chip = FloatHandlerReturnChip::<F>::new(FloatHandlerReturnFiller::new(), tester.memory_helper());

    Harness::with_capacity(executor, air, chip, MAX_INS_CAPACITY)
}

/// Core test helper that sets up memory, executes FLOAT_RETURN, and validates results
#[allow(clippy::too_many_arguments)]
fn set_and_execute<RA: Arena, E: PreflightExecutor<F, RA>>(
    tester: &mut impl TestBuilder<F>,
    executor: &mut E,
    arena: &mut RA,
    rng: &mut StdRng,
    return_addr: Option<u32>,
    register_values: Option<[u32; 31]>,
) {
    // Generate return address (4-byte aligned)
    let return_addr = return_addr.unwrap_or(rng.gen_range(0x1000..0x10000) & !3);

    // Generate 31 register values (x1-x31)
    let register_values = register_values.unwrap_or_else(|| {
        let mut regs = [0u32; 31];
        for reg in &mut regs {
            *reg = rng.gen();
        }
        regs
    });

    // Write return address to FLOAT_RETURN_ADDR
    tester.write::<4>(
        FLOAT_MEM_AS as usize,
        FLOAT_RETURN_ADDR as usize,
        return_addr.to_le_bytes().map(F::from_canonical_u8),
    );

    // Write backed-up register values to FLOAT_X0_BACKUP (31 registers, 8-byte aligned)
    for (i, &reg_value) in register_values.iter().enumerate() {
        let backup_addr = FLOAT_X0_BACKUP + ((i as u32 + 1) * 8); // x1-x31
        tester.write::<4>(
            FLOAT_MEM_AS as usize,
            backup_addr as usize,
            reg_value.to_le_bytes().map(F::from_canonical_u8),
        );
    }

    // Execute FLOAT_RETURN instruction
    let initial_pc = rng.gen_range(0x1000..0x10000) & !3;
    tester.execute_with_pc(
        executor,
        arena,
        &Instruction::from_usize(
            FloatOpcode::FLOAT_RETURN.global_opcode(),
            [0, 0, 0, 0, 0], // No operands needed for FLOAT_RETURN
        ),
        initial_pc,
    );

    // Validate PC was updated to return address
    assert_eq!(
        tester.last_to_pc().as_canonical_u32(),
        return_addr,
        "PC should be set to return address"
    );

    // Note: Register restoration is validated through memory bus interactions during proof generation
    // Attempting to read registers here would require additional setup that's tested in integration tests
}

///////////////////////////////////////////////////////////////////////////////////////
/// POSITIVE TESTS
///
/// Randomly generate computations and execute, ensuring that the generated trace
/// passes all constraints.
///////////////////////////////////////////////////////////////////////////////////////

#[test]
fn rand_float_return_test() {
    let mut rng = create_seeded_rng();
    let mut tester = VmChipTestBuilder::default();
    let mut harness = create_harness(&mut tester);

    let num_ops = 100;
    for _ in 0..num_ops {
        set_and_execute(
            &mut tester,
            &mut harness.executor,
            &mut harness.arena,
            &mut rng,
            None,
            None,
        );
    }

    let tester = tester.build().load(harness).finalize();
    tester.simple_test().expect("Verification failed");
}

///////////////////////////////////////////////////////////////////////////////////////
/// NEGATIVE TESTS
///
/// Given a valid execution, corrupt the trace and verify that constraints detect
/// the corruption.
///////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Default, PartialEq)]
struct PrankValues {
    pub return_addr: Option<u32>,
    pub restored_registers: Option<Vec<(usize, u32)>>, // (index, value) pairs to corrupt
}

fn run_negative_float_return_test(
    return_addr: Option<u32>,
    register_values: Option<[u32; 31]>,
    prank_vals: PrankValues,
    expected_error: VerificationError,
) {
    let mut rng = create_seeded_rng();
    let mut tester = VmChipTestBuilder::default();
    let mut harness = create_harness(&mut tester);

    // Execute valid operation
    set_and_execute(
        &mut tester,
        &mut harness.executor,
        &mut harness.arena,
        &mut rng,
        return_addr,
        register_values,
    );

    // Prank trace - corrupt specified values
    let adapter_width = BaseAir::<F>::width(&harness.air.adapter);
    let modify_trace = |trace: &mut DenseMatrix<F>| {
        let mut values = trace.row_slice(0).to_vec();
        let cols: &mut FloatHandlerReturnCoreCols<F> =
            values.split_at_mut(adapter_width).1.borrow_mut();

        if let Some(addr) = prank_vals.return_addr {
            cols.return_addr = F::from_canonical_u32(addr);
        }

        if let Some(ref corruptions) = prank_vals.restored_registers {
            for &(idx, value) in corruptions {
                if idx < 31 {
                    cols.restored_registers[idx] = F::from_canonical_u32(value);
                }
            }
        }

        *trace = RowMajorMatrix::new(values, trace.width());
    };

    disable_debug_builder();
    let tester = tester
        .build()
        .load_and_prank_trace(harness, modify_trace)
        .finalize();
    tester.simple_test_with_expected_error(expected_error);
}

#[test]
fn test_corrupted_return_addr() {
    // Corrupt return_addr to a different value
    run_negative_float_return_test(
        Some(0x2000),
        None,
        PrankValues {
            return_addr: Some(0x3000), // Different from actual
            ..Default::default()
        },
        VerificationError::ChallengePhaseError, // Memory bus interaction failure
    );
}

#[test]
fn test_corrupted_single_register() {
    // Corrupt a single register value
    let mut register_values = [0u32; 31];
    for (i, reg) in register_values.iter_mut().enumerate() {
        *reg = (i as u32 + 1) * 0x1000; // Deterministic values
    }

    run_negative_float_return_test(
        None,
        Some(register_values),
        PrankValues {
            return_addr: None,
            restored_registers: Some(vec![(10, 0x99999999)]), // Corrupt x11
        },
        VerificationError::ChallengePhaseError, // Memory bus interaction failure
    );
}

#[test]
fn test_corrupted_multiple_registers() {
    // Corrupt multiple register values
    let mut register_values = [0u32; 31];
    for (i, reg) in register_values.iter_mut().enumerate() {
        *reg = (i as u32 + 1) * 0x100;
    }

    run_negative_float_return_test(
        None,
        Some(register_values),
        PrankValues {
            return_addr: None,
            restored_registers: Some(vec![
                (5, 0x11111111),  // Corrupt x6
                (15, 0x22222222), // Corrupt x16
                (25, 0x33333333), // Corrupt x26
            ]),
        },
        VerificationError::ChallengePhaseError, // Memory bus interaction failure
    );
}

#[test]
fn test_corrupted_return_addr_and_registers() {
    // Corrupt both return address and register values
    let mut register_values = [0u32; 31];
    for (i, reg) in register_values.iter_mut().enumerate() {
        *reg = 0x10000000 + (i as u32);
    }

    run_negative_float_return_test(
        Some(0x4000),
        Some(register_values),
        PrankValues {
            return_addr: Some(0x5000),
            restored_registers: Some(vec![
                (0, 0xAAAAAAAA),  // Corrupt x1
                (20, 0xBBBBBBBB), // Corrupt x21
            ]),
        },
        VerificationError::ChallengePhaseError, // Memory bus interaction failure
    );
}
