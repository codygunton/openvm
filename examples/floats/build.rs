use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Path to the float library sources
    let float_lib_dir = manifest_dir.join("../../extensions/floats/guest/vendor/zisk/lib-float/c");
    let softfloat_src = float_lib_dir.join("SoftFloat-3e/source");
    let softfloat_8086 = softfloat_src.join("8086");
    let softfloat_build = float_lib_dir.join("SoftFloat-3e/build/Linux-x86_64-GCC");

    // Compiler flags for rv32imf
    let cflags = [
        "-march=rv32imf",
        "-mabi=ilp32f",
        "-ffreestanding",
        "-nostdlib",
        "-mcmodel=medany",
        "-O3",
        "-DZISK_GCC",
    ];

    let include_flags = [
        format!("-I{}", softfloat_src.join("include").display()),
        format!("-I{}", softfloat_build.display()),
        format!("-I{}", softfloat_8086.display()),
        format!("-I{}", float_lib_dir.join("src/float").display()),
    ];

    // Find all test assembly files
    let test_dir = manifest_dir.join("tests/asm");
    let test_files: Vec<_> = fs::read_dir(&test_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "S")
                .unwrap_or(false)
                && e.path().file_name().unwrap().to_str().unwrap().starts_with("test_")
        })
        .collect();

    println!("cargo:rerun-if-changed=tests/asm");
    println!("cargo:rerun-if-changed=link.ld");

    for entry in test_files {
        let test_path = entry.path();
        let test_name = test_path.file_stem().unwrap().to_str().unwrap();

        println!("cargo:rerun-if-changed={}", test_path.display());

        // Compile the test assembly file
        let test_obj = out_dir.join(format!("{}.o", test_name));
        let mut cmd = Command::new("riscv64-elf-gcc");
        cmd.args(&cflags)
            .arg("-c")
            .arg(&test_path)
            .arg("-o")
            .arg(&test_obj);

        let status = cmd.status().expect("Failed to compile assembly test");
        assert!(status.success(), "Failed to compile {}", test_name);

        // Compile float library (shared across all tests)
        compile_float_lib(&out_dir, &float_lib_dir, &cflags, &include_flags);

        // Compile SoftFloat library (shared across all tests)
        let softfloat_lib = compile_softfloat(&out_dir, &softfloat_src, &softfloat_8086, &cflags, &include_flags);

        // Link the test
        let test_elf = out_dir.join(format!("{}.elf", test_name));
        let link_script = manifest_dir.join("link.ld");

        let mut link_cmd = Command::new("riscv64-elf-gcc");
        link_cmd.args(&cflags)
            .arg(format!("-T{}", link_script.display()))
            .arg(&test_obj)
            .arg(out_dir.join("float.o"))
            .arg(out_dir.join("compiler_builtins.o"))
            .arg(&softfloat_lib)
            .arg("-lgcc")
            .arg("-o")
            .arg(&test_elf);

        let status = link_cmd.status().expect("Failed to link test");
        assert!(status.success(), "Failed to link {}", test_name);
    }
}

fn compile_float_lib(out_dir: &PathBuf, float_lib_dir: &PathBuf, cflags: &[&str], include_flags: &[String]) {
    let float_obj = out_dir.join("float.o");
    if !float_obj.exists() {
        let mut cmd = Command::new("riscv64-elf-gcc");
        cmd.args(cflags)
            .args(include_flags)
            .arg("-c")
            .arg(float_lib_dir.join("src/float/float.c"))
            .arg("-o")
            .arg(&float_obj);
        let status = cmd.status().expect("Failed to compile float.c");
        assert!(status.success());
    }

    let compiler_builtins_obj = out_dir.join("compiler_builtins.o");
    if !compiler_builtins_obj.exists() {
        let mut cmd = Command::new("riscv64-elf-gcc");
        cmd.args(cflags)
            .args(include_flags)
            .arg("-c")
            .arg("../../extensions/floats/guest/vendor/compiler_builtins.c")
            .arg("-o")
            .arg(&compiler_builtins_obj);
        let status = cmd.status().expect("Failed to compile compiler_builtins.c");
        assert!(status.success());
    }
}

