//! Verify FRI vectors match live computation.

use std::path::PathBuf;

use openvm_test_vectors::{write_vectors_json, FriFoldingVectors};
use p3_baby_bear::BabyBear;
use p3_field::{
    extension::BinomialExtensionField, BasedVectorSpace, Field, PrimeCharacteristicRing,
    TwoAdicField,
};

fn vectors_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../executable-spec/tests/test-data/fri")
}

type F = BabyBear;
type EF = BinomialExtensionField<F, 4>;

fn vec_to_ef(v: &[u32]) -> EF {
    let coeffs = [
        F::new(v[0]),
        F::new(v[1]),
        F::new(v[2]),
        F::new(v[3]),
    ];
    EF::from_basis_coefficients_fn(|i| coeffs[i])
}

#[test]
fn verify_fri_folding_vectors() {
    let json = std::fs::read_to_string(vectors_dir().join("fri_folding.json"))
        .expect("FRI folding vectors must exist — run generate_fri first");
    let vectors: FriFoldingVectors = serde_json::from_str(&json).unwrap();

    let log_domain = vectors.log_poly_size + vectors.log_blowup;
    let g = F::GENERATOR;
    let two_inv = F::TWO.inverse();

    let mut current_log_domain = log_domain;
    let mut current_shift = g;

    for round_data in &vectors.rounds {
        let n = 1usize << current_log_domain;
        let half = n / 2;
        let omega_n = F::two_adic_generator(current_log_domain);
        let beta = vec_to_ef(&round_data.challenge);

        let input: Vec<EF> = round_data.input.iter().map(|v| vec_to_ef(v)).collect();
        let expected: Vec<EF> = round_data
            .expected_output
            .iter()
            .map(|v| vec_to_ef(v))
            .collect();

        assert_eq!(input.len(), n, "round {} input size mismatch", round_data.round);
        assert_eq!(expected.len(), half, "round {} output size mismatch", round_data.round);

        // Recompute the fold and verify.
        for i in 0..half {
            let x = current_shift * omega_n.exp_u64(i as u64);
            let f_pos = input[i];
            let f_neg = input[i + half];

            let half_inv_x = EF::from(two_inv * x.inverse());
            let even = (f_pos + f_neg) * EF::from(two_inv);
            let odd = (f_pos - f_neg) * half_inv_x;
            let computed = even + beta * odd;

            assert_eq!(
                computed, expected[i],
                "FRI fold mismatch at round {}, index {}",
                round_data.round, i
            );
        }

        current_shift = current_shift * current_shift;
        current_log_domain -= 1;
    }
}

#[test]
fn generate_fri_prover() {
    let vectors = openvm_test_vectors::generate_fri_prover_vectors();

    // Sanity check: 4 folding rounds for log_poly_size=4, log_blowup=1, log_final_poly_len=0.
    assert_eq!(vectors.commit_phase_commits.len(), 4, "expected 4 commit rounds");
    assert_eq!(vectors.betas.len(), 4, "expected 4 betas");
    assert_eq!(vectors.per_round_folded_evals.len(), 4, "expected 4 rounds of folded evals");
    assert_eq!(vectors.query_proofs.len(), 2, "expected 2 query proofs");

    // Verify per-round folded eval sizes: 16, 8, 4, 2.
    assert_eq!(vectors.per_round_folded_evals[0].len(), 16);
    assert_eq!(vectors.per_round_folded_evals[1].len(), 8);
    assert_eq!(vectors.per_round_folded_evals[2].len(), 4);
    assert_eq!(vectors.per_round_folded_evals[3].len(), 2);

    // Verify final polynomial has 1 coefficient.
    assert_eq!(vectors.final_poly.len(), 1, "expected 1 final poly coefficient");

    // Verify each commit is 8 elements.
    for (i, commit) in vectors.commit_phase_commits.iter().enumerate() {
        assert_eq!(commit.len(), 8, "commit {i} should be 8 elements");
    }

    // Verify query proof structure.
    for (qi, qp) in vectors.query_proofs.iter().enumerate() {
        assert_eq!(
            qp.commit_phase_openings.len(),
            4,
            "query {qi}: expected 4 opening steps"
        );
        for (ri, step) in qp.commit_phase_openings.iter().enumerate() {
            assert_eq!(
                step.sibling_value.len(),
                4,
                "query {qi}, round {ri}: sibling should be 4 coefficients"
            );
            // Proof length = log_max_height - 1 - round_idx
            let expected_proof_len = 5 - 1 - ri;
            assert_eq!(
                step.opening_proof.len(),
                expected_proof_len,
                "query {qi}, round {ri}: expected {expected_proof_len} Merkle siblings"
            );
        }
    }

    // Write vectors to JSON.
    let path = vectors_dir().join("fri_prover.json");
    write_vectors_json(&vectors, &path).expect("Failed to write FRI prover vectors");
    eprintln!("Wrote FRI prover vectors to {}", path.display());
}
