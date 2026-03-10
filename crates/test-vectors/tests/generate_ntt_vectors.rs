use std::path::PathBuf;

use openvm_test_vectors::{generate_ntt_vectors, write_vectors_json};

fn vectors_output_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../executable-spec/tests/test-data/primitives")
}

#[test]
#[ignore] // Run via generate-test-vectors.sh
fn test_generate_ntt_vectors() {
    let vectors = generate_ntt_vectors();
    // We test sizes 4, 8, 16, 32
    assert_eq!(vectors.forward.len(), 4);
    assert_eq!(vectors.inverse.len(), 4);

    // Verify round-trip: forward then inverse should recover original
    for (fwd, inv) in vectors.forward.iter().zip(vectors.inverse.iter()) {
        assert_eq!(fwd.expected, inv.input);
        assert_eq!(fwd.input, inv.expected);
    }

    let output = vectors_output_dir().join("ntt.json");
    write_vectors_json(&vectors, &output).expect("Failed to write NTT vectors");
    println!("Wrote NTT vectors to {}", output.display());
}
