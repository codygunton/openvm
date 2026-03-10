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

/// Challenger (duplex sponge) internal state, exported for Python seeding.
#[derive(Debug, Serialize, Deserialize)]
pub struct ChallengerState {
    /// Sponge state: 16 `u32` field elements.
    pub sponge_state: Vec<u32>,
    /// Input buffer (absorbed but not yet permuted).
    pub input_buffer: Vec<u32>,
    /// Output buffer (available for squeezing).
    pub output_buffer: Vec<u32>,
}

/// Golden values for a single FRI query verification.
#[derive(Debug, Serialize, Deserialize)]
pub struct QueryVerificationGolden {
    /// Query index (derived from transcript).
    pub query_index: usize,
    /// Initial folded_eval (reduced opening, extension field `[u32; 4]`).
    pub reduced_opening: Vec<u32>,
    /// Folded eval after each round (extension field `[u32; 4]`).
    pub per_round_folded_eval: Vec<Vec<u32>>,
    /// Final folded_eval (extension field `[u32; 4]`).
    pub final_folded_eval: Vec<u32>,
    /// Expected final polynomial evaluation (extension field `[u32; 4]`).
    pub expected_final_eval: Vec<u32>,
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
    /// Challenger state at FRI entry point (after STARK transcript, before FRI).
    pub challenger_state: ChallengerState,
    /// Log2 of blowup factor.
    pub log_blowup: usize,
    /// Log2 of final polynomial length.
    pub log_final_poly_len: usize,
    /// Number of FRI queries.
    pub num_queries: usize,
    /// Log2 of max committed polynomial height (after blowup).
    pub log_max_height: usize,
    /// FRI folding challenges (betas), one per commit-phase round.
    pub betas: Vec<Vec<u32>>,
    /// Per-query golden verification data.
    pub query_verifications: Vec<QueryVerificationGolden>,
}

