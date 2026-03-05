use std::path::PathBuf;

use openvm_test_vectors::{generate_babybear_ext_field_vectors, write_vectors_json};

fn vectors_output_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../executable-spec/tests/test-data/primitives")
}

#[test]
fn test_generate_ext_field_vectors() {
    let vectors = generate_babybear_ext_field_vectors();
    assert_eq!(vectors.degree, 4);
    // 8 test elements * 8 = 64 pairs
    assert!(vectors.addition.len() >= 64);
    assert!(vectors.multiplication.len() >= 64);
    // 7 non-zero elements
    assert!(vectors.inverse.len() >= 7);

    let output = vectors_output_dir().join("babybear_ext_field.json");
    write_vectors_json(&vectors, &output).expect("Failed to write ext field vectors");
    println!("Wrote ext field vectors to {}", output.display());
}
