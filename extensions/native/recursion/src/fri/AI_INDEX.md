# FRI (Fast Reed-Solomon Interactive Oracle Proof) Component - AI Index

## Component Overview
The FRI component implements Fast Reed-Solomon Interactive Oracle Proof (FRI) verification for OpenVM's native recursion extension. FRI is a fundamental protocol for constructing succinct proofs by showing that a committed polynomial has low degree.

## Key Files

### Core Implementation
- **mod.rs**: Main FRI verification logic including query verification and batch opening verification
- **types.rs**: Core data structures for FRI configuration, proofs, challenges, and PCS rounds
- **two_adic_pcs.rs**: Two-adic polynomial commitment scheme implementation with FRI verification
- **domain.rs**: Two-adic multiplicative coset implementation for polynomial evaluation domains

### Supporting Files
- **hints.rs**: Hint management for FRI proof data
- **witness.rs**: Witness stream handling for FRI proofs

## Core Concepts

### FRI Protocol
FRI (Fast Reed-Solomon Interactive Oracle Proof) is used to prove that a committed polynomial has low degree. The protocol works through:
1. **Commit Phase**: Iteratively commits to folded polynomials
2. **Query Phase**: Verifies random queries to ensure consistency
3. **Final Polynomial**: Checks that the final polynomial matches expected degree

### Two-Adic Structure
The implementation leverages two-adic fields (fields with multiplicative subgroups of order 2^k):
- Enables efficient FFT-based polynomial operations
- Supports power-of-2 sized evaluation domains
- Allows for efficient folding in the FRI protocol

## Key Components

### 1. FRI Configuration (`FriConfigVariable`)
```rust
pub struct FriConfigVariable<C: Config> {
    pub log_blowup: usize,              // Log of blowup factor
    pub blowup: usize,                  // Blowup factor (rate)
    pub log_final_poly_len: usize,      // Log of final polynomial length
    pub num_queries: usize,             // Number of queries to verify
    pub proof_of_work_bits: usize,      // Proof of work security
    pub generators: Array<C, Felt<C::F>>, // Two-adic generators
    pub subgroups: Array<C, TwoAdicMultiplicativeCosetVariable<C>>, // Evaluation domains
}
```

### 2. FRI Proof Structure (`FriProofVariable`)
```rust
pub struct FriProofVariable<C: Config> {
    pub commit_phase_commits: Array<C, DigestVariable<C>>,     // Commitments from folding
    pub query_proofs: Array<C, FriQueryProofVariable<C>>,      // Query phase proofs
    pub final_poly: Array<C, Ext<C::F, C::EF>>,               // Final polynomial coefficients
    pub pow_witness: Felt<C::F>,                               // Proof of work witness
}
```

### 3. Domain Management (`TwoAdicMultiplicativeCosetVariable`)
```rust
pub struct TwoAdicMultiplicativeCosetVariable<C: Config> {
    pub log_n: Usize<C::N>,  // Log of domain size
    pub shift: Felt<C::F>,   // Coset shift
    pub g: Felt<C::F>,       // Generator of multiplicative subgroup
}
```

## Verification Flow

### 1. Two-Adic PCS Verification (`verify_two_adic_pcs`)
- Validates FRI proof structure
- Processes commit phase commitments
- Verifies each query independently
- Ensures final polynomial matches commitment

### 2. Query Verification (`verify_query`)
- Verifies a single FRI query through all folding rounds
- Checks Merkle authentication paths
- Validates polynomial evaluations at query points
- Ensures consistency between folding rounds

### 3. Batch Opening Verification (`verify_batch`)
- Verifies multiple polynomial openings in a single batch
- Supports both field and extension field elements
- Optimizes verification through batching

## Security Features

### Proof of Work
- Requires computational work to prevent spam attacks
- Configurable difficulty through `proof_of_work_bits`

### Soundness Parameters
- `num_queries`: Number of random queries (affects soundness error)
- `log_blowup`: Rate parameter affecting proof size and security
- Maximum two-adicity: Currently set to 27 for BabyBear field

## Performance Optimizations

### Static vs Dynamic Mode
- **Static mode**: Compile-time known parameters for optimal performance
- **Dynamic mode**: Runtime parameters for flexibility
- Conditional compilation based on `builder.flags.static_only`

### Cycle Tracking
- Fine-grained performance monitoring
- Tracks cycles for:
  - Query verification
  - Batch verification
  - Reduced opening computation
  - Generator power caching

### Memory Management
- Efficient buffer reuse for opened values
- Pre-allocated arrays for common operations
- Careful pointer management in dynamic mode

## Integration Points

### With Poseidon2
- Uses Poseidon2 for Merkle tree hashing
- Integrates with `CanPoseidon2Digest` trait
- Leverages circuit-friendly hash function

### With Challenger
- Integrates with Fiat-Shamir challenger
- Generates challenges for:
  - Folding parameters (betas)
  - Query indices
  - Random linear combinations

### With OpenVM Native Compiler
- Uses `DslVariable` derive macro for automatic variable management
- Leverages `Builder` for circuit construction
- Integrates with hint system for proof data

## Testing Infrastructure

### Test Utilities
- `build_test_fri_with_cols_and_log2_rows`: Generates test FRI proofs
- Domain assertion helpers for verification
- Support for both static and dynamic testing modes

### Test Coverage
- Two-adic FRI PCS verification
- Domain operations (splitting, shifting)
- Batch opening verification
- Integration with native circuit execution

## Usage Example

```rust
// Create FRI configuration
let config = FriConfigVariable { /* parameters */ };

// Create PCS instance
let pcs_var = TwoAdicFriPcsVariable { config };

// Verify a proof
let mut challenger = DuplexChallengerVariable::new(&mut builder);
pcs_var.verify(
    &mut builder,
    rounds,          // PCS rounds with commitments
    proof,           // FRI proof
    log_max_height,  // Maximum polynomial degree
    &mut challenger
);
```

## Important Constants

- `MAX_TWO_ADICITY = 27`: Maximum supported two-adicity
- `REDUCER_BUFFER_SIZE = 8192`: Buffer size for batch reduction
- `DIGEST_SIZE`: Size of hash digests (from circuit parameters)

## Security Considerations

1. **Parameter Selection**: Choose appropriate `num_queries` and `log_blowup` for desired security level
2. **Field Requirements**: Requires two-adic fields with sufficient two-adicity
3. **Proof Validation**: Always validate proof structure before verification
4. **Deterministic Challenges**: Uses Fiat-Shamir for non-interactive proofs