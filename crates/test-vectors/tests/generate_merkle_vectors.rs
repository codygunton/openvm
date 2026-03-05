use std::path::PathBuf;

use openvm_test_vectors::{generate_merkle_vectors, write_vectors_json};

fn vectors_output_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../executable-spec/tests/test-data/primitives")
}

#[test]
fn test_generate_merkle_vectors() {
    let vectors = generate_merkle_vectors();
    assert_eq!(vectors.hash_width, 16);
    assert_eq!(vectors.digest_size, 8);

    // 3 construction cases (4, 8, 16 leaves)
    assert_eq!(vectors.construction.len(), 3);

    // Verify root digests are 8 elements
    for case in &vectors.construction {
        assert_eq!(case.expected_root.len(), 8);
    }

    // Multiple opening proofs
    assert!(vectors.openings.len() >= 10);

    // Verify opening proof structure
    for opening in &vectors.openings {
        assert_eq!(opening.root.len(), 8);
        // Each sibling in the proof should be 8 elements
        for sibling in &opening.proof {
            assert_eq!(sibling.len(), 8);
        }
    }

    let output = vectors_output_dir().join("merkle.json");
    write_vectors_json(&vectors, &output).expect("Failed to write Merkle vectors");
    println!("Wrote Merkle vectors to {}", output.display());
}
