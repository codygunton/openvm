// OpenVM RV32F Runtime Library
// This crate provides the C runtime library for floating-point operations
//
// This is a no_std crate for bare-metal RISC-V targets

#![no_std]

use core::panic::PanicInfo;

// Panic handler for no_std environment
// This will be used if a panic occurs during floating-point operations
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // In a bare-metal environment, we just loop forever on panic
    // The actual program using this library may override this
    loop {}
}

// The actual implementation is in C files:
// - float_ops.c: Individual float operation implementations
// - dispatch_table.c: Function pointer dispatch table
// - dispatch_table.h: Header with table definitions
