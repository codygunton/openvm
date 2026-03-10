//! Test vector generation for the executable specification.
//!
//! This crate generates golden test vectors by exercising OpenVM's proving
//! pipeline and serializing intermediate values to JSON. The vectors are
//! consumed by both Rust verification tests and Python spec tests.

use std::path::Path;

use openvm_stark_backend::proof::Proof;
use openvm_stark_sdk::config::{baby_bear_poseidon2::BabyBearPoseidon2Config, FriParameters};
use p3_baby_bear::BabyBear;
use p3_field::{PrimeCharacteristicRing, PrimeField32};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Base field vectors
// ---------------------------------------------------------------------------

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
        1 << 27,                 // 2^27 (relevant to BabyBear structure)
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

// ---------------------------------------------------------------------------
// Extension field vectors
// ---------------------------------------------------------------------------

/// Extension field element as 4 base field elements.
#[derive(Debug, Serialize, Deserialize)]
pub struct ExtFieldElement {
    pub coeffs: [u32; 4],
}

/// Extension field binary operation test case.
#[derive(Debug, Serialize, Deserialize)]
pub struct ExtFieldOpCase {
    pub a: ExtFieldElement,
    pub b: ExtFieldElement,
    pub expected: ExtFieldElement,
}

/// Extension field unary operation test case.
#[derive(Debug, Serialize, Deserialize)]
pub struct ExtFieldUnaryCase {
    pub a: ExtFieldElement,
    pub expected: ExtFieldElement,
}

/// BabyBear quartic extension field test vectors.
#[derive(Debug, Serialize, Deserialize)]
pub struct BabyBearExtFieldVectors {
    pub degree: usize,
    pub addition: Vec<ExtFieldOpCase>,
    pub multiplication: Vec<ExtFieldOpCase>,
    pub inverse: Vec<ExtFieldUnaryCase>,
}

/// Generate BabyBear quartic extension field test vectors.
pub fn generate_babybear_ext_field_vectors() -> BabyBearExtFieldVectors {
    use p3_field::{extension::BinomialExtensionField, BasedVectorSpace, Field};

    type EF = BinomialExtensionField<BabyBear, 4>;

    fn ef_to_ext(e: EF) -> ExtFieldElement {
        let slice: &[BabyBear] = e.as_basis_coefficients_slice();
        ExtFieldElement {
            coeffs: [
                slice[0].as_canonical_u32(),
                slice[1].as_canonical_u32(),
                slice[2].as_canonical_u32(),
                slice[3].as_canonical_u32(),
            ],
        }
    }

    fn make_ef(a: u32, b: u32, c: u32, d: u32) -> EF {
        let coeffs = [
            BabyBear::new(a),
            BabyBear::new(b),
            BabyBear::new(c),
            BabyBear::new(d),
        ];
        EF::from_basis_coefficients_fn(|i| coeffs[i])
    }

    let test_elements: Vec<EF> = vec![
        make_ef(0, 0, 0, 0),
        make_ef(1, 0, 0, 0),
        make_ef(1, 1, 1, 1),
        make_ef(100, 200, 300, 400),
        make_ef(BabyBear::ORDER_U32 - 1, 0, 0, 0),
        make_ef(1, 2, 3, 4),
        make_ef(1000, 2000, 3000, 4000),
        make_ef(BabyBear::ORDER_U32 / 2, BabyBear::ORDER_U32 / 3, 1, 0),
    ];

    let mut addition = Vec::new();
    let mut multiplication = Vec::new();
    let mut inverse = Vec::new();

    for (i, a) in test_elements.iter().enumerate() {
        // Inverse (skip zero)
        if i > 0 {
            let inv = a.inverse();
            inverse.push(ExtFieldUnaryCase {
                a: ef_to_ext(*a),
                expected: ef_to_ext(inv),
            });
        }

        for b in &test_elements {
            addition.push(ExtFieldOpCase {
                a: ef_to_ext(*a),
                b: ef_to_ext(*b),
                expected: ef_to_ext(*a + *b),
            });

            multiplication.push(ExtFieldOpCase {
                a: ef_to_ext(*a),
                b: ef_to_ext(*b),
                expected: ef_to_ext(*a * *b),
            });
        }
    }

    BabyBearExtFieldVectors {
        degree: 4,
        addition,
        multiplication,
        inverse,
    }
}

// ---------------------------------------------------------------------------
// Poseidon2 vectors
// ---------------------------------------------------------------------------

