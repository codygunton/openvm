# FRI Component - Implementation Guide

## Overview
This guide covers implementing and extending the FRI (Fast Reed-Solomon Interactive Oracle Proof) verification system in OpenVM.

## Architecture Decisions

### Why Two-Adic Fields?
- Enables efficient FFT operations with power-of-2 sizes
- Simplifies bit-reversal and domain navigation  
- Hardware-friendly operations
- Natural fit for binary tree structures (Merkle trees)

### Static vs Dynamic Modes
The implementation supports two execution modes:

**Static Mode** (`builder.flags.static_only = true`):
- Parameters known at compile time
- Loops unrolled, branches eliminated
- Better performance but less flexible

**Dynamic Mode**:
- Parameters determined at runtime
- More flexible but requires branching
- Used for variable-sized proofs

## Core Implementation Patterns

### 1. FRI Configuration Setup

```rust
// Create FRI configuration with security parameters
fn create_fri_config<C: Config>(
    builder: &mut Builder<C>,
    security_level: usize,
) -> FriConfigVariable<C> {
    // For 128-bit security
    let (log_blowup, num_queries) = match security_level {
        128 => (2, 53),
        192 => (3, 60),
        256 => (3, 80),
        _ => panic!("Unsupported security level"),
    };

    // Pre-compute two-adic generators
    let max_degree = 27; // MAX_TWO_ADICITY
    let generators = builder.array(max_degree);
    for i in 0..max_degree {
        let g = C::F::two_adic_generator(i);
        builder.set(&generators, i, g);
    }

    FriConfigVariable {
        log_blowup,
        blowup: 1 << log_blowup,
        log_final_poly_len: 0, // Currently must be 0
        num_queries,
        proof_of_work_bits: 20,
        generators,
        subgroups: /* pre-computed subgroups */,
    }
}
```

### 2. Domain Management

```rust
// Create evaluation domain for polynomial of given degree
fn create_domain<C: Config>(
    builder: &mut Builder<C>,
    log_degree: RVar<C::N>,
) -> TwoAdicMultiplicativeCosetVariable<C> 
where
    C::F: TwoAdicField,
{
    // Get generator for domain size
    let g = C::F::two_adic_generator(log_degree.value());
    
    TwoAdicMultiplicativeCosetVariable {
        log_n: builder.eval(log_degree),
        shift: builder.eval(C::F::ONE), // No shift for standard domain
        g: builder.eval(g),
    }
}

// Create coset for quotient polynomial evaluation
fn create_quotient_domain<C: Config>(
    builder: &mut Builder<C>,
    base_domain: &TwoAdicMultiplicativeCosetVariable<C>,
    log_quotient_degree: usize,
) -> TwoAdicMultiplicativeCosetVariable<C> {
    // Shift by generator to avoid original domain
    let shift = builder.eval(base_domain.shift * C::F::GENERATOR);
    
    TwoAdicMultiplicativeCosetVariable {
        log_n: builder.eval(base_domain.log_n.clone() + log_quotient_degree),
        shift,
        g: base_domain.g, // Same generator
    }
}
```

### 3. Query Verification Implementation

```rust
// Verify a single FRI query
fn verify_fri_query<C: Config>(
    builder: &mut Builder<C>,
    config: &FriConfigVariable<C>,
    commitments: &[DigestVariable<C>],
    query_index: &Array<C, Var<C::N>>,
    query_proof: &FriQueryProofVariable<C>,
    folding_challenges: &[Ext<C::F, C::EF>],
) -> Result<Ext<C::F, C::EF>, VerificationError> {
    // Initial setup
    let mut folded_eval = /* extract from proof */;
    let mut x = /* compute evaluation point from index */;
    
    // Verify each folding round
    for (round, (commit, opening)) in commitments.iter()
        .zip(&query_proof.commit_phase_openings)
        .enumerate() 
    {
        // Verify Merkle opening
        verify_merkle_opening(builder, commit, opening)?;
        
        // Compute folding
        let beta = folding_challenges[round];
        let (eval_0, eval_1) = /* extract sibling evaluations */;
        
        // Folding formula: f(x) = f_0(x²) + β·x·f_1(x²)
        folded_eval = fold_evaluations(
            builder, eval_0, eval_1, x, beta
        );
        
        // Update x for next round
        x = builder.eval(x * x);
    }
    
    Ok(folded_eval)
}
```

### 4. Batch Opening Optimization

```rust
// Efficiently verify multiple polynomial openings
fn verify_batch_opening<C: Config>(
    builder: &mut Builder<C>,
    commitments: &[DigestVariable<C>],
    opened_values: &NestedOpenedValues<C>,
    proof: &HintSlice<C>,
) {
    if builder.flags.static_only {
        // Static path: unrolled verification
        verify_batch_static(builder, commitments, opened_values, proof);
    } else {
        // Dynamic path: flexible verification
        match opened_values {
            NestedOpenedValues::Felt(values) => {
                builder.verify_batch_felt(/* params */);
            }
            NestedOpenedValues::Ext(values) => {
                builder.verify_batch_ext(/* params */);
            }
        }
    }
}
```

### 5. Efficient Field Operations

