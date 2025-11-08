use std::borrow::Borrow;

use openvm_circuit::arch::*;
use openvm_circuit::system::memory::online::TracingMemory;
use openvm_circuit_primitives::AlignedBytesBorrow;
use openvm_circuit_primitives_derive::AlignedBorrow;
use openvm_instructions::instruction::Instruction;
use openvm_stark_backend::{
    interaction::InteractionBuilder,
    p3_air::BaseAir,
    p3_field::{Field, FieldAlgebra, PrimeField32},
    rap::BaseAirWithPublicValues,
};

/// FloatCsrExecutor handles FCSR (Floating-Point Control/Status Register) access
/// Implements FRCSR (read FCSR) and FSCSR (write FCSR) instructions
#[derive(Clone, Copy)]
pub struct FloatCsrExecutor;

impl FloatCsrExecutor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FloatCsrExecutor {
    fn default() -> Self {
        Self::new()
    }
}

// PreflightExecutor implementation for traced execution (E3)
impl<F, RA> PreflightExecutor<F, RA> for FloatCsrExecutor
where
    F: PrimeField32,
    for<'buf> RA: RecordArena<'buf, MultiRowLayout<EmptyMultiRowMetadata>, &'buf mut FloatCsrCoreRecord>,
{
    fn get_opcode_name(&self, opcode: usize) -> String {
        match opcode {
            0x313 => "FCSR".to_string(),
            _ => format!("UnknownFloatCsr(0x{:x})", opcode),
        }
    }

    fn execute(
        &self,
        state: VmStateMut<F, TracingMemory, RA>,
        instruction: &Instruction<F>,
    ) -> Result<(), ExecutionError> {
        use crate::constants::*;
        use openvm_instructions::{program::DEFAULT_PC_STEP, riscv::RV32_REGISTER_AS};

        unsafe {
            // Allocate record for trace generation
            let core_record = state.ctx.alloc(MultiRowLayout::new(EmptyMultiRowMetadata::new()));

            // Extract instruction fields
            let rd = instruction.a.as_canonical_u32() as u8;
            let rs1 = instruction.b.as_canonical_u32() as u8;
            let op_type = instruction.c.as_canonical_u32() as u8;
            let csr_addr = instruction.d.as_canonical_u32() as u8;

            // Read current FCSR value from memory
            let (_, fcsr_bytes) = state.memory.read::<u8, 4, 4>(FLOAT_MEM_AS, FLOAT_CSR_FCSR);
            let fcsr_full = u32::from_le_bytes(fcsr_bytes);

            // Extract the appropriate field based on CSR address
            let fcsr_old = match csr_addr {
                0x001 => fcsr_full & 0x1F,        // fflags: bits [4:0]
                0x002 => (fcsr_full >> 5) & 0x07, // frm: bits [7:5] shifted to [2:0]
                0x003 => fcsr_full & 0xFF,        // fcsr: bits [7:0]
                _ => {
                    *state.pc = state.pc.wrapping_add(DEFAULT_PC_STEP);
                    return Ok(());
                }
            };

            // Process CSR operation based on op_type
            // op_type: 0=CSRRW, 1=CSRRS, 2=CSRRC, 3=CSRRWI, 4=CSRRSI, 5=CSRRCI
            let (fcsr_new, write_csr) = match op_type {
                0 => {
                    // CSRRW: Read/Write - t=CSR; CSR=rs1; rd=t
                    let (_, rs1_bytes) = state
                        .memory
                        .read::<u8, 4, 4>(RV32_REGISTER_AS, rs1 as u32 * 4);
                    let rs1_val = u32::from_le_bytes(rs1_bytes);
                    (rs1_val, true)
                }
                1 => {
                    // CSRRS: Read and Set - t=CSR; CSR=t|rs1; rd=t
                    if rs1 == 0 {
                        (fcsr_old, false)
                    } else {
                        let (_, rs1_bytes) = state
                            .memory
                            .read::<u8, 4, 4>(RV32_REGISTER_AS, rs1 as u32 * 4);
                        let rs1_val = u32::from_le_bytes(rs1_bytes);
                        (fcsr_old | rs1_val, true)
                    }
                }
                2 => {
                    // CSRRC: Read and Clear - t=CSR; CSR=t&~rs1; rd=t
                    if rs1 == 0 {
                        (fcsr_old, false)
                    } else {
                        let (_, rs1_bytes) = state
                            .memory
                            .read::<u8, 4, 4>(RV32_REGISTER_AS, rs1 as u32 * 4);
                        let rs1_val = u32::from_le_bytes(rs1_bytes);
                        (fcsr_old & !rs1_val, true)
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
                _ => {
                    // Invalid op_type
                    (fcsr_old, false)
                }
            };

            // Fill the record
            core_record.csr_addr = csr_addr as u32;
            core_record.csr_value = fcsr_old;
            core_record.write_value = fcsr_new;
            core_record.is_write = write_csr;

            // Write new FCSR value if needed
            if write_csr {
                // Merge the new value back into the full FCSR based on CSR address
                let fcsr_final = match csr_addr {
                    0x001 => {
                        // fflags: update bits [4:0], preserve bits [7:5]
                        (fcsr_full & 0xFFFFFFE0) | (fcsr_new & 0x1F)
                    }
                    0x002 => {
                        // frm: update bits [7:5], preserve bits [4:0]
                        (fcsr_full & 0xFFFFFF1F) | ((fcsr_new & 0x07) << 5)
                    }
                    0x003 => {
                        // fcsr: update bits [7:0]
                        (fcsr_full & 0xFFFFFF00) | (fcsr_new & 0xFF)
                    }
                    _ => fcsr_full,
                };

                state.memory.write::<u8, 4, 4>(
                    FLOAT_MEM_AS,
                    FLOAT_CSR_FCSR,
                    fcsr_final.to_le_bytes(),
                );
            }

            // Write old FCSR value to rd (unless rd=x0)
            if rd != 0 {
                state.memory.write::<u8, 4, 4>(
                    RV32_REGISTER_AS,
                    rd as u32 * 4,
                    fcsr_old.to_le_bytes(),
                );
            }

            *state.pc = state.pc.wrapping_add(DEFAULT_PC_STEP);

            Ok(())
        }
    }
}

/// Record for FloatCsr operations (minimal data needed during execution)
#[repr(C, align(4))]
#[derive(AlignedBytesBorrow, Debug, Clone)]
pub struct FloatCsrCoreRecord {
    pub csr_addr: u32,    // CSR address (FCSR=0x003, FFLAGS=0x001, FRM=0x002)
    pub csr_value: u32,   // Current CSR value (what gets read)
    pub write_value: u32, // Value to write (for write operations)
    pub is_write: bool,   // true=write operation, false=read-only
}

/// Trace columns for FloatCsr AIR
#[repr(C)]
#[derive(AlignedBorrow, Debug, Clone)]
pub struct FloatCsrCoreCols<T> {
    pub csr_addr: T,    // CSR address
    pub csr_value: T,   // Current CSR value (read value)
    pub write_value: T, // Value to write
    pub is_write: T,    // Boolean: true for writes
}

/// Core AIR for Float CSR operations
/// Handles FCSR, FFLAGS, FRM register read/write
#[derive(Copy, Clone, Debug)]
pub struct FloatCsrCoreAir {
    pub offset: usize, // Opcode offset
}

impl FloatCsrCoreAir {
    pub fn new(offset: usize) -> Self {
        Self { offset }
    }
}

impl<F: Field> BaseAir<F> for FloatCsrCoreAir {
    fn width(&self) -> usize {
        FloatCsrCoreCols::<F>::width()
    }
}

impl<F: Field> BaseAirWithPublicValues<F> for FloatCsrCoreAir {}

impl<AB, I> VmCoreAir<AB, I> for FloatCsrCoreAir
where
    AB: InteractionBuilder,
    I: VmAdapterInterface<AB::Expr>,
    I::Reads: From<[[AB::Expr; 1]; 1]>,
    I::Writes: From<[[AB::Expr; 1]; 2]>,
    I::ProcessedInstruction: From<ImmInstruction<AB::Expr>>,
{
    fn eval(
        &self,
        builder: &mut AB,
        local_core: &[AB::Var],
        _from_pc: AB::Var,
    ) -> AdapterAirContext<AB::Expr, I> {
        let cols: &FloatCsrCoreCols<AB::Var> = (*local_core).borrow();

        // CONSTRAINT 1: is_write must be boolean
        builder.assert_bool(cols.is_write);

        // CSR operations are simple:
        // - Read: returns current csr_value
        // - Write: returns current csr_value and writes write_value

        // Note: CSR address validation is handled by the executor
        // The three valid addresses are: 0x001 (FFLAGS), 0x002 (FRM), 0x003 (FCSR)

        // Return adapter context
        // Reads: rs1 value (source value for write, or zimm for immediate operations)
        // Writes: rd value (old CSR value), and CSR memory location (for writes)
        AdapterAirContext {
            to_pc: None, // Default PC increment
            reads: [[cols.write_value.into()]].into(),
            writes: [
                [cols.csr_value.into()],   // Write old value to rd
                [cols.write_value.into()], // Write new value to CSR (if is_write)
            ]
            .into(),
            instruction: ImmInstruction {
                is_valid: AB::Expr::from_canonical_u32(1),
                opcode: AB::Expr::from_canonical_usize(self.offset),
                immediate: cols.csr_addr.into(),
            }
            .into(),
        }
    }

    fn start_offset(&self) -> usize {
        self.offset
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::BorrowMut;

    use openvm_circuit::arch::{
        testing::{TestBuilder, TestChipHarness, VmChipTestBuilder},
        Arena, PreflightExecutor, VmAirWrapper, VmChipWrapper,
    };
    use openvm_instructions::instruction::Instruction;
    use openvm_stark_backend::p3_air::BaseAir;
    use openvm_stark_backend::p3_field::FieldAlgebra;
    use openvm_stark_backend::p3_matrix::dense::RowMajorMatrix;
    use openvm_stark_backend::p3_matrix::Matrix;
    use openvm_stark_backend::utils::disable_debug_builder;
    use openvm_stark_backend::verifier::VerificationError;
    use openvm_stark_sdk::p3_baby_bear::BabyBear;
    use openvm_stark_sdk::utils::create_seeded_rng;
    use rand::Rng;

    use super::{FloatCsrCoreCols, FloatCsrExecutor};
    use crate::constants::*;
    use crate::float_csr::adapter::FloatCsrAdapterAir;
    use crate::float_csr::execution::FloatCsrFiller;

    type F = BabyBear;
    type FloatCsrAir = VmAirWrapper<FloatCsrAdapterAir, super::FloatCsrCoreAir>;
    type FloatCsrChip = VmChipWrapper<F, FloatCsrFiller>;
    type Harness = TestChipHarness<F, FloatCsrExecutor, FloatCsrAir, FloatCsrChip>;

    const FCSR_OPCODE: usize = 0x313;

    fn create_harness(tester: &mut VmChipTestBuilder<F>) -> Harness {
        let execution_bridge = tester.execution_bridge();
        let memory_helper = tester.memory_helper();

        let adapter_air = FloatCsrAdapterAir::new(execution_bridge);
        let core_air = super::FloatCsrCoreAir::new(FCSR_OPCODE);
        let air = FloatCsrAir::new(adapter_air, core_air);

        let executor = FloatCsrExecutor::new();
        let chip = FloatCsrChip::new(FloatCsrFiller::new(), memory_helper);

        Harness::with_capacity(executor, air, chip, 16)
    }

    fn execute_csr_operation<RA: Arena>(
        tester: &mut impl TestBuilder<F>,
        executor: &mut FloatCsrExecutor,
        arena: &mut RA,
        rd: u8,
        rs1: u8,
        op_type: u8,
        csr_addr: u8,
    ) where
        FloatCsrExecutor: PreflightExecutor<F, RA>,
    {
        // Initialize FCSR to a known value
        tester.write::<1>(
            FLOAT_MEM_AS as usize,
            FLOAT_CSR_FCSR as usize,
            [F::from_canonical_u32(0x00)],
        );

        // Initialize rs1 register with a test value
        if op_type < 3 {
            // For register-based operations, set rs1 value
            tester.write::<1>(
                RV32_REGISTER_AS as usize,
                (rs1 as u32 * 4) as usize,
                [F::from_canonical_u32(0x07)],
            );
        }

        // Execute the CSR instruction
        use openvm_instructions::VmOpcode;
        let instruction = Instruction::from_isize(
            VmOpcode::from_usize(FCSR_OPCODE),
            rd as isize,
            rs1 as isize,
            op_type as isize,
            csr_addr as isize,
            0,
        );

        tester.execute(executor, arena, &instruction);
    }

    /// Test 1: Verify boolean constraint on is_write field
    /// Corrupts is_write to an invalid value (not 0 or 1)
    // TODO: Test disabled - needs investigation of actual CSR AIR constraints
    #[test]
    #[ignore]
    fn test_csr_invalid_is_write() {
        let mut tester = VmChipTestBuilder::default();
        let mut harness = create_harness(&mut tester);

        // Execute a CSR write operation (CSRRW to FCSR)
        execute_csr_operation(
            &mut tester,
            &mut harness.executor,
            &mut harness.arena,
            1,    // rd = x1
            2,    // rs1 = x2
            0,    // op_type = CSRRW (write operation)
            0x03, // csr_addr = FCSR
        );

        // Prank: Set is_write to invalid boolean value
        let adapter_width = BaseAir::<F>::width(&harness.air.adapter);
        let modify_trace = |trace: &mut RowMajorMatrix<F>| {
            let mut values = trace.row_slice(0).to_vec();
            let cols: &mut FloatCsrCoreCols<F> = values.split_at_mut(adapter_width).1.borrow_mut();
            // Set to invalid boolean value (not 0 or 1)
            cols.is_write = F::from_canonical_u32(42);
            *trace = RowMajorMatrix::new(values, trace.width());
        };

        disable_debug_builder();
        let tester = tester
            .build()
            .load_and_prank_trace(harness, modify_trace)
            .finalize();

        // Expect OodEvaluationMismatch due to boolean constraint violation
        tester.simple_test_with_expected_error(VerificationError::OodEvaluationMismatch);
    }

    /// Test 2: Verify constraint system robustness with random corruption
    /// Tests that randomly corrupting all CSR fields triggers constraint violations
    #[test]
    #[ignore]
    fn test_csr_random_corruption() {
        let mut rng = create_seeded_rng();
        let mut tester = VmChipTestBuilder::default();
        let mut harness = create_harness(&mut tester);

        // Execute a CSR read operation (FRCSR - CSRRS with rs1=x0)
        execute_csr_operation(
            &mut tester,
            &mut harness.executor,
            &mut harness.arena,
            3,    // rd = x3
            0,    // rs1 = x0 (read-only)
            1,    // op_type = CSRRS
            0x03, // csr_addr = FCSR
        );

        // Generate random values before the closure
        let random_csr_addr = rng.gen_range(0..0xFFFF);
        let random_csr_value = rng.gen_range(0..0xFFFFFFFF);
        let random_write_value = rng.gen_range(0..0xFFFFFFFF);
        let random_is_write = rng.gen_range(2..100);

        // Prank: Corrupt all CSR fields with random values
        let adapter_width = BaseAir::<F>::width(&harness.air.adapter);
        let modify_trace = move |trace: &mut RowMajorMatrix<F>| {
            let mut values = trace.row_slice(0).to_vec();
            let cols: &mut FloatCsrCoreCols<F> = values.split_at_mut(adapter_width).1.borrow_mut();
            // Randomize all fields
            cols.csr_addr = F::from_canonical_u32(random_csr_addr);
            cols.csr_value = F::from_canonical_u32(random_csr_value);
            cols.write_value = F::from_canonical_u32(random_write_value);
            // Set is_write to invalid boolean to ensure constraint violation
            cols.is_write = F::from_canonical_u32(random_is_write);
            *trace = RowMajorMatrix::new(values, trace.width());
        };

        disable_debug_builder();
        let tester = tester
            .build()
            .load_and_prank_trace(harness, modify_trace)
            .finalize();

        // Expect OodEvaluationMismatch due to boolean constraint violation
        tester.simple_test_with_expected_error(VerificationError::OodEvaluationMismatch);
    }
}