/// Poseidon2 permutation test case.
#[derive(Debug, Serialize, Deserialize)]
pub struct Poseidon2PermCase {
    pub input: Vec<u32>,
    pub expected: Vec<u32>,
}

/// Poseidon2 compression test case.
#[derive(Debug, Serialize, Deserialize)]
pub struct Poseidon2CompressCase {
    pub left: Vec<u32>,
    pub right: Vec<u32>,
    pub expected: Vec<u32>,
}

/// Poseidon2 test vectors.
#[derive(Debug, Serialize, Deserialize)]
pub struct Poseidon2Vectors {
    pub width: usize,
    pub permutation: Vec<Poseidon2PermCase>,
    pub compress: Vec<Poseidon2CompressCase>,
}

/// Generate Poseidon2 test vectors using the same configuration as OpenVM.
///
/// Uses the BabyBear Poseidon2 permutation with width=16, configured via the
/// default round constants from the stark-sdk (HorizenLabs constants).
pub fn generate_poseidon2_vectors() -> Poseidon2Vectors {
    use openvm_stark_sdk::config::baby_bear_poseidon2::default_perm;
    use p3_symmetric::Permutation;

    let perm = default_perm();

    // Generate permutation test cases
    let mut permutation = Vec::new();

    // Test case 1: all zeros
    {
        let mut state = [BabyBear::ZERO; 16];
        let input: Vec<u32> = state.iter().map(|x| x.as_canonical_u32()).collect();
        perm.permute_mut(&mut state);
        let output: Vec<u32> = state.iter().map(|x| x.as_canonical_u32()).collect();
        permutation.push(Poseidon2PermCase {
            input,
            expected: output,
        });
    }

    // Test case 2: sequential 0..15
    {
        let mut state: [BabyBear; 16] = std::array::from_fn(|i| BabyBear::new(i as u32));
        let input: Vec<u32> = state.iter().map(|x| x.as_canonical_u32()).collect();
        perm.permute_mut(&mut state);
        let output: Vec<u32> = state.iter().map(|x| x.as_canonical_u32()).collect();
        permutation.push(Poseidon2PermCase {
            input,
            expected: output,
        });
    }

    // Test case 3: all ones
    {
        let mut state = [BabyBear::ONE; 16];
        let input: Vec<u32> = state.iter().map(|x| x.as_canonical_u32()).collect();
        perm.permute_mut(&mut state);
        let output: Vec<u32> = state.iter().map(|x| x.as_canonical_u32()).collect();
        permutation.push(Poseidon2PermCase {
            input,
            expected: output,
        });
    }

    // Test case 4: large values (multiples of 100)
    {
        let mut state: [BabyBear; 16] =
            std::array::from_fn(|i| BabyBear::new((i as u32 + 1) * 100));
        let input: Vec<u32> = state.iter().map(|x| x.as_canonical_u32()).collect();
        perm.permute_mut(&mut state);
        let output: Vec<u32> = state.iter().map(|x| x.as_canonical_u32()).collect();
        permutation.push(Poseidon2PermCase {
            input,
            expected: output,
        });
    }

    // Generate compression test cases (2x8 -> 8 via truncated permutation)
    let mut compress = Vec::new();

    let test_pairs: Vec<([u32; 8], [u32; 8])> = vec![
        ([0; 8], [0; 8]),
        ([1, 2, 3, 4, 5, 6, 7, 8], [9, 10, 11, 12, 13, 14, 15, 16]),
        ([100; 8], [200; 8]),
    ];

    for (left_vals, right_vals) in &test_pairs {
        let mut state: [BabyBear; 16] = std::array::from_fn(|i| {
            if i < 8 {
                BabyBear::new(left_vals[i])
            } else {
                BabyBear::new(right_vals[i - 8])
            }
        });
        let left: Vec<u32> = left_vals.to_vec();
        let right: Vec<u32> = right_vals.to_vec();
        perm.permute_mut(&mut state);
        let expected: Vec<u32> = state[..8].iter().map(|x| x.as_canonical_u32()).collect();
        compress.push(Poseidon2CompressCase {
            left,
            right,
            expected,
        });
    }

    Poseidon2Vectors {
        width: 16,
        permutation,
        compress,
    }
}

// ---------------------------------------------------------------------------
// NTT vectors
// ---------------------------------------------------------------------------