/// Generate FRI verification vectors by extracting FRI data from a Fibonacci
/// STARK proof.
///
/// Runs the same Fibonacci proof as [`generate_e2e_fibonacci_vectors`], then:
/// 1. Extracts FRI proof data (commits, final poly, query openings).
/// 2. Replays the STARK Fiat-Shamir transcript to capture the challenger's
///    internal state at the FRI entry point.
/// 3. Continues the FRI transcript to derive betas and query indices.
/// 4. Computes golden reduced openings and fold-chain intermediates per query.
pub fn generate_fri_verification_vectors() -> FriVerificationVectors {
    use openvm_stark_backend::{
        engine::StarkEngine,
        p3_challenger::{CanObserve, CanSampleBits, FieldChallenger, GrindingChallenger},
        p3_util::reverse_bits_len,
        Chip,
    };
    use openvm_stark_sdk::{
        config::{baby_bear_poseidon2::BabyBearPoseidon2Engine, FriParameters},
        dummy_airs::fib_air::chip::FibonacciChip,
        engine::StarkFriEngine,
    };
    use p3_field::{extension::BinomialExtensionField, BasedVectorSpace, Field, TwoAdicField};

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

    let vk = &vdata.data.vk;
    let proof = &vdata.data.proof;
    let fp = &vdata.fri_params;

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

    // =========================================================================
    // Replay STARK transcript to capture challenger state at FRI entry.
    //
    // Reference: stark-backend verifier/mod.rs verify_raps()
    // =========================================================================
    let mut challenger = engine.new_challenger();

    // Step 1: Observe VK pre_hash (Hash<F,F,8> → 8 field elements)
    challenger.observe(vk.pre_hash.clone());

    // Step 2: Observe num_airs
    let num_airs = proof.per_air.len();
    challenger.observe(BabyBear::from_usize(num_airs));

    // Step 3: Observe air_ids
    for ap in &proof.per_air {
        challenger.observe(BabyBear::from_usize(ap.air_id));
    }

    // Step 4: Observe public values per AIR
    for ap in &proof.per_air {
        challenger.observe_slice(&ap.public_values);
    }

    // Step 5: Observe preprocessed commits (none for Fibonacci)
    for svk in &vk.inner.per_air {
        if let Some(prep) = &svk.preprocessed_data {
            challenger.observe(prep.commit.clone());
        }
    }

    // Step 6: Observe main trace commitments
    challenger.observe_slice(&proof.commitments.main_trace);

    // Step 7: Observe log degrees
    let log_degrees: Vec<BabyBear> = proof
        .per_air
        .iter()
        .map(|ap| {
            BabyBear::from_usize(openvm_stark_backend::p3_util::log2_strict_usize(ap.degree))
        })
        .collect();
    challenger.observe_slice(&log_degrees);

    // Step 8: RAP phase — no-op for Fibonacci (no interactions).
    // FriLogUpPhase::partially_verify returns early when after_challenge is empty.

    // Step 9: Sample alpha
    let _alpha: EF = challenger.sample_algebra_element();

    // Step 10: Observe quotient commitment
    challenger.observe(proof.commitments.quotient.clone());

    // Step 11: check_witness(deep_pow_bits, deep_pow_witness)
    assert!(
        challenger.check_witness(vk.inner.deep_pow_bits, proof.opening.deep_pow_witness),
        "Deep PoW check failed during transcript replay"
    );

    // Step 12: Sample zeta
    let zeta: EF = challenger.sample_algebra_element();

    // Step 13: PCS observe — opening values in rounds order.
    // Reference: two_adic_pcs.rs TwoAdicFriPcs::verify() observes all values
    // before calling verify_fri. Rounds order: preprocessed, cached_mains,
    // common_main, after_challenge, quotient.
    //
    // For Fibonacci: common main (1 matrix, 2 points), then quotient chunks.
    for adj in &proof.opening.values.main[0] {
        challenger.observe_algebra_slice(&adj.local);
        challenger.observe_algebra_slice(&adj.next);
    }
    for air_q in &proof.opening.values.quotient {
        for chunk_vals in air_q {
            challenger.observe_algebra_slice(chunk_vals);
        }
    }

    // =========================================================================
    // Capture challenger state at FRI entry point
    // =========================================================================
    let challenger_state = ChallengerState {
        sponge_state: challenger
            .sponge_state
            .iter()
            .map(|x| x.as_canonical_u32())
            .collect(),
        input_buffer: challenger
            .input_buffer
            .iter()
            .map(|x| x.as_canonical_u32())
            .collect(),
        output_buffer: challenger
            .output_buffer
            .iter()
            .map(|x| x.as_canonical_u32())
            .collect(),
    };

    // =========================================================================
    // Continue FRI transcript: derive alpha, betas, query indices.
    //
    // Reference: p3-fri-0.4.1 verifier.rs verify_fri()
    // =========================================================================

    // Step 14: Sample FRI alpha (batch combination challenge).
    // This is the alpha used in open_input to combine polynomial evaluations,
    // distinct from the STARK alpha sampled at step 9.
    let fri_alpha: EF = challenger.sample_algebra_element();

    let log_blowup = fp.log_blowup;
    let log_final_poly_len = fp.log_final_poly_len;
    let num_queries = fp.num_queries;
    let num_rounds = fri_proof.commit_phase_commits.len();
    let log_max_height = num_rounds + log_blowup + log_final_poly_len;

    // Step 15: Derive betas (one per commit-phase round)
    let mut betas: Vec<EF> = Vec::new();
    for (comm, witness) in fri_proof
        .commit_phase_commits
        .iter()
        .zip(fri_proof.commit_pow_witnesses.iter())
    {
        challenger.observe(comm.clone());
        // commit_proof_of_work_bits = 0 → check_witness returns true immediately
        assert!(challenger.check_witness(fp.commit_proof_of_work_bits, *witness));
        let beta: EF = challenger.sample_algebra_element();
        betas.push(beta);
    }

    // Step 16: Observe final polynomial coefficients
    challenger.observe_algebra_slice(&fri_proof.final_poly);

    // Step 17: Query PoW (query_proof_of_work_bits = 0 → no-op)
    assert!(challenger.check_witness(fp.query_proof_of_work_bits, fri_proof.query_pow_witness));

    // =========================================================================
    // Per-query: derive index, compute reduced_opening, replay fold chain.
    //
    // Reference: p3-fri-0.4.1 verifier.rs verify_query(), open_input()
    //            p3-fri-0.4.1 two_adic_pcs.rs fold_row()
    // =========================================================================

    // Precompute next_zeta = zeta * omega_{trace_domain}
    let log_degree =
        openvm_stark_backend::p3_util::log2_strict_usize(proof.per_air[0].degree);
    let trace_omega = BabyBear::two_adic_generator(log_degree);
    let next_zeta: EF = zeta * EF::from(trace_omega);

    let quotient_degree = vk.inner.per_air[0].quotient_degree as usize;

    let mut query_verifications = Vec::new();

    for (qi, query_proof) in fri_proof.query_proofs.iter().enumerate() {
        // Step 18: Sample query index
        let query_index: usize = challenger.sample_bits(log_max_height);

        // -----------------------------------------------------------------
        // Compute reduced_opening via open_input logic.
        //
        // For Fibonacci: all matrices at log_height = log_max_height = 5.
        // x = GENERATOR * omega_{2^log_max}^{reverse_bits(query_index, log_max)}
        // -----------------------------------------------------------------
        let rev_idx = reverse_bits_len(query_index, log_max_height);
        let x_base = BabyBear::from_u32(31) // GENERATOR = 31
            * BabyBear::two_adic_generator(log_max_height).exp_u64(rev_idx as u64);
        let x: EF = EF::from(x_base);

        let mut ro = EF::ZERO;
        let mut alpha_pow = EF::ONE;

        // --- Common main contribution ---
        // input_proof[0] = BatchOpening for common main
        let main_row = &query_proof.input_proof[0].opened_values[0];
        let main_vals = &proof.opening.values.main[0][0];

        // Point 1: (zeta, local)
        let inv_zeta_minus_x = (zeta - x).inverse();
        for (&p_at_x, &p_at_z) in main_row.iter().zip(main_vals.local.iter()) {
            ro += alpha_pow * (p_at_z - EF::from(p_at_x)) * inv_zeta_minus_x;
            alpha_pow *= fri_alpha;
        }

        // Point 2: (next_zeta, next)
        let inv_next_minus_x = (next_zeta - x).inverse();
        for (&p_at_x, &p_at_z) in main_row.iter().zip(main_vals.next.iter()) {
            ro += alpha_pow * (p_at_z - EF::from(p_at_x)) * inv_next_minus_x;
            alpha_pow *= fri_alpha;
        }

        // --- Quotient contribution ---
        // input_proof[1] = BatchOpening for quotient
        for chunk_idx in 0..quotient_degree {
            let q_row = &query_proof.input_proof[1].opened_values[chunk_idx];
            let q_vals = &proof.opening.values.quotient[0][chunk_idx];

            for (&p_at_x, &p_at_z) in q_row.iter().zip(q_vals.iter()) {
                ro += alpha_pow * (p_at_z - EF::from(p_at_x)) * inv_zeta_minus_x;
                alpha_pow *= fri_alpha;
            }
        }

        // -----------------------------------------------------------------
        // Replay fold chain (verify_query logic)
        // -----------------------------------------------------------------
        let mut folded_eval = ro;
        let mut start_index = query_index;
        let mut per_round_folded_eval = Vec::new();

        for round_idx in 0..num_rounds {
            let log_folded_height = log_max_height - 1 - round_idx;
            let opening = &query_proof.commit_phase_openings[round_idx];
            let sibling = opening.sibling_value;
            let beta = betas[round_idx];

            // Arrange evals: evals[even] gets even-indexed value
            let index_sibling = start_index ^ 1;
            let (e0, e1) = if index_sibling % 2 == 0 {
                (sibling, folded_eval)
            } else {
                (folded_eval, sibling)
            };

            // Advance to parent index (before fold_row, matching verify_query)
            start_index >>= 1;

            // fold_row: Lagrange interpolation of (xs0, e0) and (xs1, e1) at beta
            let subgroup_start = BabyBear::two_adic_generator(log_folded_height + 1)
                .exp_u64(reverse_bits_len(start_index, log_folded_height) as u64);
            let xs0 = EF::from(subgroup_start);
            let xs1 = EF::from(-subgroup_start); // two_adic_generator(1) = -1
            folded_eval = e0 + (beta - xs0) * (e1 - e0) * (xs1 - xs0).inverse();

            // No roll-in for Fibonacci: all reduced openings at log_max_height

            per_round_folded_eval.push(ef_to_u32(&folded_eval));
        }

        // Expected final polynomial evaluation.
        // For log_final_poly_len = 0, final_poly has 1 coefficient → eval = coeff.
        let expected_final_eval = fri_proof.final_poly[0];

        assert_eq!(
            ef_to_u32(&folded_eval),
            ef_to_u32(&expected_final_eval),
            "Query {qi}: fold chain result does not match final polynomial"
        );

        query_verifications.push(QueryVerificationGolden {
            query_index,
            reduced_opening: ef_to_u32(&ro),
            per_round_folded_eval,
            final_folded_eval: ef_to_u32(&folded_eval),
            expected_final_eval: ef_to_u32(&expected_final_eval),
        });
    }

    FriVerificationVectors {
        commit_phase_commits,
        final_poly,
        queries,
        challenger_state,
        log_blowup,
        log_final_poly_len,
        num_queries,
        log_max_height,
        betas: betas.iter().map(ef_to_u32).collect(),
        query_verifications,
    }
}

