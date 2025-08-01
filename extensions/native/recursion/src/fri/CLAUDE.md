# FRI Component - Claude Working Notes

## Component Overview
This directory implements Fast Reed-Solomon Interactive Oracle Proof (FRI) verification for OpenVM's native recursion extension. FRI is used to prove that committed polynomials have low degree, which is fundamental to STARK proof systems.

## Key Implementation Details

### Core Files
- `mod.rs` - Main FRI verification logic with query and batch verification
- `types.rs` - Data structures for FRI configuration, proofs, and PCS rounds  
- `two_adic_pcs.rs` - Two-adic polynomial commitment scheme with FRI
- `domain.rs` - Two-adic multiplicative coset implementation

### Important Constants
- `MAX_TWO_ADICITY = 27` - Maximum two-adicity supported (for BabyBear field)
- `REDUCER_BUFFER_SIZE = 8192` - Buffer size for batch reduction operations
- `log_final_poly_len = 0` - Currently only supports final polynomial of length 1

## Security Considerations

### Parameter Guidelines
- `log_blowup`: Rate parameter, typically 2-4 (higher = more secure but larger proofs)
- `num_queries`: Number of queries, typically 20-100 (more = higher security)
- `proof_of_work_bits`: PoW difficulty, typically 15-20 bits

### Soundness
The soundness error is approximately: `ε ≤ (d + δ)/|F| + (1 - δ/ρ)^t`
- For 128-bit security: use log_blowup=2, num_queries=53

## Performance Notes

### Optimization Modes
- **Static mode**: Compile-time parameters, unrolled loops, constant propagation
- **Dynamic mode**: Runtime parameters, flexible but slower

### Key Optimizations
- Parallel round processing with `iter_zip!`
- Pre-allocated buffers for opened values
- Cycle tracking for performance profiling
- Efficient two-adic field operations using bit manipulation

## Common Patterns

### Verification Flow
```rust
// 1. Create FRI config
let config = FriConfigVariable { /* params */ };

// 2. Create PCS 
let pcs = TwoAdicFriPcsVariable { config };

// 3. Set up challenger
let mut challenger = DuplexChallengerVariable::new(&mut builder);

// 4. Verify
pcs.verify(&mut builder, rounds, proof, log_max_height, &mut challenger);
```

### Input Validation
Always validate proof structure:
```rust
builder.assert_usize_eq(proof.query_proofs.len(), RVar::from(config.num_queries));
builder.assert_usize_eq(proof.commit_phase_commits.len(), log_max_height);
```

## Testing
- Test utilities in `two_adic_pcs.rs::tests`
- Domain testing in `domain.rs::tests`
- Both static and dynamic modes tested

## Integration Points
- Uses Poseidon2 for Merkle hashing via `CanPoseidon2Digest`
- Integrates with Fiat-Shamir challenger for non-interactive proofs
- Works with OpenVM's hint system for proof data

## Known Limitations
- Only supports `log_final_poly_len = 0` (final polynomial length 1)
- Maximum two-adicity limited to 27 for BabyBear field
- Some operations require specific field properties (two-adicity)

## References
- Based on Plonky3 implementation
- FRI paper: "Fast Reed-Solomon Interactive Oracle Proofs of Proximity"