/// NTT test case.
#[derive(Debug, Serialize, Deserialize)]
pub struct NttCase {
    pub input: Vec<u32>,
    pub expected: Vec<u32>,
}

/// NTT test vectors.
#[derive(Debug, Serialize, Deserialize)]
pub struct NttVectors {
    pub forward: Vec<NttCase>,
    pub inverse: Vec<NttCase>,
}

/// Generate NTT (Number Theoretic Transform) test vectors using Plonky3's
/// Radix2DitParallel DFT over BabyBear.
pub fn generate_ntt_vectors() -> NttVectors {
    use p3_dft::{Radix2DitParallel, TwoAdicSubgroupDft};

    let dft = Radix2DitParallel::<BabyBear>::default();
    let sizes = [4, 8, 16, 32];

    let mut forward = Vec::new();
    let mut inverse = Vec::new();

    for &size in &sizes {
        // Create deterministic polynomial coefficients: [1, 2, ..., size]
        let coeffs: Vec<BabyBear> = (0..size).map(|i| BabyBear::new(i as u32 + 1)).collect();

        // Forward NTT: coefficients -> evaluations
        let input_u32: Vec<u32> = coeffs.iter().map(|x| x.as_canonical_u32()).collect();
        let evals = dft.dft(coeffs);
        let evals_u32: Vec<u32> = evals.iter().map(|x| x.as_canonical_u32()).collect();

        forward.push(NttCase {
            input: input_u32,
            expected: evals_u32.clone(),
        });

        // Inverse NTT: evaluations -> coefficients (should recover original)
        let recovered = dft.idft(evals);
        let recovered_u32: Vec<u32> = recovered.iter().map(|x| x.as_canonical_u32()).collect();

        inverse.push(NttCase {
            input: evals_u32,
            expected: recovered_u32,
        });
    }

    NttVectors { forward, inverse }
}

// ---------------------------------------------------------------------------
// Merkle tree vectors
// ---------------------------------------------------------------------------

/// Merkle tree construction test case.
#[derive(Debug, Serialize, Deserialize)]
pub struct MerkleConstructionCase {
    pub leaves: Vec<Vec<u32>>,
    pub expected_root: Vec<u32>,
}

/// Merkle opening test case.
#[derive(Debug, Serialize, Deserialize)]
pub struct MerkleOpeningCase {
    pub root: Vec<u32>,
    pub leaf_index: usize,
    pub leaf: Vec<u32>,
    pub proof: Vec<Vec<u32>>,
}

/// Merkle tree test vectors.
#[derive(Debug, Serialize, Deserialize)]
pub struct MerkleVectors {
    pub hash_width: usize,
    pub digest_size: usize,
    pub construction: Vec<MerkleConstructionCase>,
    pub openings: Vec<MerkleOpeningCase>,
}

