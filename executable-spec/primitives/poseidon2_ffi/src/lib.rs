//! Python FFI bindings for Poseidon2 permutation and NTT over BabyBear.
//!
//! Wraps Plonky3's production Poseidon2BabyBear<16> permutation via PyO3,
//! using the same HorizenLabs round constants as OpenVM.
//! Also provides SIMD-optimized NTT/INTT via p3-dft.

use lazy_static::lazy_static;
use openvm_stark_sdk::config::baby_bear_poseidon2::default_perm;
use openvm_stark_sdk::openvm_stark_backend::p3_field::PrimeField32;
use p3_baby_bear::BabyBear;
use p3_dft::{Radix2DitParallel, TwoAdicSubgroupDft};
use p3_matrix::dense::RowMajorMatrix;
use p3_matrix::Matrix;
use p3_symmetric::Permutation;
use pyo3::prelude::*;
use rayon::prelude::*;

type Perm = p3_baby_bear::Poseidon2BabyBear<16>;

lazy_static! {
    static ref POSEIDON2: Perm = default_perm();
    static ref DFT: Radix2DitParallel<BabyBear> = Radix2DitParallel::default();
}

/// Apply Poseidon2 permutation to a width-16 BabyBear state.
#[pyfunction]
fn poseidon2_permute(input: Vec<u32>) -> PyResult<Vec<u32>> {
    if input.len() != 16 {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "input must have exactly 16 elements",
        ));
    }
    let mut state: [BabyBear; 16] = std::array::from_fn(|i| BabyBear::new(input[i]));
    POSEIDON2.permute_mut(&mut state);
    Ok(state.iter().map(|x| x.as_canonical_u32()).collect())
}

/// Batch compress: takes N pairs of 8-element digests, returns N 8-element digests.
#[pyfunction]
fn poseidon2_compress_batch(lefts: Vec<Vec<u32>>, rights: Vec<Vec<u32>>) -> PyResult<Vec<Vec<u32>>> {
    if lefts.len() != rights.len() {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "lefts and rights must have the same length",
        ));
    }
    let perm = &*POSEIDON2;
    let results: Vec<Vec<u32>> = lefts
        .into_par_iter()
        .zip(rights.into_par_iter())
        .map(|(left, right)| {
            let mut state: [BabyBear; 16] = std::array::from_fn(|i| {
                if i < 8 {
                    BabyBear::new(left[i])
                } else {
                    BabyBear::new(right[i - 8])
                }
            });
            perm.permute_mut(&mut state);
            state[..8].iter().map(|x| x.as_canonical_u32()).collect()
        })
        .collect();
    Ok(results)
}

/// Batch hash: takes N variable-length inputs, returns N 8-element digests.
#[pyfunction]
fn poseidon2_hash_batch(inputs: Vec<Vec<u32>>) -> PyResult<Vec<Vec<u32>>> {
    let perm = &*POSEIDON2;
    let results: Vec<Vec<u32>> = inputs
        .into_par_iter()
        .map(|input| {
            let mut state = [BabyBear::default(); 16];
            let mut i = 0;
            while i < input.len() {
                for j in 0..8 {
                    if i < input.len() {
                        state[j] = BabyBear::new(input[i]);
                        i += 1;
                    }
                }
                perm.permute_mut(&mut state);
            }
            state[..8].iter().map(|x| x.as_canonical_u32()).collect()
        })
        .collect();
    Ok(results)
}

/// Forward NTT on a single column of BabyBear elements.
#[pyfunction]
fn ntt(input: Vec<u32>) -> PyResult<Vec<u32>> {
    let dft = &*DFT;
    let vals: Vec<BabyBear> = input.iter().map(|&x| BabyBear::new(x)).collect();
    let result = dft.dft(vals);
    Ok(result.iter().map(|x| x.as_canonical_u32()).collect())
}

/// Inverse NTT on a single column of BabyBear elements.
#[pyfunction]
fn intt(input: Vec<u32>) -> PyResult<Vec<u32>> {
    let dft = &*DFT;
    let vals: Vec<BabyBear> = input.iter().map(|&x| BabyBear::new(x)).collect();
    let result = dft.idft(vals);
    Ok(result.iter().map(|x| x.as_canonical_u32()).collect())
}

