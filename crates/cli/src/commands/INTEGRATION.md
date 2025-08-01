# OpenVM CLI Commands - Integration Patterns

This document outlines integration patterns and guidelines for incorporating OpenVM CLI commands into larger systems, workflows, and development environments.

## Architecture Integration Patterns

### 1. Command Pattern Implementation

The OpenVM CLI follows a consistent command pattern that can be integrated into larger applications:

```rust
// Integration trait for command execution
pub trait CommandExecutor {
    fn execute(&self) -> eyre::Result<CommandResult>;
    fn validate_args(&self) -> eyre::Result<()>;
    fn cleanup(&self) -> eyre::Result<()>;
}

// Example integration wrapper
pub struct OpenVMCommandExecutor<T: CommandExecutor> {
    command: T,
    context: ExecutionContext,
}

impl<T: CommandExecutor> OpenVMCommandExecutor<T> {
    pub fn new(command: T, context: ExecutionContext) -> Self {
        Self { command, context }
    }
    
    pub async fn execute_with_monitoring(&self) -> eyre::Result<CommandResult> {
        // Pre-execution hooks
        self.context.start_monitoring()?;
        
        // Validate before execution
        self.command.validate_args()?;
        
        // Execute with timeout and monitoring
        let result = tokio::time::timeout(
            self.context.timeout,
            self.command.execute()
        ).await??;
        
        // Post-execution cleanup
        self.command.cleanup()?;
        self.context.stop_monitoring()?;
        
        Ok(result)
    }
}
```

### 2. Plugin Architecture Integration

Commands can be extended through a plugin system:

```rust
// Plugin trait for command extensions
pub trait CommandPlugin {
    fn name(&self) -> &str;
    fn before_execute(&self, cmd: &dyn CommandExecutor) -> eyre::Result<()>;
    fn after_execute(&self, cmd: &dyn CommandExecutor, result: &CommandResult) -> eyre::Result<()>;
    fn on_error(&self, cmd: &dyn CommandExecutor, error: &eyre::Error) -> eyre::Result<()>;
}

// Plugin manager
pub struct PluginManager {
    plugins: Vec<Box<dyn CommandPlugin>>,
}

impl PluginManager {
    pub fn register_plugin(&mut self, plugin: Box<dyn CommandPlugin>) {
        self.plugins.push(plugin);
    }
    
    pub fn execute_with_plugins<T: CommandExecutor>(&self, cmd: &T) -> eyre::Result<CommandResult> {
        // Execute before hooks
        for plugin in &self.plugins {
            plugin.before_execute(cmd)?;
        }
        
        // Execute command
        let result = match cmd.execute() {
            Ok(result) => {
                // Execute after hooks
                for plugin in &self.plugins {
                    plugin.after_execute(cmd, &result)?;
                }
                result
            },
            Err(error) => {
                // Execute error hooks
                for plugin in &self.plugins {
                    plugin.on_error(cmd, &error)?;
                }
                return Err(error);
            }
        };
        
        Ok(result)
    }
}
```

## Build System Integration

### 3. Cargo Integration Patterns

#### Custom Cargo Commands
```rust
// Integration with cargo as a subcommand
// Cargo.toml
[dependencies]
cargo = "0.75"
clap = { version = "4.0", features = ["derive"] }

// src/main.rs
use cargo::ops::{compile, CompileOptions};
use clap::Parser;

#[derive(Parser)]
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
enum CargoCli {
    #[command(name = "openvm")]
    OpenVM(OpenVMArgs),
}

#[derive(Parser)]
struct OpenVMArgs {
    #[command(subcommand)]
    command: OpenVMCommand,
}

#[derive(Parser)]
enum OpenVMCommand {
    Build(BuildCmd),
    Prove(ProveCmd),
    Verify(VerifyCmd),
}

fn main() -> eyre::Result<()> {
    let CargoCli::OpenVM(args) = CargoCli::parse();
    
    match args.command {
        OpenVMCommand::Build(cmd) => cmd.run(),
        OpenVMCommand::Prove(cmd) => cmd.run(),
        OpenVMCommand::Verify(cmd) => cmd.run(),
    }
}
```

#### Build Script Integration
```rust
// build.rs
use std::process::Command;

fn main() {
    // Integration with cargo build scripts
    if std::env::var("CARGO_FEATURE_OPENVM").is_ok() {
        let output = Command::new("openvm")
            .args(&["build", "--no-transpile"])
            .output()
            .expect("Failed to execute openvm build");
            
        if !output.status.success() {
            panic!("OpenVM build failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        // Tell cargo to rerun if OpenVM config changes
        println!("cargo:rerun-if-changed=openvm.toml");
        println!("cargo:rerun-if-changed=src/lib.rs");
    }
}
```