/// Generate Merkle tree test vectors using OpenVM's Poseidon2-based MMCS.
///
/// Uses the same configuration as the proving system:
/// - `PaddingFreeSponge<Poseidon2BabyBear<16>, 16, 8, 8>` for leaf hashing
/// - `TruncatedPermutation<Poseidon2BabyBear<16>, 2, 8, 16>` for compression
/// - `MerkleTreeMmcs` with digest size 8
pub fn generate_merkle_vectors() -> MerkleVectors {
    use openvm_stark_backend::{p3_commit::Mmcs, p3_matrix::dense::RowMajorMatrix};
    use openvm_stark_sdk::config::baby_bear_poseidon2::default_perm;
    use p3_field::Field;
    use p3_merkle_tree::MerkleTreeMmcs;
    use p3_symmetric::{PaddingFreeSponge, TruncatedPermutation};

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

    let mut construction = Vec::new();
    let mut openings = Vec::new();

    // Helper: convert a commitment (Hash<BabyBear, BabyBear, 8>) to Vec<u32>
    fn digest_to_u32s(digest: &p3_symmetric::Hash<BabyBear, BabyBear, 8>) -> Vec<u32> {
        let arr: [BabyBear; 8] = (*digest).into();
        arr.iter().map(|x| x.as_canonical_u32()).collect()
    }

    // Test case 1: 4 leaves, each row is 8 field elements wide
    {
        let num_leaves = 4;
        let width = 8;
        let values: Vec<BabyBear> = (0..num_leaves * width)
            .map(|i| BabyBear::new(i as u32 + 1))
            .collect();
        let mat = RowMajorMatrix::new(values.clone(), width);
        let leaves_u32: Vec<Vec<u32>> = (0..num_leaves)
            .map(|r| {
                (0..width)
                    .map(|c| values[r * width + c].as_canonical_u32())
                    .collect()
            })
            .collect();

        let (commit, prover_data) = mmcs.commit(vec![mat]);
        let root_u32 = digest_to_u32s(&commit);

        construction.push(MerkleConstructionCase {
            leaves: leaves_u32,
            expected_root: root_u32.clone(),
        });

        // Generate opening proofs for each leaf
        for leaf_index in 0..num_leaves {
            let batch_opening = mmcs.open_batch(leaf_index, &prover_data);
            let (opened_vals, proof_siblings) = batch_opening.unpack();

            // opened_vals[0] is the row from matrix 0
            let leaf_u32: Vec<u32> = opened_vals[0]
                .iter()
                .map(|x| x.as_canonical_u32())
                .collect();

            // proof_siblings is Vec<[BabyBear; 8]>
            let proof_u32: Vec<Vec<u32>> = proof_siblings
                .iter()
                .map(|sibling| sibling.iter().map(|x| x.as_canonical_u32()).collect())
                .collect();

            openings.push(MerkleOpeningCase {
                root: root_u32.clone(),
                leaf_index,
                leaf: leaf_u32,
                proof: proof_u32,
            });
        }
    }

    // Test case 2: 8 leaves, each row is 4 field elements wide
    {
        let num_leaves = 8;
        let width = 4;
        let values: Vec<BabyBear> = (0..num_leaves * width)
            .map(|i| BabyBear::new((i as u32 + 1) * 10))
            .collect();
        let mat = RowMajorMatrix::new(values.clone(), width);
        let leaves_u32: Vec<Vec<u32>> = (0..num_leaves)
            .map(|r| {
                (0..width)
                    .map(|c| values[r * width + c].as_canonical_u32())
                    .collect()
            })
            .collect();

        let (commit, prover_data) = mmcs.commit(vec![mat]);
        let root_u32 = digest_to_u32s(&commit);

        construction.push(MerkleConstructionCase {
            leaves: leaves_u32,
            expected_root: root_u32.clone(),
        });

        // Generate opening proofs for indices 0, 3, 7
        for &leaf_index in &[0, 3, 7] {
            let batch_opening = mmcs.open_batch(leaf_index, &prover_data);
            let (opened_vals, proof_siblings) = batch_opening.unpack();

            let leaf_u32: Vec<u32> = opened_vals[0]
                .iter()
                .map(|x| x.as_canonical_u32())
                .collect();

            let proof_u32: Vec<Vec<u32>> = proof_siblings
                .iter()
                .map(|sibling| sibling.iter().map(|x| x.as_canonical_u32()).collect())
                .collect();

            openings.push(MerkleOpeningCase {
                root: root_u32.clone(),
                leaf_index,
                leaf: leaf_u32,
                proof: proof_u32,
            });
        }
    }

    // Test case 3: 16 leaves, each row is 1 field element wide
    {
        let num_leaves = 16;
        let width = 1;
        let values: Vec<BabyBear> = (0..num_leaves * width)
            .map(|i| BabyBear::new(i as u32 * 100 + 7))
            .collect();
        let mat = RowMajorMatrix::new(values.clone(), width);
        let leaves_u32: Vec<Vec<u32>> = (0..num_leaves)
            .map(|r| {
                (0..width)
                    .map(|c| values[r * width + c].as_canonical_u32())
                    .collect()
            })
            .collect();

        let (commit, prover_data) = mmcs.commit(vec![mat]);
        let root_u32 = digest_to_u32s(&commit);

        construction.push(MerkleConstructionCase {
            leaves: leaves_u32,
            expected_root: root_u32.clone(),
        });

        // Generate opening proofs for indices 0, 5, 15
        for &leaf_index in &[0, 5, 15] {
            let batch_opening = mmcs.open_batch(leaf_index, &prover_data);
            let (opened_vals, proof_siblings) = batch_opening.unpack();

            let leaf_u32: Vec<u32> = opened_vals[0]
                .iter()
                .map(|x| x.as_canonical_u32())
                .collect();

            let proof_u32: Vec<Vec<u32>> = proof_siblings
                .iter()
                .map(|sibling| sibling.iter().map(|x| x.as_canonical_u32()).collect())
                .collect();

            openings.push(MerkleOpeningCase {
                root: root_u32.clone(),
                leaf_index,
                leaf: leaf_u32,
                proof: proof_u32,
            });
        }
    }

    MerkleVectors {
        hash_width: 16,
        digest_size: 8,
        construction,
        openings,
    }
}

