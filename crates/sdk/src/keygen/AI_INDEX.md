# Keygen Component Index

## File Structure

### Core Files
- `mod.rs` - Main module with proving key structures and key generation logic
- `asm.rs` - Converts native programs to RISC-V assembly format
- `perm.rs` - AIR permutation logic for reordering by trace height
- `dummy.rs` - Dummy proof generation for determining trace heights
- `static_verifier.rs` - Halo2 static verifier key generation (EVM feature)

## Key Types

### Proving Keys
- `AppProvingKey<VC>` - Application proving key with VM config
- `AppVerifyingKey` - Application verifying key
- `AggStarkProvingKey` - Aggregation STARK proving key
- `AggProvingKey` - Full aggregation key including Halo2 (EVM feature)
- `RootVerifierProvingKey` - Root verifier with constant trace heights

### Helper Types
- `AirIdPermutation` - Manages AIR reordering by trace height
- `VmProvingKey<SC, VC>` - VM proving key wrapper

## Main Functions

### Key Generation
- `AppProvingKey::keygen()` - Generate app proving keys
- `AggStarkProvingKey::keygen()` - Generate aggregation STARK keys
- `AggProvingKey::keygen()` - Generate full aggregation keys (EVM)
- `leaf_keygen()` - Standalone leaf verifier key generation

### Utilities
- `check_recursive_verifier_size()` - Validate verifier constraints
- `program_to_asm()` - Convert programs to assembly
- `compute_root_proof_heights()` - Calculate trace heights

## Dependencies
- `openvm-circuit` - Core VM circuits
- `openvm-continuations` - Proof continuation support
- `openvm-native-circuit` - Native field circuits
- `openvm-stark-backend` - STARK proving backend
- `openvm-native-recursion` - Halo2 recursion (optional)