# OpenVM Prove Benchmarks - Quick Reference

## Running Benchmarks

### Basic Commands

```bash
# Run Fibonacci benchmark
cargo run --bin fibonacci --release

# Run with custom parameters
cargo run --bin fibonacci --release -- --app-log-blowup 3 --max-segment-length 100000

# Run with profiling
cargo run --bin fibonacci --release --features profiling -- --profiling

# Run with EVM verification
cargo run --bin fib_e2e --release --features evm
```

### CLI Parameters

| Parameter | Short | Default | Description |
|-----------|-------|---------|-------------|
| `--app-log-blowup` | `-p` | 2 | Application proof blowup factor |
| `--leaf-log-blowup` | `-g` | 2 | Leaf aggregation blowup factor |
| `--internal-log-blowup` | `-i` | 2 | Internal aggregation blowup |
| `--root-log-blowup` | `-r` | 2 | Root aggregation blowup |
| `--max-segment-length` | `-m` | None | Maximum continuation segment length |
| `--halo2-outer-k` | | 23 | Halo2 outer circuit size (2^k) |
| `--halo2-wrapper-k` | | None | Halo2 wrapper circuit size |
| `--kzg-params-dir` | | None | Directory for KZG parameters |
| `--profiling` | | false | Enable profiling metrics |

### Aggregation Tree Parameters

| Parameter | Default | Description |
|-----------|---------|-------------|
| `--num-children-leaf` | Computed | Children per leaf node |
| `--num-children-internal` | Computed | Children per internal node |
| `--num-children-root` | Computed | Children for root node |

## Available Benchmarks

### Computation Benchmarks

| Binary | Description | Required Features |
|--------|-------------|-------------------|
| `fibonacci` | Iterative Fibonacci computation | None |
| `fib_e2e` | End-to-end Fibonacci with EVM | `evm` |
| `kitchen_sink` | All extensions stress test | `evm` |

### Cryptographic Benchmarks

| Binary | Description | Key Operations |
|--------|-------------|----------------|
| `ecrecover` | ECDSA signature recovery | secp256k1 |
| `pairing` | Pairing cryptography | BN254, BLS12-381 |
| `keccak256` | Keccak hashing | Hash operations |
| `sha256` | SHA256 hashing | Hash operations |

### Data Processing Benchmarks

| Binary | Description | Focus Area |
|--------|-------------|------------|
| `base64_json` | Base64 JSON parsing | Encoding/parsing |
| `bincode` | Binary serialization | Minecraft data |
| `rkyv` | Zero-copy deserialization | Large structures |
| `regex` | Pattern matching | Email validation |

### Blockchain Benchmarks

| Binary | Description | Features |
|--------|-------------|----------|
| `revm_transfer` | EVM transfers | State transitions |
| `verify_fibair` | FibAir verification | Proof verification |

## Feature Flags

```toml
# Cargo.toml features
default = ["parallel", "jemalloc", "bench-metrics"]
bench-metrics = ["openvm-sdk/bench-metrics"]
profiling = ["openvm-sdk/profiling"]
aggregation = []  # Leaf aggregation benchmarks
evm = ["openvm-sdk/evm-verify"]
parallel = ["openvm-sdk/parallel"]
mimalloc = ["openvm-sdk/mimalloc"]
jemalloc = ["openvm-sdk/jemalloc"]
jemalloc-prof = ["openvm-sdk/jemalloc-prof"]
nightly-features = ["openvm-sdk/nightly-features"]
```

## Building Guest Programs

```bash
# Guest programs are in ../guest/<program_name>/
# Built automatically by benchmarks

# Manual build (if needed)
cd ../guest/fibonacci
cargo openvm build --release
```

## Common Usage Patterns

### Performance Testing
```bash
# Minimal proving (fast)
cargo run --bin fibonacci --release -- -p 1 -g 1

# High security (slow)
cargo run --bin fibonacci --release -- -p 4 -g 4

# With metrics collection
RUST_LOG=info cargo run --bin fibonacci --release --features bench-metrics
```

### Memory Optimization
```bash
# Use mimalloc
cargo run --bin fibonacci --release --features mimalloc

# Limit segment size
cargo run --bin fibonacci --release -- --max-segment-length 50000
```

### Debugging
```bash
# Enable debug logs
RUST_LOG=debug cargo run --bin fibonacci

# With profiling data
cargo run --bin fibonacci --features profiling -- --profiling
```

## Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `RUST_LOG` | Log level | `debug`, `info`, `warn` |
| `OUTPUT_PATH` | Metrics output directory | `/tmp/metrics` |
| `OPENVM_PARAMS_DIR` | Parameter cache | `~/.openvm/params` |

## Input Formats

### StdIn Structure
```rust
// Fibonacci input
stdin.write(&n: u64);  // Number of iterations

// Kitchen sink input
stdin.write(&data_len: u32);
stdin.write_slice(&data: &[u8]);

// Pairing input
stdin.write(&num_points: u32);
stdin.write_slice(&points: &[G1Point]);
```

## Output Metrics

### Collected Metrics
- `keygen_time_ms`: Key generation time
- `commit_exe_time_ms`: Executable commitment time
- `prove_time_ms`: Total proving time
- `verify_time_ms`: Verification time
- `peak_memory_mb`: Peak memory usage
- `proof_size_bytes`: Final proof size
- `cycles_executed`: VM cycles count

### Metric Export
```bash
# JSON format
run_with_metric_collection("/path/to/output", || { ... })

# Metrics written to: /path/to/output/metrics.json
```

## Troubleshooting

### Common Issues

| Issue | Solution |
|-------|----------|
| Out of memory | Reduce `--max-segment-length` or increase log blowup |
| Slow proving | Check if unnecessary extensions enabled |
| Build fails | Ensure guest program exists and builds |
| Verification fails | Check transpiler matches VM config |

### Debug Commands

```bash
# Check guest program builds
cd ../guest/fibonacci && cargo build --release

# Verify ELF exists
ls ../guest/fibonacci/elf/*.elf

# Test minimal config
cargo test -p openvm-benchmarks-prove
```

## Performance Tips

1. **Start Simple**: Use minimal extensions and parameters
2. **Profile First**: Run with `--profiling` to identify bottlenecks
3. **Tune Parameters**: Adjust log blowup based on security needs
4. **Use Parallelism**: Ensure `parallel` feature is enabled
5. **Choose Right Allocator**: jemalloc for multi-threaded, mimalloc for memory-constrained

## Quick Benchmark Selection

- **Testing basic proving**: Use `fibonacci`
- **Testing all extensions**: Use `kitchen_sink`
- **Testing crypto ops**: Use `ecrecover` or `pairing`
- **Testing data processing**: Use `bincode` or `rkyv`
- **Testing EVM integration**: Use `revm_transfer`