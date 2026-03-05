//! Test vector generation for the executable specification.
//!
//! This crate generates golden test vectors by exercising OpenVM's proving
//! pipeline and serializing intermediate values to JSON. The vectors are
//! consumed by both Rust verification tests and Python spec tests.

use std::path::Path;

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

    let proof = &vdata.data.proof;

    // Extract commitment metadata as canonical u32 arrays.
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
    let quotient_commitment: Vec<u32> = quotient_arr.iter().map(|x| x.as_canonical_u32()).collect();

    // Extract per-AIR metadata.
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

    // Serialize the full proof to bytes using serde_json (Proof derives Serialize).
    // We use serde_json instead of bincode because the Proof struct has complex
    // generic types and serde_json produces a stable, inspectable output.
    let proof_json_bytes = serde_json::to_vec(proof).expect("Proof should serialize to JSON bytes");
    let proof_bytes_hex = hex_encode(&proof_json_bytes);

    E2eProofVectors {
        program_name: "fibonacci_stark".to_string(),
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
fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
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
