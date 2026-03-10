//! Generate and verify multi-AIR revm_transfer proof vectors.
//!
//! This test requires the `revm-vectors` feature and is marked `#[ignore]`
//! because it is expensive (builds revm guest, proves 100 EVM transfers).
//! Optionally enable `cuda` for GPU-accelerated proving.
//!
//! ```sh
//! # CPU proving
//! cargo test --profile fast -p openvm-test-vectors --features revm-vectors \
//!     --test generate_revm_vectors -- --ignored --test-threads=1
//!
//! # GPU proving (much faster)
//! cargo test --profile fast -p openvm-test-vectors --features revm-vectors,cuda \
//!     --test generate_revm_vectors -- --ignored --test-threads=1
//! ```

use std::path::PathBuf;

use openvm_build::{build_guest_package, get_package, guest_methods, GuestOptions};
use openvm_circuit::arch::InitFileGenerator;
use openvm_sdk::{
    config::{AppFriParams, SdkVmConfig},
    Sdk, StdIn,
};
use openvm_stark_sdk::config::FriParameters;
use openvm_test_vectors::{extract_proof_vectors, hex_decode, write_vectors_json, E2eProofVectors};
use openvm_transpiler::{elf::Elf, openvm_platform::memory::MEM_SIZE};

fn e2e_output_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../executable-spec/tests/test-data/e2e")
}

fn revm_transfer_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../benchmarks/guest/revm_transfer")
}

/// Stable target directory for guest builds (enables incremental compilation).
fn guest_target_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/guest-build")
}

/// Build a guest binary package into an ELF.
fn build_guest_elf(manifest_dir: &PathBuf, config: &SdkVmConfig) -> Elf {
    config
        .write_to_init_file(manifest_dir, None)
        .expect("Failed to write init file");

    let pkg = get_package(manifest_dir);
    let target_dir = guest_target_dir();
    let guest_opts = GuestOptions::default().with_target_dir(&target_dir);

    if let Err(Some(code)) = build_guest_package(&pkg, &guest_opts, None, &None) {
        panic!("Guest build failed with exit code {code}");
    }

    let elf_path = guest_methods(&pkg, &target_dir, &guest_opts.features, &guest_opts.profile)
        .pop()
        .expect("No ELF output from guest build");

    let data = std::fs::read(&elf_path).expect("Failed to read built ELF");
    Elf::decode(&data, MEM_SIZE as u32).expect("Failed to decode ELF")
}

/// Run the revm_transfer prover and return extracted vectors.
fn prove_revm_transfer() -> E2eProofVectors {
    let mut app_config =
        SdkVmConfig::from_toml(include_str!("../../../benchmarks/guest/revm_transfer/openvm.toml"))
            .expect("Failed to parse openvm.toml");

    let manifest_dir = revm_transfer_dir();
    let elf = build_guest_elf(&manifest_dir, &app_config.app_vm_config);

    let fri_params = FriParameters {
        log_blowup: 1,
        log_final_poly_len: 0,
        num_queries: 2,
        commit_proof_of_work_bits: 0,
        query_proof_of_work_bits: 0,
    };
    app_config.app_fri_params = AppFriParams::from(fri_params);

    let sdk = Sdk::new(app_config).expect("Failed to create SDK");

    let mut prover = sdk
        .app_prover(elf)
        .expect("Failed to create app prover")
        .with_program_name("revm_transfer");
    let proof = prover.prove(StdIn::default()).expect("revm_transfer proof should succeed");

    assert!(
        !proof.per_segment.is_empty(),
        "should have at least one segment proof"
    );
    extract_proof_vectors("revm_transfer", &proof.per_segment[0], &fri_params)
}

#[test]
#[ignore] // Run via: CUDA=1 ./generate-test-vectors.sh heavy
fn generate_revm_transfer_vectors() {
    let vectors = prove_revm_transfer();

    let output = e2e_output_dir().join("revm_transfer.json");
    write_vectors_json(&vectors, &output).expect("Failed to write revm vectors");
    println!("Wrote revm_transfer vectors to {}", output.display());

    let proof_bytes = hex_decode(&vectors.proof_bytes_hex);
    let bin_output = e2e_output_dir().join("revm_transfer_proof.bin");
    std::fs::write(&bin_output, &proof_bytes).expect("Failed to write proof binary");
    println!(
        "Wrote proof binary ({} bytes) to {}",
        proof_bytes.len(),
        bin_output.display()
    );
}

#[test]
#[ignore] // Run via: ./run-tests.sh heavy
fn test_revm_transfer_proof_unchanged() {
    let bin_path = e2e_output_dir().join("revm_transfer_proof.bin");
    let committed = std::fs::read(&bin_path).unwrap_or_else(|_| {
        panic!(
            "Committed proof not found at {}.\n\
             Run: CUDA=1 ./generate-test-vectors.sh heavy",
            bin_path.display()
        )
    });

    let vectors = prove_revm_transfer();
    let current = hex_decode(&vectors.proof_bytes_hex);

    assert_eq!(
        committed.len(),
        current.len(),
        "revm_transfer proof size changed ({} -> {} bytes)",
        committed.len(),
        current.len()
    );
    assert!(
        committed == current,
        "revm_transfer proof bytes differ from committed fixture.\n\
         Prover output has changed. If intentional, run: CUDA=1 ./generate-test-vectors.sh heavy"
    );
}
