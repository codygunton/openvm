//! Generate and verify multi-AIR rv32im fibonacci proof vectors.

use std::path::PathBuf;

use openvm_build::{
    build_guest_package, get_dir_with_profile, get_package, GuestOptions, TargetFilter,
};
use openvm_circuit::{
    arch::{InitFileGenerator, Streams, OPENVM_DEFAULT_INIT_FILE_BASENAME},
    utils::{air_test_impl, test_system_config},
};
use openvm_instructions::exe::VmExe;
use openvm_rv32im_circuit::{Rv32IConfig, Rv32ImBuilder, Rv32ImConfig};
use openvm_rv32im_transpiler::{
    Rv32ITranspilerExtension, Rv32IoTranspilerExtension, Rv32MTranspilerExtension,
};
use openvm_stark_sdk::config::FriParameters;
use openvm_stark_sdk::p3_baby_bear::BabyBear;
use openvm_test_vectors::{extract_proof_vectors, hex_decode, write_vectors_json, E2eProofVectors};
use openvm_transpiler::{elf::Elf, openvm_platform::memory::MEM_SIZE, transpiler::Transpiler, FromElf};

type F = BabyBear;

fn e2e_output_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../executable-spec/tests/test-data/e2e")
}

fn programs_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../extensions/rv32im/tests/programs")
}

/// Stable target directory for guest builds (enables incremental compilation).
fn guest_target_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/guest-build")
}

/// Build a guest example program using a stable target dir for incremental builds.
fn build_example_elf(
    manifest_dir: &PathBuf,
    example_name: &str,
    init_config: &impl InitFileGenerator,
) -> Elf {
    let pkg = get_package(manifest_dir);
    let target_dir = guest_target_dir();
    let guest_opts = GuestOptions::default().with_target_dir(&target_dir);
    init_config
        .write_to_init_file(
            manifest_dir,
            Some(&format!("{OPENVM_DEFAULT_INIT_FILE_BASENAME}_{example_name}.rs")),
        )
        .expect("Failed to write init file");
    if let Err(Some(code)) = build_guest_package(
        &pkg,
        &guest_opts,
        None,
        &Some(TargetFilter {
            name: example_name.to_string(),
            kind: "example".to_string(),
        }),
    ) {
        panic!("Guest build failed with exit code {code}");
    }
    let elf_path = pkg
        .targets
        .iter()
        .find(|target| target.name == example_name)
        .map(|target| {
            get_dir_with_profile(&target_dir, "release", true)
                .join(&target.name)
                .to_path_buf()
        })
        .expect("Could not find target binary");
    let data = std::fs::read(&elf_path).expect("Failed to read built ELF");
    Elf::decode(&data, MEM_SIZE as u32).expect("Failed to decode ELF")
}

/// Run the rv32im fibonacci prover and return extracted vectors.
fn prove_rv32im_fibonacci() -> E2eProofVectors {
    let config = Rv32ImConfig {
        rv32i: Rv32IConfig {
            system: test_system_config(),
            ..Default::default()
        },
        ..Default::default()
    };

    let elf = build_example_elf(&programs_dir(), "fibonacci", &config);

    let exe = VmExe::from_elf(
        elf,
        Transpiler::<F>::default()
            .with_extension(Rv32ITranspilerExtension)
            .with_extension(Rv32IoTranspilerExtension)
            .with_extension(Rv32MTranspilerExtension),
    )
    .expect("Failed to transpile ELF");

    let fri_params = FriParameters {
        log_blowup: 1,
        log_final_poly_len: 0,
        num_queries: 2,
        commit_proof_of_work_bits: 0,
        query_proof_of_work_bits: 0,
    };

    let (_final_memory, vdata) = air_test_impl::<
        openvm_circuit::utils::TestStarkEngine,
        Rv32ImBuilder,
    >(
        fri_params,
        Rv32ImBuilder,
        config,
        exe,
        Streams::default(),
        1,
        false,
    )
    .expect("rv32im fibonacci proof should succeed");

    assert!(!vdata.is_empty(), "should have at least one segment proof");
    extract_proof_vectors(
        "rv32im_fibonacci",
        &vdata[0].data.proof,
        &vdata[0].data.vk,
        &fri_params,
    )
}

#[test]
#[ignore] // Run via generate-test-vectors.sh
fn generate_rv32im_fibonacci_vectors() {
    let vectors = prove_rv32im_fibonacci();

    let output = e2e_output_dir().join("rv32im_fibonacci.json");
    write_vectors_json(&vectors, &output).expect("Failed to write rv32im vectors");
    println!("Wrote rv32im fibonacci vectors to {}", output.display());

    let proof_bytes = hex_decode(&vectors.proof_bytes_hex);
    let bin_output = e2e_output_dir().join("rv32im_fibonacci_proof.bin");
    std::fs::write(&bin_output, &proof_bytes).expect("Failed to write proof binary");
    println!(
        "Wrote proof binary ({} bytes) to {}",
        proof_bytes.len(),
        bin_output.display()
    );
}

#[test]
fn test_rv32im_fibonacci_proof_unchanged() {
    let bin_path = e2e_output_dir().join("rv32im_fibonacci_proof.bin");
    let committed = std::fs::read(&bin_path).unwrap_or_else(|_| {
        panic!(
            "Committed proof not found at {}.\n\
             Run: ./generate-test-vectors.sh e2e",
            bin_path.display()
        )
    });

    let vectors = prove_rv32im_fibonacci();
    let current = hex_decode(&vectors.proof_bytes_hex);

    assert_eq!(
        committed.len(),
        current.len(),
        "rv32im_fibonacci proof size changed ({} -> {} bytes)",
        committed.len(),
        current.len()
    );
    assert!(
        committed == current,
        "rv32im_fibonacci proof bytes differ from committed fixture.\n\
         Prover output has changed. If intentional, run: ./generate-test-vectors.sh e2e"
    );
}
