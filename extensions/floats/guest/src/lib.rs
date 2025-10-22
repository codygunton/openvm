#![no_std]

extern crate alloc;

/// Memory-mapped float register file base address.
/// Float registers f0-f31 are stored here as u32 values (IEEE 754 binary32).
/// This matches SYS_ADDR + 0x1000 in float.h
pub const FLOAT_REGISTER_BASE: usize = 0x1F001000;

/// Raw RISC-V float instruction storage.
/// The transpiler stores the original instruction here for library decoding.
/// This matches FREG_INST in float.h (FREG_FIRST + 33 * 8)
pub const FLOAT_INST_ADDR: usize = 0x1F001108;

/// Integer register backup storage base.
/// The library uses this to backup integer registers during float operations.
/// This matches FREG_X0 in float.h (FREG_FIRST + 35 * 8)
pub const FLOAT_X0_ADDR: usize = 0x1F001118;

/// Floating point control and status register (fcsr).
/// This matches FREG_CSR in float.h (CSR_ADDR + 3 * 8)
pub const FLOAT_CSR_ADDR: usize = 0x1F008018;

/// Function pointer storage for _zisk_float handler.
/// The transpiler-generated JALR instruction loads from this address to jump to the library.
/// MUST be initialized before any float operations are executed.
pub const FLOAT_HANDLER_PTR: usize = 0x0001EC60;

// External C function - main float operation handler from Zisk library.
// This is called via JAL/JALR instruction from transpiler-generated code.
//
// The library:
// 1. Reads raw instruction from FLOAT_INST_ADDR
// 2. Decodes opcode, funct3, funct7, rd, rs1, rs2
// 3. Loads operands from float register file at FLOAT_REGISTER_BASE
// 4. Executes operation using Berkeley SoftFloat
// 5. Stores result back to float register file
// 6. Updates fcsr with exception flags
// 7. Returns via JALR return mechanism (address in x1)
#[link(name = "ziskfloat", kind = "static")]
extern "C" {
    pub fn _zisk_float();
}

/// Force the linker to include the float library by creating a reference to it.
/// Without this, the linker's dead-code elimination would remove the library
/// since nothing in Rust code directly calls _zisk_float() (only transpiler does).
#[used]
static FORCE_LINK_FLOAT_LIB: unsafe extern "C" fn() = _zisk_float;

/// Re-export openvm platform for convenience
pub use openvm_platform as platform;

/// Initialize the float subsystem by storing the _zisk_float function pointer.
/// This MUST be called before any float operations are executed.
///
/// # Safety
/// This function writes to a specific memory address (0x1EC60) that must not conflict
/// with other program memory. The address is chosen to be in high memory, far from
/// typical program addresses.
pub unsafe fn init_float_handler() {
    use core::arch::asm;

    // Get the address of _zisk_float
    let handler_addr = _zisk_float as *const () as u32;

    // Store it at the function pointer location using inline assembly
    // This ensures we generate the exact SW instruction we need
    // SW stores to address space 2 (heap) by default in RV32IM
    asm!(
        "lui  x5, 0x1e",       // Load upper bits of 0x1ec60 (x5 = t0, gives 0x1E000)
        "sw   {0}, 0xc60(x5)", // Store handler_addr to [x5 + 0xc60] = 0x1ec60
        in(reg) handler_addr,
        options(nostack),
    );
}
