// Temporary script to build the guest ELF and show its location
use std::path::PathBuf;

fn main() {
    let pkg_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    println!("Building guest binary from: {}", pkg_dir.display());

    // Build using cargo directly
    let status = std::process::Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--target")
        .arg("riscv32im-risc0-zkvm-elf")
        .current_dir(&pkg_dir)
        .env("RUSTUP_TOOLCHAIN", "nightly-2025-08-02")
        .status()
        .expect("Failed to run cargo build");

    if !status.success() {
        eprintln!("Build failed!");
        std::process::exit(1);
    }

    let elf_path = pkg_dir
        .join("target/riscv32im-risc0-zkvm-elf/release/openvm-sdk-example-test");

    println!("\nELF should be at: {}", elf_path.display());

    if elf_path.exists() {
        println!("✓ ELF file exists!");
        println!("Size: {} bytes", std::fs::metadata(&elf_path).unwrap().len());
    } else {
        println!("✗ ELF file not found!");
    }
}
