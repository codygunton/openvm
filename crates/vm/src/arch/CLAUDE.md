# OpenVM Architecture Component - Claude Instructions

## Component Purpose

The architecture component is the heart of OpenVM's virtual machine implementation. It provides the execution engine, chip integration framework, and orchestrates the entire VM lifecycle from program execution to proof generation.

## Key Concepts to Understand

1. **No-CPU Architecture**: Unlike traditional VMs, OpenVM doesn't have a central CPU. Instead, it uses a collection of specialized chips (executors) that handle specific instruction types.

2. **Adapter-Core Separation**: Every instruction executor is split into:
   - **Adapter**: Handles memory access and bus communication
   - **Core**: Performs the actual computation

3. **Execution Segmentation**: For large programs, execution can be split into segments to maintain reasonable proof sizes.

4. **Extension System**: New functionality is added through extensions that register executors and periphery chips.

## Common Implementation Tasks

### Creating a New Instruction Set Extension

1. Define your opcodes and instruction format
2. Create executor enum with derive macros
3. Implement VmExtension trait
4. Register executors with their opcodes

### Implementing an Instruction Executor

Two approaches:
1. **Direct Implementation**: Implement `InstructionExecutor` directly (simpler)
2. **Adapter-Core Pattern**: Use `VmChipWrapper` with separate adapter and core (more modular)

### Memory Access Patterns

- Always use `MemoryController` for reads/writes
- Increment timestamp for each memory operation
- Use appropriate adapter interfaces for your access pattern

## Critical Integration Points

1. **System Port**: Provides access to execution bus, program bus, and memory bridge
2. **Bus Registration**: All bus operations must be properly registered for soundness
3. **Trace Generation**: Must match AIR constraints exactly

## Performance Considerations

- Batch memory operations when possible
- Use appropriate adapter interfaces (avoid DynInterface unless necessary)
- Consider segmentation strategy for large programs

## Testing Strategy

1. Unit test executors with mock memory
2. Integration test with full VM setup
3. Constraint test to ensure soundness
4. Benchmark critical paths

## Common Pitfalls

1. **Timestamp Management**: Forgetting to increment timestamp for memory ops
2. **Opcode Registration**: Conflicting opcodes between extensions
3. **Trace Alignment**: Mismatch between execution and trace generation
4. **Bus Coordination**: Missing bus operations in constraints

## Architecture Philosophy

The architecture is designed for:
- **Modularity**: Easy to add new instruction sets
- **Soundness**: All operations are constraint-checked
- **Performance**: Parallel trace generation, efficient memory access
- **Flexibility**: Support for custom chips and exotic operations

## When Modifying This Component

1. Understand the invariants (especially around buses and memory)
2. Maintain backward compatibility for extensions
3. Update both execution and constraint code
4. Add comprehensive tests
5. Document any new patterns or requirements

## Key Files to Understand First

1. `execution.rs` - Core execution traits
2. `extensions.rs` - Extension framework
3. `integration_api.rs` - Adapter/Core pattern
4. `vm.rs` - Top-level VM implementation