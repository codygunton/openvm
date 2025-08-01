# OpenVM Prof - AI Index

## Core Components

### Main Entry Points
- `main.rs` - CLI application for processing metrics files
- `lib.rs` - Core library with MetricDb implementation

### Data Processing
- `types.rs` - Core data structures (Metric, Labels, MetricDb, etc.)
- `aggregate.rs` - Metric aggregation and statistical analysis
- `summary.rs` - GitHub-friendly summary generation

## Key Types

### Metrics Storage
- `MetricDb` - Central database for metrics
- `Metric` - Individual metric with name and value
- `Labels` - Key-value pairs for metric categorization
- `MetricsFile` - JSON input format

### Aggregation
- `GroupedMetrics` - Metrics organized by group label
- `AggregateMetrics` - Statistical summaries per group
- `Stats` - Min/max/avg/sum statistics

### Output Formats
- `MdTableCell` - Markdown table cell with optional diff
- `BencherValue` - Bencher-compatible metric format
- `BenchmarkOutput` - Full benchmark results in BMF
- `GithubSummary` - Concise summary for GitHub

## Key Functions

### MetricDb Operations
- `MetricDb::new()` - Load metrics from JSON file
- `MetricDb::apply_aggregations()` - Calculate derived metrics
- `MetricDb::generate_markdown_tables()` - Create markdown output
- `MetricDb::separate_by_label_types()` - Organize by label structure

### Aggregation
- `GroupedMetrics::new()` - Group metrics by label
- `GroupedMetrics::aggregate()` - Compute statistics
- `AggregateMetrics::compute_total()` - Calculate totals
- `AggregateMetrics::set_diff()` - Compare with previous run

### Summary Generation
- `GithubSummary::new()` - Create summary from aggregated metrics
- `GithubSummary::write_markdown()` - Generate markdown table
- `AggregateMetrics::get_summary_row()` - Extract key metrics

## Common Patterns

### Metric Processing Flow
1. Load metrics from JSON → `MetricDb::new()`
2. Apply aggregations → `apply_aggregations()`
3. Group by labels → `GroupedMetrics::new()`
4. Calculate statistics → `aggregate()`
5. Generate output → `write_markdown()` or `to_bencher_metrics()`

### Comparison Workflow
1. Load current and previous metrics
2. Calculate aggregates for both
3. Compute diffs → `set_diff()`
4. Display with color-coded changes

### Label Hierarchy
- App-level metrics (custom group names)
- `leaf` - Leaf proof metrics
- `internal.N` - Internal node metrics at height N
- `root` - Root proof metrics
- `halo2_outer` - Outer Halo2 proof
- `halo2_wrapper` - Wrapper Halo2 proof

## Important Constants

### Metric Names
- `PROOF_TIME_LABEL` = "total_proof_time_ms"
- `CELLS_USED_LABEL` = "main_cells_used"
- `CYCLES_LABEL` = "total_cycles"
- `EXECUTE_TIME_LABEL` = "execute_time_ms"
- `TRACE_GEN_TIME_LABEL` = "trace_gen_time_ms"
- `PROVE_EXCL_TRACE_TIME_LABEL` = "stark_prove_excluding_trace_time_ms"

### VM Metrics Array
- `VM_METRIC_NAMES` - Standard set of VM performance metrics