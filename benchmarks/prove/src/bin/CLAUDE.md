# OpenVM Prove Benchmarks - Development Guidelines

## Overview
This component contains benchmarks for the OpenVM proving system. When working on these benchmarks, follow these guidelines to maintain consistency and performance.

## Key Principles

### 1. Benchmark Design
- Each benchmark should test a specific capability or workload
- Keep benchmarks focused and measurable
- Provide realistic workloads that represent actual use cases
- Always include input configuration via CLI or stdin

### 2. Performance Considerations
- Use appropriate VM configurations for the workload
- Enable profiling only when needed (impacts performance)
- Choose reasonable segment lengths for continuations
- Consider memory allocator choice (jemalloc vs mimalloc)

### 3. Code Organization
- Place new benchmarks in `src/bin/`
- Use shared utilities from `util.rs`
- Follow the established pattern: parse CLI → build ELF → transpile → prove → verify

## Common Patterns

### Creating a New Benchmark

```rust
use clap::Parser;
use eyre::Result;
use openvm_benchmarks_prove::util::BenchmarkCli;

fn main() -> Result<()> {
    let args = BenchmarkCli::parse();
    
    // 1. Configure VM
    let vm_config = /* your config */;
    
    // 2. Build guest program
    let elf = args.build_bench_program("program_name", &vm_config, None)?;
    
    // 3. Transpile to executable
    let exe = VmExe::from_elf(elf, transpiler)?;
    
    // 4. Run benchmark
    args.bench_from_exe("benchmark_name", vm_config, exe, stdin)
}
```

### VM Configuration Selection

Choose minimal required extensions:
- Basic computation: RV32IM only
- Cryptography: Add specific extensions (Keccak, SHA256, etc.)
- Complex workloads: Use SdkVmConfig with all needed extensions

### Input Handling

Always provide inputs via StdIn:
```rust
let mut stdin = StdIn::default();
stdin.write(&input_data);
```

## Testing Guidelines

### 1. Benchmark Validation
- Ensure benchmark completes successfully
- Verify proof generation and verification
- Check performance metrics are reasonable

### 2. Feature Combinations
Test with different feature flags:
- Default features
- With/without aggregation
- With/without EVM verification
- Different allocators

### 3. Performance Testing
- Run with `--profiling` to collect detailed metrics
- Test with various log blowup parameters
- Measure memory usage and proving time

## Common Issues and Solutions

### 1. Out of Memory
- Reduce `max_segment_length`
- Increase log blowup parameters
- Use more efficient allocator

### 2. Slow Proving
- Check if unnecessary extensions are enabled
- Verify appropriate FRI parameters
- Consider segmentation strategy

### 3. Build Failures
- Ensure guest program exists in `../guest/`
- Check VM configuration matches program requirements
- Verify transpiler extensions match instruction set

## Performance Optimization

### 1. VM Configuration
- Only include necessary extensions
- Use appropriate field sizes for modular arithmetic
- Configure reasonable trace lengths

### 2. Proof Parameters
- Balance security and performance with log blowup
- Use standard FRI parameters unless special requirements
- Enable parallelization features

### 3. Memory Management
- Choose appropriate allocator for workload
- Configure segment lengths based on memory constraints
- Monitor peak memory usage

## Adding New Extensions

When benchmarking new VM extensions:
1. Add transpiler extension to the benchmark
2. Configure VM with the extension
3. Ensure guest program uses the extension
4. Measure overhead compared to base configuration

## Metrics Collection

Important metrics to track:
- Total proving time
- Peak memory usage
- Proof size
- Verification time
- Cycles executed
- Constraint evaluation time

Use `run_with_metric_collection` for automated metrics export.

## Integration with CI/CD

Benchmarks should:
- Complete in reasonable time for CI
- Provide consistent results
- Export metrics in standard format
- Support both debug and release builds

## Security Considerations

- Never use benchmarks with untrusted input
- Validate all guest program outputs
- Ensure deterministic execution
- Check proof verification always succeeds

## Debugging Tips

1. Enable `RUST_LOG=debug` for detailed output
2. Use smaller inputs for faster iteration
3. Check guest program independently first
4. Verify transpiler configuration matches VM

## Future Improvements

When adding benchmarks, consider:
- Real-world application scenarios
- Cross-extension interactions
- Aggregation performance
- On-chain verification costs