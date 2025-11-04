/// Base address for float register file in heap memory (SYS_ADDR)
/// Placed at 2MB to provide space for test code while staying well within 512MB limit
pub const FLOAT_REGISTER_BASE: u32 = 0x00200000;

/// Offset from SYS_ADDR to FREG_FIRST
/// Matches C code: #define FREG_FIRST (SYS_ADDR + 0x1000)
pub const FREG_FIRST_OFFSET: u32 = 0x1000;

/// CSR register location
pub const FLOAT_CSR_FCSR: u32 = FLOAT_REGISTER_BASE + 0x8000 + 24;  // 0x00208018 - FCSR register location

/// Address where float instruction encoding is stored for handler
/// Matches C code: FREG_INST (FREG_FIRST + 33 * 8)
pub const FLOAT_INST_ADDR: u32 = FLOAT_REGISTER_BASE + FREG_FIRST_OFFSET + 33 * 8;

/// Integer register backup storage base
/// The handler writes integer register results here (like float comparison results)
/// Matches C code: FREG_X0 (FREG_FIRST + 35 * 8)
pub const FLOAT_X0_BACKUP: u32 = FLOAT_REGISTER_BASE + FREG_FIRST_OFFSET + 35 * 8;

/// Address of pointer to float library entry point (_zisk_float)
/// This should match where float_init.S stores the entry pointer
/// Placed at 1MB to provide space for test code while keeping binary size reasonable
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
/// Matches C code: fregs[n] accesses (FREG_FIRST + n * 8)
#[inline]
pub const fn float_reg_addr(freg: u8) -> u32 {
    FLOAT_REGISTER_BASE + FREG_FIRST_OFFSET + (freg as u32) * 8
}

/// Number of limbs per RV32 register
pub const RV32_REGISTER_NUM_LIMBS: usize = 1;
