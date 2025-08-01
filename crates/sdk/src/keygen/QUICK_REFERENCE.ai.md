# Keygen Quick Reference

## Essential Functions

### Generate App Proving Key
```rust
use openvm_sdk::keygen::AppProvingKey;
use openvm_sdk::config::AppConfig;

let app_pk = AppProvingKey::keygen(app_config);
let app_vk = app_pk.get_app_vk();
let commitment = app_pk.commit_in_bn254();
```

### Generate Aggregation Keys
```rust
use openvm_sdk::keygen::AggStarkProvingKey;
use openvm_sdk::config::AggStarkConfig;

let agg_pk = AggStarkProvingKey::keygen(agg_config);
let internal_commit = agg_pk.internal_program_commit();
```

### Generate EVM Keys (feature = "evm-prove")
```rust
use openvm_sdk::keygen::AggProvingKey;
use openvm_sdk::config::AggConfig;

let agg_pk = AggProvingKey::keygen(config, &params_reader, &pv_handler);
```

## Key Types Quick Reference

| Type | Purpose | Key Methods |
|------|---------|-------------|
| `AppProvingKey<VC>` | App proving | `keygen()`, `get_app_vk()`, `commit_in_bn254()` |
| `AppVerifyingKey` | App verification | Used by verifiers |
| `AggStarkProvingKey` | STARK aggregation | `keygen()`, `internal_program_commit()` |
| `AggProvingKey` | Full aggregation | `keygen()` (includes Halo2) |
| `RootVerifierProvingKey` | Root verification | `air_id_permutation()` |

## Common Patterns

### Check Verifier Size
```rust
check_recursive_verifier_size(&vk, fri_params, next_log_blowup);
```

### Convert Program to Assembly
```rust
use openvm_sdk::keygen::asm::program_to_asm;

let asm_code = program_to_asm(program);
```

### Generate Leaf Keys Only
```rust
use openvm_sdk::keygen::leaf_keygen;

let leaf_pk = leaf_keygen(fri_params, leaf_vm_config);
```

## AIR Permutation
```rust
use openvm_sdk::keygen::perm::AirIdPermutation;

let perm = AirIdPermutation::compute(&heights);
let special_ids = perm.get_special_air_ids();
perm.permute(&mut air_array);
```

## FRI Parameters
```rust
use openvm_stark_sdk::config::fri_params::*;

// Common configurations
let fri_params = standard_fri_params_with_100_bits_conjectured_security(log_blowup);
```

## Memory Estimates

| Operation | Typical Memory Usage |
|-----------|---------------------|
| App keygen | 100MB - 1GB |
| Leaf keygen | 500MB - 2GB |
| Internal keygen | 1GB - 5GB |
| Root keygen | 2GB - 8GB |
| Halo2 keygen | 10GB - 64GB |

## Common Errors

| Error | Cause | Solution |
|-------|-------|----------|
| "Recursive verifier size too large" | FRI parameters too aggressive | Increase log_blowup |
| "Max constraint degree exceeded" | Complex constraints | Adjust FRI max degree |
| "Out of memory" | Large Halo2 keys | Use machine with more RAM |
| "AIR not found" | Permutation issue | Check special AIR tracking |

## Performance Tips

1. **Pre-generate Keys**: Generate once, serialize, and reuse
2. **Parallel Generation**: Use separate threads for independent keys
3. **Memory Mapping**: Use memory-mapped files for large keys
4. **Incremental Generation**: Generate keys as needed, not all upfront

## Debug Commands

```rust
// Log key sizes
tracing::info!("App PK size: {} MB", app_pk.size_hint() / 1_000_000);

// Verify key consistency
assert_eq!(app_pk.app_fri_params(), config.app_fri_params.fri_params);

// Check AIR permutation
let perm = root_pk.air_id_permutation();
tracing::debug!("AIR order: {:?}", perm.perm);
```