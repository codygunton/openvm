//! Generate and verify multi-AIR rv32im fibonacci proof vectors.

use std::path::PathBuf;

use openvm_build::{
    build_guest_package, get_dir_with_profile, get_package, GuestOptions, TargetFilter,
};
use openvm_circuit::{
    arch::{
        execution_mode::Segment, InitFileGenerator, Streams, OPENVM_DEFAULT_INIT_FILE_BASENAME,
    },
    utils::{test_system_config, TestStarkEngine},
};
use openvm_instructions::exe::VmExe;
use openvm_rv32im_circuit::{Rv32IConfig, Rv32ImBuilder, Rv32ImConfig};
use openvm_rv32im_transpiler::{
    Rv32ITranspilerExtension, Rv32IoTranspilerExtension, Rv32MTranspilerExtension,
};
use openvm_stark_backend::engine::StarkEngine;
use openvm_stark_backend::p3_matrix::Matrix;
use openvm_stark_sdk::config::FriParameters;
use openvm_stark_sdk::engine::StarkFriEngine;
use openvm_stark_sdk::p3_baby_bear::BabyBear;
use openvm_test_vectors::{
    extract_proof_vectors, hex_decode, matrix_to_u32_vecs, write_vectors_json,
    write_vectors_json_compact, AirInputVectors, E2eProofVectors, ProverInputVectors,
};
use openvm_transpiler::{
    elf::Elf, openvm_platform::memory::MEM_SIZE, transpiler::Transpiler, FromElf,
};

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

/// Build the rv32im fibonacci config, ELF, and executable.
fn build_rv32im_fibonacci() -> (Rv32ImConfig, VmExe<F>, FriParameters) {
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

    (config, exe, fri_params)
}

/// Run the rv32im fibonacci prover with trace extraction.
///
/// Replicates the `air_test_impl` pipeline but intercepts the ProvingContext
/// before `engine.prove()` to extract raw input traces for the Python prover.
fn prove_rv32im_fibonacci_with_traces() -> (E2eProofVectors, ProverInputVectors) {
    use openvm_circuit::arch::{PreflightExecutionOutput, VirtualMachine};
    use openvm_stark_backend::proof::Proof;

    let (config, exe, fri_params) = build_rv32im_fibonacci();

    let engine = TestStarkEngine::new(fri_params);
    let (mut vm, pk) = VirtualMachine::<TestStarkEngine, Rv32ImBuilder>::new_with_keygen(
        engine,
        Rv32ImBuilder,
        config,
    )
    .expect("Failed to create VM with keygen");
    let vk = pk.get_vk();

    let metered_ctx = vm.build_metered_ctx(&exe);
    let (segments, _) = vm
        .metered_interpreter(&exe)
        .expect("Failed to create metered interpreter")
        .execute_metered(Streams::<F>::default(), metered_ctx)
        .expect("Failed to execute metered");

    let cached_program_trace = vm.commit_program_on_device(&exe.program);
    vm.load_program(cached_program_trace);
    let mut preflight_interpreter = vm
        .preflight_interpreter(&exe)
        .expect("Failed to create preflight interpreter");

    let mut state = Some(vm.create_initial_state(&exe, Streams::<F>::default()));
    let mut proofs: Vec<Proof<_>> = Vec::new();
    let mut prover_inputs = None;

    for segment in segments {
        let Segment {
            num_insns,
            trace_heights,
            ..
        } = segment;
        let from_state = Option::take(&mut state).unwrap();
        vm.transport_init_memory_to_device(&from_state.memory);
        let PreflightExecutionOutput {
            system_records,
            record_arenas,
            to_state,
        } = vm
            .execute_preflight(
                &mut preflight_interpreter,
                from_state,
                Some(num_insns),
                &trace_heights,
            )
            .expect("Failed to execute preflight");
        state = Some(to_state);

        let ctx = vm
            .generate_proving_ctx(system_records, record_arenas)
            .expect("Failed to generate proving context");

        // Extract raw input traces before proving consumes the context.
        if prover_inputs.is_none() {
            let device_pk = vm.pk();
            let per_air: Vec<AirInputVectors> = ctx
                .per_air
                .iter()
                .map(|(air_id, air_ctx)| {
                    let common_main =
                        air_ctx
                            .common_main
                            .as_ref()
                            .map(|m| matrix_to_u32_vecs(&m.values, m.width()));
                    let cached_mains: Vec<Vec<Vec<u32>>> = air_ctx
                        .cached_mains
                        .iter()
                        .map(|ctd| matrix_to_u32_vecs(&ctd.trace.values, ctd.trace.width()))
                        .collect();
                    let preprocessed =
                        device_pk.per_air[*air_id]
                            .preprocessed_data
                            .as_ref()
                            .map(|pd| matrix_to_u32_vecs(&pd.trace.values, pd.trace.width()));
                    AirInputVectors {
                        air_id: *air_id,
                        common_main,
                        cached_mains,
                        preprocessed,
                    }
                })
                .collect();
            prover_inputs = Some(ProverInputVectors { per_air });
        }

        let proof = vm.engine.prove(vm.pk(), ctx);
        proofs.push(proof);
    }

    assert!(!proofs.is_empty(), "should have at least one segment proof");
    vm.verify(&vk, &proofs)
        .expect("segment proofs should verify");

    let vectors = extract_proof_vectors("rv32im_fibonacci", &proofs[0], &vk, &fri_params);
    let prover_inputs = prover_inputs.expect("prover_inputs should be captured");
    (vectors, prover_inputs)
}

/// Run the rv32im fibonacci prover (without trace extraction, for proof comparison).
fn prove_rv32im_fibonacci() -> E2eProofVectors {
    use openvm_circuit::utils::air_test_impl;

    let (config, exe, fri_params) = build_rv32im_fibonacci();

    let (_final_memory, vdata) = air_test_impl::<TestStarkEngine, Rv32ImBuilder>(
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
    let (vectors, prover_inputs) = prove_rv32im_fibonacci_with_traces();

    // Summarize trace data
    let total_cells: usize = prover_inputs
        .per_air
        .iter()
        .map(|a| {
            let cm = a
                .common_main
                .as_ref()
                .map_or(0, |m| m.len() * m.first().map_or(0, |r| r.len()));
            let cached: usize = a
                .cached_mains
                .iter()
                .map(|m| m.len() * m.first().map_or(0, |r| r.len()))
                .sum();
            let prep = a
                .preprocessed
                .as_ref()
                .map_or(0, |m| m.len() * m.first().map_or(0, |r| r.len()));
            cm + cached + prep
        })
        .sum();
    println!(
        "Extracted traces for {} AIRs, {} total cells",
        prover_inputs.per_air.len(),
        total_cells
    );

    // Write proof vectors (pretty-printed, ~2MB)
    let output = e2e_output_dir().join("rv32im_fibonacci.json");
    write_vectors_json(&vectors, &output).expect("Failed to write rv32im vectors");
    println!("Wrote rv32im fibonacci vectors to {}", output.display());

    // Write prover input traces (compact JSON, ~15MB)
    let traces_output = e2e_output_dir().join("rv32im_fibonacci_traces.json");
    write_vectors_json_compact(&prover_inputs, &traces_output)
        .expect("Failed to write prover input traces");
    println!(
        "Wrote prover input traces to {}",
        traces_output.display()
    );

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
