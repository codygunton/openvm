# OpenVM Architecture (arch) Component - AI Documentation

## Component Overview

The `arch` component implements the core virtual machine architecture for OpenVM, providing the execution engine, chip integration framework, and memory management interfaces. It uses a modular design that allows seamless integration of custom instruction sets without forking the core VM.

### Key Responsibilities

1. **VM Execution Engine**: Manages program execution, segmentation, and proof generation
2. **Chip Integration**: Provides framework for integrating instruction executors and periphery chips
3. **Memory Architecture**: Coordinates with memory controller for state management
4. **Continuation Support**: Handles multi-segment execution with state persistence
5. **Proof Generation**: Orchestrates trace generation and proof input preparation

## Architecture Diagrams

### Core VM Architecture
```
┌─────────────────────────────────────────────────────────────┐
│                      VirtualMachine                          │
│  ┌─────────────────┐              ┌───────────────────┐    │
│  │   VmExecutor    │              │   StarkEngine     │    │
│  │                 │              │                   │    │
│  │  - Execute      │              │  - Prove          │    │
│  │  - Segment      │              │  - Verify         │    │
│  │  - Generate     │              │                   │    │
│  └────────┬────────┘              └─────────┬─────────┘    │
│           │                                  │               │
│           ▼                                  ▼               │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              VmChipComplex                           │   │
│  │  ┌─────────────┐  ┌──────────────┐  ┌────────────┐ │   │
│  │  │ SystemBase  │  │  Inventory   │  │   Config   │ │   │
│  │  │             │  │              │  │            │ │   │
│  │  │ - Program   │  │ - Executors  │  │ - Memory   │ │   │
│  │  │ - Memory    │  │ - Periphery  │  │ - System   │ │   │
│  │  │ - Connector │  │              │  │            │ │   │
│  │  └─────────────┘  └──────────────┘  └────────────┘ │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### Chip Integration Framework
```
┌─────────────────────────────────────────────────────┐
│                 VmChipWrapper                        │
│  ┌──────────────────┐      ┌──────────────────┐   │
│  │  AdapterChip     │      │   CoreChip       │   │
│  │                  │      │                  │   │
│  │ - Memory Access  │      │ - Execute        │   │
│  │ - Bus Interface  │      │ - Constraints    │   │
│  │ - Trace Gen      │      │ - Public Values  │   │
│  └────────┬─────────┘      └─────────┬────────┘   │
│           │                           │             │
│           ▼                           ▼             │
│  ┌──────────────────────────────────────────────┐  │
│  │            Integration Interface              │  │
│  │   - Reads    - Writes    - Instructions     │  │
│  └──────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────┘
```

### Execution Flow
```
┌──────────┐    ┌───────────────┐    ┌──────────────┐
│ Program  │───▶│ ExecutionBus  │───▶│ Instruction  │
│  Chip    │    │               │    │  Executor    │
└──────────┘    └───────────────┘    └──────┬───────┘
                                             │
                ┌───────────────┐            ▼
                │ MemoryBridge  │◀───────────────────┐
                │               │                     │
                └───────┬───────┘    ┌──────────────┐│
                        │            │   Memory     ││
                        └───────────▶│ Controller   ││
                                    └──────────────┘│
                                                    │
                ┌───────────────┐    ┌─────────────┘
                │ ConnectorChip │◀───┘
                │               │
                └───────────────┘
```

## Module Structure

### Core Modules

1. **config.rs**: VM configuration and initialization
   - `VmConfig` trait for extensible configurations
   - System configuration with memory and continuation settings
   - Init file generation for guest programs

2. **execution.rs**: Execution infrastructure
   - `InstructionExecutor` trait for chip integration
   - Execution bus for state transitions
   - Error handling for execution failures

3. **extensions.rs**: Extension and chip management
   - `VmExtension` trait for modular extensions
   - Chip inventory management
   - System base components (memory, program, connector)

4. **vm.rs**: Virtual machine implementation
   - `VirtualMachine` struct combining executor and prover
   - Segment execution and proof generation
   - Verification logic for continuations

5. **segment.rs**: Execution segmentation
   - `ExecutionSegment` for bounded execution chunks
   - Segmentation strategies for continuation support
   - Trace cell tracking for dynamic segmentation

6. **integration_api.rs**: Chip integration framework
   - Adapter/Core chip separation
   - Interface definitions for memory access patterns
   - Trace generation coordination

### Sub-modules

- **hasher/**: Cryptographic hash functions (Poseidon2)
- **testing/**: Testing utilities and mock implementations

## Key Design Patterns

### 1. Extension Pattern
```rust
trait VmExtension<F: PrimeField32> {
    type Executor: InstructionExecutor<F> + AnyEnum;
    type Periphery: AnyEnum;
    