/// Batch NTT: perform forward NTT on multiple columns in parallel.
#[pyfunction]
fn ntt_batch(columns: Vec<Vec<u32>>) -> PyResult<Vec<Vec<u32>>> {
    let dft = &*DFT;
    let results: Vec<Vec<u32>> = columns
        .into_par_iter()
        .map(|col| {
            let vals: Vec<BabyBear> = col.iter().map(|&x| BabyBear::new(x)).collect();
            let result = dft.dft(vals);
            result.iter().map(|x| x.as_canonical_u32()).collect()
        })
        .collect();
    Ok(results)
}

/// Batch INTT: perform inverse NTT on multiple columns in parallel.
#[pyfunction]
fn intt_batch(columns: Vec<Vec<u32>>) -> PyResult<Vec<Vec<u32>>> {
    let dft = &*DFT;
    let results: Vec<Vec<u32>> = columns
        .into_par_iter()
        .map(|col| {
            let vals: Vec<BabyBear> = col.iter().map(|&x| BabyBear::new(x)).collect();
            let result = dft.idft(vals);
            result.iter().map(|x| x.as_canonical_u32()).collect()
        })
        .collect();
    Ok(results)
}

/// Coset LDE batch: INTT + pad + coset-shift + NTT on extended domain.
///
/// Takes multiple columns, performs coset LDE on each in parallel.
/// This is the exact operation used in PCS commit (INTT, pad, shift, NTT).
///
/// shift: the coset shift factor (GENERATOR / domain.shift), as u32.
/// log_blowup: log2 of the blowup factor.
///
/// Returns columns after LDE (each column has n * 2^log_blowup elements).
#[pyfunction]
fn coset_lde_batch(
    columns: Vec<Vec<u32>>,
    shift: u32,
    log_blowup: usize,
) -> PyResult<Vec<Vec<u32>>> {
    let dft = &*DFT;
    let shift_field = BabyBear::new(shift);

    // Use p3-dft's coset_lde_batch which handles INTT + pad + coset-shift + NTT.
    // We pack all columns into a single RowMajorMatrix so p3-dft can process them
    // together (it already uses internal parallelism).
    if columns.is_empty() {
        return Ok(vec![]);
    }
    let n = columns[0].len();
    let num_cols = columns.len();

    // Build row-major matrix: n rows x num_cols columns
    let mut flat: Vec<BabyBear> = Vec::with_capacity(n * num_cols);
    for row in 0..n {
        for col in &columns {
            flat.push(BabyBear::new(col[row]));
        }
    }
    let mat = RowMajorMatrix::new(flat, num_cols);
    let result = dft.coset_lde_batch(mat, log_blowup, shift_field);
    let result_mat = result.to_row_major_matrix();
    let n_ext = n << log_blowup;

    // Extract columns from the result
    let mut out_columns: Vec<Vec<u32>> = vec![Vec::with_capacity(n_ext); num_cols];
    for row in 0..n_ext {
        for c in 0..num_cols {
            out_columns[c].push(result_mat.values[row * num_cols + c].as_canonical_u32());
        }
    }
    Ok(out_columns)
}

/// Set the number of rayon threads to use for batch operations.
#[pyfunction]
fn set_num_threads(n: usize) -> PyResult<()> {
    rayon::ThreadPoolBuilder::new()
        .num_threads(n)
        .build_global()
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("{}", e)))
}

/// Python module exposing Poseidon2 BabyBear permutation and NTT.
#[pymodule]
fn poseidon2_ffi(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(poseidon2_permute, m)?)?;
    m.add_function(wrap_pyfunction!(poseidon2_compress_batch, m)?)?;
    m.add_function(wrap_pyfunction!(poseidon2_hash_batch, m)?)?;
    m.add_function(wrap_pyfunction!(ntt, m)?)?;
    m.add_function(wrap_pyfunction!(intt, m)?)?;
    m.add_function(wrap_pyfunction!(ntt_batch, m)?)?;
    m.add_function(wrap_pyfunction!(intt_batch, m)?)?;
    m.add_function(wrap_pyfunction!(coset_lde_batch, m)?)?;
    m.add_function(wrap_pyfunction!(set_num_threads, m)?)?;
    m.add("WIDTH", 16u32)?;
    m.add("RATE", 8u32)?;
    m.add("DIGEST_SIZE", 8u32)?;
    Ok(())
}