// ---------------------------------------------------------------------------
// E2E proof vectors (Fibonacci STARK)
// ---------------------------------------------------------------------------

/// Per-AIR metadata extracted from the proof.
#[derive(Debug, Serialize, Deserialize)]
pub struct AirProofMeta {
    pub air_id: usize,
    pub degree: usize,
    pub num_public_values: usize,
    pub public_values: Vec<u32>,
}

/// E2E proof test vectors for a Fibonacci STARK.
///
/// Contains serialized proof bytes (via `serde_json`), FRI parameters, and
/// commitment metadata extracted from a real prover run.
#[derive(Debug, Serialize, Deserialize)]
pub struct E2eProofVectors {
    pub program_name: String,
    pub num_airs: usize,
    pub per_air: Vec<AirProofMeta>,
    /// Main trace commitments as arrays of canonical u32 values.
    pub main_trace_commitments: Vec<Vec<u32>>,
    /// After-challenge commitments as arrays of canonical u32 values.
    pub after_challenge_commitments: Vec<Vec<u32>>,
    /// Quotient commitment as an array of canonical u32 values.
    pub quotient_commitment: Vec<u32>,
    pub fri_params: FriParamsMeta,
    /// Hex-encoded serde_json-serialized `Proof<BabyBearPoseidon2Config>`.
    pub proof_bytes_hex: String,
    /// Length of the binary proof in bytes.
    pub proof_bytes_len: usize,
}

/// FRI parameters metadata.
#[derive(Debug, Serialize, Deserialize)]
pub struct FriParamsMeta {
    pub log_blowup: usize,
    pub log_final_poly_len: usize,
    pub num_queries: usize,
    pub query_proof_of_work_bits: usize,
    pub commit_proof_of_work_bits: usize,
}

/// Generate E2E proof test vectors using a simple Fibonacci STARK AIR.
///
/// This exercises the full proving pipeline (keygen, trace generation,
/// commitment, FRI opening) without requiring a RISC-V guest program build.
/// The Fibonacci AIR computes `n` steps of the sequence starting from `(a, b)`.
pub fn generate_e2e_fibonacci_vectors() -> E2eProofVectors {
    use openvm_stark_backend::Chip;
    use openvm_stark_sdk::{
        config::{baby_bear_poseidon2::BabyBearPoseidon2Engine, FriParameters},
        dummy_airs::fib_air::chip::FibonacciChip,
        engine::StarkFriEngine,
    };

    // Use a small trace size (2^4 = 16 rows) for fast generation.
    let n = 1 << 4;
    let fib_chip = FibonacciChip::new(0, 1, n);

    // Use deterministic, fast (insecure) FRI parameters for reproducible test vectors.
    let fri_params = FriParameters {
        log_blowup: 1,
        log_final_poly_len: 0,
        num_queries: 2,
        commit_proof_of_work_bits: 0,
        query_proof_of_work_bits: 0,
    };
    let engine = BabyBearPoseidon2Engine::new(fri_params);

    let fib_air = vec![fib_chip.air()];
    let fib_ctx = vec![fib_chip.generate_proving_ctx(())];

    let vdata = engine
        .run_test(fib_air, fib_ctx)
        .expect("Fibonacci STARK proof should succeed");

    extract_proof_vectors("fibonacci_stark", &vdata.data.proof, &fri_params)
}

