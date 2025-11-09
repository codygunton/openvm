use std::borrow::BorrowMut;

use openvm_circuit::arch::{
    testing::{TestBuilder, TestChipHarness, VmChipTestBuilder},
    Arena, ExecutionBridge, PreflightExecutor,
};
use openvm_circuit::system::memory::SharedMemoryHelper;
use openvm_instructions::{instruction::Instruction, riscv::RV32_REGISTER_AS, VmOpcode};
use openvm_stark_backend::{
    p3_air::BaseAir,
    p3_field::{FieldAlgebra, PrimeField32},
    p3_matrix::{dense::RowMajorMatrix, Matrix},
    utils::disable_debug_builder,
    verifier::VerificationError,
};
use openvm_stark_sdk::{p3_baby_bear::BabyBear, utils::create_seeded_rng};
use rand::{rngs::StdRng, Rng};
use test_case::test_case;

use super::*;
use crate::{
    constants::*,
    float_csr::{
        FloatCsrAdapterAir, FloatCsrAir, FloatCsrChip, FloatCsrCoreCols, FloatCsrExecutor,
    },
};

const MAX_INS_CAPACITY: usize = 128;
const FCSR_OPCODE: usize = 0x313;
type F = BabyBear;
type Harness = TestChipHarness<F, FloatCsrExecutor, FloatCsrAir, FloatCsrChip<F>>;

/// Creates (Air, Executor, Chip) from bridges
fn create_harness_fields(
    execution_bridge: ExecutionBridge,
    memory_helper: SharedMemoryHelper<F>,
) -> (FloatCsrAir, FloatCsrExecutor, FloatCsrChip<F>) {
    let adapter_air = FloatCsrAdapterAir::new(execution_bridge);
    let core_air = FloatCsrCoreAir::new(FCSR_OPCODE);
    let air = FloatCsrAir::new(adapter_air, core_air);

    let executor = FloatCsrExecutor::new();
    let chip = FloatCsrChip::new(FloatCsrFiller::new(), memory_helper);

    (air, executor, chip)
}

/// Wrapper using VmChipTestBuilder
fn create_harness(tester: &mut VmChipTestBuilder<F>) -> Harness {
    let execution_bridge = tester.execution_bridge();
    let memory_helper = tester.memory_helper();

    let (air, executor, chip) = create_harness_fields(execution_bridge, memory_helper);

    Harness::with_capacity(executor, air, chip, MAX_INS_CAPACITY)
}

