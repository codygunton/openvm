//! Verify NTT vectors match live Plonky3 code.

use std::path::PathBuf;

use openvm_test_vectors::NttVectors;
use p3_baby_bear::BabyBear;
use p3_dft::{Radix2DitParallel, TwoAdicSubgroupDft};
use p3_field::PrimeField32;

fn vectors_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../executable-spec/tests/test-data/primitives")
}

#[test]
fn verify_ntt_vectors() {
    let json =
        std::fs::read_to_string(vectors_dir().join("ntt.json")).expect("NTT vectors must exist");
    let vectors: NttVectors = serde_json::from_str(&json).unwrap();

    let dft = Radix2DitParallel::<BabyBear>::default();

    for case in &vectors.forward {
        let coeffs: Vec<BabyBear> = case.input.iter().map(|&v| BabyBear::new(v)).collect();
        let evals = dft.dft(coeffs);
        let result: Vec<u32> = evals.iter().map(|x| x.as_canonical_u32()).collect();
        assert_eq!(
            result,
            case.expected,
            "forward NTT mismatch for size {}",
            case.input.len()
        );
    }

    for case in &vectors.inverse {
        let evals: Vec<BabyBear> = case.input.iter().map(|&v| BabyBear::new(v)).collect();
        let coeffs = dft.idft(evals);
        let result: Vec<u32> = coeffs.iter().map(|x| x.as_canonical_u32()).collect();
        assert_eq!(
            result,
            case.expected,
            "inverse NTT mismatch for size {}",
            case.input.len()
        );
    }
}