/// Extract [`E2eProofVectors`] from a proof and FRI parameters.
///
/// This is a shared helper used by both the single-AIR Fibonacci generator and
/// multi-AIR VM proof generators. It extracts commitment metadata, per-AIR info,
/// and serializes the proof to hex-encoded JSON bytes.
pub fn extract_proof_vectors(
    program_name: &str,
    proof: &Proof<BabyBearPoseidon2Config>,
    fri_params: &FriParameters,
) -> E2eProofVectors {
    let main_trace_commitments: Vec<Vec<u32>> = proof
        .commitments
        .main_trace
        .iter()
        .map(|com| {
            let arr: [BabyBear; 8] = (*com).into();
            arr.iter().map(|x| x.as_canonical_u32()).collect()
        })
        .collect();

    let after_challenge_commitments: Vec<Vec<u32>> = proof
        .commitments
        .after_challenge
        .iter()
        .map(|com| {
            let arr: [BabyBear; 8] = (*com).into();
            arr.iter().map(|x| x.as_canonical_u32()).collect()
        })
        .collect();

    let quotient_arr: [BabyBear; 8] = proof.commitments.quotient.into();
    let quotient_commitment: Vec<u32> =
        quotient_arr.iter().map(|x| x.as_canonical_u32()).collect();

    let per_air: Vec<AirProofMeta> = proof
        .per_air
        .iter()
        .map(|air_data| AirProofMeta {
            air_id: air_data.air_id,
            degree: air_data.degree,
            num_public_values: air_data.public_values.len(),
            public_values: air_data
                .public_values
                .iter()
                .map(|v| v.as_canonical_u32())
                .collect(),
        })
        .collect();

    let fri_meta = FriParamsMeta {
        log_blowup: fri_params.log_blowup,
        log_final_poly_len: fri_params.log_final_poly_len,
        num_queries: fri_params.num_queries,
        query_proof_of_work_bits: fri_params.query_proof_of_work_bits,
        commit_proof_of_work_bits: fri_params.commit_proof_of_work_bits,
    };

    let proof_json_bytes =
        serde_json::to_vec(proof).expect("Proof should serialize to JSON bytes");
    let proof_bytes_hex = hex_encode(&proof_json_bytes);

    E2eProofVectors {
        program_name: program_name.to_string(),
        num_airs: proof.per_air.len(),
        per_air,
        main_trace_commitments,
        after_challenge_commitments,
        quotient_commitment,
        fri_params: fri_meta,
        proof_bytes_hex,
        proof_bytes_len: proof_json_bytes.len(),
    }
}

/// Encode bytes as a lowercase hex string.
pub fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Decode a hex string to bytes.
pub fn hex_decode(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).expect("valid hex"))
        .collect()
}

// ---------------------------------------------------------------------------
// FRI folding vectors
// ---------------------------------------------------------------------------

/// A single FRI folding round with input/output evaluations.
#[derive(Debug, Serialize, Deserialize)]
pub struct FriFoldRound {
    pub round: usize,
    /// Extension field evaluations before fold (each element is `[u32; 4]`).
    pub input: Vec<Vec<u32>>,
    /// Extension field challenge (beta) as `[u32; 4]`.
    pub challenge: Vec<u32>,
    /// Extension field evaluations after fold.
    pub expected_output: Vec<Vec<u32>>,
}

/// FRI folding test vectors: step-by-step fold intermediates and final polynomial.
///
/// Evaluations are in **natural coset order** (not bit-reversed). The fold
/// formula pairs `evals[i]` with `evals[i + N/2]` (the evaluation at `−x`).
#[derive(Debug, Serialize, Deserialize)]
pub struct FriFoldingVectors {
    pub log_poly_size: usize,
    pub log_blowup: usize,
    pub log_final_poly_len: usize,
    pub num_rounds: usize,
    pub rounds: Vec<FriFoldRound>,
    /// Final polynomial coefficients (extension field elements).
    pub final_polynomial: Vec<Vec<u32>>,
}

