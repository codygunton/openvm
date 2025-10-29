/// Base address for float register file in heap memory
/// Placed at 2MB to provide space for test code while staying well within 512MB limit
pub const FLOAT_REGISTER_BASE: u32 = 0x00200000;
pub const FLOAT_CSR_FCSR: u32 = FLOAT_REGISTER_BASE + 0x8000 + 24;  // 0x00208018 - FCSR register location

/// Address where float instruction encoding is stored for handler
/// Placed 0x108 bytes after register base (after 32 registers * 8 bytes + padding)
pub const FLOAT_INST_ADDR: u32 = 0x00200108;

/// Integer register backup storage base
/// The handler writes integer register results here (like float comparison results)
/// This matches FREG_X0 in the guest code (offset 0x118 from register base)
pub const FLOAT_X0_BACKUP: u32 = 0x00200118;

/// Address of pointer to float library entry point (_zisk_float)
/// This should match where the linker places .float_lib_entry section
/// Placed at 1MB to avoid overlap with test code while keeping binary size reasonable
pub const FLOAT_LIB_ENTRY_PTR: u32 = 0x0010_0000;
pub const FLOAT_SAVED_X1: u32 = 0x1F001200;  // Scratch location to save x1 during float handler calls
pub const FLOAT_RETURN_ADDR: u32 = 0x1F001204;  // Scratch location to save actual return address
pub const FLOAT_SAVED_REGS_BASE: u32 = 0x1F001210;  // Save area for caller-saved registers during float handler calls
pub const FLOAT_TRAMPOLINE_PC: u32 = 0xF0000000;  // Special PC value for return trampoline

/// Heap memory address space ID
pub const FLOAT_MEM_AS: u32 = 2;

/// RISC-V register address space ID
pub const RV32_REGISTER_AS: u32 = 1;

/// Convert float register index (0-31) to memory address
/// Note: The float handler (SoftFloat C code) expects registers to be 8 bytes apart
/// (it uses uint64_t array), even though we only store 4 bytes per register.
#[inline]
pub const fn float_reg_addr(freg: u8) -> u32 {
    FLOAT_REGISTER_BASE + (freg as u32) * 8
}

/// Number of limbs per RV32 register
pub const RV32_REGISTER_NUM_LIMBS: usize = 1;