    fn build(&self, builder: &mut VmInventoryBuilder<F>) 
        -> Result<VmInventory<Self::Executor, Self::Periphery>, VmInventoryError>;
}
```

### 2. Adapter-Core Separation
```rust
// Adapter handles memory and buses
trait VmAdapterChip<F> {
    fn preprocess(&mut self, memory: &mut MemoryController<F>, instruction: &Instruction<F>)
        -> Result<(Reads, ReadRecord)>;
    fn postprocess(&mut self, memory: &mut MemoryController<F>, ...) 
        -> Result<(ExecutionState, WriteRecord)>;
}

// Core handles computation
trait VmCoreChip<F, I: VmAdapterInterface<F>> {
    fn execute_instruction(&self, instruction: &Instruction<F>, from_pc: u32, reads: I::Reads)
        -> Result<(AdapterRuntimeContext<F, I>, Self::Record)>;
}
```

### 3. Segmentation Strategy
```rust
trait SegmentationStrategy {
    fn should_segment(&self, air_names: &[String], trace_heights: &[usize], trace_cells: &[usize]) -> bool;
    fn stricter_strategy(&self) -> Arc<dyn SegmentationStrategy>;
}
```

## Integration Points

### 1. Memory System
- Uses `MemoryController` for all memory operations
- Memory bridge for batch memory access
- Offline memory for trace generation

### 2. Program Management
- `ProgramChip` stores and serves instructions
- Program bus for instruction fetching
- Cached program traces for optimization

### 3. Extension Integration
- Extensions register executors with opcodes
- Phantom instruction support for custom operations
- Shared resources through system port

### 4. Proof System
- Generates `ProofInput` for STARK backend
- Manages AIR ordering and trace alignment
- Handles public values and continuations

## Configuration Options

### System Configuration
```rust
SystemConfig {
    max_constraint_degree: usize,      // Max polynomial degree
    continuation_enabled: bool,         // Multi-segment support
    memory_config: MemoryConfig,        // Memory parameters
    num_public_values: usize,          // Public output slots
    profiling: bool,                   // Performance metrics
    segmentation_strategy: Arc<dyn>,   // Segment control
}
```

### Memory Configuration
```rust
MemoryConfig {
    as_height: usize,         // Address space height (2^n addresses)
    as_offset: u32,          // Address space offset (usually 1)
    pointer_max_bits: usize, // Max pointer size
    clk_max_bits: usize,     // Max timestamp bits
    decomp: usize,           // Range check decomposition
    max_access_adapter_n: usize, // Max adapter size
    access_capacity: usize,  // Expected memory accesses
}
```

## Performance Considerations

1. **Trace Generation**: Parallel trace row generation using rayon
2. **Segmentation**: Dynamic segmentation based on trace cells
3. **Memory Access**: Batch memory operations through adapters
4. **Caching**: Program trace caching for repeated proofs

## Error Handling

### Execution Errors
- `PcNotFound`: Invalid program counter
- `DisabledOperation`: Unsupported opcode
- `PublicValueIndexOutOfBounds`: Invalid public value access
- `DidNotTerminate`: Program didn't complete

### Generation Errors
- `TraceHeightsLimitExceeded`: Segment too large
- `Execution`: Wrapped execution errors

### Verification Errors
- `ProgramCommitMismatch`: Different programs in segments
- `InitialMemoryRootMismatch`: Continuation state mismatch
- `IsTerminateMismatch`: Unexpected termination

## Usage Examples

### Basic VM Setup
```rust
let config = SystemConfig::default().with_continuations();
let engine = BabyBearBlake3Engine::new(proving_config);
let vm = VirtualMachine::new(engine, config);
```

### Extension Integration
```rust
impl VmExtension<F> for MyExtension {
    type Executor = MyExecutor<F>;
    type Periphery = MyPeriphery<F>;
    
    fn build(&self, builder: &mut VmInventoryBuilder<F>) -> Result<...> {
        let executor = MyExecutor::new(builder.system_port());
        builder.add_executor(executor, [MY_OPCODE])?;
        Ok(VmInventory::new())
    }
}
```

### Segment Execution
```rust
let segments = vm.executor.execute_segments(exe, input)?;
for segment in segments {
    let proof_input = segment.generate_proof_input(cached_program)?;
    // Generate proof...
}
```

## Testing Support

The component provides testing utilities in the `testing` module:
- Mock execution environments
- Test adapters and cores
- Trace verification helpers

## Dependencies

- `openvm-instructions`: Instruction definitions
- `openvm-circuit`: Circuit primitives
- `openvm-stark-backend`: STARK proving system
- `p3-baby-bear`: Field implementation
- Standard library collections and synchronization