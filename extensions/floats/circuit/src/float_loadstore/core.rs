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

