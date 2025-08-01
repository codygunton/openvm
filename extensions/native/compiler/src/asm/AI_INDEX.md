# ASM Component - AI Index

## Component Overview
The ASM (Assembly) component is a critical part of the OpenVM native compiler infrastructure that translates high-level DSL IR operations into low-level assembly instructions. It provides an efficient compilation pipeline for converting abstract operations into executable assembly code with support for field elements, extension field elements, control flow, memory management, and cryptographic operations.

## Purpose
- Compile DSL IR operations into assembly instructions
- Manage stack-based memory allocation and addressing
- Handle control flow (branches, loops, conditionals)
- Support field and extension field arithmetic
- Integrate cryptographic primitives (Poseidon2, FRI)
- Provide debugging and tracing capabilities

## Key Files

### mod.rs (14 lines)
**Purpose**: Module declaration and public re-exports
- Re-exports all submodules for external access
- Serves as the main entry point for the ASM component

### config.rs (18 lines)
**Purpose**: Configuration types for assembly compilation
- `AsmConfig<F, EF>`: Generic configuration type parameterized by field and extension field
- Implements the `Config` trait for type-safe compilation

### builder.rs (25 lines)
**Purpose**: High-level builder interface for assembly compilation
- `AsmBuilder<F, EF>`: Type alias for Builder with AsmConfig
- `compile_isa()`: Compiles operations into a Program
- `compile_isa_with_options()`: Compilation with custom options (word size, etc.)

### instruction.rs (401 lines)
**Purpose**: Assembly instruction definitions and formatting
- `AsmInstruction<F, EF>`: Comprehensive enum of all assembly instructions
- Field operations: LoadFI, StoreFI, AddF, SubF, MulF, DivF
- Extension operations: LoadEI, StoreEI, AddE, SubE, MulE, DivE
- Control flow: Jump, Bne, Beq, branches with immediates
- Cryptographic: Poseidon2Permute, FriReducedOpening, VerifyBatch
- Debug/utility: PrintF, PrintV, HintBits, CycleTracker
- Formatting methods for human-readable assembly output

### code.rs (69 lines)
**Purpose**: Assembly code structure and organization
- `BasicBlock<F, EF>`: Container for instructions with debug info
- `AssemblyCode<F, EF>`: Complete assembly program with blocks and labels
- Display implementation for pretty-printing assembly code

### utils.rs (35 lines)
**Purpose**: Utility types for assembly compilation
- `IndexTriple<F>`: Represents memory indexing (variable or constant)
- `ValueOrConst<F, EF>`: Discriminated union for values vs constants
- `MemIndex` helper methods for frame pointer calculation

### compiler.rs (1128 lines) - Core Component
**Purpose**: Main compilation engine that transforms IR to assembly
- `AsmCompiler<F, EF>`: Stateful compiler with basic blocks and labels
- Memory layout constants:
  - MEMORY_BITS: 29 (512MB address space)
  - HEAP_START_ADDRESS: 1 << 24
  - STACK_TOP: Below heap for local variables
- Frame pointer methods for Var, Felt, Ext, Ptr types
- `build()`: Main compilation method processing DslIr operations
- Control flow compilation: IfCompiler, ZipForCompiler
- Memory management: alloc(), heap pointer tracking
- Extension field operations with specialized methods
- Assertion and error handling with trap mechanism

## Architecture Patterns

### Memory Layout
```
High Memory (MEMORY_TOP)
├── Heap (grows upward from HEAP_START_ADDRESS)
├── Heap Pointer (HEAP_PTR)
├── Utility Register (A0)
├── Stack (local variables)
└── Low Memory
```

### Frame Pointer Allocation
- Vars: Positions 1, 2, 9, 10, 17, 18... (8n + 1, 8n + 2)
- Felts: Positions 3, 4, 11, 12, 19, 20... (8n + 3, 8n + 4)
- Exts: Positions 5-8, 13-16, 21-24... (8n + 5 through 8n + 8)

### Compilation Flow
1. Initialize with trap block for assertions
2. Process each DslIr operation sequentially
3. Generate appropriate assembly instructions
4. Handle control flow with label management
5. Optimize extension field operations
6. Track debug information throughout

## Integration Points
- Receives: DslIr operations from IR layer
- Produces: AssemblyCode for conversion layer
- Uses: openvm_circuit for Program structure
- Uses: openvm_stark_backend for field types
- Integrates with: Native compiler pipeline

## Design Decisions
- Stack-based allocation for efficiency
- Separate instruction types for fields vs extensions
- Label-based control flow for flexibility
- Debug info preservation for development
- Trap mechanism for runtime assertions
- Specialized extension field arithmetic methods