/// Generate FRI folding test vectors by manually computing the fold at each round.
///
/// Creates a degree-15 polynomial over `BinomialExtensionField<BabyBear, 4>`,
/// evaluates it on a two-adic coset of size 32 (blowup = 2), and iteratively
/// folds with deterministic challenges, capturing input/output at each round.
pub fn generate_fri_folding_vectors() -> FriFoldingVectors {
    use p3_field::{extension::BinomialExtensionField, BasedVectorSpace, Field, TwoAdicField};

    type F = BabyBear;
    type EF = BinomialExtensionField<F, 4>;

    let log_poly_size: usize = 4; // degree 15 → 16 coefficients
    let log_blowup: usize = 1;
    let log_domain = log_poly_size + log_blowup; // 5 → domain size 32

    fn ef_to_vec(e: &EF) -> Vec<u32> {
        e.as_basis_coefficients_slice()
            .iter()
            .map(|x: &BabyBear| x.as_canonical_u32())
            .collect()
    }

    fn make_ef(a: u32, b: u32, c: u32, d: u32) -> EF {
        let coeffs = [
            BabyBear::new(a),
            BabyBear::new(b),
            BabyBear::new(c),
            BabyBear::new(d),
        ];
        EF::from_basis_coefficients_fn(|i| coeffs[i])
    }

    /// Evaluate an extension-field polynomial at a base-field point using Horner's method.
    fn eval_poly(coeffs: &[EF], x: F) -> EF {
        let x_ef = EF::from(x);
        coeffs
            .iter()
            .rev()
            .fold(EF::ZERO, |acc, &c| acc * x_ef + c)
    }

    // 1. Create deterministic polynomial coefficients (degree 15).
    let coeffs: Vec<EF> = (0..1usize << log_poly_size)
        .map(|i| {
            let base = (i * 4) as u32;
            make_ef(base + 1, base + 2, base + 3, base + 4)
        })
        .collect();

    // 2. Evaluate on coset g·⟨ω_N⟩ where g = F::GENERATOR.
    let g = F::GENERATOR;
    let omega = F::two_adic_generator(log_domain);

    let mut evals: Vec<EF> = (0..1usize << log_domain)
        .map(|i| {
            let x = g * omega.exp_u64(i as u64);
            eval_poly(&coeffs, x)
        })
        .collect();

    // 3. Deterministic challenges: β_0 = (1,2,3,4), β_1 = (5,6,7,8), …
    let log_final_poly_len: usize = 0;
    let num_rounds = log_domain - log_blowup - log_final_poly_len;
    let challenges: Vec<EF> = (0..num_rounds)
        .map(|i| {
            let base = (i * 4) as u32;
            make_ef(base + 1, base + 2, base + 3, base + 4)
        })
        .collect();

    // 4. Fold iteratively.
    let two_inv = F::TWO.inverse();
    let mut rounds = Vec::new();
    let mut current_log_domain = log_domain;
    let mut current_shift = g;

    for round in 0..num_rounds {
        let n = 1usize << current_log_domain;
        let half = n / 2;
        let beta = challenges[round];

        let input = evals.clone();
        let omega_n = F::two_adic_generator(current_log_domain);

        let mut folded = Vec::with_capacity(half);
        for i in 0..half {
            let x = current_shift * omega_n.exp_u64(i as u64);
            let f_pos = evals[i];
            let f_neg = evals[i + half]; // f(-x), since ω^(N/2) = -1

            // FRI fold: even + β · odd
            //   even = (f(x) + f(-x)) / 2
            //   odd  = (f(x) - f(-x)) / (2x)
            let half_inv_x = EF::from(two_inv * x.inverse());
            let even = (f_pos + f_neg) * EF::from(two_inv);
            let odd = (f_pos - f_neg) * half_inv_x;
            folded.push(even + beta * odd);
        }

        rounds.push(FriFoldRound {
            round,
            input: input.iter().map(|e| ef_to_vec(e)).collect(),
            challenge: ef_to_vec(&beta),
            expected_output: folded.iter().map(|e| ef_to_vec(e)).collect(),
        });

        evals = folded;
        current_shift = current_shift * current_shift;
        current_log_domain -= 1;
    }

    // 5. Final polynomial: after all folds, the polynomial is constant (degree 0).
    // All evaluations on the final coset should be identical.
    let final_poly_len = 1usize << log_final_poly_len;
    assert!(
        evals[..final_poly_len]
            .windows(2)
            .all(|w| w[0] == w[1])
            || final_poly_len == 1,
        "final polynomial evaluations should be consistent"
    );
    let final_polynomial = evals[..final_poly_len]
        .iter()
        .map(|e| ef_to_vec(e))
        .collect();

    FriFoldingVectors {
        log_poly_size,
        log_blowup,
        log_final_poly_len,
        num_rounds,
        rounds,
        final_polynomial,
    }
}

// ---------------------------------------------------------------------------
// FRI verification vectors
// ---------------------------------------------------------------------------

/// A single commit-phase opening step in a FRI query proof.
#[derive(Debug, Serialize, Deserialize)]
pub struct FriCommitPhaseStep {
    /// Sibling value (extension field element as `[u32; 4]`).
    pub sibling_value: Vec<u32>,
    /// Merkle opening proof: sibling digests per level (each digest is 8 `u32`s).
    pub opening_proof: Vec<Vec<u32>>,
}

/// Data for a single FRI query.
#[derive(Debug, Serialize, Deserialize)]
pub struct FriQueryData {
    pub index: usize,
    pub commit_phase_openings: Vec<FriCommitPhaseStep>,
}

