# OpenVM Prof Integration Guide

## Overview

This guide covers how to integrate `openvm-prof` into various development workflows, CI/CD pipelines, and monitoring systems for comprehensive zkVM performance analysis.

## Core Integration Patterns

### 1. Development Workflow Integration

#### Local Development Setup
```bash
# Add to your ~/.bashrc or ~/.zshrc
alias ovprof='cargo run -p openvm-prof --'
alias ovprof-compare='ovprof --json-paths latest.json --prev-json-paths baseline.json'

# Set up local benchmark directory structure
mkdir -p benchmarks/{current,baseline,reports}
```

#### Pre-commit Hook Integration
```bash
#!/bin/sh
# .git/hooks/pre-commit
# Run performance benchmarks before commits

echo "Running performance benchmarks..."

# Run critical benchmarks
cargo run --release --bin critical_benchmarks -- --output-metrics benchmarks/current/pre_commit.json

# Compare against baseline
if [ -f "benchmarks/baseline/pre_commit.json" ]; then
    cargo run -p openvm-prof -- \
        --json-paths benchmarks/current/pre_commit.json \
        --prev-json-paths benchmarks/baseline/pre_commit.json \
        --names "Pre-commit Check" \
        summary --benchmark-results-link "local" --summary-md-path /dev/stdout
    
    # Optional: Fail commit if major regression detected
    if cargo run -p openvm-prof -- \
        --json-paths benchmarks/current/pre_commit.json \
        --prev-json-paths benchmarks/baseline/pre_commit.json 2>&1 | grep -q "color: red"; then
        echo "Performance regression detected. Commit blocked."
        echo "Run 'git commit --no-verify' to bypass this check."
        exit 1
    fi
fi
```

### 2. CI/CD Pipeline Integration

#### GitHub Actions Integration

```yaml
# .github/workflows/performance.yml
name: Performance Benchmarks

on:
  pull_request:
    branches: [main]
  push:
    branches: [main]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    timeout-minutes: 60
    
    steps:
    - uses: actions/checkout@v4
      with:
        fetch-depth: 0  # Needed for baseline comparison
        
    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        override: true
        
    - name: Cache dependencies
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target/
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
        
    - name: Build benchmark binaries
      run: cargo build --release --bin benchmark_suite
      
    - name: Download baseline metrics
      if: github.event_name == 'pull_request'
      run: |
        # Download baseline from main branch artifact
        gh run download --name benchmark-results --repo ${{ github.repository }} || true
      env:
        GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        
    - name: Run benchmarks
      run: |
        mkdir -p benchmark_results
        ./target/release/benchmark_suite --output-dir benchmark_results
        
    - name: Generate performance report
      run: |
        # Find all generated metrics files
        METRICS_FILES=$(find benchmark_results -name "*.json" | tr '\n' ',' | sed 's/,$//')
        BASELINE_FILES=""
        
        if [ -d "baseline_results" ]; then
          BASELINE_FILES=$(find baseline_results -name "*.json" | tr '\n' ',' | sed 's/,$//')
          BASELINE_ARGS="--prev-json-paths $BASELINE_FILES"
        fi
        
        cargo run -p openvm-prof -- \
          --json-paths "$METRICS_FILES" \
          $BASELINE_ARGS \
          --output-json benchmark_output.json \
          summary \
          --benchmark-results-link "${{ github.server_url }}/${{ github.repository }}/actions/runs/${{ github.run_id }}" \
          --summary-md-path performance_summary.md
          
    - name: Upload benchmark artifacts
      uses: actions/upload-artifact@v3
      with:
        name: benchmark-results
        path: |
          benchmark_results/
          benchmark_output.json
          performance_summary.md
        retention-days: 30
        
    - name: Comment PR with results
      if: github.event_name == 'pull_request'
      uses: actions/github-script@v7
      with:
        script: |
          const fs = require('fs');
          
          if (fs.existsSync('performance_summary.md')) {
            const summary = fs.readFileSync('performance_summary.md', 'utf8');
            
            // Find existing performance comment
            const comments = await github.rest.issues.listComments({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
            });
            
            const existingComment = comments.data.find(comment => 
              comment.body.includes('📊 Performance Summary')
            );
            
            if (existingComment) {
              // Update existing comment
              await github.rest.issues.updateComment({
                comment_id: existingComment.id,
                owner: context.repo.owner,
                repo: context.repo.repo,
                body: summary
              });
            } else {
              // Create new comment
              await github.rest.issues.createComment({
                issue_number: context.issue.number,
                owner: context.repo.owner,
                repo: context.repo.repo,
                body: summary
              });
            }
          }
```

#### GitLab CI Integration

