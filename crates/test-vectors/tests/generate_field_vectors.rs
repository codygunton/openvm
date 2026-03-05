use std::path::PathBuf;

use openvm_test_vectors::{generate_babybear_field_vectors, write_vectors_json};

fn vectors_output_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../executable-spec/tests/test-data/primitives")
}

#[test]
fn test_generate_field_vectors() {
    let vectors = generate_babybear_field_vectors();
    assert_eq!(vectors.modulus, (1u64 << 31) - (1u64 << 27) + 1);
    assert!(vectors.addition.len() >= 100);
    assert!(vectors.inverse.len() >= 9);

    let output = vectors_output_dir().join("babybear_field.json");
    write_vectors_json(&vectors, &output).expect("Failed to write field vectors");
    println!("Wrote field vectors to {}", output.display());
}
