# OpenVM Architecture Component - AI Index

## Quick Navigation

### Core Concepts
- [Component Overview](AI_DOCS.md#component-overview)
- [Architecture Diagrams](AI_DOCS.md#architecture-diagrams)
- [Key Design Patterns](AI_DOCS.md#key-design-patterns)

### Main Types
- [`VirtualMachine`](AI_DOCS.md#vm.rs) - Main VM entry point
- [`VmExecutor`](AI_DOCS.md#vm.rs) - Execution engine
- [`VmChipComplex`](AI_DOCS.md#extensions.rs) - Chip container
- [`ExecutionSegment`](AI_DOCS.md#segment.rs) - Execution chunk
- [`SystemConfig`](AI_DOCS.md#config.rs) - VM configuration

### Key Traits
- [`VmConfig`](AI_DOCS.md#config.rs) - Configuration trait
- [`InstructionExecutor`](AI_DOCS.md#execution.rs) - Instruction execution
- [`VmExtension`](AI_DOCS.md#extensions.rs) - Extension integration
- [`VmAdapterChip`](AI_DOCS.md#integration_api.rs) - Memory adapter
- [`VmCoreChip`](AI_DOCS.md#integration_api.rs) - Core computation
- [`SegmentationStrategy`](AI_DOCS.md#segment.rs) - Segment control

### Integration Interfaces
- [`VmAdapterInterface`](AI_DOCS.md#integration_api.rs) - Adapter API
- [`BasicAdapterInterface`](AI_DOCS.md#integration_api.rs) - Simple reads/writes
- [`VecHeapAdapterInterface`](AI_DOCS.md#integration_api.rs) - Vector operations
- [`DynAdapterInterface`](AI_DOCS.md#integration_api.rs) - Dynamic interface

### Execution Components
- [`ExecutionBus`](AI_DOCS.md#execution.rs) - State transitions
- [`ExecutionBridge`](AI_DOCS.md#execution.rs) - Bus coordination
- [`ExecutionState`](AI_DOCS.md#execution.rs) - PC and timestamp
- [`AdapterRuntimeContext`](AI_DOCS.md#integration_api.rs) - Runtime data

### System Components
- [`SystemBase`](AI_DOCS.md#extensions.rs) - Core system chips
- [`SystemExecutor`](AI_DOCS.md#extensions.rs) - System executors
- [`SystemPeriphery`](AI_DOCS.md#extensions.rs) - System periphery
- [`SystemPort`](AI_DOCS.md#extensions.rs) - System resources

### Configuration
- [`MemoryConfig`](AI_DOCS.md#configuration-options) - Memory settings
- [`SystemTraceHeights`](AI_DOCS.md#config.rs) - Trace sizing
- [`VmComplexTraceHeights`](AI_DOCS.md#extensions.rs) - Full heights

### Error Types
- [`ExecutionError`](AI_DOCS.md#error-handling) - Runtime errors
- [`GenerationError`](AI_DOCS.md#error-handling) - Trace generation
- [`VmInventoryError`](AI_DOCS.md#extensions.rs) - Chip conflicts
- [`VmVerificationError`](AI_DOCS.md#error-handling) - Proof verification

### Common Tasks

#### Setting Up a VM
```rust
let config = SystemConfig::default();
let engine = BabyBearBlake3Engine::new(proving_config);
let vm = VirtualMachine::new(engine, config);
```

#### Creating an Extension
```rust
impl VmExtension<F> for MyExtension {
    type Executor = MyExecutor<F>;
    type Periphery = ();
    
    fn build(&self, builder: &mut VmInventoryBuilder<F>) -> Result<...> {
        // Register executors...
    }
}
```

#### Implementing an Executor
```rust
impl<F> InstructionExecutor<F> for MyExecutor<F> {
    fn execute(&mut self, memory: &mut MemoryController<F>, 
               instruction: &Instruction<F>, 
               from_state: ExecutionState<u32>) -> Result<ExecutionState<u32>> {
        // Execute instruction...
    }
}
```

#### Creating Adapter/Core Pair
```rust
// Adapter for memory access
impl<F> VmAdapterChip<F> for MyAdapter<F> {
    type Interface = BasicAdapterInterface<F, MinimalInstruction<F>, 2, 1, 1, 1>;
    // Implement preprocess/postprocess...
}

// Core for computation
impl<F, I> VmCoreChip<F, I> for MyCore<F> {
    fn execute_instruction(&self, ...) -> Result<...> {
        // Core logic...
    }
}
```

### File Structure
```
arch/
├── mod.rs                # Module exports
├── config.rs            # Configuration types
├── execution.rs         # Execution infrastructure
├── extensions.rs        # Extension framework
├── vm.rs               # Virtual machine implementation
├── segment.rs          # Segmentation logic
├── integration_api.rs  # Chip integration API
├── hasher/            # Hash functions
│   ├── mod.rs
│   └── poseidon2.rs
└── testing/           # Test utilities
    ├── mod.rs
    ├── execution/
    ├── memory/
    └── program/
```

### Key Constants
- `DEFAULT_MAX_SEGMENT_LEN`: 4194204 (2^22 - 100)
- `DEFAULT_MAX_CELLS_PER_CHIP_IN_SEGMENT`: ~503M
- `SEGMENT_CHECK_INTERVAL`: 100 instructions
- `POSEIDON2_WIDTH`: 16
- `DEFAULT_MAX_NUM_PUBLIC_VALUES`: 32

### AIR IDs (Global)
- `PROGRAM_AIR_ID`: 0
- `CONNECTOR_AIR_ID`: 1
- `PUBLIC_VALUES_AIR_ID`: 2
- `BOUNDARY_AIR_ID`: PUBLIC_VALUES_AIR_ID + 1 + offset
- `MERKLE_AIR_ID`: CONNECTOR_AIR_ID + 1 + offset