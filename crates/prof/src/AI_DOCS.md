# OpenVM Prof - AI Documentation

## Overview

OpenVM Prof is a profiling and benchmarking tool for analyzing performance metrics of OpenVM proofs. It processes metrics files (JSON format) to generate detailed performance reports, aggregate statistics, and markdown-formatted benchmark summaries.

## Key Components

### Core Functionality (`lib.rs`)
- **MetricDb**: Central database for storing and processing metrics
  - Loads metrics from JSON files using memory-mapped I/O for efficiency
  - Processes counter and gauge metrics
  - Applies aggregations (e.g., calculating total proof time)
  - Organizes metrics by label types for structured reporting
  - Generates markdown tables for visualization

### Command-Line Interface (`main.rs`)
- Supports multiple metrics files with optional comparison to previous runs
- Generates benchmark outputs in Bencher Metric Format (BMF)
- Creates markdown reports with detailed and aggregated metrics
- Includes a `summary` subcommand for GitHub-friendly summaries

### Data Types (`types.rs`)
- **Metric**: Basic metric with name and value
- **Labels**: Key-value pairs for categorizing metrics
- **MdTableCell**: Table cell with value and optional diff for comparisons
- **BencherValue**: Output format compatible with Bencher benchmarking tool
- **MetricsFile**: JSON structure for input metrics (counters and gauges)

### Aggregation System (`aggregate.rs`)
- **GroupedMetrics**: Groups metrics by "group" label
- **AggregateMetrics**: Computes statistics (sum, max, min, avg) per group
- **Stats**: Statistical summary for each metric
- Calculates total and parallel proof times across all groups
- Supports diff calculation between runs
- Exports to Bencher-compatible format

### Summary Generation (`summary.rs`)
- **GithubSummary**: Creates concise summaries for GitHub comments/PRs
- **BenchSummaryMetrics**: Organizes metrics by proof stages (app, leaf, internal, root, halo2)
- Tracks key metrics: proof time, cycles, cells used
- Supports diff visualization for performance comparisons

## Key Metrics Tracked

1. **Proof Time Metrics**:
   - `total_proof_time_ms`: Total time for proof generation
   - `execute_time_ms`: Execution time
   - `trace_gen_time_ms`: Trace generation time
   - `stark_prove_excluding_trace_time_ms`: Proving time excluding trace

2. **Resource Metrics**:
   - `main_cells_used`: Number of cells used in the proof
   - `total_cycles`: Total execution cycles

3. **Detailed Timing**:
   - `main_trace_commit_time_ms`
   - `generate_perm_trace_time_ms`
   - `perm_trace_commit_time_ms`
   - `quotient_poly_compute_time_ms`
   - `quotient_poly_commit_time_ms`
   - `pcs_opening_time_ms`

## Usage Patterns

1. **Basic profiling**:
   ```bash
   openvm-prof --json-paths metrics1.json,metrics2.json
   ```

2. **With comparison**:
   ```bash
   openvm-prof --json-paths new.json --prev-json-paths old.json
   ```

3. **Generate GitHub summary**:
   ```bash
   openvm-prof --json-paths metrics.json summary --benchmark-results-link https://...
   ```

## Output Formats

1. **Markdown Tables**: Human-readable performance reports
2. **Bencher JSON**: Machine-readable format for CI/CD integration
3. **GitHub Summary**: Concise tables optimized for PR comments

## Architecture Decisions

- Uses memory-mapped files for efficient large metrics file processing
- Groups metrics hierarchically (app → leaf → internal → root → halo2)
- Supports parallel proof time estimation
- Designed for integration with CI/CD benchmarking workflows