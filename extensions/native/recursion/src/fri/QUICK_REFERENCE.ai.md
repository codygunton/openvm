# FRI Component - Quick Reference

## Core Types

### FriConfigVariable
```rust
FriConfigVariable<C> {
    log_blowup: usize,              // Rate (2-4 typical)
    blowup: usize,                  // 2^log_blowup
    log_final_poly_len: usize,      // Must be 0 currently
    num_queries: usize,             // Security parameter (20-100)
    proof_of_work_bits: usize,      // PoW difficulty (15-20)
    generators: Array<C, Felt<C::F>>, // Pre-computed generators
    subgroups: Array<C, TwoAdicMultiplicativeCosetVariable<C>>,
}
```

### FriProofVariable
```rust
FriProofVariable<C> {
    commit_phase_commits: Array<C, DigestVariable<C>>,
    query_proofs: Array<C, FriQueryProofVariable<C>>,
    final_poly: Array<C, Ext<C::F, C::EF>>,
    pow_witness: Felt<C::F>,
}
```

### TwoAdicMultiplicativeCosetVariable
```rust
TwoAdicMultiplicativeCosetVariable<C> {
    log_n: Usize<C::N>,    // Domain size = 2^log_n
    shift: Felt<C::F>,     // Coset shift
    g: Felt<C::F>,         // Generator
}
```

## Key Functions

### Main Verification
```rust
// Verify FRI proof
verify_two_adic_pcs<C>(
    builder: &mut Builder<C>,
    config: &FriConfigVariable<C>,
    rounds: Array<C, TwoAdicPcsRoundVariable<C>>,
    proof: FriProofVariable<C>,
    log_max_height: RVar<C::N>,
    challenger: &mut impl ChallengerVariable<C>,
)

// Verify single query
verify_query<C>(
    builder: &mut Builder<C>,
    config: &FriConfigVariable<C>,
    commit_phase_commits: &Array<C, DigestVariable<C>>,
    index_bits: &Array<C, Var<C::N>>,
    proof: &FriQueryProofVariable<C>,
    betas: &Array<C, Ext<C::F, C::EF>>,
    betas_squared: &Array<C, Ext<C::F, C::EF>>,
    reduced_openings: &Array<C, Ext<C::F, C::EF>>,
    log_max_lde_height: RVar<C::N>,
    i_plus_one_arr: &Array<C, Usize<C::N>>,
) -> Ext<C::F, C::EF>

// Verify batch opening
verify_batch<C>(
    builder: &mut Builder<C>,
    commit: &DigestVariable<C>,
    dimensions: Array<C, DimensionsVariable<C>>,
    index_bits: Array<C, Var<C::N>>,
    opened_values: &NestedOpenedValues<C>,
    proof: &HintSlice<C>,
)
```

### Domain Operations
```rust
// Create natural domain
let domain = pcs.natural_domain_for_log_degree(builder, log_degree);

// Create disjoint domain
let disjoint = domain.create_disjoint_domain(
    builder, 
    log_degree, 
    Some(fri_config)
);

// Split domain
let subdomains = domain.split_domains(
    builder,
    log_num_chunks,
    num_chunks
);

// Get selectors at point
let selectors = domain.selectors_at_point(builder, point);

// Get zero polynomial at point
let zp = domain.zp_at_point(builder, point);
```

## Constants

```rust
MAX_TWO_ADICITY = 27          // Max two-adicity for BabyBear
REDUCER_BUFFER_SIZE = 8192    // Buffer for batch reduction
DIGEST_SIZE                   // From circuit config
```

## Common Patterns

### Basic FRI Verification
```rust
// 1. Setup
let config = FriConfigVariable { /* params */ };
let pcs = TwoAdicFriPcsVariable { config };

// 2. Create challenger
let mut challenger = DuplexChallengerVariable::new(&mut builder);

// 3. Verify
pcs.verify(
    &mut builder,
    rounds,
    proof,
    log_max_height,
    &mut challenger
);
```

### Input Validation
```rust
// Always validate proof structure
builder.assert_usize_eq(
    proof.query_proofs.len(), 
    RVar::from(config.num_queries)
);
builder.assert_usize_eq(
    proof.commit_phase_commits.len(), 
    log_max_height
);
builder.assert_var_eq(
    RVar::from(config.log_final_poly_len), 
    RVar::zero()
);
```

### Challenge Generation
```rust
// Observe then sample
challenger.observe_digest(builder, commit);
let beta = challenger.sample_ext(builder);
let beta_squared = builder.eval(beta * beta);
```

### Field Operations
```rust
// Power of 2 exponentiation
let x_squared = builder.exp_power_of_2_v(x, 1);

// Bit-reversed indexing
let point = builder.exp_bits_big_endian(gen, &index_bits);

// Generator for size 2^k
let g = config.get_two_adic_generator(builder, k);
```

## Security Parameters

### 128-bit Security
```rust
FriConfigVariable {
    log_blowup: 2,         // 4x
    num_queries: 53,       
    proof_of_work_bits: 20,
    // ...
}
```

### 192-bit Security
```rust
FriConfigVariable {
    log_blowup: 3,         // 8x
    num_queries: 60,
    proof_of_work_bits: 24,
    // ...
}
```

### 256-bit Security
```rust
FriConfigVariable {
    log_blowup: 3,         // 8x
    num_queries: 80,
    proof_of_work_bits: 30,
    // ...
}
```

## Performance Tips

### Use Static Mode When Possible
```rust
builder.flags.static_only = true; // For compile-time params
```

### Enable Cycle Tracking
```rust
builder.cycle_tracker_start("my-operation");
// ... work ...
builder.cycle_tracker_end("my-operation");
```

### Batch Operations
```rust
// Process multiple items together
iter_zip!(builder, items1, items2).for_each(|ptrs, builder| {
    // Parallel processing
});
```

### Pre-allocate Buffers
```rust
// Allocate once, reuse many times
let buffer = builder.array(REDUCER_BUFFER_SIZE);
```

## Error Handling

### Common Assertions
```rust
// Proof structure
builder.assert_usize_eq(actual_len, expected_len);

// Field values
builder.assert_felt_eq(computed, expected);
builder.assert_ext_eq(computed_ext, expected_ext);

// Domain parameters
builder.assert_var_eq(domain.log_n, expected_log_n);
```

### Debugging
```rust
// Add cycle tracking
builder.cycle_tracker_start("suspect-operation");

// Print intermediate values (debug only)
if cfg!(debug_assertions) {
    println!("Value: {:?}", value);
}

// Assert invariants
debug_assert_eq!(list.len(), expected_len);
```

## Integration Examples

### With STARK Verifier
```rust
let fri_pcs = TwoAdicFriPcsVariable { 
    config: fri_config 
};

stark_verifier.verify_with_pcs(
    &mut builder,
    &fri_pcs,
    proof,
    &mut challenger
);
```

### With Poseidon2
```rust
// FRI uses Poseidon2 for hashing
let digest = values.p2_digest(builder);
challenger.observe_digest(builder, digest);
```

### With Hint System
```rust
// Read proof data from hints
let proof = InnerFriProof::read(&mut builder);
```

## Quick Formulas

### Soundness Error
```
ε ≤ (d + δ)/|F| + (1 - δ/ρ)^t
```
- d: degree bound
- δ: code distance  
- ρ: blowup factor
- t: num queries

### Security Level
```
security_bits ≈ min(
    log2(field_size),
    num_queries * log2(blowup)
)
```

### Proof Size
```
size ≈ num_queries * (
    log_max_height * hash_size + 
    final_poly_size
)
```