# STARK Recursion Component - AI Documentation Index

## Component Overview

The STARK recursion component within OpenVM's native recursion extension provides functionality for recursively verifying STARK proofs. This enables proof composition and aggregation, allowing multiple STARK proofs to be verified within a single proof.

## Core Purpose

This component implements STARK verifiers that can run inside the OpenVM zkVM, enabling recursive proof verification. It supports both inner (BabyBear) and outer (BN254) field configurations for different stages of the recursive verification pipeline.

## Key Files

### Main Implementation
- **`mod.rs`** - Core STARK verifier implementation with `VerifierProgram` and `StarkVerifier` structs
- **`outer.rs`** - Specialized verifier for outer field (BN254) static verification

## Architecture Components

### 1. VerifierProgram (`mod.rs`)
- Builds verification programs for BabyBear field configuration
- Manages proof reading, PCS initialization, and verification orchestration
- Supports cycle tracking for performance monitoring

### 2. StarkVerifier (`mod.rs`)
- Core verification logic implementing the STARK verification protocol
- Handles multi-trace verification with RAP (Randomized Air with Preprocessing) constraints
- Manages challenge sampling, commitment verification, and constraint checking

### 3. Outer Verifier (`outer.rs`)
- Static verifier for BN254 field configuration
- Used in the final verification stage for EVM/on-chain verification
- Generates Halo2 circuit operations

## Key Features

1. **Multi-Trace Support**: Verifies multiple AIR (Algebraic Intermediate Representation) traces in a single proof
2. **Height Constraints**: Validates trace height constraints across different AIRs
3. **Challenge Phases**: Supports multiple challenge phases with proof-of-work
4. **Quotient Verification**: Recomputes and verifies quotient polynomials
5. **FRI-based PCS**: Uses Fast Reed-Solomon Interactive Oracle Proofs for polynomial commitments

## Integration Points

- **FRI Module**: Uses `TwoAdicFriPcsVariable` for polynomial commitment verification
- **Challenger**: Integrates with duplex and multi-field challengers for Fiat-Shamir
- **Halo2**: Generates circuit operations for final EVM verification (static-verifier feature)

## Usage Context

This component is used when:
1. Building recursive proofs that verify other STARK proofs
2. Aggregating multiple proofs into a single succinct proof
3. Creating proof chains for complex verification scenarios
4. Generating EVM-verifiable proofs through the Halo2 backend

## Dependencies

- OpenVM STARK backend for proof structures and verification logic
- Native compiler for building verification circuits
- Various cryptographic primitives (Poseidon2, FRI, etc.)

## Security Considerations

- Implements careful validation of proof shapes and constraints
- Ensures all AIRs in multi-trace proofs are properly verified
- Validates height constraints to prevent malformed proofs
- Uses established cryptographic protocols (FRI, Fiat-Shamir)