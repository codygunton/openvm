//! Python FFI bindings for Poseidon2 permutation over BabyBear.
//!
//! Wraps Plonky3's production Poseidon2BabyBear<16> permutation via PyO3,
//! using the same HorizenLabs round constants as OpenVM.

use lazy_static::lazy_static;
use openvm_stark_sdk::config::baby_bear_poseidon2::default_perm;
use openvm_stark_sdk::openvm_stark_backend::p3_field::PrimeField32;
use p3_baby_bear::BabyBear;
use p3_symmetric::Permutation;
use pyo3::prelude::*;

type Perm = p3_baby_bear::Poseidon2BabyBear<16>;

lazy_static! {
    static ref POSEIDON2: Perm = default_perm();
}

/// Apply Poseidon2 permutation to a width-16 BabyBear state.
///
/// Takes 16 u32 field elements, returns 16 u32 field elements.
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

/// Python module exposing Poseidon2 BabyBear permutation.
#[pymodule]
fn poseidon2_ffi(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(poseidon2_permute, m)?)?;
    m.add("WIDTH", 16u32)?;
    m.add("RATE", 8u32)?;
    m.add("DIGEST_SIZE", 8u32)?;
    Ok(())
}