### 4. Make Integration
```makefile
# Makefile integration
OPENVM := openvm
CARGO := cargo

.PHONY: build prove verify clean setup

# Default target
all: build prove verify

# Build OpenVM program
build:
	$(OPENVM) build --release

# Generate keys if they don't exist
keys:
	@if [ ! -f app.pk ] || [ ! -f app.vk ]; then \
		$(OPENVM) keygen; \
	fi

# Generate proof
prove: build keys
	$(OPENVM) prove --output proof.bin

# Verify proof
verify: prove
	$(OPENVM) verify --proof proof.bin

# Setup development environment
setup:
	$(CARGO) install --path crates/cli
	$(OPENVM) init --template default

# Clean build artifacts
clean:
	$(CARGO) clean
	rm -rf target/openvm/
	rm -f *.pk *.vk *.proof *.vmexe

# Development workflow
dev: build
	$(OPENVM) run --debug

# Production deployment
deploy: build keys
	$(OPENVM) prove --output production.proof
	$(OPENVM) verify --proof production.proof
	@echo "Ready for deployment"

# Testing
test: build
	$(CARGO) test
	$(OPENVM) run --test
```

## CI/CD Integration Patterns

### 5. GitHub Actions Integration
```yaml
# .github/workflows/openvm.yml
name: OpenVM Build and Prove

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  build-and-prove:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        rust-version: [1.82, stable]
        
    steps:
    - uses: actions/checkout@v4
    
    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: ${{ matrix.rust-version }}
        override: true
        components: rustfmt, clippy
    
    - name: Cache Cargo dependencies
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target/
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
    
    - name: Install OpenVM CLI
      run: cargo install --path crates/cli
    
    - name: Check formatting
      run: cargo fmt --all -- --check
    
    - name: Run clippy
      run: cargo clippy --all-features -- -D warnings
    
    - name: Build OpenVM program
      run: openvm build --release
    
    - name: Generate keys
      run: openvm keygen
    
    - name: Run tests
      run: |
        cargo test --all
        openvm run --test
    
    - name: Generate proof
      run: openvm prove --output proof.bin
    
    - name: Verify proof
      run: openvm verify --proof proof.bin
    
    - name: Upload artifacts
      uses: actions/upload-artifact@v3
      with:
        name: openvm-artifacts-${{ matrix.rust-version }}
        path: |
          target/openvm/release/*.vmexe
          *.pk
          *.vk
          proof.bin
```

### 6. GitLab CI Integration
```yaml
# .gitlab-ci.yml
stages:
  - build
  - test
  - prove
  - verify
  - deploy

variables:
  CARGO_HOME: "${CI_PROJECT_DIR}/.cargo"
  RUST_VERSION: "1.82"

cache:
  paths:
    - .cargo/
    - target/

before_script:
  - curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  - source ~/.cargo/env
  - rustup toolchain install $RUST_VERSION
  - rustup default $RUST_VERSION
  - cargo install --path crates/cli

build:
  stage: build
  script:
    - cargo build --release
    - openvm build --release
  artifacts:
    paths:
      - target/openvm/release/
    expire_in: 1 hour

test:
  stage: test
  script:
    - cargo test --all
    - openvm run --test
  dependencies:
    - build

keygen:
  stage: test
  script:
    - openvm keygen
  artifacts:
    paths:
      - "*.pk"
      - "*.vk"
    expire_in: 1 hour
  dependencies:
    - build

prove:
  stage: prove
  script:
    - openvm prove --output proof.bin
  artifacts:
    paths:
      - proof.bin
    expire_in: 1 hour
  dependencies:
    - build
    - keygen

verify:
  stage: verify
  script:
    - openvm verify --proof proof.bin
  dependencies:
    - prove

deploy:
  stage: deploy
  script:
    - echo "Deploying OpenVM artifacts"
    - # Upload to deployment target
  only:
    - main
  dependencies:
    - verify
```

## Development Environment Integration

