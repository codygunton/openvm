# OpenVM Prof - Claude Instructions

## Component Overview

This is the OpenVM profiling tool (`openvm-prof`) that processes performance metrics from zkVM executions and generates detailed benchmark reports. It's a standalone CLI tool for analyzing proof generation performance.

## Key Responsibilities

1. **Metrics Processing**: Parse JSON metrics files from proof generation
2. **Statistical Analysis**: Calculate min/max/avg/sum statistics per metric
3. **Comparison**: Compute diffs between benchmark runs
4. **Report Generation**: Create markdown and JSON outputs for CI/CD

## Architecture Notes

### Data Flow
1. Metrics JSON → MetricDb (memory-mapped loading)
2. MetricDb → GroupedMetrics (organize by labels)
3. GroupedMetrics → AggregateMetrics (compute statistics)
4. AggregateMetrics → Output formats (Markdown/Bencher JSON)

### Key Design Decisions
- Memory-mapped files for efficient large file processing
- Hierarchical grouping (app → leaf → internal → root → halo2)
- Automatic proof time aggregation across components
- Diff calculation for performance regression detection

## Important Patterns

### Metric Naming Convention
All time metrics use `_ms` suffix:
- `total_proof_time_ms`
- `execute_time_ms`
- `trace_gen_time_ms`

### Group Hierarchy
Groups are ordered by proof generation stages:
1. App-specific groups (custom names)
2. `leaf` - Segment proofs
3. `internal.N` - Recursion tree nodes
4. `root` - Final STARK proof
5. `halo2_outer`, `halo2_wrapper` - SNARK wrapping

### Parallel Time Estimation
- Serial time: Sum of all proof times
- Parallel time: Max proof time per group + serial execution time

## Common Tasks

### Adding New Metrics
1. Define constant in `aggregate.rs`
2. Add to `VM_METRIC_NAMES` if standard
3. Handle in aggregation logic if needed

### Customizing Output
1. Modify markdown generation in `MetricDb::generate_markdown_tables()`
2. Update summary format in `summary.rs`
3. Extend Bencher output in `to_bencher_metrics()`

## Integration Points

### Input Format
Expects metrics in Prometheus-style JSON:
```json
{
  "counter": [...],
  "gauge": [...]
}
```

### Output Formats
1. **Markdown**: Human-readable tables with diffs
2. **Bencher JSON**: CI/CD benchmark tracking
3. **GitHub Summary**: Concise PR comments

## Performance Considerations

- Uses `memmap2` for efficient file I/O
- Minimal memory allocation during processing
- Suitable for processing GB-scale metrics files

## Testing Guidelines

When modifying:
1. Test with real metrics files from benchmark runs
2. Verify diff calculations with known changes
3. Check markdown formatting renders correctly
4. Ensure Bencher JSON validates

## Common Pitfalls

1. **Missing Metrics**: Always handle Option types gracefully
2. **Label Assumptions**: Don't assume all metrics have "group" label
3. **Time Units**: Be consistent with milliseconds vs seconds
4. **Diff Signs**: Negative = improvement, Positive = regression