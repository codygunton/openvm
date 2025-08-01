# OpenVM Toolchain Component Index

## Component Location
`crates/toolchain/openvm/src/`

## Purpose
Standard library for OpenVM guest programs providing runtime, I/O, serialization, and platform abstractions for zkVM execution.

## Key Files

### Core Runtime
- **lib.rs** - Main entry point, runtime setup, entry! macro
- **process.rs** - Exit and panic functions
- **memcpy.s** - Assembly-optimized memory copy
- **memset.s** - Assembly-optimized memory set

### I/O Subsystem
- **io/mod.rs** - I/O operations, hint reading, output revelation
- **io/read.rs** - Reader implementation for deserialization

### Serialization
- **serde/** - Word-aligned serialization subsystem (see serde/AI_INDEX.md)

### Platform Support
- **getrandom.rs** - Random number generation for zkVM
- **pal_abi.rs** - Platform abstraction layer ABI
- **host.rs** - Host-side implementations for testing
- **utils.rs** - Non-zkVM utility functions

## Public API

### Macros
- `entry!(main)` - Define guest program entry point
- `init!()` - Include generated initialization code
- `print!`/`println!` - Debug output (re-exported)

### I/O Functions
- `io::read<T>() -> T` - Read and deserialize input
- `io::read_vec() -> Vec<u8>` - Read variable-length bytes
- `io::read_u32() -> u32` - Read single word
- `io::hint_load_by_key(&[u8])` - Load hints by key
- `io::reveal_bytes32([u8; 32])` - Reveal 32-byte output
- `io::reveal_u32(u32, usize)` - Reveal word at index
- `io::store_u32_to_native(u32, u32)` - Store to native address

### Process Control
- `process::exit()` - Exit with code 0
- `process::panic()` - Exit with code 1

### Utilities
- `memory_barrier<T>(*const T)` - Memory ordering barrier

### Re-exports
- `openvm_platform` as `platform`
- All of `openvm_rv32im_guest` (when target_os = "zkvm")

## Key Features
- No-std by default with optional std support
- Target-specific compilation (zkVM vs host)
- Word-aligned I/O operations
- Custom entry point and runtime setup
- Configurable heap allocators
- Assembly-optimized memory operations

## Configuration

### Features
- `default` = ["getrandom-unsupported"]
- `getrandom-unsupported` - Error-only getrandom backend
- `heap-embedded-alloc` - Linked-list allocator with free
- `std` - Standard library support

### Target-Specific Behavior
- `target_os = "zkvm"` - Full zkVM functionality
- Other targets - Host-side implementations for testing

## Dependencies
- `openvm-platform` (with rust-runtime)
- `openvm-custom-insn`
- `openvm-rv32im-guest`
- `serde` (with alloc)
- `bytemuck` (with extern_crate_alloc)
- `getrandom` (optional, zkVM only)

## Usage Context
Core dependency for all OpenVM guest programs:
- Entry point setup
- Runtime initialization
- Guest-host communication
- Debug output
- Program termination

## Testing
Supports dual-target compilation enabling:
- Host-side unit tests
- zkVM integration tests
- Mock hint injection
- Debug output capture

## Related Components
- `openvm-platform` - Platform constants and runtime
- `openvm-rv32im-guest` - RISC-V guest instructions
- `openvm-sdk` - Higher-level SDK functionality
- Guest program crates using this as dependency