# OpenVM Native Recursion Extension - File Index

## Core Files

### lib.rs
- **Purpose**: Module declarations and exports
- **Key exports**: Public API for the recursion extension
- **Constants**: `OUTER_DIGEST_SIZE` for digest configuration

### types.rs
- **Purpose**: Core type definitions for recursion
- **Types**:
  - `InnerConfig`: Configuration for inner recursion (BabyBear)
  - `StarkVerificationAdvice`: Constants for single STARK verification
  - `MultiStarkVerificationAdvice`: Constants for multi-STARK verification
  - `VerifierSinglePreprocessedDataInProgram`: Preprocessed data wrapper

### vars.rs
- **Purpose**: Variable types for proof components in the compiler IR
- **Key types**:
  - `StarkProofVariable`: Main proof structure
  - `CommitmentsVariable`: Commitment organization
  - `AirProofDataVariable`: Per-AIR proof data
  - `AdjacentOpenedValuesVariable`: Opened polynomial values

## Verification

### stark/mod.rs
- **Purpose**: Main STARK verification logic
- **Key components**:
  - `VerifierProgram`: Entry point for verification programs
  - `StarkVerifier`: Core verification implementation
  - Constraint checking and proof validation

### stark/outer.rs
- **Purpose**: Outer recursion verification for final proofs
- **Features**: Bn254 field operations for EVM compatibility

## Challenger

### challenger/mod.rs
- **Purpose**: Challenge generation interface
- **Trait**: `ChallengerVariable` for Fiat-Shamir challenges

### challenger/duplex.rs
- **Purpose**: Duplex sponge construction for challenges
- **Implementation**: Poseidon2-based challenger

### challenger/multi_field32.rs
- **Purpose**: Multi-field challenger implementation
- **Features**: Efficient challenge generation for 32-bit fields

## FRI (Fast Reed-Solomon IOP)

### fri/mod.rs
- **Purpose**: FRI protocol implementation
- **Components**: Two-adic FRI PCS variable types

### fri/two_adic_pcs.rs
- **Purpose**: Two-adic polynomial commitment scheme
- **Key struct**: `TwoAdicFriPcsVariable`

### fri/types.rs
- **Purpose**: FRI-specific type definitions
- **Types**: Domain variables, round variables, proof structures

### fri/domain.rs
- **Purpose**: Two-adic multiplicative coset implementation
- **Features**: Domain arithmetic and coset operations

### fri/witness.rs
- **Purpose**: Witness generation for FRI protocol
- **Traits**: Witnessable implementations for FRI types

### fri/hints.rs
- **Purpose**: Hint generation for FRI verification
- **Features**: Optimized hint computation

## Configuration

### config/mod.rs
- **Purpose**: Configuration module exports

### config/outer.rs
- **Purpose**: Outer recursion configuration
- **Key types**:
  - `OuterConfig`: Bn254-based configuration
  - Type aliases for outer recursion components
  - Conversion functions from verifying keys

## Utilities

### utils.rs
- **Purpose**: General utility functions
- **Functions**: Configuration builders, helper methods

### helper.rs
- **Purpose**: Helper functions for verification
- **Features**: Auxiliary verification utilities

### hints.rs
- **Purpose**: Hint system for optimized verification
- **Traits**: `Hintable` for efficient proof hints

### digest.rs
- **Purpose**: Digest value handling
- **Types**: `DigestVal` enum for field-agnostic digests

### witness.rs
- **Purpose**: Witness generation utilities
- **Features**: Proof witness construction

### view.rs
- **Purpose**: View helpers for proof data
- **Functions**: `get_advice_per_air` for advice extraction

### commit.rs
- **Purpose**: Commitment scheme abstractions
- **Interfaces**: PCS variable traits

### folder.rs
- **Purpose**: Constraint folding implementation
- **Type**: `RecursiveVerifierConstraintFolder`

### outer_poseidon2.rs
- **Purpose**: Outer Poseidon2 permutation
- **Features**: Optimized Poseidon2 for Bn254

## Testing

### tests.rs
- **Purpose**: Integration tests for recursion
- **Coverage**: End-to-end verification tests

### testing_utils.rs
- **Purpose**: Testing utilities and helpers
- **Features**: Test data generation, mock proofs

## Halo2 Integration (Feature-gated)

### halo2/mod.rs
- **Purpose**: Halo2 proof system integration
- **Features**: Static verifier support

### halo2/verifier.rs
- **Purpose**: Halo2 verifier implementation
- **Features**: Aggregation circuit generation

### halo2/wrapper.rs
- **Purpose**: Halo2 wrapper utilities
- **Features**: Proof format conversion

### halo2/utils.rs
- **Purpose**: Halo2-specific utilities
- **Functions**: Helper methods for Halo2 integration

### halo2/testing_utils.rs
- **Purpose**: Halo2 testing utilities
- **Features**: Test helpers for Halo2 proofs

### halo2/tests/
- **Purpose**: Halo2 integration tests
- **Files**:
  - `mod.rs`: Test module exports
  - `multi_field32.rs`: Multi-field tests
  - `outer_poseidon2.rs`: Poseidon2 tests
  - `stark.rs`: STARK-to-Halo2 tests

## Sub-module Documentation

The recursion extension includes AI documentation for key sub-modules:
- `fri/`: FRI protocol implementation docs
- `halo2/`: Halo2 integration docs
- `stark/`: STARK verification docs

Each sub-module contains:
- `AI_DOCS.md`: Component overview
- `AI_INDEX.md`: File inventory
- `IMPLEMENTATION_GUIDE.ai.md`: Technical details
- `QUICK_REFERENCE.ai.md`: Usage patterns
- `CLAUDE.md`: AI assistant instructions