```yaml
# .gitlab-ci.yml
stages:
  - build
  - benchmark
  - report

variables:
  CARGO_HOME: ${CI_PROJECT_DIR}/.cargo

benchmark:
  stage: benchmark
  image: rust:latest
  cache:
    paths:
      - .cargo/
      - target/
  script:
    - cargo build --release --bin benchmark_suite
    - mkdir -p benchmark_results
    - ./target/release/benchmark_suite --output-dir benchmark_results
    
    # Download baseline from previous successful pipeline
    - |
      if [ "$CI_COMMIT_REF_NAME" != "main" ]; then
        curl -f "${CI_API_V4_URL}/projects/${CI_PROJECT_ID}/jobs/artifacts/main/download?job=benchmark" \
          -H "PRIVATE-TOKEN: ${CI_JOB_TOKEN}" \
          -o baseline.zip || true
        if [ -f baseline.zip ]; then
          unzip -o baseline.zip
          mv benchmark_results baseline_results
          mkdir benchmark_results
          ./target/release/benchmark_suite --output-dir benchmark_results
        fi
      fi
      
    - |
      METRICS_FILES=$(find benchmark_results -name "*.json" | tr '\n' ',' | sed 's/,$//')
      BASELINE_ARGS=""
      if [ -d "baseline_results" ]; then
        BASELINE_FILES=$(find baseline_results -name "*.json" | tr '\n' ',' | sed 's/,$//')
        BASELINE_ARGS="--prev-json-paths $BASELINE_FILES"
      fi
      
      cargo run -p openvm-prof -- \
        --json-paths "$METRICS_FILES" \
        $BASELINE_ARGS \
        --output-json benchmark_output.json \
        summary \
        --benchmark-results-link "${CI_PIPELINE_URL}" \
        --summary-md-path performance_summary.md
        
  artifacts:
    name: "benchmark-results-${CI_COMMIT_SHORT_SHA}"
    paths:
      - benchmark_results/
      - benchmark_output.json
      - performance_summary.md
    expire_in: 1 week
    reports:
      junit: benchmark_results/junit.xml  # If you generate JUnit format
  only:
    - merge_requests
    - main
```

### 3. Monitoring and Alerting Integration

#### Prometheus/Grafana Integration

```rust
// src/monitoring.rs
use openvm_prof::{types::MetricDb, aggregate::GroupedMetrics};
use prometheus::{Gauge, GaugeVec, register_gauge, register_gauge_vec};

pub struct PrometheusExporter {
    total_proof_time: Gauge,
    proof_time_by_group: GaugeVec,
}

impl PrometheusExporter {
    pub fn new() -> prometheus::Result<Self> {
        Ok(Self {
            total_proof_time: register_gauge!(
                "openvm_total_proof_time_seconds",
                "Total proof generation time in seconds"
            )?,
            proof_time_by_group: register_gauge_vec!(
                "openvm_proof_time_by_group_seconds",
                "Proof generation time by group in seconds",
                &["group", "benchmark"]
            )?,
        })
    }
    
    pub fn export_metrics(&self, metrics_path: &str, benchmark_name: &str) -> eyre::Result<()> {
        let db = MetricDb::new(metrics_path)?;
        let grouped = GroupedMetrics::new(&db, "group")?;
        let aggregated = grouped.aggregate();
        
        // Export total time
        self.total_proof_time.set(aggregated.total_proof_time.val);
        
        // Export per-group times
        for (group_name, metrics) in &aggregated.by_group {
            if let Some(stats) = metrics.get("total_proof_time_ms") {
                self.proof_time_by_group
                    .with_label_values(&[group_name, benchmark_name])
                    .set(stats.sum.val / 1000.0);  // Convert ms to seconds
            }
        }
        
        Ok(())
    }
}
```

#### Slack/Discord Alerting

