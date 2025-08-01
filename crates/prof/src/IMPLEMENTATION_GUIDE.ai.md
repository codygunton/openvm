# OpenVM Prof - Implementation Guide

## Adding New Metrics

### 1. Define the Metric Constant
Add to `aggregate.rs`:
```rust
pub const YOUR_METRIC_LABEL: &str = "your_metric_name_ms";
```

### 2. Include in VM_METRIC_NAMES
If it's a standard VM metric, add to the array:
```rust
pub const VM_METRIC_NAMES: &[&str] = &[
    // ... existing metrics
    YOUR_METRIC_LABEL,
];
```

### 3. Handle in Aggregations
If the metric needs special aggregation logic, modify `MetricDb::apply_aggregations()`:
```rust
pub fn apply_aggregations(&mut self) {
    for metrics in self.flat_dict.values_mut() {
        // Add your aggregation logic
        let your_metric = get(YOUR_METRIC_LABEL);
        // Process as needed
    }
}
```

## Adding New Output Formats

### 1. Create Format Structure
Define in `types.rs`:
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct YourFormat {
    // Define fields matching your format requirements
}
```

### 2. Add Conversion Method
In `aggregate.rs` or appropriate location:
```rust
impl AggregateMetrics {
    pub fn to_your_format(&self) -> YourFormat {
        // Convert aggregated metrics to your format
    }
}
```

### 3. Wire to CLI
Add command-line option in `main.rs`:
```rust
#[arg(long)]
output_your_format: Option<PathBuf>,

// In main():
if let Some(path) = args.output_your_format {
    let formatted = aggregated.to_your_format();
    fs::write(&path, serde_json::to_string_pretty(&formatted)?)?;
}
```

## Customizing Metric Groups

### 1. Modify Grouping Logic
The current implementation groups by "group" label. To customize:
```rust
// In GroupedMetrics::new()
let group_name = labels.get("your_custom_label");
```

### 2. Adjust Group Ordering
Modify `group_weight()` in `aggregate.rs`:
```rust
pub(crate) fn group_weight(name: &str) -> usize {
    // Add your custom ordering logic
    if name.contains("your_pattern") {
        return PRIORITY;
    }
    // ... existing logic
}
```

## Processing New Metric Types

### 1. Extend MetricsFile
Add to `types.rs`:
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct MetricsFile {
    #[serde(default)]
    pub counter: Vec<MetricEntry>,
    #[serde(default)]
    pub gauge: Vec<MetricEntry>,
    #[serde(default)]
    pub histogram: Vec<HistogramEntry>, // New type
}
```

### 2. Process in MetricDb::new()
```rust
// Process histograms
for entry in metrics.histogram {
    // Extract percentiles or other statistics
    let labels = Labels::from(entry.labels);
    db.add_to_flat_dict(labels, format!("{}_p99", entry.metric), entry.p99);
}
```

## Optimizing Performance

### 1. Large File Handling
Current implementation uses memory-mapped files:
```rust
let mmap = unsafe { Mmap::map(&file)? };
```

For extremely large files, consider streaming:
```rust
use serde_json::Deserializer;
let stream = Deserializer::from_reader(file).into_iter::<MetricEntry>();
```

### 2. Parallel Processing
For multiple files:
```rust
use rayon::prelude::*;

let results: Vec<_> = paths
    .par_iter()
    .map(|path| MetricDb::new(path))
    .collect::<Result<Vec<_>>>()?;
```

## Custom Visualizations

### 1. Add Visualization Method
```rust
impl MetricDb {
    pub fn generate_custom_output(&self) -> String {
        // Create custom visualization
        // Access self.dict_by_label_types for organized data
    }
}
```

### 2. Special Formatting
For specific metric types:
```rust
impl MetricDb {
    pub fn format_duration(ms: f64) -> String {
        if ms > 1000.0 {
            format!("{:.2}s", ms / 1000.0)
        } else {
            format!("{:.0}ms", ms)
        }
    }
}
```

## Error Handling Best Practices

### 1. Graceful Degradation
When metrics are missing:
```rust
let metric = metrics.get(METRIC_NAME).unwrap_or(&default_value);
```

### 2. Validation
Add validation for critical metrics:
```rust
if !self.by_group.values().any(|m| m.contains_key(PROOF_TIME_LABEL)) {
    return Err(eyre!("No proof time metrics found"));
}
```

## Testing Strategies

### 1. Unit Tests
Test metric aggregation:
```rust
#[test]
fn test_aggregation() {
    let mut db = MetricDb::default();
    db.add_to_flat_dict(
        Labels::from(vec![["group".into(), "test".into()]]),
        "metric".into(),
        100.0
    );
    db.apply_aggregations();
    // Assert expected results
}
```

### 2. Integration Tests
Test full pipeline:
```rust
#[test]
fn test_full_processing() {
    let temp_file = create_test_metrics_file();
    let db = MetricDb::new(&temp_file).unwrap();
    let grouped = GroupedMetrics::new(&db, "group").unwrap();
    let aggregated = grouped.aggregate();
    // Verify output format
}
```