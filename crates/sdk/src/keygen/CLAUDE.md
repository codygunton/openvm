# Keygen Component Instructions

## Architecture  
Claude MUST read the `./CURSOR.mdc` file before making any changes to this component.

## Overview
The keygen module is responsible for generating proving keys for the multi-level proof system in OpenVM. It manages the complex hierarchy of verifiers and their corresponding proving keys.

## Key Responsibilities
1. Generate proving keys for app, leaf, internal, and root verifiers
2. Manage AIR permutations for optimal proof generation
3. Handle dummy proof generation for key generation
4. Convert native programs to RISC-V assembly format

## Critical Implementation Details

### Proving Key Hierarchy
- **App Proving Key**: For user applications
- **Leaf Verifier**: Verifies app proofs
- **Internal Verifier**: Aggregates leaf proofs
- **Root Verifier**: Final aggregation layer
- **Static Verifier** (EVM feature): Halo2-based verifier for on-chain verification

### AIR Permutation Strategy
AIRs are reordered by trace height (largest first) for the root verifier to ensure consistent trace heights required by the static verifier.

### Memory Management
Key generation is memory-intensive, especially for Halo2 proving keys (>10GB). Use Arc for shared ownership of large structures.

## Best Practices
1. Always check FRI parameters against max constraint degrees
2. Validate recursive verifier sizes to prevent soundness issues
3. Use dummy proofs to determine trace heights before actual proving
4. Ensure special AIRs (program, connector, public values) are correctly tracked after permutation

## Common Pitfalls
1. Not handling AIR permutation correctly can break proof verification
2. Insufficient memory allocation for Halo2 key generation
3. Mismatched FRI parameters between verifier levels
4. Not rounding trace heights to powers of 2 for dummy proofs