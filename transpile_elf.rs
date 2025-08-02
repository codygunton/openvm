use std::env;
use std::fs;
use eyre::Result;
use openvm_circuit::arch::instructions::exe::VmExe;
use openvm_transpiler::{Elf, FromElf, Transpiler};
use openvm_rv32im_transpiler::{
    Rv32ITranspilerExtension, Rv32MTranspilerExtension, Rv32IoTranspilerExtension
};
use openvm_stark_backend::p3_field::AbstractField;
use openvm_sdk::F;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <elf-file>", args[0]);
        eprintln!("Outputs: <elf-file>.vmexe");
        std::process::exit(1);
    }

    let elf_path = &args[1];
    let vmexe_path = format!("{}.vmexe", elf_path);

    // Read and decode the ELF file
    println!("Reading ELF file: {}", elf_path);
    let elf_data = fs::read(elf_path)?;
    let elf = Elf::decode(&elf_data, 0x20000000)?;

    // Create transpiler with RISC-V extensions
    println!("Transpiling to OpenVM format...");
    let exe = VmExe::from_elf(
        elf,
        Transpiler::<F>::default()
            .with_extension(Rv32ITranspilerExtension)
            .with_extension(Rv32MTranspilerExtension)
            .with_extension(Rv32IoTranspilerExtension),
    )?;

    // Serialize and write the VmExe
    println!("Writing to: {}", vmexe_path);
    openvm_sdk::fs::write_exe_to_file(exe, vmexe_path)?;

    println!("Transpilation complete!");
    Ok(())
}