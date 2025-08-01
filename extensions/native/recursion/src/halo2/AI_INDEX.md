# Halo2 Component AI Index

## Overview
The Halo2 component provides integration between OpenVM's native recursion framework and the Halo2 proof system. It enables generating SNARK proofs for OpenVM STARK verifiers using the Halo2 proving system with KZG commitments on the BN254 curve.

## Key Components

### Core Types
- **Halo2Prover**: Main prover that generates Halo2 proofs from DSL operations
- **Halo2ProvingPinning**: Metadata and proving key for Halo2 circuits
- **DslOperations**: Compiled DSL operations from the native compiler
- **RawEvmProof**: Proof format suitable for EVM verification

### Modules
- `mod.rs`: Main module with prover implementation
- `verifier.rs`: Halo2 verifier circuit generation for STARK proofs
- `wrapper.rs`: Wrapper circuit for aggregating proofs and EVM deployment
- `utils.rs`: KZG parameters management and verification utilities
- `testing_utils.rs`: Testing utilities for Halo2 circuits

## Purpose
Enables recursive proof composition by converting STARK proofs into Halo2 SNARKs, which can be efficiently verified on-chain (especially on Ethereum via EVM).

## Integration Points
- Uses `openvm-native-compiler` for constraint compilation
- Integrates with `snark-verifier-sdk` for Halo2 circuit construction
- Supports BabyBear field arithmetic with BN254 scalar field
- Provides EVM-compatible proof generation

## Key Features
- Mock proving for testing
- Keygen and proving with configurable parameters
- EVM verifier generation
- Proof aggregation via wrapper circuits
- Parameter caching and management