```python
#!/usr/bin/env python3
# scripts/alert_on_regression.py
import json
import subprocess
import sys
import requests
from typing import Dict, Any

def analyze_performance(current_metrics: str, baseline_metrics: str) -> Dict[str, Any]:
    """Run openvm-prof and return analysis results."""
    cmd = [
        "cargo", "run", "-p", "openvm-prof", "--",
        "--json-paths", current_metrics,
        "--prev-json-paths", baseline_metrics,
        "--output-json", "analysis.json"
    ]
    
    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode != 0:
        raise Exception(f"Analysis failed: {result.stderr}")
    
    with open("analysis.json") as f:
        return json.load(f)

def send_slack_alert(webhook_url: str, message: str):
    """Send alert to Slack."""
    payload = {
        "text": "🚨 Performance Regression Detected",
        "blocks": [
            {
                "type": "section",
                "text": {
                    "type": "mrkdwn",
                    "text": message
                }
            }
        ]
    }
    
    response = requests.post(webhook_url, json=payload)
    response.raise_for_status()

def check_regression_threshold(results: Dict[str, Any], threshold: float = 0.1) -> bool:
    """Check if any benchmark exceeded regression threshold."""
    for app_name, metrics in results.items():
        if "total_proof_time" in metrics:
            current_time = metrics["total_proof_time"]["value"]
            # Estimate baseline from diff if available
            if hasattr(metrics["total_proof_time"], "diff"):
                baseline_time = current_time - metrics["total_proof_time"]["diff"]
                regression_percent = (current_time - baseline_time) / baseline_time
                
                if regression_percent > threshold:
                    return True, f"{app_name}: {regression_percent:.1%} slower"
    
    return False, None

if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("Usage: alert_on_regression.py <current_metrics.json> <baseline_metrics.json>")
        sys.exit(1)
    
    current, baseline = sys.argv[1], sys.argv[2]
    webhook_url = os.environ.get("SLACK_WEBHOOK_URL")
    
    try:
        results = analyze_performance(current, baseline)
        has_regression, details = check_regression_threshold(results)
        
        if has_regression and webhook_url:
            message = f"Performance regression detected: {details}"
            send_slack_alert(webhook_url, message)
            print("Alert sent to Slack")
        elif has_regression:
            print(f"Regression detected: {details}")
            sys.exit(1)
        else:
            print("No significant performance regressions detected")
            
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)
```

### 4. IDE and Editor Integration

#### VS Code Integration

```json
// .vscode/tasks.json
{
    "version": "2.0.0",
    "tasks": [
        {
            "label": "Benchmark Current",
            "type": "shell",
            "command": "cargo",
            "args": [
                "run", "--release", "--bin", "benchmark_suite", "--",
                "--output-metrics", "current_benchmark.json"
            ],
            "group": "build",
            "presentation": {
                "echo": true,
                "reveal": "always",
                "focus": false,
                "panel": "shared"
            }
        },
        {
            "label": "Analyze Performance",
            "type": "shell",
            "command": "cargo",
            "args": [
                "run", "-p", "openvm-prof", "--",
                "--json-paths", "current_benchmark.json",
                "--prev-json-paths", "baseline_benchmark.json",
                "--names", "Current vs Baseline"
            ],
            "group": "build",
            "dependsOn": "Benchmark Current",
            "presentation": {
                "echo": true,
                "reveal": "always",
                "focus": true,
                "panel": "shared"
            }
        }
    ]
}
```

#### Vim/Neovim Integration

```lua
-- .config/nvim/lua/openvm.lua
local M = {}

function M.run_benchmark()
    vim.cmd('!cargo run --release --bin benchmark_suite -- --output-metrics current.json')
end

function M.analyze_performance()
    local baseline_exists = vim.fn.filereadable('baseline.json') == 1
    local cmd = 'cargo run -p openvm-prof -- --json-paths current.json'
    
    if baseline_exists then
        cmd = cmd .. ' --prev-json-paths baseline.json'
    end
    
    vim.cmd('!' .. cmd)
end

function M.set_baseline()
    vim.cmd('!cp current.json baseline.json')
    print("Baseline updated")
end

-- Key mappings
vim.keymap.set('n', '<leader>br', M.run_benchmark, { desc = 'Run benchmark' })
vim.keymap.set('n', '<leader>ba', M.analyze_performance, { desc = 'Analyze performance' })
vim.keymap.set('n', '<leader>bs', M.set_baseline, { desc = 'Set baseline' })

return M
```

### 5. Database Integration

#### PostgreSQL Schema for Metrics Storage

```sql
-- migrations/001_create_benchmark_tables.sql
CREATE TABLE benchmark_runs (
    id SERIAL PRIMARY KEY,
    commit_hash VARCHAR(40) NOT NULL,
    branch_name VARCHAR(255) NOT NULL,
    run_timestamp TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    total_proof_time_ms DECIMAL(12,3),
    total_parallel_proof_time_ms DECIMAL(12,3),
    metadata JSONB
);

CREATE TABLE benchmark_groups (
    id SERIAL PRIMARY KEY,
    run_id INTEGER REFERENCES benchmark_runs(id),
    group_name VARCHAR(255) NOT NULL,
    metrics JSONB NOT NULL
);

CREATE INDEX idx_benchmark_runs_commit ON benchmark_runs(commit_hash);
CREATE INDEX idx_benchmark_runs_timestamp ON benchmark_runs(run_timestamp);
CREATE INDEX idx_benchmark_groups_run_group ON benchmark_groups(run_id, group_name);
```

