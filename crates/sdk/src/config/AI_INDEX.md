# OpenVM SDK Config Component - AI Index

## Component Overview
The config component provides configuration management for the OpenVM SDK, handling both application-level and aggregation-level configurations.

## Core Purpose
- Define configuration structures for app VMs and aggregation layers
- Manage FRI parameters for different proof levels
- Configure VM extensions and their initialization
- Handle aggregation tree structure configuration

## Key Structures

### Application Configuration
- `AppConfig<VC>`: Main app configuration with FRI params and VM config
- `SdkVmConfig`: Modular VM configuration with optional extensions
- `AppFriParams`/`LeafFriParams`: FRI parameter wrappers

### Aggregation Configuration
- `AggConfig`: Combined STARK and SNARK aggregation config
- `AggStarkConfig`: STARK-specific aggregation parameters
- `Halo2Config`: SNARK wrapper configuration
- `AggregationTreeConfig`: Tree structure for recursive aggregation

### VM Extension System
- `SdkVmConfigExecutor<F>`: Enum of all available executors
- `SdkVmConfigPeriphery<F>`: Enum of all available peripheries
- Extension support: RV32I/M, IO, Keccak, SHA256, Native, BigInt, Modular, Fp2, Pairing, ECC

## Configuration Flow
1. User creates `AppConfig` with app-specific VM config
2. System generates aggregation config with default or custom params
3. VM complex created from config with selected extensions
4. Init files generated for algebraic extensions if needed

## Key Features
- Modular extension system with automatic dependency handling
- Configurable FRI parameters for different security levels
- Automatic transpiler configuration based on enabled extensions
- Init file generation for complex field extensions

## Dependencies
- `openvm-circuit`: Core VM architecture
- `openvm-stark-sdk`: FRI parameter configuration
- Extension crates: Each extension has circuit/executor/periphery crates
- `openvm-transpiler`: For transpilation configuration

## Usage Context
This component is used by the SDK to:
- Configure app VMs with specific instruction sets
- Set up aggregation infrastructure
- Generate proving/verifying keys
- Initialize algebraic field extensions