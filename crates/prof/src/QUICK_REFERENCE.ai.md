# OpenVM Prof - Quick Reference

## CLI Usage

```bash
# Basic profiling
openvm-prof --json-paths metrics.json

# Multiple files with names
openvm-prof --json-paths app1.json,app2.json --names "App 1,App 2"

# Compare with previous run
openvm-prof --json-paths new.json --prev-json-paths old.json

# Generate Bencher output
openvm-prof --json-paths metrics.json --output-json bencher.json

# Create GitHub summary
openvm-prof --json-paths metrics.json summary \
    --benchmark-results-link https://github.com/... \
    --summary-md-path summary.md
```

## Key Data Structures

```rust
// Basic metric
Metric {
    name: String,
    value: f64,
}

// Labels for categorization
Labels(Vec<(String, String)>)

// Table cell with optional diff
MdTableCell {
    val: f64,
    diff: Option<f64>,
}

// Statistical summary
Stats {
    sum: MdTableCell,
    max: MdTableCell,
    min: MdTableCell,
    avg: MdTableCell,
    count: usize,
}
```

## Common Operations

### Load and Process Metrics
```rust
// Load from file
let db = MetricDb::new("metrics.json")?;

// Group by label
let grouped = GroupedMetrics::new(&db, "group")?;

// Calculate statistics
let aggregated = grouped.aggregate();
```

### Generate Output
```rust
// Markdown table
let mut writer = Vec::new();
aggregated.write_markdown(&mut writer, VM_METRIC_NAMES)?;

// Bencher format
let bencher = aggregated.to_bencher_metrics();

// GitHub summary
let summary = GithubSummary::new(&names, &metrics, &paths, &link);
```

### Compare Runs
```rust
// Load both metrics
let current = MetricDb::new("current.json")?;
let previous = MetricDb::new("previous.json")?;

// Calculate diffs
let mut current_agg = GroupedMetrics::new(&current, "group")?.aggregate();
let prev_agg = GroupedMetrics::new(&previous, "group")?.aggregate();
current_agg.set_diff(&prev_agg);
```

## Metric Labels

Standard group labels:
- Custom app names (determined by `group_weight() == 0`)
- `leaf` - Leaf proofs
- `internal.0`, `internal.1`, ... - Internal nodes by height
- `root` - Root proof
- `halo2_outer` - Outer Halo2 proof
- `halo2_wrapper` - Wrapper Halo2 proof
- `*keygen*` - Key generation (excluded from totals)

## Output Examples

### Markdown Summary
```
| Summary | Proof Time (s) | Parallel Proof Time (s) |
|:---|---:|---:|
| Total | 45.23 | 12.34 |
| app_name | 40.12 | 10.23 |
| leaf | 3.45 | 1.23 |
| root | 1.66 | 0.88 |
```

### Detailed Metrics
```
| group |||||
|:---|---:|---:|---:|---:|
|metric|avg|sum|max|min|
| `total_proof_time_ms` | 1234 | 4936 | 1500 | 1000 |
| `main_cells_used` | 50000 | 200000 | 55000 | 45000 |
```

### Diff Display
- Green: Performance improvement (negative diff)
- Red: Performance regression (positive diff)
- Format: `value (+diff [+X.X%])`

## JSON Input Format

```json
{
  "counter": [
    {
      "labels": [["group", "app_name"], ["segment", "0"]],
      "metric": "total_cycles",
      "value": "123456"
    }
  ],
  "gauge": [
    {
      "labels": [["group", "leaf"]],
      "metric": "main_cells_used",
      "value": "50000"
    }
  ]
}
```

## Environment Variables

None required. All configuration via CLI arguments.

## File Outputs

- `*.md` - Markdown reports (same name as input JSON)
- `*.json` - Bencher format output (specified by `--output-json`)
- Console output - When no output file specified

## Performance Tips

1. **Memory Usage**: Files are memory-mapped, suitable for large metrics
2. **Batch Processing**: Process multiple files in single invocation
3. **Selective Metrics**: Use `VM_METRIC_NAMES` to limit output columns

## Common Issues

1. **Missing Metrics**: Gracefully handled, shows default/empty values
2. **Label Mismatch**: Metrics without expected labels go to "ungrouped"
3. **Format Errors**: Clear error messages for malformed JSON

## Integration Examples

### CI/CD Pipeline
```yaml
- name: Run benchmarks
  run: cargo bench --bench vm_bench

- name: Process metrics  
  run: openvm-prof --json-paths metrics.json --output-json bencher.json

- name: Upload results
  run: bencher upload bencher.json
```

### PR Comment
```bash
openvm-prof --json-paths new.json --prev-json-paths main.json \
    summary --benchmark-results-link ${{ github.event.pull_request.html_url }}
```