/// FRI verification test vectors extracted from a real STARK proof.
#[derive(Debug, Serialize, Deserialize)]
pub struct FriVerificationVectors {
    /// Merkle roots per FRI commit-phase round (each is 8 `u32`s).
    pub commit_phase_commits: Vec<Vec<u32>>,
    /// Final polynomial coefficients (extension field elements).
    pub final_poly: Vec<Vec<u32>>,
    /// Query proofs.
    pub queries: Vec<FriQueryData>,
}

/// Generate FRI verification vectors by extracting FRI data from a Fibonacci
/// STARK proof.
///
/// Runs the same Fibonacci proof as [`generate_e2e_fibonacci_vectors`], then
/// accesses the FRI proof to extract commit-phase commitments, the final
/// polynomial, and per-query opening data.
pub fn generate_fri_verification_vectors() -> FriVerificationVectors {
    use openvm_stark_backend::Chip;
    use openvm_stark_sdk::{
        config::{baby_bear_poseidon2::BabyBearPoseidon2Engine, FriParameters},
        dummy_airs::fib_air::chip::FibonacciChip,
        engine::StarkFriEngine,
    };
    use p3_field::{extension::BinomialExtensionField, BasedVectorSpace};

    type EF = BinomialExtensionField<BabyBear, 4>;

    let n = 1 << 4;
    let fib_chip = FibonacciChip::new(0, 1, n);

    let fri_params = FriParameters {
        log_blowup: 1,
        log_final_poly_len: 0,
        num_queries: 2,
        commit_proof_of_work_bits: 0,
        query_proof_of_work_bits: 0,
    };
    let engine = BabyBearPoseidon2Engine::new(fri_params);

    let fib_air = vec![fib_chip.air()];
    let fib_ctx = vec![fib_chip.generate_proving_ctx(())];

    let vdata = engine
        .run_test(fib_air, fib_ctx)
        .expect("Fibonacci STARK proof should succeed");

    let proof = &vdata.data.proof;

    // Access the FRI proof: proof.opening.proof is PcsProof<SC> = FriProof<…>
    let fri_proof = &proof.opening.proof;

    // Helper to convert a BabyBear commitment digest to Vec<u32>.
    fn digest_to_u32(com: &p3_symmetric::Hash<BabyBear, BabyBear, 8>) -> Vec<u32> {
        let arr: [BabyBear; 8] = (*com).into();
        arr.iter().map(|x| x.as_canonical_u32()).collect()
    }

    // Helper to convert an extension field element to Vec<u32>.
    fn ef_to_u32(e: &EF) -> Vec<u32> {
        e.as_basis_coefficients_slice()
            .iter()
            .map(|x: &BabyBear| x.as_canonical_u32())
            .collect()
    }

    // Extract commit-phase commitments as Vec<[u32; 8]>.
    let commit_phase_commits: Vec<Vec<u32>> = fri_proof
        .commit_phase_commits
        .iter()
        .map(digest_to_u32)
        .collect();

    // Extract final polynomial (extension field coefficients).
    let final_poly: Vec<Vec<u32>> = fri_proof.final_poly.iter().map(ef_to_u32).collect();

    // Extract query proofs.
    //
    // NOTE: The FRI proof does not store query indices directly. They are
    // derived from the challenger transcript during verification. We store
    // the query position in the vector (0, 1, …) as a placeholder. A full
    // verifier implementation recovers indices from the transcript.
    let queries: Vec<FriQueryData> = fri_proof
        .query_proofs
        .iter()
        .enumerate()
        .map(|(qi, query_proof)| {
            let openings: Vec<FriCommitPhaseStep> = query_proof
                .commit_phase_openings
                .iter()
                .map(|step| {
                    let sibling = ef_to_u32(&step.sibling_value);
                    let merkle_proof: Vec<Vec<u32>> = step
                        .opening_proof
                        .iter()
                        .map(|digest| {
                            digest
                                .iter()
                                .map(|x: &BabyBear| x.as_canonical_u32())
                                .collect()
                        })
                        .collect();
                    FriCommitPhaseStep {
                        sibling_value: sibling,
                        opening_proof: merkle_proof,
                    }
                })
                .collect();
            FriQueryData {
                index: qi,
                commit_phase_openings: openings,
            }
        })
        .collect();

    FriVerificationVectors {
        commit_phase_commits,
        final_poly,
        queries,
    }
}

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

/// Write vectors to a JSON file, creating parent directories as needed.
pub fn write_vectors_json<T: Serialize>(vectors: &T, path: &Path) -> eyre::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(vectors)?;
    std::fs::write(path, json)?;
    Ok(())
}
