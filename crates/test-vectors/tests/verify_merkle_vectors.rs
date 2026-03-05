//! Verify Merkle tree vectors match live Plonky3 code.

use std::path::PathBuf;

use openvm_stark_backend::{p3_commit::Mmcs, p3_matrix::dense::RowMajorMatrix};
use openvm_stark_sdk::config::baby_bear_poseidon2::default_perm;
use openvm_test_vectors::MerkleVectors;
use p3_baby_bear::BabyBear;
use p3_field::{Field, PrimeField32};
use p3_merkle_tree::MerkleTreeMmcs;
use p3_symmetric::{PaddingFreeSponge, TruncatedPermutation};

fn vectors_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../executable-spec/tests/test-data/primitives")
}

#[test]
fn verify_merkle_vectors() {
    type Val = BabyBear;
    type PackedVal = <Val as Field>::Packing;
    type Perm = p3_baby_bear::Poseidon2BabyBear<16>;
    type MyHash = PaddingFreeSponge<Perm, 16, 8, 8>;
    type MyCompress = TruncatedPermutation<Perm, 2, 8, 16>;
    type MyMmcs = MerkleTreeMmcs<PackedVal, PackedVal, MyHash, MyCompress, 8>;

    let perm: Perm = default_perm();
    let hash = MyHash::new(perm.clone());
    let compress = MyCompress::new(perm);
    let mmcs = MyMmcs::new(hash, compress);

    let json = std::fs::read_to_string(vectors_dir().join("merkle.json"))
        .expect("Merkle vectors must exist");
    let vectors: MerkleVectors = serde_json::from_str(&json).unwrap();

    for case in &vectors.construction {
        let num_leaves = case.leaves.len();
        let width = case.leaves[0].len();
        let values: Vec<BabyBear> = case
            .leaves
            .iter()
            .flat_map(|row| row.iter().map(|&v| BabyBear::new(v)))
            .collect();
        let mat = RowMajorMatrix::new(values, width);
        let (commit, _) = mmcs.commit(vec![mat]);

        fn digest_to_u32s(d: &p3_symmetric::Hash<BabyBear, BabyBear, 8>) -> Vec<u32> {
            let arr: [BabyBear; 8] = (*d).into();
            arr.iter().map(|x| x.as_canonical_u32()).collect()
        }

        let root = digest_to_u32s(&commit);
        assert_eq!(
            root, case.expected_root,
            "Merkle root mismatch for {} leaves x {} width",
            num_leaves, width
        );
    }
}
