# OpenVM Prof Component AI Documentation

## Component Overview

The `openvm-prof` component is a performance analysis tool for OpenVM zkVM executions. It processes performance metrics from proof generation and produces detailed benchmark reports for analyzing zkVM performance characteristics.

## Architecture & Design

### Core Components

#### 1. MetricDb (`lib.rs`, `types.rs`)
- **Purpose**: Central data structure for loading and organizing performance metrics
- **Key Features**:
  - Memory-mapped file loading for efficient processing of large metric files
  - Hierarchical metric organization by labels
  - Custom aggregation logic for proof time calculations
  - Markdown table generation for human-readable output

#### 2. GroupedMetrics (`aggregate.rs`)
- **Purpose**: Groups metrics by configurable labels (typically "group") for analysis
- **Key Features**:
  - Separates metrics by group labels (app, leaf, internal, root, halo2)
  - Handles ungrouped metrics separately
  - Provides foundation for statistical aggregation

#### 3. AggregateMetrics (`aggregate.rs`)
- **Purpose**: Computes statistical summaries (min/max/avg/sum) for grouped metrics
- **Key Features**:
  - Calculates total proof time across all groups
  - Estimates parallel proof time using maximum times per group
  - Supports diff calculation between benchmark runs
  - Converts between time units (ms to seconds)

#### 4. CLI Interface (`main.rs`)
- **Purpose**: Command-line interface for processing metrics files
- **Key Features**:
  - Batch processing of multiple metrics files
  - Optional comparison with previous benchmark runs
  - Multiple output formats (Markdown, Bencher JSON)
  - GitHub summary generation for CI/CD integration

### Data Flow Architecture

```
Metrics JSON Files → MetricDb → GroupedMetrics → AggregateMetrics → Output Formats
                                     ↓
                                 Statistical
                                 Analysis
                                     ↓
                              Markdown Tables
                              Bencher JSON
                              GitHub Summary
```

## Key Data Structures

### Labels
- Represents metric labels as key-value pairs
- Supports custom sorting (prioritizes "group" label)
- Implements hash and equality for use as HashMap keys

### Metric
- Simple structure containing metric name and numeric value
- Foundation for all performance measurements

### MdTableCell
- Enhanced numeric value with optional diff from previous run
- Supports formatted display with color-coded changes
- Core type for markdown table generation

### Stats
- Statistical aggregation container (sum, min, max, avg, count)
- Supports diff calculation between benchmark runs
- Automatic finalization to compute averages

## Performance Metrics

### Standard VM Metrics
The component recognizes these core performance metrics:

#### Time Metrics (all in milliseconds)
- `execute_time_ms`: VM execution time
- `trace_gen_time_ms`: Trace generation time  
- `prove_excl_trace_time_ms`: Proof generation excluding trace
- `total_proof_time_ms`: Aggregated total (execute + trace_gen + prove_excl_trace)

#### Other Metrics
- `total_cycles`: Execution cycles
- `total_instructions`: Instruction count
- Memory-related metrics
- Custom application metrics

### Group Hierarchy
Performance metrics are organized by execution stage groups:

1. **App Groups**: Custom application-specific proofs
2. **leaf**: Segment-level STARK proofs
3. **internal.N**: Recursion tree intermediate nodes
4. **root**: Final STARK proof aggregation
5. **halo2_outer**: SNARK wrapper proof
6. **halo2_wrapper**: Final SNARK proof
7. **keygen**: Key generation (excluded from totals)

### Time Calculations

#### Serial Proof Time
Sum of all proof times across all groups (excluding keygen):
```rust
total_serial_time = sum(group.proof_time for group in all_groups if not keygen)
```

#### Parallel Proof Time  
Estimated parallel execution time accounting for concurrent proof generation:
```rust
total_parallel_time = sum(max(group.proof_time) for group in all_groups if not keygen)
```

## Input/Output Formats

### Input: Prometheus-Style JSON
```json
{
  "counter": [
    {
      "labels": [["group", "leaf"], ["segment", "0"]],
      "metric": "total_cycles",
      "value": "1000000"
    }
  ],
  "gauge": [
    {
      "labels": [["group", "leaf"]],
      "metric": "execute_time_ms", 
      "value": "150.5"
    }
  ]
}
```

