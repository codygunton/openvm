/// Base address for float register file in heap memory
pub const FLOAT_REGISTER_BASE: u32 = 0x1F001000;

/// Address where float instruction encoding is stored for handler
pub const FLOAT_INST_ADDR: u32 = 0x1F001108;

/// Address of pointer to float library entry point (_zisk_float)
pub const FLOAT_LIB_ENTRY_PTR: u32 = 0x0001_EC60;

/// Heap memory address space ID
pub const FLOAT_MEM_AS: u32 = 2;

/// RISC-V register address space ID
pub const RV32_REGISTER_AS: u32 = 1;

/// Convert float register index (0-31) to memory address
#[inline]
pub const fn float_reg_addr(freg: u8) -> u32 {
    FLOAT_REGISTER_BASE + (freg as u32) * 4
}

/// Number of limbs per RV32 register
pub const RV32_REGISTER_NUM_LIMBS: usize = 1;
