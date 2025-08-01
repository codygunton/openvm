# OpenVM SDK - AI Index

## Component Overview
The OpenVM SDK provides a high-level interface for building, executing, proving, and verifying zkVM programs. It abstracts away low-level details while exposing essential functionality for developers.

## Core Module Structure

### Main Entry Point
- `lib.rs` - Main SDK API through `GenericSdk<E>` struct and type aliases
  - Primary interface for all SDK operations
  - Generic over STARK engine types
  - Provides methods for build, transpile, execute, prove, verify

### Key Modules

#### Configuration (`config/`)
- `mod.rs` - Configuration types for App VM, Aggregation, and STARK parameters
  - `AppConfig<VC>` - Application VM configuration
  - `AggConfig` - Aggregation configuration (STARK + Halo2)
  - `AggStarkConfig` - STARK aggregation parameters
  - `AggregationTreeConfig` - Tree structure for proof aggregation
- `global.rs` - Global SDK configuration (currently uses subdirectories)

#### Key Generation (`keygen/`)
- `mod.rs` - Proving and verifying key generation
  - `AppProvingKey<VC>` - Application proving keys
  - `AppVerifyingKey` - Application verification keys
  - `AggProvingKey` - Aggregation proving keys (EVM feature)
  - `AggStarkProvingKey` - STARK aggregation keys
- `asm.rs` - Assembly generation utilities
- `dummy.rs` - Dummy proof generation for keygen
- `perm.rs` - AIR ID permutation utilities
- `static_verifier.rs` - Static verifier key generation (EVM feature)

#### Proving (`prover/`)
- `mod.rs` - Main prover exports
- `app.rs` - Application-level proving
  - `AppProver<VC, E>` - Generates app VM proofs
- `stark.rs` - STARK proof generation
  - `StarkProver<VC, E>` - E2E STARK proving
- `agg.rs` - Proof aggregation
  - `AggProver<E>` - Aggregates multiple proofs
- `root.rs` - Root proof generation
  - `RootProver<E>` - Final root proofs
- `halo2.rs` - Halo2 SNARK proving (EVM feature)
  - `Halo2Prover` - STARK-to-SNARK conversion
- `vm/` - VM proving utilities
  - `types.rs` - Core proving types
  - `local.rs` - Local VM execution
  - `mod.rs` - VM module exports

#### Core Types and Utilities
- `types.rs` - SDK-specific types
  - `EvmProof` - EVM-compatible proof format
  - `VmStarkProofBytes` - Serializable STARK proofs
  - `ProofData` - Raw proof data
- `stdin.rs` - Standard input handling
  - `StdIn` - Input stream management
- `commit.rs` - Commitment utilities
  - `AppExecutionCommit` - Application execution commitments
  - `CommitBytes` - Commitment byte representation
- `codec.rs` - Encoding/decoding utilities
- `fs.rs` - File system utilities for proof/key storage

### Binary Executables (`bin/`)
- `app_prover.rs` - CLI for generating application proofs
- `program_executor.rs` - CLI for executing VM programs

## Key Type Relationships

### SDK Hierarchy
```
GenericSdk<E: StarkFriEngine>
├── Configuration Management
│   ├── AppConfig<VC>
│   ├── AggConfig
│   └── AggregationTreeConfig
├── Key Generation
│   ├── AppProvingKey<VC> / AppVerifyingKey
│   └── AggProvingKey / AggStarkProvingKey
├── Proving Pipeline
│   ├── AppProver → ContinuationVmProof
│   ├── StarkProver → VmStarkProof
│   └── Halo2Prover → EvmProof
└── Commitment System
    ├── AppExecutionCommit
    └── NonRootCommittedExe
```

### Data Flow
1. **Build**: Source → Elf → VmExe
2. **Execute**: VmExe + StdIn → Public Values
3. **Prove**: VmExe + StdIn → Proof
4. **Verify**: Proof + VK → Verified Payload

## Feature-Gated Functionality

### `evm-prove` Feature
- Enables EVM proof generation
- Adds Halo2 proving capabilities
- Includes static verifier support

### `evm-verify` Feature
- Enables EVM proof verification
- Generates Solidity verifier contracts
- Includes contract interfaces

## Integration Points

### External Dependencies
- `openvm-circuit` - Core circuit definitions
- `openvm-stark-backend` - STARK proving backend
- `openvm-native-recursion` - Recursion support
- `openvm-continuations` - Continuation proof types
- `openvm-build` - Build system integration

### Contract Templates
- `contracts/template/OpenVmHalo2Verifier.sol` - EVM verifier template
- `contracts/src/IOpenVmHalo2Verifier.sol` - Verifier interface
- `contracts/abi/` - Contract ABIs

## Common Usage Patterns

### Basic Proving Flow
```rust
let sdk = Sdk::new();
let elf = sdk.build(...);
let exe = sdk.transpile(elf, transpiler);
let committed_exe = sdk.commit_app_exe(fri_params, exe);
let app_pk = sdk.app_keygen(config);
let proof = sdk.generate_app_proof(app_pk, committed_exe, inputs);
```

### EVM Proof Generation
```rust
let agg_pk = sdk.agg_keygen(config, reader, handler);
let evm_proof = sdk.generate_evm_proof(reader, app_pk, app_exe, agg_pk, inputs);
```