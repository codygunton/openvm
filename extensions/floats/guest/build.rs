use std::env;
use std::path::PathBuf;

fn main() {
    let target = env::var("TARGET").unwrap();

    // Only compile for RISC-V targets
    if !target.starts_with("riscv32") {
        return;
    }

    let lib_float_dir = PathBuf::from("vendor/zisk/lib-float/c");
    let softfloat_src = lib_float_dir.join("SoftFloat-3e/source");
    let softfloat_8086 = softfloat_src.join("8086");
    let softfloat_build = lib_float_dir.join("SoftFloat-3e/build/Linux-x86_64-GCC");

    // Configure cc for cross-compilation
    // Note: riscv64-elf-gcc is installed on the host machine
    let mut build = cc::Build::new();
    build
        .compiler("riscv64-elf-gcc") // Use installed compiler (supports rv32 via -march)
        .target(&target)
        .opt_level(3)
        .flag("-march=rv32imf")
        .flag("-mabi=ilp32f")
        .flag("-ffreestanding")
        .flag("-mcmodel=medany")
        .flag("-DZISK_GCC");

    // Include paths
    build.include(softfloat_src.join("include"));
    build.include(&softfloat_build);
    build.include(&softfloat_8086);
    build.include(lib_float_dir.join("src/float"));

    // Add main float handler
    build.file(lib_float_dir.join("src/float/float.c"));

    // Add compiler builtins that Rust doesn't provide for C linkage
    build.file("vendor/compiler_builtins.c");

    // Add all SoftFloat f32 operations
    let f32_files = [
        "f32_add",
        "f32_sub",
        "f32_mul",
        "f32_div",
        "f32_sqrt",
        "f32_mulAdd",
        "f32_eq",
        "f32_lt",
        "f32_le",
        "f32_to_i32",
        "f32_to_ui32",
        "f32_to_i64",
        "f32_to_ui64",
        "i32_to_f32",
        "ui32_to_f32",
        "i64_to_f32",
        "ui64_to_f32",
        "f32_to_f64",
        "f64_to_f32",
    ];

    for file in &f32_files {
        build.file(softfloat_src.join(format!("{}.c", file)));
    }

    // Add f64 operations (required by float.c even though we only support f32)
    let f64_files = [
        "f64_add",
        "f64_sub",
        "f64_mul",
        "f64_div",
        "f64_sqrt",
        "f64_mulAdd",
        "f64_eq",
        "f64_lt",
        "f64_le",
        "f64_to_i32",
        "f64_to_ui32",
        "f64_to_i64",
        "f64_to_ui64",
        "i32_to_f64",
        "ui32_to_f64",
        "i64_to_f64",
        "ui64_to_f64",
    ];

    for file in &f64_files {
        build.file(softfloat_src.join(format!("{}.c", file)));
    }

    // Add SoftFloat support functions
    let support_files = [
        "s_addM",
        "s_subM",
        "s_negXM",
        "s_addMagsF32",
        "s_subMagsF32",
        "s_mulAddF32",
        "s_normRoundPackToF32",
        "s_roundPackToF32",
        "s_normSubnormalF32Sig",
        "s_addMagsF64",
        "s_subMagsF64",
        "s_mulAddF64",
        "s_normRoundPackToF64",
        "s_roundPackToF64",
        "s_normSubnormalF64Sig",
        "s_shiftLeftM",
        "s_shortShiftLeftM",
        "s_shortShiftRightM",
        "s_shortShiftRightJamM",
        "s_shiftRightJam32",
        "s_shiftRightJam64",
        "s_shiftRightJamM",
        "s_shortShiftRightJam64",
        "s_roundToI32",
        "s_roundToUI32",
        "s_roundMToI64",
        "s_roundMToUI64",
        "s_countLeadingZeros32",
        "s_countLeadingZeros64",
        "s_countLeadingZeros8",
        "s_mul64To128M",
        "s_approxRecip32_1",
        "s_approxRecip_1Ks",
        "s_approxRecipSqrt32_1",
        "s_approxRecipSqrt_1Ks",
        "softfloat_state",
    ];

    for file in &support_files {
        build.file(softfloat_src.join(format!("{}.c", file)));
    }

    // Add 8086-specific files
    let files_8086 = [
        "s_propagateNaNF32UI",
        "s_propagateNaNF64UI",
        "softfloat_raiseFlags",
        "s_commonNaNToF32UI",
        "s_commonNaNToF64UI",
        "s_f32UIToCommonNaN",
        "s_f64UIToCommonNaN",
    ];

    for file in &files_8086 {
        build.file(softfloat_8086.join(format!("{}.c", file)));
    }

    // Compile to static library
    build.compile("ziskfloat");

    // Force the linker to include the entire library, preventing garbage collection
    // This ensures _zisk_float is available even if the linker thinks it's unused
    println!("cargo:rustc-link-arg=-Wl,--whole-archive");
    println!("cargo:rustc-link-arg=-lziskfloat");
    println!("cargo:rustc-link-arg=-Wl,--no-whole-archive");

    // Tell cargo to recompile if library sources change
    println!("cargo:rerun-if-changed={}", lib_float_dir.display());
    println!("cargo:rerun-if-changed=vendor/zisk/lib-float/c/src/float/float.h");
    println!("cargo:rerun-if-changed=vendor/zisk/lib-float/c/src/float/float.c");
}
