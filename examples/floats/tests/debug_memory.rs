use std::fs;
use std::path::PathBuf;

use openvm_circuit::utils::test_system_config;
use openvm_native_compiler::conversion::CompilerOptions;
use openvm_sdk::{
    config::{AppConfig, SdkSystemConfig, SdkVmConfig},
    Sdk,
};
use openvm_stark_sdk::config::FriParameters;

#[test]
fn debug_test_fadd_memory_init() {
    // Find the compiled ELF in OUT_DIR
    let out_dir = PathBuf::from(env!("OUT_DIR"));
    let elf_path = out_dir.join("test_fadd.elf");

    eprintln!("[DEBUG] Reading ELF from: {}", elf_path.display());
    let elf_bytes = fs::read(&elf_path).expect(&format!("Failed to read {}", elf_path.display()));

    // Create test configuration
    let mut system_config = test_system_config()
        .with_max_segment_len(1000);
    system_config.memory_config.addr_spaces[2].num_cells = 1 << 24;

    let app_vm_config = SdkVmConfig::builder()
        .system(SdkSystemConfig { config: system_config })
        .rv32i(Default::default())
        .rv32m(Default::default())
        .rv32f(Default::default())
        .io(Default::default())
        .build()
        .optimize();

    let app_config = AppConfig {
        app_fri_params: FriParameters::new_for_testing(1).into(),
        app_vm_config,
        leaf_fri_params: FriParameters::new_for_testing(2).into(),
        compiler_options: CompilerOptions::default(),
    };

    eprintln!("[DEBUG] Creating SDK...");
    let sdk = Sdk::new(app_config).unwrap();

    // Convert ELF to VmExe
    eprintln!("[DEBUG] Converting ELF to VmExe...");
    let exe = sdk.convert_to_exe(elf_bytes).unwrap();

    eprintln!("[DEBUG] Checking init_memory contents:");
    eprintln!("  PC start: {:#x}", exe.pc_start);
    eprintln!("  Program base: {:#x}", exe.program.pc_base);
    eprintln!("  Program length: {} instructions", exe.program.instructions_and_debug_infos.len());

    // Check for float_lib_entry pointer at 0x00100000
    let float_lib_entry_addr = 0x00100000u32;
    let rv32_memory_as = 2u32;

    eprintln!("[DEBUG] Checking for float_lib_entry at address {:#x}:", float_lib_entry_addr);
    for offset in 0..4 {
        let key = (rv32_memory_as, float_lib_entry_addr + offset);
        if let Some(&byte) = exe.init_memory.get(&key) {
            eprintln!("  [{:#x}] = {:#04x}", float_lib_entry_addr + offset, byte);
        } else {
            eprintln!("  [{:#x}] = NOT IN INIT_MEMORY", float_lib_entry_addr + offset);
        }
    }

    // Reconstruct the pointer value
    let mut handler_addr_bytes = [0u8; 4];
    for i in 0..4 {
        let key = (rv32_memory_as, float_lib_entry_addr + i);
        handler_addr_bytes[i as usize] = exe.init_memory.get(&key).copied().unwrap_or(0);
    }
    let handler_addr = u32::from_le_bytes(handler_addr_bytes);
    eprintln!("[DEBUG] Reconstructed handler address: {:#x}", handler_addr);
    eprintln!("[DEBUG] Expected handler address: {:#x}", 0x00010108u32);

    // Check if handler code exists in program
    if handler_addr >= exe.program.pc_base {
        let handler_idx = ((handler_addr - exe.program.pc_base) / 4) as usize;
        eprintln!("[DEBUG] Handler instruction index: {}", handler_idx);
        eprintln!("[DEBUG] Total instructions: {}", exe.program.instructions_and_debug_infos.len());

        if handler_idx < exe.program.instructions_and_debug_infos.len() {
            if let Some((insn, _)) = &exe.program.instructions_and_debug_infos[handler_idx] {
                eprintln!("[DEBUG] Handler first instruction: {:?}", insn);
            } else {
                eprintln!("[DEBUG] Handler first instruction: MISSING");
            }
        } else {
            eprintln!("[DEBUG] Handler index out of bounds!");
        }
    }

    eprintln!("[DEBUG] Total entries in init_memory: {}", exe.init_memory.len());
    eprintln!("[DEBUG] Sample of init_memory addresses:");
    for ((as_id, addr), byte) in exe.init_memory.iter().take(20) {
        eprintln!("  AS={}, addr={:#x}: {:#04x}", as_id, addr, byte);
    }
}
