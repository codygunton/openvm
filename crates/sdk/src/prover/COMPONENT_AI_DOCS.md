# Prover Component Documentation

## Overview
The prover component is the core proving system for OpenVM, responsible for generating zero-knowledge proofs for virtual machine execution. It provides a multi-layered proof generation system supporting both STARK and Halo2 backends.

## Architecture

### Core Components

#### AppProver (`app.rs`)
- **Purpose**: Generates application-level proofs for VM execution
- **Key Features**:
  - Support for continuation proofs (multi-segment execution)
  - Single-segment proof generation
  - Program name tracking for debugging/metrics
- **Main Types**: `AppProver<VC, E>`

#### StarkProver (`stark.rs`) 
- **Purpose**: Orchestrates the full STARK proving pipeline
- **Key Features**:
  - Combines app and aggregation proving
  - Validates compatibility between proving keys
  - Generates proofs for outer recursion
- **Main Types**: `StarkProver<VC, E>`

#### AggStarkProver (`agg.rs`)
- **Purpose**: Handles proof aggregation using STARK backend
- **Key Features**:
  - Aggregation tree-based proof combination
  - Root proof generation
  - Batch verification support

#### EvmHalo2Prover (`mod.rs` - evm module)
- **Purpose**: Generates EVM-compatible proofs using Halo2
- **Key Features**:  
  - STARK-to-Halo2 proof conversion
  - EVM verifier compatibility
  - Integration with aggregation pipeline

#### VM Provers (`vm/` directory)
- **VmLocalProver**: Local proving for single VM instances
- **ContinuationVmProver**: Handles multi-segment continuations
- **SingleSegmentVmProver**: Single execution segment proving

### Type System

#### Generic Parameters
- `VC: VmConfig<F>`: VM configuration defining instruction set and peripherals
- `E: StarkFriEngine<SC>`: STARK proving engine with FRI backend
- `SC`: STARK configuration (field and hash settings)
- `F`: Base field for VM operations

#### Key Traits Required
- `VC::Executor: Chip<SC>`: Execution unit must be provable
- `VC::Periphery: Chip<SC>`: Peripheral components must be provable

## Proof Generation Flow

### Standard Flow
1. **App Proof**: `AppProver` generates proof for VM execution
2. **Aggregation**: `AggStarkProver` combines multiple proofs
3. **Root Proof**: Final aggregated proof for verification

### EVM Flow  
1. **STARK Pipeline**: Standard flow through `StarkProver`
2. **Halo2 Conversion**: `Halo2Prover` converts to EVM-compatible format
3. **EVM Proof**: Final proof verifiable on Ethereum

## Configuration

### Proving Keys
- `AppProvingKey<VC>`: Contains VM-specific proving parameters
- `AggStarkProvingKey`: Aggregation circuit proving key  
- `AggProvingKey`: Combined STARK and Halo2 proving keys

### Validation
- FRI parameter compatibility between app and aggregation
- Public value count matching between layers
- Configuration consistency checks

## Error Handling
- Assertion-based validation for configuration mismatches
- Panic on incompatible proving key combinations
- Continuation mode validation

## Performance Considerations
- Metrics integration with `bench-metrics` feature
- FRI blowup factor tracking
- Tracing spans for performance monitoring
- Program name labeling for debugging

## Integration Points
- **Input**: `StdIn` for program inputs
- **Output**: Various proof types (`ContinuationVmProof`, `Proof<SC>`, `EvmProof`)
- **Keys**: Proving keys from keygen module
- **Config**: Aggregation tree configuration