### Output Formats

#### 1. Markdown Tables
Human-readable tables with statistical summaries and diffs:
- Grouped by metric labels
- Color-coded performance changes
- Collapsible detailed metrics sections

#### 2. Bencher JSON (BMF Format)
Machine-readable format for CI/CD benchmark tracking:
```json
{
  "app_name": {
    "total_proof_time": {"value": 45.2},
    "leaf": {
      "execute_time_ms": {"value": 150.5}
    }
  }
}
```

#### 3. GitHub Summary
Concise markdown for PR comments and GitHub Actions summaries.

## Key Algorithms

### Custom Label Sorting
Ensures consistent ordering with "group" label prioritized:
```rust
fn custom_sort_label_keys(keys: &mut [String]) {
    keys.sort_by_key(|key| {
        if key == "group" { (0, key.clone()) } 
        else { (1, key.clone()) }
    });
}
```

### Group Weight Ordering  
Defines display order for proof generation stages:
```rust
fn group_weight(name: &str) -> usize {
    // keygen > halo2_wrapper > halo2_outer > root > internal.N > leaf > app
}
```

### Diff Calculation
Performance regression detection:
```rust
diff = current_value - previous_value
diff_percent = diff / previous_value * 100.0
// Negative = improvement, Positive = regression
```

## Integration Points

### CLI Usage Patterns
```bash
# Single metrics file analysis
openvm-prof --json-paths metrics.json

# Comparison with previous run
openvm-prof --json-paths metrics.json --prev-json-paths prev_metrics.json

# Batch processing with custom names
openvm-prof --json-paths a.json,b.json --names "App A,App B"

# Generate GitHub summary
openvm-prof summary --benchmark-results-link "https://..." --summary-md-path summary.md
```

### Memory-Mapped File Processing
Uses `memmap2` for efficient processing of large metrics files:
```rust
let file = File::open(metrics_file)?;
let mmap = unsafe { Mmap::map(&file)? };
let metrics: MetricsFile = serde_json::from_slice(&mmap)?;
```

## Error Handling Patterns

### File Processing
- Graceful handling of missing previous metrics files (new benchmarks)
- Validation of array length consistency for batch operations
- Memory-mapped file error propagation

### Metric Processing  
- Zero-value counter filtering to reduce noise
- Missing metric graceful handling with Option types
- Label validation and custom sorting

### Output Generation
- UTF-8 string conversion validation
- File write error handling
- JSON serialization error propagation

## Performance Considerations

### Scalability
- Memory-mapped files handle GB-scale metrics efficiently
- Minimal memory allocation during processing
- Stream-based output generation

### Processing Efficiency
- Single-pass metric aggregation
- Lazy evaluation of statistical calculations
- Efficient HashMap-based grouping

## Testing & Validation

### Test Data
- `data/metrics.example.json`: Sample metrics file
- `data/metrics.prev.example.json`: Previous run comparison data

### Validation Points
- Statistical calculation accuracy
- Markdown formatting correctness
- Bencher JSON schema compliance
- Diff calculation validation

## Security Considerations

### File Access
- Memory-mapped files require appropriate file permissions
- Input validation for JSON parsing
- Safe handling of file paths and names

### Data Processing
- No sensitive data exposure in output formats
- Controlled memory usage patterns
- Safe numeric conversions and formatting

## Extension Points

### Adding New Metrics
1. Define metric constants in `aggregate.rs`
2. Add to `VM_METRIC_NAMES` for standard processing
3. Implement custom aggregation logic if needed
4. Update output formatting in relevant generators

### Custom Output Formats
1. Implement new format generator in appropriate module
2. Add CLI option for format selection
3. Extend main processing loop to handle new format

### New Statistical Functions
1. Extend `Stats` struct with new calculations
2. Update aggregation logic in `AggregateMetrics`
3. Modify output formatters to display new statistics

This component serves as the primary performance analysis tool for OpenVM, providing essential insights into zkVM execution characteristics and proof generation performance.