### 7. VS Code Integration
```json
// .vscode/tasks.json
{
    "version": "2.0.0",
    "tasks": [
        {
            "label": "OpenVM: Build",
            "type": "shell",
            "command": "openvm",
            "args": ["build"],
            "group": {
                "kind": "build",
                "isDefault": true
            },
            "presentation": {
                "echo": true,
                "reveal": "always",
                "focus": false,
                "panel": "shared"
            },
            "problemMatcher": ["$rustc"]
        },
        {
            "label": "OpenVM: Build and Run",
            "type": "shell",
            "command": "openvm",
            "args": ["build", "&&", "openvm", "run"],
            "group": "build",
            "dependsOn": "OpenVM: Build"
        },
        {
            "label": "OpenVM: Generate Keys",
            "type": "shell",
            "command": "openvm",
            "args": ["keygen"],
            "group": "build"
        },
        {
            "label": "OpenVM: Full Workflow",
            "dependsOrder": "sequence",
            "dependsOn": [
                "OpenVM: Build",
                "OpenVM: Generate Keys",
                "OpenVM: Prove",
                "OpenVM: Verify"
            ]
        },
        {
            "label": "OpenVM: Prove",
            "type": "shell",
            "command": "openvm",
            "args": ["prove"],
            "group": "test"
        },
        {
            "label": "OpenVM: Verify",
            "type": "shell",
            "command": "openvm",
            "args": ["verify"],
            "group": "test"
        }
    ]
}
```

```json
// .vscode/launch.json
{
    "version": "0.2.0",
    "configurations": [
        {
            "name": "Debug OpenVM Build",
            "type": "lldb",
            "request": "launch",
            "program": "${workspaceFolder}/target/debug/openvm",
            "args": ["build", "--verbose"],
            "cwd": "${workspaceFolder}",
            "sourceLanguages": ["rust"]
        },
        {
            "name": "Debug OpenVM Run",
            "type": "lldb",
            "request": "launch",
            "program": "${workspaceFolder}/target/debug/openvm",
            "args": ["run", "--debug"],
            "cwd": "${workspaceFolder}",
            "sourceLanguages": ["rust"]
        }
    ]
}
```

### 8. Docker Integration Patterns

#### Development Container
```dockerfile
# Dockerfile.dev
FROM rust:1.82

# Install system dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Install OpenVM CLI
COPY . /openvm
WORKDIR /openvm
RUN cargo install --path crates/cli

# Setup development environment
WORKDIR /workspace
VOLUME ["/workspace"]

ENTRYPOINT ["openvm"]
```

#### Production Container
```dockerfile
# Dockerfile.prod
FROM rust:1.82-slim as builder

WORKDIR /app
COPY . .

# Build OpenVM CLI
RUN cargo build --release --bin openvm

# Build the program
RUN ./target/release/openvm build --release

FROM ubuntu:22.04

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy OpenVM binary and artifacts
COPY --from=builder /app/target/release/openvm /usr/local/bin/
COPY --from=builder /app/target/openvm/release/ /app/

WORKDIR /app

# Generate keys
RUN openvm keygen

CMD ["openvm", "prove"]
```

#### Docker Compose Integration
```yaml
# docker-compose.yml
version: '3.8'

services:
  openvm-dev:
    build:
      context: .
      dockerfile: Dockerfile.dev
    volumes:
      - .:/workspace
      - cargo-cache:/usr/local/cargo/registry
    working_dir: /workspace
    command: ["build", "--watch"]
    
  openvm-prover:
    build:
      context: .
      dockerfile: Dockerfile.prod
    volumes:
      - ./proofs:/app/proofs
    environment:
      - RUST_LOG=info
    command: ["prove", "--output-dir", "/app/proofs"]
    
  openvm-verifier:
    build:
      context: .
      dockerfile: Dockerfile.prod
    volumes:
      - ./proofs:/app/proofs
    command: ["verify", "--proof-dir", "/app/proofs"]
    depends_on:
      - openvm-prover

volumes:
  cargo-cache:
```

## Web Service Integration

### 9. REST API Integration
```rust
// Web service integration
use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio::process::Command;

#[derive(Debug, Deserialize)]
struct BuildRequest {
    config: Option<String>,
    release: Option<bool>,
}

#[derive(Debug, Serialize)]
struct BuildResponse {
    success: bool,
    output_path: Option<String>,
    error: Option<String>,
}

async fn build_handler(
    State(app_state): State<AppState>,
    Json(request): Json<BuildRequest>,
) -> Result<Json<BuildResponse>, StatusCode> {
    let mut cmd = Command::new("openvm");
    cmd.arg("build");
    
    if request.release.unwrap_or(false) {
        cmd.arg("--release");
    }
    
    if let Some(config) = request.config {
        cmd.args(&["--config", &config]);
    }
    
    match cmd.output().await {
        Ok(output) => {
            if output.status.success() {
                Ok(Json(BuildResponse {
                    success: true,
                    output_path: Some("target/openvm/release/program.vmexe".to_string()),
                    error: None,
                }))
            } else {
                Ok(Json(BuildResponse {
                    success: false,
                    output_path: None,
                    error: Some(String::from_utf8_lossy(&output.stderr).to_string()),
                }))
            }
        }
        Err(e) => Ok(Json(BuildResponse {
            success: false,
            output_path: None,
            error: Some(e.to_string()),
        })),
    }
}

pub fn create_app() -> Router {
    Router::new()
        .route("/build", axum::routing::post(build_handler))
        .route("/prove", axum::routing::post(prove_handler))
        .route("/verify", axum::routing::post(verify_handler))
}
```

