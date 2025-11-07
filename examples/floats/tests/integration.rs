use std::fs;
use std::path::PathBuf;

use openvm_circuit::utils::test_system_config;
use openvm_native_compiler::conversion::CompilerOptions;
use openvm_sdk::{
    config::{AppConfig, SdkSystemConfig, SdkVmConfig},
    CpuSdk, Sdk, StdIn,
};
use openvm_stark_sdk::config::FriParameters;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

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
            let passed = output.len() == 32 && output.iter().all(|&x| x == 0);
            eprintln!(
                "DEBUG {}: output len={}, all zeros={}, result={}",
                test_name,
                output.len(),
                output.iter().all(|&x| x == 0),
                passed
            );
            passed
        }
        Err(e) => {
            eprintln!("Test {} failed with error: {:?}", test_name, e);
            false
        }
    }
}

fn run_asm_proof_test(test_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for TRACE level logs
    let _ = tracing_subscriber::registry()
        .with(fmt::layer().with_writer(std::io::stderr))
        .with(
            EnvFilter::from_default_env()
                .add_directive("openvm_circuit::arch::interpreter_preflight=trace".parse().unwrap()),
        )
        .try_init();

    // Find the compiled ELF in OUT_DIR
    let out_dir = PathBuf::from(env!("OUT_DIR"));
    let elf_path = out_dir.join(format!("{}.elf", test_name));

    // Read the ELF file
    let elf_bytes = fs::read(&elf_path).expect(&format!("Failed to read {}", elf_path.display()));

    // Create a test configuration with float extension (similar to SDK integration tests)
    // Need to increase memory size for address space 2 to cover float registers at 0x00200000
    let mut system_config = test_system_config().with_max_segment_len(1000); // Increase segment length for float operations

    // Increase RV32_MEMORY_AS (address space 2) to at least 8MB to cover float registers
    // Float registers start at 0x00200000 (2MB), so we need at least 2MB + register file size
    system_config.memory_config.addr_spaces[2].num_cells = 1 << 24; // 16MB worth of cells

    let app_vm_config = SdkVmConfig::builder()
        .system(SdkSystemConfig {
            config: system_config,
        })
        .rv32i(Default::default())
        .rv32m(Default::default())
        .rv32f(Default::default()) // Add float extension
        .io(Default::default())
        .build()
        .optimize();

    let app_config = AppConfig {
        app_fri_params: FriParameters::new_for_testing(1).into(),
        app_vm_config,
        leaf_fri_params: FriParameters::new_for_testing(2).into(),
        compiler_options: CompilerOptions::default(),
    };

    eprintln!("[PROOF TEST] Creating SDK...");
    let sdk = Sdk::new(app_config)?;
    eprintln!("[PROOF TEST] SDK created successfully");

    eprintln!(
        "[PROOF TEST] Starting proof generation for {}...",
        test_name
    );
    eprintln!("[PROOF TEST] ELF size: {} bytes", elf_bytes.len());

    // Generate proof
    eprintln!("[PROOF TEST] Calling sdk.prove()...");
    let (proof, app_commit) = sdk.prove(elf_bytes, StdIn::default())?;
    eprintln!("[PROOF TEST] ✓ Proof generated successfully!");

    eprintln!("[PROOF TEST] Generating verification key...");
    let (_agg_pk, agg_vk) = sdk.agg_keygen()?;
    eprintln!("[PROOF TEST] Verification key generated");

    eprintln!("[PROOF TEST] Verifying proof...");
    Sdk::verify_proof(&agg_vk, app_commit, &proof)?;
    eprintln!(
        "[PROOF TEST] ✓ Proof verified successfully for {}!",
        test_name
    );

    Ok(())
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

// Proof test for test_fadd
#[test]
fn test_fadd_proof() {
    run_asm_proof_test("test_fadd").expect("Failed to generate or verify proof for test_fadd");
}
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
