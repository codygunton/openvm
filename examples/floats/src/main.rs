#![no_main]
#![no_std]

use core::ptr::{read_volatile, write_volatile};

openvm::entry!(main);

// Global inputs and outputs to prevent constant folding
static mut INPUTS: [f32; 2] = [1.5, 2.5];
static mut RESULT: [f32; 4] = [0.0, 0.0, 0.0, 0.0];

pub fn main() {
    #[cfg(target_os = "zkvm")]
    unsafe {
        // Initialize float handler FIRST
        openvm_floats_guest::init_float_handler();

        // Use volatile reads to prevent constant folding
        let a: f32 = read_volatile(&INPUTS[0]);
        let b: f32 = read_volatile(&INPUTS[1]);

        // These operations compile to RISC-V F extension instructions
        // which the transpiler intercepts and routes to the float library
        let sum = a + b; // FADD.S
        let diff = a - b; // FSUB.S
        let prod = a * b; // FMUL.S
        let quot = a / b; // FDIV.S

        // Use volatile writes to prevent optimizer elimination
        write_volatile(&mut RESULT[0], sum);
        write_volatile(&mut RESULT[1], diff);
        write_volatile(&mut RESULT[2], prod);
        write_volatile(&mut RESULT[3], quot);
    }
}