// ---------------------------------------------------------------------------
// FRI prover vectors
// ---------------------------------------------------------------------------

/// FRI prover golden vectors: commit phase outputs, intermediate folds, query proofs.
///
/// Generated from a standalone synthetic polynomial to test the FRI prover
/// implementation independently of the full STARK pipeline. Evaluations are
/// in **bit-reversed order** matching the Plonky3 FRI prover's input format.
#[derive(Debug, Serialize, Deserialize)]
pub struct FriProverVectors {
    pub log_poly_size: usize,
    pub log_blowup: usize,
    pub log_final_poly_len: usize,
    pub num_queries: usize,

    /// Extension field polynomial coefficients `[c0,c1,c2,c3]` each.
    pub poly_coefficients: Vec<Vec<u32>>,
    /// Evaluations on subgroup in bit-reversed order (FRI input).
    pub input_evals_bit_reversed: Vec<Vec<u32>>,

    /// Merkle root per commit-phase round (8-element digests).
    pub commit_phase_commits: Vec<Vec<u32>>,
    /// Folding challenge per round `[c0,c1,c2,c3]` each.
    pub betas: Vec<Vec<u32>>,
    /// Folded evaluations after each round (bit-reversed).
    pub per_round_folded_evals: Vec<Vec<Vec<u32>>>,
    /// Final polynomial coefficients (extension field elements).
    pub final_poly: Vec<Vec<u32>>,

