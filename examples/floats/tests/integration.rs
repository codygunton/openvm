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
            output.len() == 32 && output.iter().all(|&x| x == 0)
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
            assert!(run_asm_test(stringify!($test_name)), "Assembly test {} failed", stringify!($test_name));
        }
    };
}

// Define all assembly tests
asm_test!(test_basic_ops);
asm_test!(test_fma);
asm_test!(test_comparisons);
asm_test!(test_nan);
