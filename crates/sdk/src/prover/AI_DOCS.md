# OpenVM SDK Prover Component

## Overview

The prover component in the OpenVM SDK is responsible for generating cryptographic proofs for VM execution. It provides a modular architecture that supports multiple proof systems including STARK proofs for VM execution, aggregation proofs, and optional Halo2 proofs for EVM compatibility.

## Architecture

### Core Components

1. **App Prover** (`app.rs`)
   - Generates proofs for application-level VM execution
   - Handles both single-segment and continuation-based execution
   - Manages VM configuration and execution parameters

2. **STARK Prover** (`stark.rs`)
   - Coordinates between app proving and aggregation proving
   - Generates root proofs for outer recursion
   - Ensures compatibility between app and aggregation parameters

3. **Aggregation Prover** (`agg.rs`)
   - Handles proof aggregation for multiple VM segments
   - Generates leaf proofs from continuation segments
   - Produces final aggregated proofs

4. **Root Prover** (`root.rs`)
   - Manages root-level proof generation
   - Handles verification input generation

5. **Halo2 Prover** (`halo2.rs`) - Feature-gated under `evm-prove`
   - Provides EVM-compatible proof generation
   - Wraps STARK proofs for on-chain verification

6. **VM Module** (`vm/`)
   - Contains VM-specific proving logic
   - Implements continuation and single-segment provers
   - Manages proving keys and types

## Key Concepts

### Proof Types

1. **App Proof**: Proof of VM execution for a specific program
2. **Leaf Proof**: Individual segment proofs in continuation-based execution
3. **Aggregated Proof**: Combined proof of multiple segments
4. **Root Proof**: Final proof for outer recursion
5. **EVM Proof**: Ethereum-compatible proof (when `evm-prove` feature is enabled)

### Continuation Support

The prover supports continuation-based execution, allowing large programs to be split into segments:
- Each segment generates its own proof
- Segments are linked through continuation logic
- Final aggregation combines all segment proofs

### Configuration

The prover uses several configuration types:
- `VmConfig`: VM-specific configuration
- `AppProvingKey`: Application proving parameters
- `AggStarkProvingKey`: Aggregation proving parameters
- `AggregationTreeConfig`: Tree structure for aggregation

## Dependencies

- `openvm-circuit`: Core circuit definitions and VM architecture
- `openvm-stark-backend`: STARK proof system backend
- `openvm-stark-sdk`: STARK proving engine and utilities
- `openvm-continuations`: Continuation and verification logic
- `openvm-native-recursion`: Native field recursion (for Halo2)

## Usage Flow

1. **Initialization**: Create prover with proving keys and configuration
2. **Execution**: Generate app proof from input (`StdIn`)
3. **Aggregation**: Aggregate proofs if using continuations
4. **Finalization**: Generate root proof or EVM-compatible proof

## Feature Flags

- `evm-prove`: Enables Halo2 proving for EVM compatibility
- `bench-metrics`: Enables performance metrics collection

## Performance Considerations

- FRI parameters affect proof size and generation time
- Continuation segment size impacts memory usage
- Aggregation tree depth affects recursion overhead

## Security Notes

- Proving keys must be generated from trusted setup
- Parameters must match between app and aggregation layers
- Public values count must be consistent across components