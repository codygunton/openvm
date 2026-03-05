//! Verify Poseidon2 vectors match live Plonky3 code.

use std::path::PathBuf;

use openvm_stark_sdk::config::baby_bear_poseidon2::default_perm;
use openvm_test_vectors::Poseidon2Vectors;
use p3_baby_bear::BabyBear;
use p3_field::PrimeField32;
use p3_symmetric::Permutation;

fn vectors_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../executable-spec/tests/test-data/primitives")
}

#[test]
fn verify_poseidon2_vectors() {
    let json = std::fs::read_to_string(vectors_dir().join("poseidon2.json"))
        .expect("Poseidon2 vectors must exist");
    let vectors: Poseidon2Vectors = serde_json::from_str(&json).unwrap();

    let perm = default_perm();

    for case in &vectors.permutation {
        let mut state: [BabyBear; 16] = std::array::from_fn(|i| BabyBear::new(case.input[i]));
        perm.permute_mut(&mut state);
        let result: Vec<u32> = state.iter().map(|x| x.as_canonical_u32()).collect();
        assert_eq!(
            result, case.expected,
            "permutation mismatch for input {:?}",
            case.input
        );
    }

    for case in &vectors.compress {
        let mut state: [BabyBear; 16] = std::array::from_fn(|i| {
            if i < 8 {
                BabyBear::new(case.left[i])
            } else {
                BabyBear::new(case.right[i - 8])
            }
        });
        perm.permute_mut(&mut state);
        let result: Vec<u32> = state[..8].iter().map(|x| x.as_canonical_u32()).collect();
        assert_eq!(result, case.expected, "compress mismatch");
    }
}
