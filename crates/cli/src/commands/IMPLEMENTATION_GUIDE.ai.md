# OpenVM CLI Commands - Implementation Guide

## Command Implementation Patterns

### 1. Basic Command Structure

```rust
use clap::Parser;
use eyre::Result;

#[derive(Parser)]
#[command(name = "mycommand", about = "Short description")]
pub struct MyCommand {
    #[arg(long, help = "Description of option")]
    option: Option<String>,
    
    #[clap(flatten)]
    common_args: CommonArgs,
}

impl MyCommand {
    pub fn run(&self) -> Result<()> {
        // Implementation
        Ok(())
    }
}
```

### 2. Build Integration Pattern

```rust
pub fn build_and_run(args: &RunArgs, cargo_args: &RunCargoArgs) -> Result<()> {
    // Determine if build is needed
    let exe_path = if let Some(exe) = &args.exe {
        exe.clone()
    } else {
        // Build and get output path
        let target_name = get_single_target_name(cargo_args)?;
        let build_args = args.clone().into();
        let cargo_args = cargo_args.clone().into();
        let output_dir = build(&build_args, &cargo_args)?;
        output_dir.join(format!("{}.vmexe", target_name))
    };
    
    // Use the executable
    let exe = read_exe_from_file(&exe_path)?;
    // ... rest of implementation
}
```

### 3. Config Loading Pattern

```rust
pub fn load_config(args: &CommandArgs) -> Result<AppConfig<SdkVmConfig>> {
    let (_, manifest_dir) = get_manifest_path_and_dir(&args.manifest_path)?;
    
    let config_path = args.config
        .clone()
        .unwrap_or_else(|| manifest_dir.join("openvm.toml"));
    
    if config_path.exists() {
        read_to_struct_toml(config_path)
    } else {
        println!("Config not found, using defaults");
        Ok(default_app_config())
    }
}
```

### 4. Key Management Pattern

```rust
pub fn load_or_generate_keys(
    pk_path: &Option<PathBuf>,
    cargo_args: &CargoArgs,
) -> Result<Arc<AppProvingKey<SdkVmConfig>>> {
    let (manifest_path, _) = get_manifest_path_and_dir(&cargo_args.manifest_path)?;
    let target_dir = get_target_dir(&cargo_args.target_dir, &manifest_path);
    
    let pk_path = pk_path
        .clone()
        .unwrap_or_else(|| get_app_pk_path(&target_dir));
    
    if pk_path.exists() {
        Ok(Arc::new(read_app_pk_from_file(pk_path)?))
    } else {
        Err(eyre::eyre!(
            "Proving key not found at {:?}. Run 'cargo openvm keygen' first.",
            pk_path
        ))
    }
}
```

### 5. Output Management Pattern

```rust
pub fn write_output_with_optional_copy(
    data: &impl Serialize,
    default_path: &Path,
    output_dir: &Option<PathBuf>,
    filename: &str,
) -> Result<()> {
    // Write to default location
    write_to_file_json(default_path, data)?;
    
    // Copy to output dir if specified
    if let Some(output_dir) = output_dir {
        create_dir_all(output_dir)?;
        copy(default_path, output_dir.join(filename))?;
    }
    
    Ok(())
}
```

### 6. Multi-Stage Command Pattern

```rust
#[derive(Parser)]
enum ProveSubCommand {
    App {
        #[clap(flatten)]
        args: AppProveArgs,
    },
    Stark {
        #[clap(flatten)]
        args: StarkProveArgs,
    },
    Evm {
        #[clap(flatten)]
        args: EvmProveArgs,
    },
}

impl ProveCmd {
    pub fn run(&self) -> Result<()> {
        match &self.command {
            ProveSubCommand::App { args } => self.prove_app(args),
            ProveSubCommand::Stark { args } => self.prove_stark(args),
            ProveSubCommand::Evm { args } => self.prove_evm(args),
        }
    }
}
```

### 7. Workspace Handling Pattern

```rust
pub fn get_workspace_targets(cargo_args: &CargoArgs) -> Result<Vec<Target>> {
    let (manifest_path, manifest_dir) = get_manifest_path_and_dir(&cargo_args.manifest_path)?;
    let workspace_root = get_workspace_root(&manifest_path);
    
    let packages = if cargo_args.workspace || manifest_dir == workspace_root {
        get_workspace_packages(manifest_dir)
    } else {
        vec![get_package(manifest_dir)]
    };
    
    packages
        .into_iter()
        .filter(|pkg| {
            (cargo_args.package.is_empty() || cargo_args.package.contains(&pkg.name))
                && !cargo_args.exclude.contains(&pkg.name)
        })
        .flat_map(|pkg| pkg.targets)
        .collect()
}
```

### 8. Progress Reporting Pattern

