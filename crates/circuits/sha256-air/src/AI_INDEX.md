# openvm-sha256-air: Component Overview

## Purpose
This component implements the SHA-256 compression function as an Arithmetic Intermediate Representation (AIR) for use in OpenVM's zkVM framework. It provides a constraint-based implementation without message padding, allowing for efficient zero-knowledge proof generation of SHA-256 computations.

## Architecture Summary
The component follows a columnar architecture where SHA-256 computations are laid out in a trace matrix:
- **17 rows per block**: 16 round rows + 1 digest row
- **4 rounds per row**: Processes 64 SHA-256 rounds efficiently
- **Bitwise operations**: Integrated with OpenVM's bitwise lookup tables
- **Self-interaction bus**: For block-to-block hash chaining

## Key Files
- `air.rs`: Core AIR constraints and evaluation logic
- `columns.rs`: Column layout structures for round and digest rows
- `trace.rs`: Trace generation for SHA-256 blocks
- `utils.rs`: SHA-256 functions and helper utilities

## Main Types
- `Sha256Air`: Main AIR implementation with constraints
- `Sha256RoundCols<T>`: Column layout for computation rounds
- `Sha256DigestCols<T>`: Column layout for block finalization
- `Sha256FlagsCols<T>`: Control flags and indices

## Integration Points
- Uses `openvm-circuit-primitives` for bitwise operations
- Integrates with `openvm-stark-backend` for proof generation
- Provides trace generation compatible with OpenVM's zkVM

## Design Philosophy
- **No-padding design**: Expects pre-padded 512-bit message blocks
- **Efficient constraints**: Minimizes constraint degree through careful column layout
- **Modular integration**: Works seamlessly with other OpenVM components