// Build script to compile the C runtime library
// Only performs cross-compilation when building for RISC-V target

fn main() {
    let target = std::env::var("TARGET").unwrap_or_default();

    // Only cross-compile when target is RISC-V
    // For host builds (x86_64, aarch64, etc.), skip compilation
    if target.starts_with("riscv") {
        // Use cc crate to compile C files to RISC-V
        cc::Build::new()
            .compiler("riscv64-unknown-elf-gcc")
            .target("riscv32im-unknown-none-elf")
            .opt_level(2)  // -O2 for tail call optimization
            .flag("-march=rv32im")
            .flag("-mabi=ilp32")
            .file("src/fcsr.c")
            .file("src/float_ops.c")
            .file("src/dispatch_table.c")
            .compile("openvm_rv32f_runtime");

        println!("cargo:rerun-if-changed=src/fcsr.c");
        println!("cargo:rerun-if-changed=src/fcsr.h");
        println!("cargo:rerun-if-changed=src/float_ops.c");
        println!("cargo:rerun-if-changed=src/dispatch_table.c");
        println!("cargo:rerun-if-changed=src/dispatch_table.h");
        println!("cargo:rerun-if-changed=linker.ld");
    } else {
        // Building for host (x86_64, etc.) - skip C compilation
        // The library won't have actual runtime functions, but allows
        // development and testing of transpiler without cross-compiler
        println!("cargo:warning=Skipping C runtime compilation for non-RISC-V target: {}", target);
        println!("cargo:warning=Runtime functions will not be available (transpiler-only build)");
    }
}