fn compile_softfloat(
    out_dir: &PathBuf,
    softfloat_src: &PathBuf,
    softfloat_8086: &PathBuf,
    cflags: &[&str],
    include_flags: &[String],
) -> PathBuf {
    let lib_path = out_dir.join("libsoftfloat.a");

    if lib_path.exists() {
        return lib_path;
    }

    // Compile essential SoftFloat files
    let softfloat_files = vec![
        // Core operations
        "f32_add.c", "f64_add.c", "f32_sub.c", "f64_sub.c",
        "f32_mul.c", "f64_mul.c", "f32_div.c", "f64_div.c",
        "f32_sqrt.c", "f64_sqrt.c", "f32_mulAdd.c", "f64_mulAdd.c",
        // Comparisons
        "f32_eq.c", "f64_eq.c", "f32_lt.c", "f64_lt.c",
        "f32_le.c", "f64_le.c",
        // Conversions
        "f32_to_f64.c", "f64_to_f32.c",
        "f32_to_i32.c", "f32_to_ui32.c", "f32_to_i64.c", "f32_to_ui64.c",
        "f64_to_i32.c", "f64_to_ui32.c", "f64_to_i64.c", "f64_to_ui64.c",
        "i32_to_f32.c", "ui32_to_f32.c", "i64_to_f32.c", "ui64_to_f32.c",
        "i32_to_f64.c", "ui32_to_f64.c", "i64_to_f64.c", "ui64_to_f64.c",
        // Internal helpers (partial list - add more as needed)
        "s_addMagsF32.c", "s_addMagsF64.c", "s_subMagsF32.c", "s_subMagsF64.c",
        "s_mulAddF32.c", "s_mulAddF64.c",
        "s_normRoundPackToF32.c", "s_normRoundPackToF64.c",
        "s_roundPackToF32.c", "s_roundPackToF64.c",
        "s_normSubnormalF32Sig.c", "s_normSubnormalF64Sig.c",
        "s_shiftRightJam32.c", "s_shiftRightJam64.c", "s_shortShiftRightJam64.c",
        "s_countLeadingZeros32.c", "s_countLeadingZeros64.c", "s_countLeadingZeros8.c",
        "s_approxRecip32_1.c", "s_approxRecipSqrt32_1.c",
        "s_approxRecip_1Ks.c", "s_approxRecipSqrt_1Ks.c",
        "s_roundToI32.c", "s_roundToUI32.c", "s_roundToI64.c", "s_roundToUI64.c",
        "s_addM.c", "s_subM.c", "s_mul64To128M.c",
        "s_shortShiftLeftM.c", "s_shortShiftRightM.c",
        "s_shortShiftRightJamM.c", "s_shiftRightJamM.c", "s_shiftLeftM.c",
        "s_negXM.c", "s_roundMToI64.c", "s_roundMToUI64.c",
        "softfloat_state.c",
    ];

    let softfloat_8086_files = vec![
        "s_propagateNaNF32UI.c", "s_propagateNaNF64UI.c",
        "softfloat_raiseFlags.c",
        "s_commonNaNToF32UI.c", "s_commonNaNToF64UI.c",
        "s_f32UIToCommonNaN.c", "s_f64UIToCommonNaN.c",
    ];

    let mut object_files = Vec::new();

    for file in &softfloat_files {
        let obj = out_dir.join(file.replace(".c", ".o"));
        let mut cmd = Command::new("riscv64-elf-gcc");
        cmd.args(cflags)
            .args(include_flags)
            .arg("-c")
            .arg(softfloat_src.join(file))
            .arg("-o")
            .arg(&obj);
        let status = cmd.status().expect("Failed to compile SoftFloat file");
        assert!(status.success());
        object_files.push(obj);
    }

    for file in &softfloat_8086_files {
        let obj = out_dir.join(format!("8086_{}", file.replace(".c", ".o")));
        let mut cmd = Command::new("riscv64-elf-gcc");
        cmd.args(cflags)
            .args(include_flags)
            .arg("-c")
            .arg(softfloat_8086.join(file))
            .arg("-o")
            .arg(&obj);
        let status = cmd.status().expect("Failed to compile SoftFloat 8086 file");
        assert!(status.success());
        object_files.push(obj);
    }

    // Create library
    let mut ar_cmd = Command::new("riscv64-elf-ar");
    ar_cmd.arg("rcs").arg(&lib_path);
    for obj in &object_files {
        ar_cmd.arg(obj);
    }
    let status = ar_cmd.status().expect("Failed to create SoftFloat library");
    assert!(status.success());

    lib_path
}