```rust
// Common two-adic field operations
impl<C: Config> TwoAdicOps<C> {
    // Compute g^(2^k) efficiently
    fn generator_power_of_2(
        builder: &mut Builder<C>,
        g: Felt<C::F>,
        k: usize,
    ) -> Felt<C::F> {
        let mut result = g;
        for _ in 0..k {
            result = builder.eval(result * result);
        }
        result
    }
    
    // Bit-reversed indexing for FFT
    fn bit_reverse_index(
        builder: &mut Builder<C>,
        index: &Array<C, Var<C::N>>,
        bits: usize,
    ) -> Array<C, Var<C::N>> {
        let reversed = builder.array(bits);
        for i in 0..bits {
            let bit = builder.get(index, bits - 1 - i);
            builder.set(&reversed, i, bit);
        }
        reversed
    }
}
```

## Performance Optimization Strategies

### 1. Memory Management

```rust
// Pre-allocate buffers for repeated operations
struct FriVerifierBuffers<C: Config> {
    opened_values: Array<C, Array<C, Felt<C::F>>>,
    alpha_powers: Vec<Ext<C::F, C::EF>>,
    reduced_openings: Array<C, Ext<C::F, C::EF>>,
}

impl<C: Config> FriVerifierBuffers<C> {
    fn new(builder: &mut Builder<C>, max_degree: usize) -> Self {
        Self {
            opened_values: builder.array(REDUCER_BUFFER_SIZE),
            alpha_powers: Vec::with_capacity(max_degree),
            reduced_openings: builder.array(MAX_TWO_ADICITY + 1),
        }
    }
}
```

### 2. Parallel Processing

```rust
// Process independent rounds in parallel
fn verify_rounds_parallel<C: Config>(
    builder: &mut Builder<C>,
    rounds: &[Round<C>],
) {
    // Use iter_zip! for parallel iteration
    iter_zip!(builder, rounds, round_contexts)
        .for_each(|ptr_vec, builder| {
            let round = builder.iter_ptr_get(&rounds, ptr_vec[0]);
            let context = builder.iter_ptr_get(&round_contexts, ptr_vec[1]);
            
            // Process round independently
            process_round(builder, round, context);
        });
}
```

### 3. Cycle Tracking

```rust
// Add cycle tracking for performance analysis
fn critical_operation<C: Config>(builder: &mut Builder<C>) {
    builder.cycle_tracker_start("critical-op");
    
    // Perform operation
    expensive_computation(builder);
    
    builder.cycle_tracker_end("critical-op");
}
```

## Common Pitfalls and Solutions

### 1. Incorrect Domain Setup
**Problem**: Using wrong generator or shift values
**Solution**: Always use pre-computed generators from config

### 2. Memory Aliasing
**Problem**: Modifying shared buffers incorrectly
**Solution**: Clone arrays when modification needed:
```rust
let buffer_copy = if builder.flags.static_only {
    builder.array(size) // New allocation
} else {
    builder.eval(buffer.clone()) // Safe copy
};
```

### 3. Field Overflow
**Problem**: Operations exceeding field modulus
**Solution**: Use proper field arithmetic:
```rust
// Wrong: direct multiplication
let result = a * b;

// Correct: field multiplication
let result = builder.eval(a * b);
```

## Extension Points

### 1. Custom Folding Strategies

```rust
trait FoldingStrategy<C: Config> {
    fn fold(
        &self,
        builder: &mut Builder<C>,
        evals: &[Ext<C::F, C::EF>],
        challenge: Ext<C::F, C::EF>,
    ) -> Ext<C::F, C::EF>;
}

// Implement alternative folding
struct AdaptiveFolding;
impl<C: Config> FoldingStrategy<C> for AdaptiveFolding {
    fn fold(/* params */) -> Ext<C::F, C::EF> {
        // Custom folding logic
    }
}
```

### 2. Domain Extensions

```rust
// Extend domain functionality
trait DomainExtensions<C: Config> {
    fn create_twin_domain(&self, builder: &mut Builder<C>) -> Self;
    fn shift_by_root_of_unity(&self, builder: &mut Builder<C>, k: usize) -> Self;
}
```

### 3. Proof Compression

```rust
// Implement proof compression
struct CompressedFriProof<C: Config> {
    // Store only essential data
    commit_roots: Vec<DigestVariable<C>>,
    query_indices: BitVec,
    final_poly_hash: DigestVariable<C>,
}
```

## Testing Strategies

### 1. Unit Tests
```rust
#[test]
fn test_folding_correctness() {
    // Test individual folding step
}

#[test]
fn test_domain_operations() {
    // Test domain arithmetic
}
```

### 2. Integration Tests
```rust
#[test]
fn test_full_fri_verification() {
    // End-to-end FRI proof verification
}
```

### 3. Fuzzing
```rust
#[test]
fn fuzz_fri_parameters() {
    // Test with random parameters
    for _ in 0..100 {
        let params = random_fri_params();
        verify_fri_soundness(params);
    }
}
```

## Debugging Tips

### 1. Enable Verbose Logging
```rust
if cfg!(debug_assertions) {
    println!("FRI round {}: folded_eval = {:?}", round, folded_eval);
}
```

### 2. Assert Intermediate Values
```rust
// Add assertions for debugging
builder.assert_ext_eq(
    computed_value,
    expected_value,
    "Folding mismatch at round {}",
    round
);
```

### 3. Use Cycle Tracking
Monitor performance bottlenecks with cycle tracking enabled.

## Security Checklist

- [ ] Validate all proof inputs
- [ ] Use sufficient number of queries for target security
- [ ] Implement proof-of-work correctly
- [ ] Check field arithmetic doesn't overflow
- [ ] Verify Merkle paths completely
- [ ] Use proper challenge generation (Fiat-Shamir)
- [ ] Test with adversarial inputs