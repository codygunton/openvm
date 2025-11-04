use std::fs;
use std::path::PathBuf;

use openvm_sdk::{config::AppConfig, CpuSdk, StdIn};

fn run_asm_test(test_name: &str) -> bool {
    // Find the compiled ELF in OUT_DIR
    let out_dir = PathBuf::from(env!("OUT_DIR"));
    let elf_path = out_dir.join(format!("{}.elf", test_name));

    // Read the ELF file
    let elf_bytes = fs::read(&elf_path).expect(&format!("Failed to read {}", elf_path.display()));

    // Create OpenVM configuration for RV32IMF
    let app_config = AppConfig::riscv32();

    // Create SDK and run the program
    let sdk = CpuSdk::new(app_config).expect("Failed to create SDK");

    // Execute the ELF (it automatically converts to ExecutableFormat)
    let result = sdk.execute(elf_bytes, StdIn::default());

    match result {
        Ok(output) => {
            // Check the exit code from the output
            // The test uses .insn i 0x0b, 0, x0, x0, <code>
            // Success is code 0, failure is code 1
            // We need to check the execution result
            eprintln!(
                "{} passed with: output len={}, all zeros={}",
                test_name,
                output.len(),
                output.iter().all(|&x| x == 0),
            );
            true
        }
        Err(e) => {
            eprintln!("Test {} failed with error: {:?}", test_name, e);
            false
        }
    }
}

macro_rules! asm_test {
    ($test_name:ident) => {
        #[test]
        fn $test_name() {
            assert!(
                run_asm_test(stringify!($test_name)),
                "Assembly test {} failed",
                stringify!($test_name)
            );
        }
    };
}

macro_rules! asm_test_fail {
    ($test_name:ident) => {
        #[test]
        fn $test_name() {
            assert!(
                !run_asm_test(stringify!($test_name)),
                "Assembly test {} should have failed",
                stringify!($test_name)
            );
        }
    };
}

// Basic operations
asm_test!(test_flw_fsw);
asm_test!(test_fadd);
asm_test_fail!(test_fadd_fail);
asm_test!(test_fsub);
asm_test!(test_fmul);
asm_test!(test_fdiv);

// Fused multiply-add operations
asm_test!(test_fmadd);
asm_test!(test_fmsub);
asm_test!(test_fnmsub);
asm_test!(test_fnmadd);

// Comparison operations
asm_test!(test_feq);
asm_test!(test_flt);
asm_test!(test_fle);

// Conversion operations
asm_test!(test_fcvt_w_s);
asm_test!(test_fcvt_wu_s);
asm_test!(test_fcvt_s_w);
asm_test!(test_fcvt_s_wu);

// Edge cases
asm_test!(test_nan);