/// Core test helper for CSR operations
/// Tests all 6 CSR operation types: CSRRW, CSRRS, CSRRC, CSRRWI, CSRRSI, CSRRCI
fn set_and_execute<RA: Arena, E: PreflightExecutor<F, RA>>(
    tester: &mut impl TestBuilder<F>,
    executor: &mut E,
    arena: &mut RA,
    rng: &mut StdRng,
    rd: Option<u8>,
    rs1: Option<u8>,
    op_type: u8,
    csr_addr: u8,
    initial_csr: Option<u32>,
    initial_rs1: Option<u32>,
) {
    use openvm_instructions::program::DEFAULT_PC_STEP;

    let rd = rd.unwrap_or_else(|| rng.gen_range(1..32));
    let rs1 = rs1.unwrap_or_else(|| rng.gen_range(1..32));
    let initial_csr = initial_csr.unwrap_or_else(|| rng.gen_range(0..256));
    let initial_rs1 = initial_rs1.unwrap_or_else(|| rng.gen_range(0..256));

    // Write initial CSR value to FLOAT_CSR_FCSR
    tester.write(
        FLOAT_MEM_AS as usize,
        FLOAT_CSR_FCSR as usize,
        initial_csr.to_le_bytes().map(F::from_canonical_u8),
    );

    // Write initial rs1 value to register (for non-immediate operations)
    if op_type < 3 {
        tester.write(
            RV32_REGISTER_AS as usize,
            (rs1 as u32 * 4) as usize,
            initial_rs1.to_le_bytes().map(F::from_canonical_u8),
        );
    }

    // Extract the appropriate CSR field based on csr_addr
    let fcsr_old = match csr_addr {
        0x001 => initial_csr & 0x1F,        // fflags: bits [4:0]
        0x002 => (initial_csr >> 5) & 0x07, // frm: bits [7:5] shifted to [2:0]
        0x003 => initial_csr & 0xFF,        // fcsr: bits [7:0]
        _ => panic!("Invalid CSR address: 0x{:03x}", csr_addr),
    };

    // Calculate expected new CSR value based on op_type
    let (fcsr_new, write_csr) = match op_type {
        0 => {
            // CSRRW: Read/Write - t=CSR; CSR=rs1; rd=t
            (initial_rs1, true)
        }
        1 => {
            // CSRRS: Read and Set - t=CSR; CSR=t|rs1; rd=t
            if rs1 == 0 {
                (fcsr_old, false)
            } else {
                (fcsr_old | initial_rs1, true)
            }
        }
        2 => {
            // CSRRC: Read and Clear - t=CSR; CSR=t&~rs1; rd=t
            if rs1 == 0 {
                (fcsr_old, false)
            } else {
                (fcsr_old & !initial_rs1, true)
            }
        }
        3 => {
            // CSRRWI: Read/Write Immediate - t=CSR; CSR=zimm; rd=t
            (rs1 as u32, true)
        }
        4 => {
            // CSRRSI: Read and Set Immediate - t=CSR; CSR=t|zimm; rd=t
            if rs1 == 0 {
                (fcsr_old, false)
            } else {
                (fcsr_old | (rs1 as u32), true)
            }
        }
        5 => {
            // CSRRCI: Read and Clear Immediate - t=CSR; CSR=t&~zimm; rd=t
            if rs1 == 0 {
                (fcsr_old, false)
            } else {
                (fcsr_old & !(rs1 as u32), true)
            }
        }
        _ => panic!("Invalid op_type: {}", op_type),
    };

    // Execute CSR instruction
    let instruction = Instruction {
        opcode: VmOpcode::from_usize(FCSR_OPCODE),
        a: F::from_canonical_u8(rd),
        b: F::from_canonical_u8(rs1),
        c: F::from_canonical_u8(op_type),
        d: F::from_canonical_u8(csr_addr),
        e: F::ZERO,
        f: F::ZERO,
        g: F::ZERO,
    };

    tester.execute(executor, arena, &instruction);

    // Validate rd receives old CSR value (unless rd=x0)
    if rd != 0 {
        let rd_bytes: [F; 4] = tester.read(RV32_REGISTER_AS as usize, (rd as u32 * 4) as usize);
        let rd_value = u32::from_le_bytes(rd_bytes.map(|f| f.as_canonical_u32() as u8));
        assert_eq!(
            rd_value, fcsr_old,
            "rd should contain old CSR value. op_type={}, csr_addr=0x{:03x}, expected={}, got={}",
            op_type, csr_addr, fcsr_old, rd_value
        );
    }

    // Validate CSR updated correctly
    if write_csr {
        let fcsr_bytes: [F; 4] = tester.read(FLOAT_MEM_AS as usize, FLOAT_CSR_FCSR as usize);
        let fcsr_full = u32::from_le_bytes(fcsr_bytes.map(|f| f.as_canonical_u32() as u8));

        // Extract the updated field
        let fcsr_actual = match csr_addr {
            0x001 => fcsr_full & 0x1F,        // fflags: bits [4:0]
            0x002 => (fcsr_full >> 5) & 0x07, // frm: bits [7:5] shifted to [2:0]
            0x003 => fcsr_full & 0xFF,        // fcsr: bits [7:0]
            _ => panic!("Invalid CSR address: 0x{:03x}", csr_addr),
        };

        assert_eq!(
            fcsr_actual, fcsr_new,
            "CSR value mismatch. op_type={}, csr_addr=0x{:03x}, expected={}, got={}",
            op_type, csr_addr, fcsr_new, fcsr_actual
        );
    }
}

