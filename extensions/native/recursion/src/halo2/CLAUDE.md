# Halo2 Component Instructions for Claude

## Component Overview
This is the Halo2 integration component for OpenVM's native recursion framework. It provides SNARK proof generation using the Halo2 proof system with KZG commitments on BN254.

## Key Responsibilities
When working with this component, you should:

1. **Maintain Field Compatibility**: Always ensure proper conversion between BabyBear (STARK field) and BN254Fr (SNARK field)
2. **Preserve Circuit Determinism**: Never modify circuit construction in ways that could change the circuit structure between keygen and proving
3. **Handle Parameters Carefully**: KZG parameters are large files that should be cached and reused

## Code Standards

### Import Organization
```rust
// External crates first
use itertools::Itertools;
use serde::{Deserialize, Serialize};

// OpenVM dependencies
use openvm_native_compiler::{...};
use openvm_stark_backend::{...};

// Snark verifier SDK
use snark_verifier_sdk::{...};

// Local imports
use crate::halo2::{...};
```

### Error Handling
- Use `anyhow::Result` for high-level functions
- Provide context with `.context()` for errors
- Never panic in library code except for programmer errors

### Testing Requirements
- Always test with mock prover before real proving
- Include both unit tests and integration tests
- Test edge cases like empty witnesses and maximum circuit sizes

## Common Tasks

### Adding New Verifier Types
When implementing a new verifier:
1. Create a new struct wrapping `Halo2VerifierProvingKey`
2. Implement witness generation for your proof type
3. Add integration tests with your proof system
4. Document public API thoroughly

### Optimizing Circuit Performance
1. Profile with mock prover first to understand constraints
2. Minimize advice column usage
3. Use lookup tables for repeated operations
4. Cache proving keys and parameters

### EVM Integration
When working on EVM features:
1. Ensure feature flags are properly gated (`#[cfg(feature = "evm-prove")]`)
2. Test generated verifiers with actual EVM execution
3. Minimize calldata size for gas efficiency

## Architecture Constraints

1. **No CPU Architecture**: This component doesn't implement CPU opcodes
2. **Proof Composition**: Designed for recursive proof composition, not standalone proving
3. **Fixed Field**: Always uses BN254 scalar field for Halo2 circuits

## Security Considerations

1. **Trusted Setup**: KZG parameters must come from a trusted ceremony
2. **Deterministic Proving**: Circuit structure must be identical between keygen and proving
3. **Public Input Ordering**: Maintain consistent ordering of public values
4. **No Secret Leakage**: Never log or expose witness values

## Performance Guidelines

1. **Parameter Selection**: Choose minimal k that satisfies constraints
2. **Witness Generation**: Can be parallelized for better performance
3. **Caching Strategy**: Cache parameters, proving keys, and even compiled circuits
4. **Profiling**: Use `profiling = true` flag to collect metrics

## Common Pitfalls to Avoid

1. **Mismatched Break Points**: Always use the same break points for keygen and proving
2. **Field Element Confusion**: BabyBear elements must be properly embedded into BN254Fr
3. **Circuit Non-determinism**: Avoid any randomness in circuit construction
4. **Parameter Corruption**: Always verify parameter file integrity

## Testing Checklist

- [ ] Mock prover passes
- [ ] Real prover generates valid proofs
- [ ] Verifier accepts valid proofs
- [ ] Verifier rejects invalid proofs
- [ ] EVM verifier (if applicable) works correctly
- [ ] Performance meets requirements
- [ ] No memory leaks or excessive allocations

## Integration Notes

This component integrates with:
- `openvm-native-compiler`: For constraint compilation
- `openvm-stark-backend`: For STARK proof structures
- `snark-verifier-sdk`: For Halo2 circuit construction
- EVM: For on-chain verification (optional)

Always maintain compatibility with these dependencies.