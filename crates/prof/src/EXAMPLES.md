# OpenVM Prof Usage Examples

## Basic Usage Examples

### Example 1: Analyze Single Metrics File

```bash
# Analyze metrics from a single benchmark run
openvm-prof --json-paths benchmark_metrics.json

# Output: Creates benchmark_metrics.md with performance tables
```

**Expected Output Structure:**
```markdown
# Benchmark Results

## Summary
- Total Proof Time: 45.32s
- Parallel Proof Time: 23.15s

## Performance by Group
| group | total_proof_time_ms | execute_time_ms | trace_gen_time_ms |
|-------|-------------------|----------------|------------------|
| app   | 15,230           | 5,120          | 8,340           |
| leaf  | 28,450           | 12,340         | 14,200          |
| root  | 1,890            | 450            | 1,200           |

<details>
<summary>Detailed Metrics</summary>
[Additional detailed metrics tables]
</details>
```

### Example 2: Compare Current vs Previous Performance

```bash
# Compare current benchmark against previous run
openvm-prof \
  --json-paths current_metrics.json \
  --prev-json-paths previous_metrics.json

# Shows performance changes with color-coded diffs
```

**Sample Output with Diffs:**
```markdown
| group | total_proof_time_ms |
|-------|-------------------|
| app   | <span style='color: green'>(-1,230 [-7.5%])</span> 15,230 |
| leaf  | <span style='color: red'>(+2,450 [+9.4%])</span> 28,450 |
```

### Example 3: Batch Processing Multiple Benchmarks

```bash
# Process multiple benchmark files at once
openvm-prof \
  --json-paths app1_metrics.json,app2_metrics.json,app3_metrics.json \
  --names "Fibonacci,Sorting,Matrix Mult" \
  --output-json combined_results.json
```

**Generated Files:**
- `app1_metrics.md` - Fibonacci benchmark results
- `app2_metrics.md` - Sorting benchmark results  
- `app3_metrics.md` - Matrix multiplication results
- `combined_results.json` - Bencher format for CI/CD

## Advanced Usage Examples

### Example 4: CI/CD Integration with GitHub Actions

```yaml
# .github/workflows/benchmark.yml
name: Performance Benchmarks

on: [pull_request]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    
    - name: Run benchmarks
      run: |
        # Run your benchmarks and generate metrics.json
        cargo run --release -- benchmark --output-metrics metrics.json
        
    - name: Generate performance report
      run: |
        cargo run -p openvm-prof -- \
          --json-paths metrics.json \
          --prev-json-paths baseline_metrics.json \
          --output-json benchmark_results.json \
          summary \
          --benchmark-results-link "https://github.com/${{ github.repository }}/actions/runs/${{ github.run_id }}" \
          --summary-md-path performance_summary.md
          
    - name: Comment PR with results
      uses: actions/github-script@v7
      with:
        script: |
          const fs = require('fs');
          const summary = fs.readFileSync('performance_summary.md', 'utf8');
          github.rest.issues.createComment({
            issue_number: context.issue.number,
            owner: context.repo.owner,
            repo: context.repo.repo,
            body: summary
          });
```

### Example 5: Custom Metrics Analysis

```bash
# Analyze custom application metrics with specific naming
openvm-prof \
  --json-paths fibonacci_n30.json,fibonacci_n40.json \
  --names "Fib(30),Fib(40)" \
  --output-json fibonacci_scaling.json
```

**Input Metrics Structure:**
```json
{
  "counter": [
    {
      "labels": [["group", "fibonacci_app"], ["input_size", "30"]],
      "metric": "recursion_depth",
      "value": "30"
    }
  ],
  "gauge": [
    {
      "labels": [["group", "fibonacci_app"]],
      "metric": "execute_time_ms",
      "value": "1250.5"
    }
  ]
}
```

### Example 6: Performance Regression Detection

```bash
# Detect performance regressions with threshold alerting
openvm-prof \
  --json-paths new_metrics.json \
  --prev-json-paths baseline_metrics.json \
  --names "Performance Test" \
  summary \
  --benchmark-results-link "https://internal-ci.example.com/build/123" \
  --summary-md-path regression_report.md

# Check for regressions in the output
if grep -q "color: red" regression_report.md; then
  echo "Performance regression detected!"
  exit 1
fi
```

## Real-World Benchmark Scenarios

### Example 7: zkVM Application Performance Suite

```bash
#!/bin/bash
# benchmark_suite.sh - Comprehensive performance testing

APPS=("fibonacci" "sha256" "ecdsa" "merkle_proof")
SIZES=("small" "medium" "large")

for app in "${APPS[@]}"; do
  for size in "${SIZES[@]}"; do
    echo "Benchmarking $app with $size input..."
    
    # Generate metrics
    cargo run --release --bin $app -- \
      --input-size $size \
      --output-metrics "${app}_${size}_metrics.json"
  done
done

# Analyze all results
openvm-prof \
  --json-paths $(printf "%s_small_metrics.json," "${APPS[@]}" | sed 's/,$//') \
  --names $(printf "%s(small)," "${APPS[@]}" | sed 's/,$//') \
  --output-json small_benchmarks.json

# Compare against previous baseline
openvm-prof \
  --json-paths $(printf "%s_small_metrics.json," "${APPS[@]}" | sed 's/,$//') \
  --prev-json-paths $(printf "baseline_%s_small_metrics.json," "${APPS[@]}" | sed 's/,$//') \
  --names $(printf "%s(small)," "${APPS[@]}" | sed 's/,$//') \
  summary \
  --benchmark-results-link "https://ci.example.com" \
  --summary-md-path weekly_performance_report.md
```

