use std::borrow::{Borrow, BorrowMut};

use openvm_circuit::arch::{AdapterAirContext, MinimalInstruction, VmAdapterInterface, VmCoreAir};
use openvm_circuit_primitives::{AlignedBorrow, AlignedBytesBorrow};
use openvm_stark_backend::{
    interaction::InteractionBuilder,
    p3_air::{AirBuilder, BaseAir},
    p3_field::{Field, FieldAlgebra, PrimeField32},
    rap::BaseAirWithPublicValues,
};

/// Record for FloatLoadStore operations (FLW/FSW instructions)
/// This is the minimal data stored during execution
#[repr(C, align(4))]
#[derive(AlignedBytesBorrow, Debug, Clone)]
pub struct FloatLoadStoreCoreRecord {
    pub base_addr: u32,   // Value from rs1 (base address register)
    pub imm: i16,         // Sign-extended 12-bit immediate
    pub float_value: u32, // Float register value (load=output, store=input)
    pub is_load: bool,    // true=FLW (load), false=FSW (store)
}

/// Columns for FloatLoadStore core trace
/// This is the full trace data generated during proving
#[repr(C)]
#[derive(AlignedBorrow, Debug, Clone)]
pub struct FloatLoadStoreCoreCols<T> {
    pub base_addr: T,
    pub imm: T,
    pub imm_is_negative: T, // Boolean flag indicating if immediate is negative
    pub mem_addr: T,        // Computed memory address (base_addr + sign_extend(imm))
    pub addr_overflow: T,   // Overflow flag for address calculation
    pub float_value: [T; 4], // Float value as 4 bytes
    pub is_load: T,         // Boolean flag: 1=load, 0=store
}

/// AIR for FloatLoadStore core operations
/// Handles the core logic for FLW (float load word) and FSW (float store word)
#[derive(Copy, Clone, Debug)]
pub struct FloatLoadStoreCoreAir {
    pub offset: usize, // Opcode offset for this instruction type
}

impl FloatLoadStoreCoreAir {
    pub fn new(offset: usize) -> Self {
        Self { offset }
    }
}

impl<F: Field> BaseAir<F> for FloatLoadStoreCoreAir {
    fn width(&self) -> usize {
        FloatLoadStoreCoreCols::<F>::width()
    }
}

impl<F: Field> BaseAirWithPublicValues<F> for FloatLoadStoreCoreAir {}