```rust
// src/database.rs
use sqlx::{PgPool, Row};
use serde_json::Value;
use crate::aggregate::AggregateMetrics;

pub struct BenchmarkDb {
    pool: PgPool,
}

impl BenchmarkDb {
    pub async fn new(database_url: &str) -> sqlx::Result<Self> {
        let pool = PgPool::connect(database_url).await?;
        Ok(Self { pool })
    }
    
    pub async fn store_benchmark(
        &self,
        commit_hash: &str,
        branch: &str,
        metrics: &AggregateMetrics,
    ) -> sqlx::Result<i32> {
        let run_id: i32 = sqlx::query!(
            "INSERT INTO benchmark_runs (commit_hash, branch_name, total_proof_time_ms, total_parallel_proof_time_ms) 
             VALUES ($1, $2, $3, $4) RETURNING id",
            commit_hash,
            branch,
            metrics.total_proof_time.val as f64 * 1000.0,
            metrics.total_par_proof_time.val as f64 * 1000.0
        )
        .fetch_one(&self.pool)
        .await?
        .id;
        
        for (group_name, group_metrics) in &metrics.by_group {
            let metrics_json = serde_json::to_value(group_metrics)?;
            sqlx::query!(
                "INSERT INTO benchmark_groups (run_id, group_name, metrics) VALUES ($1, $2, $3)",
                run_id,
                group_name,
                metrics_json
            )
            .execute(&self.pool)
            .await?;
        }
        
        Ok(run_id)
    }
    
    pub async fn get_baseline_for_branch(&self, branch: &str) -> sqlx::Result<Option<AggregateMetrics>> {
        // Implementation to retrieve baseline metrics
        todo!()
    }
}
```

### 6. Custom Metrics Collection

#### Application Instrumentation

```rust
// src/instrumentation.rs
use std::time::Instant;
use serde_json::{json, Value};
use std::collections::HashMap;

pub struct MetricsCollector {
    metrics: HashMap<String, Value>,
    timers: HashMap<String, Instant>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
            timers: HashMap::new(),
        }
    }
    
    pub fn start_timer(&mut self, name: &str) {
        self.timers.insert(name.to_string(), Instant::now());
    }
    
    pub fn end_timer(&mut self, name: &str, labels: Vec<[String; 2]>) {
        if let Some(start_time) = self.timers.remove(name) {
            let duration_ms = start_time.elapsed().as_millis() as f64;
            self.record_gauge(name, duration_ms, labels);
        }
    }
    
    pub fn record_counter(&mut self, name: &str, value: f64, labels: Vec<[String; 2]>) {
        let entry = json!({
            "labels": labels,
            "metric": name,
            "value": value.to_string()
        });
        
        self.metrics
            .entry("counter".to_string())
            .or_insert_with(|| json!([]))
            .as_array_mut()
            .unwrap()
            .push(entry);
    }
    
    pub fn record_gauge(&mut self, name: &str, value: f64, labels: Vec<[String; 2]>) {
        let entry = json!({
            "labels": labels,
            "metric": name,
            "value": value.to_string()
        });
        
        self.metrics
            .entry("gauge".to_string())
            .or_insert_with(|| json!([]))
            .as_array_mut()
            .unwrap()
            .push(entry);
    }
    
    pub fn export_to_file(&self, path: &str) -> std::io::Result<()> {
        std::fs::write(path, serde_json::to_string_pretty(&self.metrics)?)
    }
}

// Usage example in your application
pub fn benchmark_fibonacci(n: u32, collector: &mut MetricsCollector) -> u64 {
    let labels = vec![
        ["group".to_string(), "fibonacci_app".to_string()],
        ["input_size".to_string(), n.to_string()],
    ];
    
    collector.start_timer("execute_time_ms");
    
    let result = fibonacci_recursive(n);
    
    collector.end_timer("execute_time_ms", labels.clone());
    collector.record_counter("total_calculations", n as f64, labels);
    
    result
}
```

## Best Practices

### 1. Baseline Management
- Store baselines in version control for reproducibility
- Update baselines regularly but deliberately
- Use multiple baseline strategies (daily, release, feature-branch)

### 2. Metrics Naming Conventions
- Use consistent suffixes (`_ms` for milliseconds, `_bytes` for memory)
- Include meaningful labels for grouping and filtering
- Separate application metrics from system metrics

### 3. Alerting Thresholds
- Set different thresholds for different metric types
- Account for natural variance in performance measurements  
- Use percentage-based thresholds rather than absolute values

### 4. Performance Analysis Workflow
1. Run benchmarks consistently (same hardware, clean environment)
2. Compare against multiple baselines (last commit, last release, long-term trend)
3. Investigate regressions promptly with detailed profiling
4. Document performance characteristics for future reference

This integration guide provides the foundation for incorporating `openvm-prof` into your development and deployment workflows, enabling continuous performance monitoring and regression detection.