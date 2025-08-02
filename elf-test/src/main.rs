use std::env;
use std::fs;
use eyre::Result;
use openvm_circuit::arch::instructions::exe::VmExe;
use openvm_transpiler::{elf::Elf, transpiler::Transpiler, FromElf};
use openvm_rv32im_transpiler::{
    Rv32ITranspilerExtension, Rv32MTranspilerExtension, Rv32IoTranspilerExtension
};
use openvm_sdk::{StdIn, Sdk, F};

// Parse ELF to find signature symbols
fn find_signature_bounds(elf_data: &[u8]) -> Option<(u32, u32)> {
    use object::{Object, ObjectSymbol};
    
    let obj = object::File::parse(elf_data).ok()?;
    
    let mut begin_addr = None;
    let mut end_addr = None;
    
    for symbol in obj.symbols() {
        if let Ok(name) = symbol.name() {
            if name == "begin_signature" {
                begin_addr = Some(symbol.address() as u32);
            } else if name == "end_signature" {
                end_addr = Some(symbol.address() as u32);
            }
        }
    }
    
    match (begin_addr, end_addr) {
        (Some(begin), Some(end)) if begin < end => Some((begin, end)),
        _ => None,
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <elf-file> [signature-output]", args[0]);
        std::process::exit(1);
    }

    let elf_path = &args[1];
    let signature_path = args.get(2);

    // Read the ELF data
    println!("Reading ELF: {}", elf_path);
    let elf_data = fs::read(elf_path)?;
    
    // Find signature bounds from ELF symbols
    let (begin, end) = find_signature_bounds(&elf_data)
        .expect("Failed to find begin_signature/end_signature symbols in ELF");
    
    let size = (end - begin) as usize;
    println!("Found signature region: 0x{:08x} - 0x{:08x} ({} bytes)", begin, end, size);
    
    // Set environment variables for the SDK to use
    std::env::set_var("RISC0_SIG_BEGIN_ADDR", begin.to_string());
    std::env::set_var("RISC0_SIG_SIZE", size.to_string());
    
    // Decode the ELF - use 0x80000000 as base for RISC-V tests
    let elf = Elf::decode(&elf_data, 0x80000000)?;

    // Transpile to VmExe
    println!("Transpiling...");
    let exe = VmExe::from_elf(
        elf,
        Transpiler::<F>::default()
            .with_extension(Rv32ITranspilerExtension)
            .with_extension(Rv32MTranspilerExtension)
            .with_extension(Rv32IoTranspilerExtension),
    )?;

    // Use SDK to execute - this is simplified since we just need basic RV32IM support
    let sdk = Sdk::new();
    
    // Create config using builder pattern from examples
    let vm_config = openvm_sdk::config::SdkVmConfig::builder()
        .system(Default::default())
        .rv32i(openvm_sdk::config::UnitStruct::default())
        .rv32m(Default::default())
        .io(openvm_sdk::config::UnitStruct::default())
        .build();
    
    if let Some(sig_path) = signature_path {
        println!("Running with signature extraction to: {}", sig_path);
        sdk.execute_with_signature(
            exe,
            vm_config,
            StdIn::default(),
            Some(std::path::Path::new(sig_path))
        )?;
        println!("Signature written to: {}", sig_path);
    } else {
        println!("Running without signature extraction...");
        sdk.execute(exe, vm_config, StdIn::default())?;
    }

    println!("Done!");
    Ok(())
}