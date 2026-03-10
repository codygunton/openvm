//! Generate FRI folding and verification test vectors.

use std::path::PathBuf;

use openvm_test_vectors::{
    generate_fri_folding_vectors, generate_fri_verification_vectors, write_vectors_json,
};

fn fri_output_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../executable-spec/tests/test-data/fri")
}

#[test]
#[ignore] // Run via generate-test-vectors.sh
fn test_generate_fri_folding_vectors() {
    let vectors = generate_fri_folding_vectors();

    // Sanity checks.
    assert_eq!(vectors.log_poly_size, 4);
    assert_eq!(vectors.log_blowup, 1);
    assert_eq!(vectors.num_rounds, 4);
    assert_eq!(vectors.rounds.len(), 4);

    // Round 0: 32 evaluations → 16.
    assert_eq!(vectors.rounds[0].input.len(), 32);
    assert_eq!(vectors.rounds[0].expected_output.len(), 16);

    // Round 1: 16 → 8.
    assert_eq!(vectors.rounds[1].input.len(), 16);
    assert_eq!(vectors.rounds[1].expected_output.len(), 8);

    // Round 2: 8 → 4.
    assert_eq!(vectors.rounds[2].input.len(), 8);
    assert_eq!(vectors.rounds[2].expected_output.len(), 4);

    // Round 3: 4 → 2.
    assert_eq!(vectors.rounds[3].input.len(), 4);
    assert_eq!(vectors.rounds[3].expected_output.len(), 2);

    // Each extension field element should have 4 components.
    for round in &vectors.rounds {
        assert_eq!(round.challenge.len(), 4);
        for eval in &round.input {
            assert_eq!(eval.len(), 4);
        }
        for eval in &round.expected_output {
            assert_eq!(eval.len(), 4);
        }
    }

    // Final polynomial: constant (1 coefficient).
    assert_eq!(vectors.final_polynomial.len(), 1);
    assert_eq!(vectors.final_polynomial[0].len(), 4);

    // Consistency: each round's output matches the next round's input.
    for i in 0..vectors.rounds.len() - 1 {
        assert_eq!(
            vectors.rounds[i].expected_output,
            vectors.rounds[i + 1].input,
            "round {} output should match round {} input",
            i,
            i + 1
        );
    }

    let output = fri_output_dir().join("fri_folding.json");
    write_vectors_json(&vectors, &output).expect("Failed to write FRI folding vectors");
    println!("Wrote FRI folding vectors to {}", output.display());
}

#[test]
#[ignore] // Run via generate-test-vectors.sh
fn test_generate_fri_verification_vectors() {
    let vectors = generate_fri_verification_vectors();

    // Should have commit-phase commitments (one per FRI round).
    assert!(
        !vectors.commit_phase_commits.is_empty(),
        "should have at least one commit-phase commitment"
    );
    for commit in &vectors.commit_phase_commits {
        assert_eq!(
            commit.len(),
            8,
            "each commitment digest should have 8 elements"
        );
    }

    // Final polynomial should have extension field elements.
    assert!(
        !vectors.final_poly.is_empty(),
        "final polynomial should not be empty"
    );
    for coeff in &vectors.final_poly {
        assert_eq!(
            coeff.len(),
            4,
            "each extension field element should have 4 components"
        );
    }

    // Should have query proofs.
    assert_eq!(
        vectors.queries.len(),
        2,
        "should have 2 query proofs (num_queries=2)"
    );
    for query in &vectors.queries {
        assert!(
            !query.commit_phase_openings.is_empty(),
            "each query should have commit-phase openings"
        );
        for step in &query.commit_phase_openings {
            assert_eq!(step.sibling_value.len(), 4);
            for digest in &step.opening_proof {
                assert_eq!(digest.len(), 8, "Merkle sibling digest should be 8 u32s");
            }
        }
    }

    let output = fri_output_dir().join("fri_verification.json");
    write_vectors_json(&vectors, &output).expect("Failed to write FRI verification vectors");
    println!(
        "Wrote FRI verification vectors to {}",
        output.display()
    );
}
