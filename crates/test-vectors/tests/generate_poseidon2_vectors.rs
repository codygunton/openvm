use std::path::PathBuf;

use openvm_test_vectors::{generate_poseidon2_vectors, write_vectors_json};

fn vectors_output_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../executable-spec/tests/test-data/primitives")
}

#[test]
fn test_generate_poseidon2_vectors() {
    let vectors = generate_poseidon2_vectors();
    assert_eq!(vectors.width, 16);
    assert!(vectors.permutation.len() >= 4);
    assert!(vectors.compress.len() >= 3);

    // Verify input/output sizes
    for case in &vectors.permutation {
        assert_eq!(case.input.len(), 16);
        assert_eq!(case.expected.len(), 16);
    }
    for case in &vectors.compress {
        assert_eq!(case.left.len(), 8);
        assert_eq!(case.right.len(), 8);
        assert_eq!(case.expected.len(), 8);
    }

    let output = vectors_output_dir().join("poseidon2.json");
    write_vectors_json(&vectors, &output).expect("Failed to write poseidon2 vectors");
    println!("Wrote poseidon2 vectors to {}", output.display());
}
