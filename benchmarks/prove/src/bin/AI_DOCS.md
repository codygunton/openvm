# OpenVM Prove Benchmarks - Detailed Documentation

## Overview

The `openvm-benchmarks-prove` crate provides a comprehensive suite of benchmarks for testing and measuring the performance of the OpenVM proving system. These benchmarks demonstrate various capabilities of the zkVM, from simple arithmetic operations to complex cryptographic computations.

## Architecture

### Core Components

#### BenchmarkCli (`util.rs`)
The main CLI interface for configuring and running benchmarks with the following parameters:

- **Log Blowup Parameters**: Controls proof size/performance tradeoff
  - `app_log_blowup`: Application level (default: 2)
  - `leaf_log_blowup`: Aggregation leaf level (default: 2)
  - `internal_log_blowup`: Internal aggregation level (default: 2)
  - `root_log_blowup`: Root aggregation level (default: 2)

- **Halo2 Parameters**: For final proof aggregation
  - `halo2_outer_k`: Outer circuit size
  - `halo2_wrapper_k`: Wrapper circuit size
  - `kzg_params_dir`: KZG parameters directory

- **Execution Parameters**:
  - `max_segment_length`: Maximum segment size for continuations
  - `profiling`: Enable performance profiling

#### Proof Generation Flow (`bench_from_exe`)

1. **Key Generation**: Generate proving keys from VM configuration
2. **EXE Commitment**: Create committed executable with cached trace
3. **Execution**: Run the program with provided inputs
4. **Trace Generation**: Generate execution trace
5. **STARK Proof**: Generate STARK proofs for each segment
6. **Verification**: Verify all proofs including boundary conditions
7. **Aggregation** (optional): Generate leaf aggregation proofs

### Benchmark Types

#### 1. Basic Computation (`fibonacci.rs`)
- Tests basic arithmetic and control flow
- Uses RV32IM instruction set
- Configurable iteration count via stdin

#### 2. End-to-End with EVM (`fib_e2e.rs`, `kitchen_sink.rs`)
- Full pipeline including EVM verification
- Requires `evm` feature flag
- Tests Halo2 aggregation and on-chain verification

#### 3. Cryptographic Operations

**ECDSA (`ecrecover.rs`)**:
- Signature verification
- Key recovery
- Secp256k1 curve operations

**Pairing Cryptography (`pairing.rs`)**:
- BN254 and BLS12-381 curves
- Miller loop computation
- Final exponentiation

**Hashing**:
- Keccak256 hashing
- SHA256 hashing

#### 4. Data Processing

**Serialization (`bincode.rs`, `rkyv.rs`)**:
- Binary encoding/decoding
- Zero-copy deserialization
- Large data structure handling

**Text Processing**:
- Base64 encoding/decoding (`base64_json.rs`)
- Regular expression matching (`regex.rs`)

#### 5. EVM Integration (`revm_transfer.rs`)
- EVM state transitions
- Transfer operations
- Blockchain integration

## Configuration

### VM Configuration

Each benchmark can configure its VM with different extensions:

```rust
SdkVmConfig::builder()
    .system(SystemConfig::default().with_continuations())
    .rv32i(Default::default())
    .rv32m(Default::default())
    .io(Default::default())
    .keccak(Default::default())
    .sha256(Default::default())
    .bigint(Default::default())
    .modular(ModularExtension::new(moduli))
    .fp2(Fp2Extension::new(moduli))
    .ecc(WeierstrassExtension::new(curves))
    .pairing(PairingExtension::new(curves))
    .build()
```

### FRI Parameters

The framework uses FRI (Fast Reed-Solomon IOP) with configurable security levels:
- Standard 100-bit conjectured security
- Configurable log blowup factors
- Adjustable constraint degrees

### Profiling Support

When `profiling` is enabled:
- Cycle tracking in VM execution
- Detailed performance metrics
- Memory usage statistics
- Constraint evaluation timings

## Performance Considerations

### Memory Usage
- Benchmarks handle large proof sizes
- Configurable segment lengths for memory management
- Optional allocators: jemalloc, mimalloc

### Parallelization
- Multi-threaded proof generation
- Parallel constraint evaluation
- Configurable via `parallel` feature

### Optimization Features
- `nightly-features`: Rust nightly optimizations
- `bench-metrics`: Additional performance metrics
- Profile-guided optimization support

## Building and Running

### Build Requirements
- Guest programs in `../guest/` directory
- ELF compilation via cargo-openvm
- VM configuration files

### Execution Flow
1. Build guest program to ELF
2. Transpile ELF for target VM config
3. Run benchmark with configuration
4. Collect and report metrics

### Feature Flags
- `default`: ["parallel", "jemalloc", "bench-metrics"]
- `aggregation`: Enable proof aggregation benchmarks
- `evm`: EVM verification support
- `profiling`: Performance profiling

## Integration Points

### With OpenVM SDK
- Uses SDK for proof generation/verification
- Leverages SDK VM configurations
- Integrates with SDK proving pipeline

### With Guest Programs
- Reads compiled ELFs from guest directory
- Supports custom initialization files
- Handles program input via StdIn

### With Proof Systems
- STARK backend for main proofs
- Halo2 for aggregation
- Native recursion for composition