    /// Per-query opening data.
    pub query_proofs: Vec<FriQueryData>,
}

/// Generate FRI prover golden vectors by manually running the commit phase
/// and query phase with a deterministic synthetic polynomial.
///
/// Parameters: `log_poly_size=4`, `log_blowup=1`, `log_final_poly_len=0`,
/// `num_queries=2` — giving 4 folding rounds (32 → 16 → 8 → 4 → 2).
pub fn generate_fri_prover_vectors() -> FriProverVectors {
    use openvm_stark_backend::{
        p3_challenger::{CanObserve, CanSampleBits, DuplexChallenger, FieldChallenger},
        p3_commit::Mmcs,
        p3_matrix::dense::RowMajorMatrix,
        p3_util::{log2_strict_usize, reverse_slice_index_bits},
    };
    use openvm_stark_sdk::config::baby_bear_poseidon2::default_perm;
    use p3_dft::{Radix2DitParallel, TwoAdicSubgroupDft};
    use p3_field::{extension::BinomialExtensionField, BasedVectorSpace, Field, TwoAdicField};
    use p3_merkle_tree::MerkleTreeMmcs;
    use p3_symmetric::{PaddingFreeSponge, TruncatedPermutation};

    type F = BabyBear;
    type EF = BinomialExtensionField<F, 4>;
    type PackedVal = <F as Field>::Packing;
    type Perm = p3_baby_bear::Poseidon2BabyBear<16>;
    type MyHash = PaddingFreeSponge<Perm, 16, 8, 8>;
    type MyCompress = TruncatedPermutation<Perm, 2, 8, 16>;
    type MyMmcs = MerkleTreeMmcs<PackedVal, PackedVal, MyHash, MyCompress, 8>;
    type MyChal = DuplexChallenger<F, Perm, 16, 8>;

    let log_poly_size: usize = 4;
    let log_blowup: usize = 1;
    let log_final_poly_len: usize = 0;
    let num_queries: usize = 2;
    let log_domain = log_poly_size + log_blowup; // 5, domain size = 32
    let blowup = 1usize << log_blowup;
    let final_poly_len = 1usize << log_final_poly_len;

    fn ef_to_vec(e: &EF) -> Vec<u32> {
        e.as_basis_coefficients_slice()
            .iter()
            .map(|x: &BabyBear| x.as_canonical_u32())
            .collect()
    }

    fn make_ef(a: u32, b: u32, c: u32, d: u32) -> EF {
        let coeffs = [F::new(a), F::new(b), F::new(c), F::new(d)];
        EF::from_basis_coefficients_fn(|i| coeffs[i])
    }

    // 1. Create deterministic polynomial coefficients (degree 15, 16 coefficients).
    //    Same pattern as generate_fri_folding_vectors.
    let poly_coefficients: Vec<EF> = (0..1usize << log_poly_size)
        .map(|i| {
            let base = (i * 4) as u32;
            make_ef(base + 1, base + 2, base + 3, base + 4)
        })
        .collect();

    // 2. Evaluate on subgroup H = <two_adic_generator(log_domain)> in natural order.
    //    Note: the FRI prover works on subgroup evaluations (not coset), because
    //    the PCS converts coset evals to subgroup evals via change of variables.
    let omega = F::two_adic_generator(log_domain);
    let domain_size = 1usize << log_domain;

    let eval_at = |x: F| -> EF {
        let x_ef = EF::from(x);
        poly_coefficients
            .iter()
            .rev()
            .fold(EF::ZERO, |acc, &c| acc * x_ef + c)
    };

    let evals_natural: Vec<EF> = (0..domain_size)
        .map(|i| eval_at(omega.exp_u64(i as u64)))
        .collect();

    // 3. Bit-reverse evaluations to match FRI prover input format.
    let mut evals_br = evals_natural.clone();
    reverse_slice_index_bits(&mut evals_br);

    // 4. Set up MMCS and challenger.
    let perm: Perm = default_perm();
    let hash = MyHash::new(perm.clone());
    let compress = MyCompress::new(perm.clone());
    let mmcs = MyMmcs::new(hash, compress);
    let mut challenger = MyChal::new(perm.clone());

    // 5. Commit phase: iterative folding with Merkle commitments.
    let mut folded: Vec<EF> = evals_br.clone();
    let mut commits: Vec<Vec<u32>> = Vec::new();
    let mut betas: Vec<Vec<u32>> = Vec::new();
    let mut per_round_folded_evals: Vec<Vec<Vec<u32>>> = Vec::new();
    let mut prover_data_vec: Vec<_> = Vec::new(); // Store Merkle tree data for queries

    while folded.len() > blowup * final_poly_len {
        let height = folded.len() / 2;
        let log_height = log2_strict_usize(height);

        // Flatten EF pairs to base field for MMCS commitment.
        // Each row: [lo_c0, lo_c1, lo_c2, lo_c3, hi_c0, hi_c1, hi_c2, hi_c3]
        let leaf_data: Vec<F> = folded
            .chunks(2)
            .flat_map(|pair| {
                let lo_coeffs = pair[0].as_basis_coefficients_slice().to_vec();
                let hi_coeffs = pair[1].as_basis_coefficients_slice().to_vec();
                lo_coeffs.into_iter().chain(hi_coeffs)
            })
            .collect();
        let mat = RowMajorMatrix::new(leaf_data, 8);

        // Commit and observe.
        let (commit, prover_data) = mmcs.commit(vec![mat]);
        let commit_arr: [F; 8] = commit.into();
        let commit_u32: Vec<u32> = commit_arr.iter().map(|x| x.as_canonical_u32()).collect();
        challenger.observe(commit);
        commits.push(commit_u32);

        // grind(0) is a no-op for commit_proof_of_work_bits = 0.

        // Sample folding challenge.
        let beta: EF = challenger.sample_algebra_element();
        betas.push(ef_to_vec(&beta));

        // Fold using fold_matrix math.
        let g_inv = F::two_adic_generator(log_height + 1).inverse();
        let half = F::TWO.inverse();
        let mut halve_inv_powers: Vec<F> = Vec::with_capacity(height);
        let mut val = half;
        for _ in 0..height {
            halve_inv_powers.push(val);
            val *= g_inv;
        }
        reverse_slice_index_bits(&mut halve_inv_powers);

        let mut new_folded = Vec::with_capacity(height);
        for i in 0..height {
            let lo = folded[2 * i];
            let hi = folded[2 * i + 1];
            let hip = EF::from(halve_inv_powers[i]);
            let result = (lo + hi).halve() + (lo - hi) * beta * hip;
            new_folded.push(result);
        }
        folded = new_folded;

        per_round_folded_evals.push(folded.iter().map(|e| ef_to_vec(e)).collect());
        prover_data_vec.push(prover_data);
    }

    // 6. Compute final polynomial via IDFT.
    let mut final_evals = folded[..final_poly_len].to_vec();
    reverse_slice_index_bits(&mut final_evals);
    let dft = Radix2DitParallel::<F>::default();
    let final_poly_ef: Vec<EF> = dft.idft_algebra(final_evals);

    // Observe final polynomial in challenger (matches challenger.observe_algebra_slice).
    challenger.observe_algebra_slice::<EF>(&final_poly_ef);

    let final_poly: Vec<Vec<u32>> = final_poly_ef.iter().map(|e| ef_to_vec(e)).collect();

    // grind(0) is a no-op for query_proof_of_work_bits = 0.

    // 7. Query phase: sample indices and generate opening proofs.
    let log_max_height = log2_strict_usize(evals_br.len());
    let num_rounds = commits.len();
    let mut query_proofs: Vec<FriQueryData> = Vec::new();

    for _ in 0..num_queries {
        let query_index: usize = challenger.sample_bits(log_max_height);

        let mut openings: Vec<FriCommitPhaseStep> = Vec::new();
        for round_idx in 0..num_rounds {
            let index_i = query_index >> round_idx;
            let index_i_sibling = index_i ^ 1;
            let index_pair = index_i >> 1;

            // Open the Merkle tree at index_pair.
            let batch_opening =
                mmcs.open_batch(index_pair, &prover_data_vec[round_idx]);
            let (opened_vals, proof_siblings) = batch_opening.unpack();

            // opened_vals[0] is the base-field row (8 elements).
            let row = &opened_vals[0];
            assert_eq!(row.len(), 8, "Expected 8 base field elements per row");

            // Reconstruct the two EF elements.
            let ef0 = EF::from_basis_coefficients_fn(|j| row[j]);
            let ef1 = EF::from_basis_coefficients_fn(|j| row[4 + j]);

            // Sibling value: if index_i is even, sibling is hi (ef1); if odd, sibling is lo (ef0).
            let sibling = if index_i_sibling % 2 == 0 { ef0 } else { ef1 };

            let proof_u32: Vec<Vec<u32>> = proof_siblings
                .iter()
                .map(|digest| digest.iter().map(|x: &F| x.as_canonical_u32()).collect())
                .collect();

            openings.push(FriCommitPhaseStep {
                sibling_value: ef_to_vec(&sibling),
                opening_proof: proof_u32,
            });
        }

        query_proofs.push(FriQueryData {
            index: query_index,
            commit_phase_openings: openings,
        });
    }

    FriProverVectors {
        log_poly_size,
        log_blowup,
        log_final_poly_len,
        num_queries,
        poly_coefficients: poly_coefficients.iter().map(|e| ef_to_vec(e)).collect(),
        input_evals_bit_reversed: evals_br.iter().map(|e| ef_to_vec(e)).collect(),
        commit_phase_commits: commits,
        betas,
        per_round_folded_evals,
        final_poly,
        query_proofs,
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
