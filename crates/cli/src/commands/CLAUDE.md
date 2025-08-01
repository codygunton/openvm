# OpenVM CLI Commands - Claude Instructions

## Component Overview
This directory contains the core CLI command implementations for OpenVM. Each command is self-contained in its own module and follows consistent patterns for argument parsing, execution, and error handling.

## Key Principles

### 1. Command Consistency
- Every command struct ends with `Cmd` suffix
- Every command implements a `run()` method returning `Result<()>`
- Use `#[derive(Parser)]` with clap for argument parsing
- Group related arguments using `#[clap(flatten)]`

### 2. Cargo Integration
- Commands that build should accept standard Cargo arguments
- Respect user's Cargo configuration and environment
- Use `openvm_build` crate for build operations
- Forward Cargo output appropriately

### 3. Error Handling
- Provide actionable error messages
- Suggest next steps when operations fail
- Use `eyre::Result` for error propagation
- Include context with `.context()` or `.wrap_err()`

### 4. File Management
- Use consistent paths: `target/openvm/{profile}/`
- Create directories before writing files
- Support both absolute and relative paths
- Clean up temporary files on error

## Command Implementation Guidelines

### Adding New Commands

1. **Create Module File**
   ```rust
   // commands/newcmd.rs
   use clap::Parser;
   use eyre::Result;
   
   #[derive(Parser)]
   #[command(name = "newcmd", about = "Description")]
   pub struct NewCmd {
       // Arguments
   }
   
   impl NewCmd {
       pub fn run(&self) -> Result<()> {
           // Implementation
       }
   }
   ```

2. **Export in mod.rs**
   ```rust
   mod newcmd;
   pub use newcmd::*;
   ```

3. **Add to Main CLI** (in `bin/cli.rs`)

### Argument Organization

Group arguments logically:
```rust
#[derive(Parser)]
pub struct MyCmd {
    // Command-specific args first
    #[arg(long, help = "...", help_heading = "Command Options")]
    my_option: String,
    
    // Flattened common args
    #[clap(flatten)]
    build_args: BuildArgs,
    
    #[clap(flatten)]
    cargo_args: CargoArgs,
}
```

### Build Integration

When commands need to build:
```rust
// Reuse build command logic
let output_dir = build(&build_args, &cargo_args)?;

// Or get target information
let (manifest_path, manifest_dir) = get_manifest_path_and_dir(&cargo_args.manifest_path)?;
let target_dir = get_target_dir(&cargo_args.target_dir, &manifest_path);
```

### Output Handling

1. **Progress Messages**
   ```rust
   println!("[openvm] Building the package...");
   // operation
   println!("[openvm] Successfully built the packages");
   ```

2. **File Outputs**
   ```rust
   let output_path = target_output_dir.join(format!("{}.vmexe", target_name));
   write_exe_to_file(exe, &output_path)?;
   ```

3. **Optional Output Dirs**
   ```rust
   if let Some(output_dir) = &args.output_dir {
       create_dir_all(output_dir)?;
       copy(&source, output_dir.join(&filename))?;
   }
   ```

## Common Patterns

### Config Loading
```rust
let app_config = read_config_toml_or_default(
    args.config
        .to_owned()
        .unwrap_or_else(|| manifest_dir.join("openvm.toml")),
)?;
```

### Target Resolution
```rust
let target_name = get_single_target_name(&cargo_args)?;
```

### Key Management
```rust
let app_pk_path = if let Some(path) = &args.app_pk {
    path.to_path_buf()
} else {
    get_app_pk_path(&target_dir)
};
```

## Testing Guidelines

1. **Unit Tests**: Test argument parsing and validation
2. **Integration Tests**: Test full command execution
3. **Error Cases**: Test missing files, invalid configs
4. **Edge Cases**: Empty workspaces, multiple targets

## Performance Considerations

1. **Lazy Loading**: Don't load files until needed
2. **Parallel Operations**: Use async where beneficial
3. **Caching**: Reuse build artifacts and keys
4. **Memory**: Be mindful of proof generation memory usage

## Security Notes

1. **Input Validation**: Validate all file paths and inputs
2. **Key Handling**: Never log or expose private keys
3. **Proof Integrity**: Always verify proof structure
4. **File Permissions**: Respect system file permissions

## Maintenance

### Deprecation
When deprecating commands or options:
1. Mark with `#[deprecated]` attribute
2. Provide migration path in help text
3. Support old behavior for 2 releases
4. Document in CHANGELOG

### Version Compatibility
- Maintain backward compatibility in file formats
- Version proving artifacts if format changes
- Provide migration tools when needed

## Common Issues and Solutions

### Issue: Multiple Targets Found
**Solution**: Add explicit target detection logic and helpful error messages

### Issue: Missing Config File
**Solution**: Fall back to default configuration with warning

### Issue: Proof Generation OOM
**Solution**: Document memory requirements and suggest workarounds

### Issue: Build Cache Conflicts
**Solution**: Use profile-specific output directories

## Code Review Checklist

- [ ] Command has clear help text
- [ ] Arguments are properly grouped
- [ ] Error messages are actionable
- [ ] File paths are handled correctly
- [ ] Progress is reported to user
- [ ] Resources are cleaned up
- [ ] Tests cover main paths
- [ ] Documentation is updated