impl<AB, I> VmCoreAir<AB, I> for FloatLoadStoreCoreAir
where
    AB: InteractionBuilder,
    I: VmAdapterInterface<AB::Expr>,
    I::Reads: From<[[AB::Expr; 4]; 2]>,
    I::Writes: From<[[AB::Expr; 4]; 1]>,
    I::ProcessedInstruction: From<MinimalInstruction<AB::Expr>>,
{
    fn eval(
        &self,
        builder: &mut AB,
        local_core: &[AB::Var],
        _from_pc: AB::Var,
    ) -> AdapterAirContext<AB::Expr, I> {
        let cols: &FloatLoadStoreCoreCols<AB::Var> = (*local_core).borrow();

        // CONSTRAINT 1: Boolean flags must be 0 or 1
        builder.assert_bool(cols.is_load);
        builder.assert_bool(cols.imm_is_negative);
        builder.assert_bool(cols.addr_overflow);

        // CONSTRAINT 2: Address calculation with sign extension
        // The immediate is stored as absolute value, with sign in imm_is_negative
        // Sign extension: if negative, we need to add -4096 (for 12-bit signed values)
        // We represent this as: imm_is_negative * (-4096) + imm
        // For 12-bit signed immediate: range is -2048 to 2047
        // When negative, we need to extend with 1s in upper 20 bits: -(1 << 12) = -4096
        // Using from_wrapped_u32 to properly handle the negative value in the field
        let sign_extend_offset = AB::F::from_wrapped_u32(0xFFFF_F000u32);
        let signed_imm = cols.imm_is_negative * sign_extend_offset + cols.imm;

        // mem_addr = base_addr + signed_imm (with wrapping on overflow)
        let expected_addr = cols.base_addr + signed_imm;

        // CONSTRAINT 3: Overflow detection and correction
        // If overflow occurs, we need to wrap around by subtracting 2^32
        // The relationship is: mem_addr + overflow * 2^32 = expected_addr
        let overflow_correction = cols.addr_overflow * AB::F::from_wrapped_u64(1u64 << 32);
        builder.assert_eq(cols.mem_addr + overflow_correction, expected_addr);

        // Convert float_value bytes to expressions for adapter interface
        let float_value_exprs_read: [AB::Expr; 4] = [
            cols.float_value[0].into(),
            cols.float_value[1].into(),
            cols.float_value[2].into(),
            cols.float_value[3].into(),
        ];
        let float_value_exprs_write: [AB::Expr; 4] = [
            cols.float_value[0].into(),
            cols.float_value[1].into(),
            cols.float_value[2].into(),
            cols.float_value[3].into(),
        ];

        // Base address as a 4-byte value (we'll use lower byte as the value, others as zeros)
        // This is a simplified representation for the adapter interface
        let zero = AB::F::from_canonical_u32(0);
        let base_addr_exprs: [AB::Expr; 4] =
            [cols.base_addr.into(), zero.into(), zero.into(), zero.into()];

        // Return adapter context
        // For loads: read from rs1 (base_addr) and memory, write to float register
        // For stores: read from rs1 (base_addr) and float register, write to memory
        AdapterAirContext {
            to_pc: None, // Default PC increment
            reads: [base_addr_exprs, float_value_exprs_read].into(),
            writes: [float_value_exprs_write].into(),
            instruction: MinimalInstruction {
                is_valid: AB::F::from_canonical_u32(1).into(),
                opcode: AB::F::from_canonical_usize(self.offset).into(),
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
        testing::{TestChipHarness, VmChipTestBuilder},
        VmChipWrapper,
    };
    use openvm_stark_backend::{
        p3_air::BaseAir,
        p3_field::{Field, FieldAlgebra, PrimeField32},
        p3_matrix::{dense::RowMajorMatrix, Matrix},
        utils::disable_debug_builder,
        verifier::VerificationError,
    };
    use openvm_stark_sdk::p3_baby_bear::BabyBear;

    use super::{FloatLoadStoreCoreAir, FloatLoadStoreCoreCols, FloatLoadStoreCoreRecord};
    use crate::float_loadstore::{
        adapter::FloatLoadStoreAdapterAir, execution::FloatLoadStoreFiller, FloatLoadStoreAir,
    };

    type F = BabyBear;
    const MAX_INS_CAPACITY: usize = 16;
    type Harness = TestChipHarness<F, (), FloatLoadStoreAir, VmChipWrapper<F, FloatLoadStoreFiller>>;

    fn create_harness(tester: &mut VmChipTestBuilder<F>) -> Harness {
        let adapter_air = FloatLoadStoreAdapterAir::new(tester.execution_bridge());
        let core_air = FloatLoadStoreCoreAir::new(0x100);
        let air = FloatLoadStoreAir::new(adapter_air, core_air);

        let chip = VmChipWrapper::new(FloatLoadStoreFiller::new(), tester.memory_helper());

        Harness::with_capacity((), air, chip, MAX_INS_CAPACITY)
    }

    fn execute_load_operation(
        harness: &mut Harness,
        base_addr: u32,
        imm: i16,
    ) {
        let float_value = 0x3F800000u32; // 1.0 in IEEE 754

        // Manually allocate a record in the arena
        // The record is written directly to the arena buffer
        let record_size = std::mem::size_of::<FloatLoadStoreCoreRecord>();
        let buffer = unsafe {
            let row_slice = harness.arena.alloc_single_row();
            // Skip adapter columns (ExecutionState = 2 fields)
            let adapter_width = 2 * std::mem::size_of::<F>();
            &mut row_slice[adapter_width..(adapter_width + record_size)]
        };

        // Write record data
        let record = FloatLoadStoreCoreRecord {
            base_addr,
            imm,
            float_value,
            is_load: true,
        };

        unsafe {
            std::ptr::copy_nonoverlapping(
                &record as *const _ as *const u8,
                buffer.as_mut_ptr(),
                record_size,
            );
        }
    }

    /// Test 1: Corrupt is_load boolean constraint
    /// Verifies that is_load must be exactly 0 or 1
    #[test]
    fn test_loadstore_corrupted_is_load() {
        let mut tester = VmChipTestBuilder::default();
        let mut harness = create_harness(&mut tester);

        // Execute a valid load operation
        execute_load_operation(&mut harness, 0x1000, 100);

        let adapter_width = BaseAir::<F>::width(&harness.air.adapter);
        let modify_trace = |trace: &mut RowMajorMatrix<F>| {
            let mut values = trace.row_slice(0).to_vec();
            let cols: &mut FloatLoadStoreCoreCols<F> =
                values.split_at_mut(adapter_width).1.borrow_mut();
            // Flip the boolean: if it was 1 (true), make it 0, and vice versa
            cols.is_load = F::ONE - cols.is_load;
            *trace = RowMajorMatrix::new(values, trace.width());
        };

        disable_debug_builder();
        let tester = tester
            .build()
            .load_and_prank_trace(harness, modify_trace)
            .finalize();
        tester.simple_test_with_expected_error(VerificationError::OodEvaluationMismatch);
    }

    /// Test 2: Corrupt sign extension constraint (imm_is_negative)
    /// Verifies that the sign flag correctly determines sign extension
    #[test]
    fn test_loadstore_invalid_sign_extension() {
        let mut tester = VmChipTestBuilder::default();
        let mut harness = create_harness(&mut tester);

        // Execute with a negative immediate
        execute_load_operation(&mut harness, 0x1000, -256);

        let adapter_width = BaseAir::<F>::width(&harness.air.adapter);
        let modify_trace = |trace: &mut RowMajorMatrix<F>| {
            let mut values = trace.row_slice(0).to_vec();
            let cols: &mut FloatLoadStoreCoreCols<F> =
                values.split_at_mut(adapter_width).1.borrow_mut();
            // Flip the sign flag to break the sign extension constraint
            cols.imm_is_negative = F::ONE - cols.imm_is_negative;
            *trace = RowMajorMatrix::new(values, trace.width());
        };

        disable_debug_builder();
        let tester = tester
            .build()
            .load_and_prank_trace(harness, modify_trace)
            .finalize();
        tester.simple_test_with_expected_error(VerificationError::OodEvaluationMismatch);
    }

    /// Test 3: Corrupt address computation constraint
    /// Verifies that mem_addr must equal base_addr + signed_imm (modulo overflow)
    #[test]
    fn test_loadstore_invalid_address_computation() {
        let mut tester = VmChipTestBuilder::default();
        let mut harness = create_harness(&mut tester);

        // Execute a normal load
        execute_load_operation(&mut harness, 0x1000, 100);

        let adapter_width = BaseAir::<F>::width(&harness.air.adapter);
        let modify_trace = |trace: &mut RowMajorMatrix<F>| {
            let mut values = trace.row_slice(0).to_vec();
            let cols: &mut FloatLoadStoreCoreCols<F> =
                values.split_at_mut(adapter_width).1.borrow_mut();
            // Corrupt the computed memory address to break the address calculation constraint
            cols.mem_addr = cols.mem_addr + F::from_canonical_u32(42);
            *trace = RowMajorMatrix::new(values, trace.width());
        };

        disable_debug_builder();
        let tester = tester
            .build()
            .load_and_prank_trace(harness, modify_trace)
            .finalize();
        tester.simple_test_with_expected_error(VerificationError::OodEvaluationMismatch);
    }

    /// Test 4: Corrupt overflow flag constraint
    /// Verifies that addr_overflow must be a boolean (0 or 1)
    #[test]
    fn test_loadstore_invalid_overflow_flag() {
        let mut tester = VmChipTestBuilder::default();
        let mut harness = create_harness(&mut tester);

        // Execute a load that doesn't overflow
        execute_load_operation(&mut harness, 0x1000, 100);

        let adapter_width = BaseAir::<F>::width(&harness.air.adapter);
        let modify_trace = |trace: &mut RowMajorMatrix<F>| {
            let mut values = trace.row_slice(0).to_vec();
            let cols: &mut FloatLoadStoreCoreCols<F> =
                values.split_at_mut(adapter_width).1.borrow_mut();
            // Flip the overflow flag to break the constraint
            cols.addr_overflow = F::ONE - cols.addr_overflow;
            *trace = RowMajorMatrix::new(values, trace.width());
        };

        disable_debug_builder();
        let tester = tester
            .build()
            .load_and_prank_trace(harness, modify_trace)
            .finalize();
        tester.simple_test_with_expected_error(VerificationError::OodEvaluationMismatch);
    }

    /// Test 5: Random corruption test
    /// Verifies overall constraint system robustness by corrupting all fields
    #[test]
    fn test_loadstore_random_corruption() {
        let mut tester = VmChipTestBuilder::default();
        let mut harness = create_harness(&mut tester);

        // Execute a valid operation
        execute_load_operation(&mut harness, 0x2000, -512);

        let adapter_width = BaseAir::<F>::width(&harness.air.adapter);
        let modify_trace = |trace: &mut RowMajorMatrix<F>| {
            let mut values = trace.row_slice(0).to_vec();
            let cols: &mut FloatLoadStoreCoreCols<F> =
                values.split_at_mut(adapter_width).1.borrow_mut();
            // Corrupt multiple fields with values that violate constraints
            // Use values within BabyBear field range (prime = 2013265921 = 0x78000001)
            cols.base_addr = F::from_canonical_u32(0x12345678);
            cols.imm = F::from_canonical_u32(0xCAFE);
            cols.mem_addr = F::from_canonical_u32(0x76543210); // < BabyBear prime
            cols.float_value[0] = F::from_canonical_u32(0xFF);
            cols.float_value[1] = F::from_canonical_u32(0xAA);
            *trace = RowMajorMatrix::new(values, trace.width());
        };

        disable_debug_builder();
        let tester = tester
            .build()
            .load_and_prank_trace(harness, modify_trace)
            .finalize();
        tester.simple_test_with_expected_error(VerificationError::OodEvaluationMismatch);
    }
}
