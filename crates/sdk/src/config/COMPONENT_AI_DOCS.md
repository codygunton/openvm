# OpenVM SDK Config Component

## Overview
The SDK config component provides configuration management for OpenVM applications and aggregation systems. It defines configuration structures for application VMs, aggregation trees, and STARK/SNARK parameters used throughout the OpenVM ecosystem.

## Architecture
This component is located at `crates/sdk/src/config/` and serves as the central configuration hub for:
- **Application Configuration**: Settings for application VMs with customizable FRI parameters
- **Aggregation Configuration**: STARK and Halo2 configurations for proof aggregation
- **VM Configuration**: Extensible VM configuration with support for multiple instruction set extensions
- **Global Configuration**: System-wide settings and default parameters

## Key Components

### Configuration Types
- `AppConfig<VC>`: Application-level configuration with FRI parameters and VM config
- `AggConfig`: Aggregation configuration combining STARK and Halo2 settings
- `SdkVmConfig`: Main VM configuration supporting multiple extensions
- `AggregationTreeConfig`: Tree structure configuration for proof aggregation

### Extension Support
The configuration system supports the following VM extensions:
- **RISC-V**: rv32i, rv32m, I/O operations
- **Cryptography**: Keccak256, SHA256
- **Arithmetic**: Native field operations, BigInt (Int256), modular arithmetic
- **Advanced Math**: Fp2 extensions, elliptic curve operations, pairing operations
- **Type Casting**: CastF extension for field element conversions

### Default Parameters
- App FRI blowup: 1
- Leaf FRI blowup: 1  
- Internal FRI blowup: 2
- Root FRI blowup: 3
- Aggregation tree: 1 child per leaf, 3 children per internal node
- Security level: 100-bit conjectured security

## Key Files
- `mod.rs`: Main configuration structures and aggregation settings
- `global.rs`: Global VM configuration with all extension support
- `AI_DOCS.md`: Component-level AI documentation (existing)
- `AI_INDEX.md`: Component index and navigation (existing)

## Usage Context
This component is fundamental to OpenVM's operation, providing the configuration layer that:
1. Defines VM capabilities through extension selection
2. Sets cryptographic parameters for security
3. Configures aggregation trees for scalable proof systems
4. Manages transpiler settings for ELF-to-bytecode conversion

## Integration Points
- Used by the OpenVM CLI for build and prove operations
- Integrated with circuit implementations for constraint system setup
- Connected to transpiler components for instruction set configuration
- Utilized by aggregation systems for proof tree construction

## Security Considerations
- All FRI parameters default to 100-bit conjectured security
- Cryptographic configurations are audited by Cantina and Axiom team
- Extension configurations must maintain security properties
- Profiling mode available for debugging but should be disabled in production