/// Positive tests for all 6 CSR operation types across 3 CSR registers
#[test_case(0, 0x003; "CSRRW FCSR")]
#[test_case(0, 0x001; "CSRRW FFLAGS")]
#[test_case(0, 0x002; "CSRRW FRM")]
#[test_case(1, 0x003; "CSRRS FCSR")]
#[test_case(1, 0x001; "CSRRS FFLAGS")]
#[test_case(1, 0x002; "CSRRS FRM")]
#[test_case(2, 0x003; "CSRRC FCSR")]
#[test_case(2, 0x001; "CSRRC FFLAGS")]
#[test_case(2, 0x002; "CSRRC FRM")]
#[test_case(3, 0x003; "CSRRWI FCSR")]
#[test_case(3, 0x001; "CSRRWI FFLAGS")]
#[test_case(3, 0x002; "CSRRWI FRM")]
#[test_case(4, 0x003; "CSRRSI FCSR")]
#[test_case(4, 0x001; "CSRRSI FFLAGS")]
#[test_case(4, 0x002; "CSRRSI FRM")]
#[test_case(5, 0x003; "CSRRCI FCSR")]
#[test_case(5, 0x001; "CSRRCI FFLAGS")]
#[test_case(5, 0x002; "CSRRCI FRM")]
fn rand_csr_test(op_type: u8, csr_addr: u8) {
    let mut rng = create_seeded_rng();
    let mut tester = VmChipTestBuilder::default();
    let mut harness = create_harness(&mut tester);

    for _ in 0..100 {
        set_and_execute(
            &mut tester,
            &mut harness.executor,
            &mut harness.arena,
            &mut rng,
            None,
            None,
            op_type,
            csr_addr,
            None,
            None,
        );
    }

    let tester = tester.build().load(harness).finalize();
    tester.simple_test().expect("Test should pass");
}

/// Struct to hold values for pranking the trace
struct PrankValues {
    is_write: Option<u32>,
    csr_addr: Option<u32>,
    csr_value: Option<u32>,
}

/// Helper function to run negative tests that corrupt the trace
fn run_negative_csr_test(prank: PrankValues) {
    let mut rng = create_seeded_rng();
    let mut tester = VmChipTestBuilder::default();
    let mut harness = create_harness(&mut tester);

    // Execute one valid CSR operation
    set_and_execute(
        &mut tester,
        &mut harness.executor,
        &mut harness.arena,
        &mut rng,
        Some(1),
        Some(2),
        0,     // CSRRW
        0x003, // FCSR
        Some(0x12),
        Some(0x34),
    );

    // Prank the trace
    let adapter_width = BaseAir::<F>::width(&harness.air.adapter);
    let modify_trace = move |trace: &mut RowMajorMatrix<F>| {
        let mut values = trace.row_slice(0).to_vec();
        let cols: &mut FloatCsrCoreCols<F> = values.split_at_mut(adapter_width).1.borrow_mut();

        if let Some(is_write) = prank.is_write {
            cols.is_write = F::from_canonical_u32(is_write);
        }
        if let Some(csr_addr) = prank.csr_addr {
            cols.csr_addr = F::from_canonical_u32(csr_addr);
        }
        if let Some(csr_value) = prank.csr_value {
            cols.csr_value = F::from_canonical_u32(csr_value);
        }

        *trace = RowMajorMatrix::new(values, trace.width());
    };

    disable_debug_builder();
    let tester = tester
        .build()
        .load_and_prank_trace(harness, modify_trace)
        .finalize();

    tester.simple_test_with_expected_error(VerificationError::OodEvaluationMismatch);
}

/// Negative test: Corrupt is_write to non-boolean value
#[test]
fn test_invalid_is_write() {
    run_negative_csr_test(PrankValues {
        is_write: Some(42), // Invalid boolean value
        csr_addr: None,
        csr_value: None,
    });
}

/// Negative test: Set out-of-range CSR value
#[test]
fn test_invalid_csr_value() {
    let mut rng = create_seeded_rng();
    run_negative_csr_test(PrankValues {
        is_write: None,
        csr_addr: None,
        csr_value: Some(rng.gen_range(0x1000000..0xFFFFFFFF)), // Very large value
    });
}
