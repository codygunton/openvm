use std::path::PathBuf;

use openvm_test_vectors::{generate_e2e_fibonacci_vectors, write_vectors_json};

fn e2e_output_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../executable-spec/tests/test-data/e2e")
}

#[test]
fn test_generate_e2e_fibonacci_vectors() {
    let vectors = generate_e2e_fibonacci_vectors();

    // Sanity checks on the generated proof vectors.
    assert_eq!(vectors.program_name, "fibonacci_stark");
    assert!(
        vectors.num_airs > 0,
        "proof should contain at least one AIR"
    );
    assert!(
        !vectors.proof_bytes_hex.is_empty(),
        "proof bytes should not be empty"
    );
    assert!(
        vectors.proof_bytes_len > 0,
        "proof byte length should be positive"
    );

    // Verify main trace commitment structure.
    assert!(
        !vectors.main_trace_commitments.is_empty(),
        "should have at least one main trace commitment"
    );
    for commit in &vectors.main_trace_commitments {
        assert_eq!(commit.len(), 8, "commitment digest should have 8 elements");
    }

    // Verify quotient commitment structure.
    assert_eq!(
        vectors.quotient_commitment.len(),
        8,
        "quotient commitment digest should have 8 elements"
    );

    // Verify per-AIR metadata.
    assert_eq!(vectors.per_air.len(), vectors.num_airs);
    let fib_air = &vectors.per_air[0];
    assert_eq!(fib_air.degree, 16, "Fibonacci trace should have 16 rows");
    assert_eq!(
        fib_air.num_public_values, 3,
        "FibonacciAir should have 3 public values (a, b, result)"
    );
    // Public values: a=0, b=1, last_row_right=987
    // With 16 rows starting from (0,1), the last row is (610, 987).
    assert_eq!(fib_air.public_values[0], 0, "initial a should be 0");
    assert_eq!(fib_air.public_values[1], 1, "initial b should be 1");
    assert_eq!(
        fib_air.public_values[2], 987,
        "last row right value should be 987 (F_16)"
    );

    // Verify FRI parameters match what we specified.
    assert_eq!(vectors.fri_params.log_blowup, 1);
    assert_eq!(vectors.fri_params.num_queries, 2);
    assert_eq!(vectors.fri_params.query_proof_of_work_bits, 0);
    assert_eq!(vectors.fri_params.commit_proof_of_work_bits, 0);

    // Write the JSON output.
    let output = e2e_output_dir().join("fibonacci_stark.json");
    write_vectors_json(&vectors, &output).expect("Failed to write E2E vectors");
    println!("Wrote E2E Fibonacci STARK vectors to {}", output.display());

    // Also write the raw proof bytes to a binary file for binary comparison tests.
    let proof_bytes = hex_decode(&vectors.proof_bytes_hex);
    let bin_output = e2e_output_dir().join("fibonacci_stark_proof.bin");
    std::fs::write(&bin_output, &proof_bytes).expect("Failed to write proof binary");
    println!(
        "Wrote proof binary ({} bytes) to {}",
        proof_bytes.len(),
        bin_output.display()
    );
}

/// Decode a hex string to bytes.
fn hex_decode(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).expect("valid hex"))
        .collect()
}