### Example 8: Memory Usage Analysis

```json
// Custom metrics for memory analysis
{
  "gauge": [
    {
      "labels": [["group", "memory_analysis"], ["phase", "execution"]],
      "metric": "peak_memory_mb",
      "value": "2048.5"
    },
    {
      "labels": [["group", "memory_analysis"], ["phase", "trace_generation"]],
      "metric": "peak_memory_mb", 
      "value": "4096.2"
    },
    {
      "labels": [["group", "memory_analysis"], ["phase", "proving"]],
      "metric": "peak_memory_mb",
      "value": "8192.1"
    }
  ]
}
```

```bash
# Analyze memory usage patterns
openvm-prof \
  --json-paths memory_metrics.json \
  --names "Memory Usage Analysis"
```

## Output Format Examples

### Example 9: Bencher JSON Output

```json
{
  "fibonacci_app": {
    "total_proof_time": {
      "value": 45.32
    },
    "total_par_proof_time": {
      "value": 23.15  
    },
    "leaf": {
      "execute_time_ms": {
        "value": 1250.5
      },
      "trace_gen_time_ms": {
        "value": 2340.2
      }
    }
  }
}
```

### Example 10: GitHub Summary Format

```markdown
## 📊 Performance Summary

### ⏱️ Proof Generation Times
- **Total Serial Time**: 45.32s <span style='color: green'>(-2.1s [-4.4%])</span>
- **Total Parallel Time**: 23.15s <span style='color: green'>(-1.2s [-4.9%])</span>

### 🎯 Key Improvements
- Fibonacci execution 7.5% faster
- Memory usage reduced by 12%

### ⚠️ Performance Regressions  
- Leaf proof generation 9.4% slower (investigate memory pressure)

[📈 Full Benchmark Results](https://ci.example.com/benchmark/123)
```

## Integration Patterns

### Example 11: Rust Integration

```rust
use openvm_prof::{types::MetricDb, aggregate::GroupedMetrics};

fn analyze_benchmark_results(metrics_path: &str) -> eyre::Result<()> {
    // Load metrics
    let db = MetricDb::new(metrics_path)?;
    
    // Group by application
    let grouped = GroupedMetrics::new(&db, "group")?;
    
    // Generate statistics
    let aggregated = grouped.aggregate();
    
    // Extract key metrics
    let total_time = aggregated.total_proof_time.val;
    let parallel_time = aggregated.total_par_proof_time.val;
    
    println!("Total proof time: {:.2}s", total_time);
    println!("Parallel proof time: {:.2}s", parallel_time);
    
    // Check for performance thresholds
    if total_time > 60.0 {
        eprintln!("Warning: Proof time exceeds 60s threshold");
    }
    
    Ok(())
}
```

### Example 12: Python Analysis Script

```python
#!/usr/bin/env python3
import json
import subprocess
import sys

def run_prof_analysis(metrics_files, prev_files=None):
    """Run openvm-prof and return parsed results."""
    cmd = ["cargo", "run", "-p", "openvm-prof", "--"]
    cmd.extend(["--json-paths", ",".join(metrics_files)])
    
    if prev_files:
        cmd.extend(["--prev-json-paths", ",".join(prev_files)])
    
    cmd.extend(["--output-json", "results.json"])
    
    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode != 0:
        print(f"Error: {result.stderr}", file=sys.stderr)
        return None
    
    with open("results.json") as f:
        return json.load(f)

# Usage
metrics = ["app1.json", "app2.json"]
baseline = ["baseline1.json", "baseline2.json"]

results = run_prof_analysis(metrics, baseline)
if results:
    for app_name, app_metrics in results.items():
        total_time = app_metrics["total_proof_time"]["value"]
        print(f"{app_name}: {total_time:.2f}s")
```

## Troubleshooting Examples

### Example 13: Handling Missing Previous Metrics

```bash
# When some benchmarks are new and don't have baselines
openvm-prof \
  --json-paths new_app.json,existing_app.json \
  --prev-json-paths nonexistent.json,baseline_existing.json \
  --names "New Feature,Existing App"

# The tool gracefully handles missing baseline files
# New benchmarks will show metrics without diffs
# Existing benchmarks will show performance comparisons
```

### Example 14: Large Metrics File Processing

```bash
# For very large metrics files (>1GB), use memory-mapped processing
export RUST_LOG=info

openvm-prof \
  --json-paths huge_benchmark_metrics.json \
  --names "Large Scale Test"

# Monitor memory usage and processing time
# The tool uses memory-mapped files for efficiency
```

These examples demonstrate the flexibility and power of `openvm-prof` for various performance analysis scenarios, from simple single-file analysis to complex CI/CD integration and custom metric processing.