use std::path::PathBuf;

use openvm_test_vectors::{generate_e2e_fibonacci_vectors, hex_decode, write_vectors_json};

fn e2e_output_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../executable-spec/tests/test-data/e2e")
}

#[test]
#[ignore] // Run via generate-test-vectors.sh
fn test_generate_e2e_fibonacci_vectors() {
    let vectors = generate_e2e_fibonacci_vectors();

    let output = e2e_output_dir().join("fibonacci_stark.json");
    write_vectors_json(&vectors, &output).expect("Failed to write E2E vectors");
    println!("Wrote E2E Fibonacci STARK vectors to {}", output.display());

    let proof_bytes = hex_decode(&vectors.proof_bytes_hex);
    let bin_output = e2e_output_dir().join("fibonacci_stark_proof.bin");
    std::fs::write(&bin_output, &proof_bytes).expect("Failed to write proof binary");
    println!(
        "Wrote proof binary ({} bytes) to {}",
        proof_bytes.len(),
        bin_output.display()
    );
}

#[test]
fn test_e2e_fibonacci_proof_unchanged() {
    let bin_path = e2e_output_dir().join("fibonacci_stark_proof.bin");
    let committed = std::fs::read(&bin_path).unwrap_or_else(|_| {
        panic!(
            "Committed proof not found at {}.\n\
             Run: ./generate-test-vectors.sh e2e",
            bin_path.display()
        )
    });

    let vectors = generate_e2e_fibonacci_vectors();
    let current = hex_decode(&vectors.proof_bytes_hex);

    assert_eq!(
        committed.len(),
        current.len(),
        "fibonacci_stark proof size changed ({} -> {} bytes)",
        committed.len(),
        current.len()
    );
    assert!(
        committed == current,
        "fibonacci_stark proof bytes differ from committed fixture.\n\
         Prover output has changed. If intentional, run: ./generate-test-vectors.sh e2e"
    );
}