```rust
pub fn execute_with_progress<T>(
    description: &str,
    operation: impl FnOnce() -> Result<T>,
) -> Result<T> {
    println!("[openvm] {}...", description);
    let result = operation()?;
    println!("[openvm] Successfully completed {}", description.to_lowercase());
    Ok(result)
}
```

### 9. File Discovery Pattern

```rust
pub fn find_proof_file(proof_path: &Option<PathBuf>, extension: &str) -> Result<PathBuf> {
    if let Some(path) = proof_path {
        Ok(path.clone())
    } else {
        let files = get_files_with_ext(Path::new("."), extension)?;
        match files.len() {
            0 => Err(eyre::eyre!("No {} file found", extension)),
            1 => Ok(files[0].clone()),
            _ => Err(eyre::eyre!(
                "Multiple {} files found, specify with --proof",
                extension
            )),
        }
    }
}
```

### 10. Async Command Pattern

```rust
#[derive(Parser)]
pub struct AsyncCommand {
    // args
}

impl AsyncCommand {
    pub async fn run(&self) -> Result<()> {
        // Use tokio runtime if needed
        let runtime = tokio::runtime::Runtime::new()?;
        runtime.block_on(self.async_operation())
    }
    
    async fn async_operation(&self) -> Result<()> {
        // Async implementation
        Ok(())
    }
}
```

## Error Handling Patterns

### Contextual Errors
```rust
operation()
    .context("Failed to perform operation")?;

operation()
    .wrap_err_with(|| format!("Failed to process {}", filename))?;
```

### User-Friendly Errors
```rust
if !path.exists() {
    return Err(eyre::eyre!(
        "File not found: {:?}\nDid you run 'cargo openvm build' first?",
        path
    ));
}
```

### Recovery Suggestions
```rust
match operation() {
    Ok(result) => Ok(result),
    Err(e) if e.to_string().contains("out of memory") => {
        Err(eyre::eyre!(
            "Out of memory during proving. Try:\n\
            1. Increase system memory\n\
            2. Use a smaller test case\n\
            3. Run with --release profile"
        ))
    }
    Err(e) => Err(e),
}
```

## Testing Patterns

### Command Test Structure
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_command_parsing() {
        let cmd = MyCommand::parse_from(&[
            "mycommand",
            "--option", "value",
        ]);
        assert_eq!(cmd.option, Some("value".to_string()));
    }
    
    #[test]
    fn test_command_execution() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let cmd = MyCommand {
            option: Some("test".to_string()),
            output_dir: Some(temp_dir.path().to_path_buf()),
        };
        
        cmd.run()?;
        
        // Verify outputs
        assert!(temp_dir.path().join("output.file").exists());
        Ok(())
    }
}
```

### Integration Test Pattern
```rust
#[test]
fn test_full_workflow() -> Result<()> {
    // Build
    let build_cmd = BuildCmd::parse_from(&["build"]);
    build_cmd.run()?;
    
    // Run
    let run_cmd = RunCmd::parse_from(&["run"]);
    run_cmd.run()?;
    
    // Verify outputs
    Ok(())
}
```

## Performance Optimization Patterns

### Lazy Loading
```rust
struct CommandContext {
    config: OnceCell<AppConfig>,
    keys: OnceCell<AppProvingKey>,
}

impl CommandContext {
    fn config(&self) -> Result<&AppConfig> {
        self.config.get_or_try_init(|| {
            // Load config only when needed
            read_config_toml_or_default("openvm.toml")
        })
    }
}
```

### Parallel Processing
```rust
use rayon::prelude::*;

let results: Vec<_> = targets
    .par_iter()
    .map(|target| process_target(target))
    .collect::<Result<Vec<_>>>()?;
```

## Common Implementation Tasks

### Adding a New Proof Type
1. Add variant to proof subcommand enum
2. Implement proof generation logic
3. Add serialization support
4. Update verify command
5. Document memory requirements

### Adding Build Options
1. Add field to `BuildArgs` struct
2. Update `Default` implementation
3. Pass option to build system
4. Update help text
5. Add to config if persistent

### Integrating External Tools
1. Check tool availability
2. Provide installation instructions
3. Handle tool failures gracefully
4. Support tool version requirements
5. Cache tool outputs

## Best Practices

1. **Fail Fast**: Validate inputs early
2. **Progress Updates**: Keep users informed
3. **Atomic Operations**: Complete fully or rollback
4. **Resource Cleanup**: Use RAII patterns
5. **Consistent Naming**: Follow project conventions
6. **Helpful Errors**: Guide users to solutions
7. **Performance**: Profile before optimizing
8. **Testing**: Cover edge cases
9. **Documentation**: Update with code
10. **Compatibility**: Maintain backwards compatibility