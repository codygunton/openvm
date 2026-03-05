//! Test vector generation for the executable specification.
//!
//! This crate generates golden test vectors by exercising OpenVM's proving
//! pipeline and serializing intermediate values to JSON. The vectors are
//! consumed by both Rust verification tests and Python spec tests.

use std::path::Path;

use p3_baby_bear::BabyBear;
use p3_field::PrimeField32;
use serde::{Deserialize, Serialize};

/// A single field arithmetic test case.
#[derive(Debug, Serialize, Deserialize)]
pub struct FieldOpCase {
    pub a: u32,
    pub b: u32,
    pub expected: u32,
}

/// A single unary field operation test case (negation, inverse).
#[derive(Debug, Serialize, Deserialize)]
pub struct FieldUnaryCase {
    pub a: u32,
    pub expected: u32,
}

/// BabyBear field test vectors.
#[derive(Debug, Serialize, Deserialize)]
pub struct BabyBearFieldVectors {
    pub modulus: u64,
    pub addition: Vec<FieldOpCase>,
    pub subtraction: Vec<FieldOpCase>,
    pub multiplication: Vec<FieldOpCase>,
    pub negation: Vec<FieldUnaryCase>,
    pub inverse: Vec<FieldUnaryCase>,
}

/// Generate BabyBear base field test vectors.
pub fn generate_babybear_field_vectors() -> BabyBearFieldVectors {
    use p3_field::Field;

    let test_values: Vec<u32> = vec![
        0,
        1,
        2,
        100,
        1000,
        BabyBear::ORDER_U32 - 1, // p - 1
        BabyBear::ORDER_U32 - 2, // p - 2
        BabyBear::ORDER_U32 / 2, // ~p/2
        1 << 27,                  // 2^27 (relevant to BabyBear structure)
        (1 << 27) - 1,
    ];

    let mut addition = Vec::new();
    let mut subtraction = Vec::new();
    let mut multiplication = Vec::new();
    let mut negation = Vec::new();
    let mut inverse = Vec::new();

    for &a_val in &test_values {
        let a = BabyBear::new(a_val);

        // Negation
        let neg_a = -a;
        negation.push(FieldUnaryCase {
            a: a_val,
            expected: neg_a.as_canonical_u32(),
        });

        // Inverse (skip 0)
        if a_val != 0 {
            let inv_a = a.inverse();
            inverse.push(FieldUnaryCase {
                a: a_val,
                expected: inv_a.as_canonical_u32(),
            });
        }

        for &b_val in &test_values {
            let b = BabyBear::new(b_val);

            addition.push(FieldOpCase {
                a: a_val,
                b: b_val,
                expected: (a + b).as_canonical_u32(),
            });

            subtraction.push(FieldOpCase {
                a: a_val,
                b: b_val,
                expected: (a - b).as_canonical_u32(),
            });

            multiplication.push(FieldOpCase {
                a: a_val,
                b: b_val,
                expected: (a * b).as_canonical_u32(),
            });
        }
    }

    BabyBearFieldVectors {
        modulus: BabyBear::ORDER_U32 as u64,
        addition,
        subtraction,
        multiplication,
        negation,
        inverse,
    }
}

/// Write vectors to a JSON file, creating parent directories as needed.
pub fn write_vectors_json<T: Serialize>(vectors: &T, path: &Path) -> eyre::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(vectors)?;
    std::fs::write(path, json)?;
    Ok(())
}
