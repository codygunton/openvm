//! Verify BabyBear field vectors match live Plonky3 code.

use std::path::PathBuf;

use openvm_test_vectors::{BabyBearExtFieldVectors, BabyBearFieldVectors};
use p3_baby_bear::BabyBear;
use p3_field::{extension::BinomialExtensionField, BasedVectorSpace, Field, PrimeField32};

fn vectors_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../executable-spec/tests/test-data/primitives")
}

#[test]
fn verify_base_field_vectors() {
    let json = std::fs::read_to_string(vectors_dir().join("babybear_field.json"))
        .expect("Field vectors must exist — run generate-test-vectors.sh");
    let vectors: BabyBearFieldVectors = serde_json::from_str(&json).unwrap();

    assert_eq!(vectors.modulus, (1u64 << 31) - (1u64 << 27) + 1);

    for case in &vectors.addition {
        let a = BabyBear::new(case.a);
        let b = BabyBear::new(case.b);
        assert_eq!(
            (a + b).as_canonical_u32(),
            case.expected,
            "add: a={}, b={}",
            case.a,
            case.b
        );
    }
    for case in &vectors.subtraction {
        let a = BabyBear::new(case.a);
        let b = BabyBear::new(case.b);
        assert_eq!(
            (a - b).as_canonical_u32(),
            case.expected,
            "sub: a={}, b={}",
            case.a,
            case.b
        );
    }
    for case in &vectors.multiplication {
        let a = BabyBear::new(case.a);
        let b = BabyBear::new(case.b);
        assert_eq!(
            (a * b).as_canonical_u32(),
            case.expected,
            "mul: a={}, b={}",
            case.a,
            case.b
        );
    }
    for case in &vectors.negation {
        let a = BabyBear::new(case.a);
        assert_eq!((-a).as_canonical_u32(), case.expected, "neg: a={}", case.a);
    }
    for case in &vectors.inverse {
        let a = BabyBear::new(case.a);
        assert_eq!(
            a.inverse().as_canonical_u32(),
            case.expected,
            "inv: a={}",
            case.a
        );
    }
}

#[test]
fn verify_ext_field_vectors() {
    type EF = BinomialExtensionField<BabyBear, 4>;

    fn make_ef(coeffs: &[u32; 4]) -> EF {
        let bb = [
            BabyBear::new(coeffs[0]),
            BabyBear::new(coeffs[1]),
            BabyBear::new(coeffs[2]),
            BabyBear::new(coeffs[3]),
        ];
        EF::from_basis_coefficients_fn(|i| bb[i])
    }

    fn ef_to_u32s(e: EF) -> [u32; 4] {
        let s: &[BabyBear] = e.as_basis_coefficients_slice();
        [
            s[0].as_canonical_u32(),
            s[1].as_canonical_u32(),
            s[2].as_canonical_u32(),
            s[3].as_canonical_u32(),
        ]
    }

    let json = std::fs::read_to_string(vectors_dir().join("babybear_ext_field.json"))
        .expect("Ext field vectors must exist");
    let vectors: BabyBearExtFieldVectors = serde_json::from_str(&json).unwrap();

    for case in &vectors.addition {
        let a = make_ef(&case.a.coeffs);
        let b = make_ef(&case.b.coeffs);
        assert_eq!(ef_to_u32s(a + b), case.expected.coeffs, "ext add failed");
    }
    for case in &vectors.multiplication {
        let a = make_ef(&case.a.coeffs);
        let b = make_ef(&case.b.coeffs);
        assert_eq!(ef_to_u32s(a * b), case.expected.coeffs, "ext mul failed");
    }
    for case in &vectors.inverse {
        let a = make_ef(&case.a.coeffs);
        assert_eq!(
            ef_to_u32s(a.inverse()),
            case.expected.coeffs,
            "ext inv failed"
        );
    }
}
