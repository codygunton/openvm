# OpenVM Prove Benchmarks - AI Documentation Index

## Component Overview
The OpenVM Prove benchmarks demonstrate and measure the performance of proving various programs using the OpenVM zkVM framework. This component contains executable benchmarks that compile guest programs, generate proofs, and verify them using different VM configurations.

## Key Files

### Core Infrastructure
- `util.rs` - Shared benchmark utilities including CLI parsing, configuration, and proof generation
- `lib.rs` - Module exports

### Benchmark Programs
- `fibonacci.rs` - Proves execution of Fibonacci computation using RV32IM
- `fib_e2e.rs` - End-to-end Fibonacci benchmark with EVM verification
- `kitchen_sink.rs` - Comprehensive benchmark using all available extensions (crypto, pairing, etc.)
- `ecrecover.rs` - ECDSA signature recovery benchmark
- `pairing.rs` - Pairing-based cryptography benchmark
- `regex.rs` - Regular expression matching benchmark
- `revm_transfer.rs` - EVM transfer operation benchmark
- `base64_json.rs` - Base64 JSON parsing benchmark
- `bincode.rs` - Binary encoding/decoding benchmark
- `rkyv.rs` - Zero-copy deserialization benchmark
- `verify_fibair.rs` - FibAir proof verification benchmark

## Architecture

### Benchmark Flow
1. Parse CLI arguments for configuration
2. Build guest program ELF from source
3. Transpile ELF to VM executable
4. Generate proving key (keygen)
5. Commit to executable
6. Execute program with inputs
7. Generate STARK proof
8. Verify proof
9. Optionally aggregate proofs

### Key Components Used
- **VM Configs**: RV32IM, SDK VM with extensions
- **Proof Systems**: STARK with FRI, optional Halo2 aggregation
- **Extensions**: Crypto (Keccak, SHA256), BigInt, Modular arithmetic, ECC, Pairings

## Dependencies
- OpenVM SDK for proving/verification
- VM circuits (RV32IM, crypto extensions)
- Transpiler for ELF to VM conversion
- STARK backend for proof generation
- Native recursion for aggregation

## Usage Context
These benchmarks are used to:
- Measure proving performance
- Test different VM configurations
- Validate proof generation and verification
- Profile resource usage
- Demonstrate OpenVM capabilities