### 10. GraphQL Integration
```rust
// GraphQL integration
use async_graphql::{Context, Object, Result, Schema};
use tokio::process::Command;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn build_status(&self, ctx: &Context<'_>) -> Result<BuildStatus> {
        // Check if build artifacts exist
        let vmexe_exists = tokio::fs::metadata("target/openvm/release/program.vmexe")
            .await
            .is_ok();
            
        Ok(BuildStatus {
            built: vmexe_exists,
            last_build: None, // Could track timestamps
        })
    }
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn build_program(&self, config: Option<String>) -> Result<BuildResult> {
        let mut cmd = Command::new("openvm");
        cmd.arg("build");
        
        if let Some(config_path) = config {
            cmd.args(&["--config", &config_path]);
        }
        
        let output = cmd.output().await?;
        
        Ok(BuildResult {
            success: output.status.success(),
            message: if output.status.success() {
                "Build completed successfully".to_string()
            } else {
                String::from_utf8_lossy(&output.stderr).to_string()
            },
        })
    }
    
    async fn generate_proof(&self) -> Result<ProofResult> {
        let output = Command::new("openvm")
            .args(&["prove"])
            .output()
            .await?;
            
        Ok(ProofResult {
            success: output.status.success(),
            proof_path: if output.status.success() {
                Some("proof.bin".to_string())
            } else {
                None
            },
        })
    }
}

pub type AppSchema = Schema<QueryRoot, MutationRoot, async_graphql::EmptySubscription>;
```

## Monitoring and Observability Integration

### 11. Metrics Integration
```rust
// Prometheus metrics integration
use prometheus::{Counter, Histogram, Registry};
use std::time::Instant;

pub struct CommandMetrics {
    pub build_counter: Counter,
    pub prove_counter: Counter,
    pub verify_counter: Counter,
    pub build_duration: Histogram,
    pub prove_duration: Histogram,
    pub verify_duration: Histogram,
}

impl CommandMetrics {
    pub fn new(registry: &Registry) -> eyre::Result<Self> {
        let build_counter = Counter::new("openvm_builds_total", "Total number of builds")?;
        let prove_counter = Counter::new("openvm_proofs_total", "Total number of proofs generated")?;
        let verify_counter = Counter::new("openvm_verifications_total", "Total number of verifications")?;
        
        let build_duration = Histogram::new("openvm_build_duration_seconds", "Build duration in seconds")?;
        let prove_duration = Histogram::new("openvm_prove_duration_seconds", "Proof generation duration in seconds")?;
        let verify_duration = Histogram::new("openvm_verify_duration_seconds", "Verification duration in seconds")?;
        
        registry.register(Box::new(build_counter.clone()))?;
        registry.register(Box::new(prove_counter.clone()))?;
        registry.register(Box::new(verify_counter.clone()))?;
        registry.register(Box::new(build_duration.clone()))?;
        registry.register(Box::new(prove_duration.clone()))?;
        registry.register(Box::new(verify_duration.clone()))?;
        
        Ok(Self {
            build_counter,
            prove_counter,
            verify_counter,
            build_duration,
            prove_duration,
            verify_duration,
        })
    }
    
    pub fn record_build<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = f();
        self.build_duration.observe(start.elapsed().as_secs_f64());
        self.build_counter.inc();
        result
    }
}
```

### 12. Logging Integration
```rust
// Structured logging integration
use tracing::{info, error, span, Level};
use tracing_subscriber::fmt::format::FmtSpan;

pub fn setup_logging() -> eyre::Result<()> {
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .with_level(true)
        .with_target(true)
        .json()
        .init();
    
    Ok(())
}

// Enhanced command execution with tracing
impl BuildCmd {
    #[tracing::instrument(skip(self))]
    pub fn run(&self) -> Result<()> {
        let span = span!(Level::INFO, "openvm_build", 
            config = ?self.build_args.config,
            release = self.cargo_args.release
        );
        let _enter = span.enter();
        
        info!("Starting OpenVM build");
        
        match build(&self.build_args, &self.cargo_args) {
            Ok(_) => {
                info!("OpenVM build completed successfully");
                Ok(())
            }
            Err(e) => {
                error!(error = %e, "OpenVM build failed");
                Err(e)
            }
        }
    }
}
```

These integration patterns provide comprehensive guidance for incorporating OpenVM CLI commands into various development workflows and system architectures. Each pattern is designed to be modular and adaptable to specific use cases while maintaining consistency with OpenVM's design principles.