# SHA256 AIR Integration Guide

## Overview

This guide covers integration patterns, best practices, and common use cases for the SHA256 AIR component within the OpenVM ecosystem and standalone applications.

## Integration Architecture

### Core Dependencies

```rust
[dependencies]
openvm-sha256-air = { workspace = true }
openvm-circuit-primitives = { workspace = true }
openvm-stark-backend = { workspace = true }
sha2 = { version = "0.10", features = ["compress"] }
```

### Basic Integration Pattern

```rust
use openvm_circuit_primitives::bitwise_op_lookup::{
    BitwiseOperationLookupBus, SharedBitwiseOperationLookupChip
};
use openvm_sha256_air::{Sha256Air, generate_trace};
use openvm_stark_backend::interaction::BusIndex;

pub struct Sha256Integration {
    pub sha256_air: Sha256Air,
    pub bitwise_chip: SharedBitwiseOperationLookupChip<8>,
    pub bitwise_bus: BitwiseOperationLookupBus,
}

impl Sha256Integration {
    pub fn new(bitwise_bus_idx: BusIndex, self_bus_idx: BusIndex) -> Self {
        let bitwise_bus = BitwiseOperationLookupBus::new(bitwise_bus_idx);
        let bitwise_chip = SharedBitwiseOperationLookupChip::new(bitwise_bus);
        let sha256_air = Sha256Air::new(bitwise_bus, self_bus_idx);
        
        Self {
            sha256_air,
            bitwise_chip,
            bitwise_bus,
        }
    }
}
```

## OpenVM Framework Integration

### 1. VM Chip Integration

```rust
use openvm_circuit::arch::VirtualMachine;
use openvm_circuit_primitives::SubAir;
use openvm_stark_backend::{Chip, AirRef};
use std::sync::Arc;

pub struct Sha256VmChip {
    sha256_air: Sha256Air,
    bitwise_chip: SharedBitwiseOperationLookupChip<8>,
    execution_records: Vec<Sha256ExecutionRecord>,
}

#[derive(Clone, Debug)]
pub struct Sha256ExecutionRecord {
    pub input_blocks: Vec<[u8; 64]>,
    pub is_last_message: bool,
    pub pc: u32,
    pub cycle: u32,
}

impl<SC: StarkGenericConfig> Chip<SC> for Sha256VmChip
where
    SC::Val: PrimeField32,
{
    fn air(&self) -> AirRef<SC> {
        Arc::new(Sha256VmAir {
            sha256_air: self.sha256_air.clone(),
        })
    }

    fn generate_air_proof_input(self) -> AirProofInput<SC> {
        // Convert execution records to trace format
        let records: Vec<_> = self.execution_records
            .into_iter()
            .flat_map(|record| {
                record.input_blocks.into_iter().enumerate().map(|(i, block)| {
                    let is_last_block = i == record.input_blocks.len() - 1 && record.is_last_message;
                    (block, is_last_block)
                })
            })
            .collect();

        let trace = generate_trace(&self.sha256_air, self.bitwise_chip, records);
        AirProofInput::simple_no_pis(trace)
    }
}
```

### 2. Instruction Set Extension

```rust
use openvm_circuit::arch::{AdapterAirContext, AdapterRuntimeContext};

pub struct Sha256Instruction;

impl Sha256Instruction {
    pub const OPCODE: u32 = 0x1000_0000; // Custom opcode
    
    pub fn execute_sha256_block(
        &self,
        rt: &mut AdapterRuntimeContext,
        input_addr: u32,
        output_addr: u32,
        is_last_block: bool,
    ) -> Result<(), VMError> {
        // Read input block from memory
        let input_block = rt.slice_unsafe(input_addr, 64)?;
        
        // Record execution for trace generation
        let record = Sha256ExecutionRecord {
            input_blocks: vec![input_block.try_into().unwrap()],
            is_last_message: is_last_block,
            pc: rt.pc(),
            cycle: rt.cycle(),
        };
        
        // Store record for later trace generation
        rt.record_sha256_execution(record);
        
        // Compute and write result (or defer to prove time)
        let prev_hash = rt.read_sha256_state();
        let result = Sha256Air::get_block_hash(&prev_hash, input_block);
        rt.write_slice(output_addr, &result.as_bytes())?;
        
        Ok(())
    }
}
```

## Bus Management and Interactions

### 1. Bus Index Management

```rust
pub struct BusManager {
    next_bus_idx: BusIndex,
}

impl BusManager {
    pub fn new() -> Self {
        Self { next_bus_idx: 0 }
    }
    
    pub fn allocate_bus(&mut self) -> BusIndex {
        let bus_idx = self.next_bus_idx;
        self.next_bus_idx += 1;
        bus_idx
    }
}

// Usage in system setup
let mut bus_manager = BusManager::new();
let bitwise_bus_idx = bus_manager.allocate_bus();
let sha256_self_bus_idx = bus_manager.allocate_bus();
let memory_bus_idx = bus_manager.allocate_bus();
```

### 2. Cross-Component Interactions

```rust
pub struct SystemIntegration {
    pub memory_chip: MemoryChip,
    pub sha256_chip: Sha256VmChip,
    pub bitwise_chip: SharedBitwiseOperationLookupChip<8>,
}

impl SystemIntegration {
    pub fn setup_interactions(&mut self) {
        // SHA256 depends on bitwise operations
        self.sha256_chip.set_bitwise_dependency(&self.bitwise_chip);
        
        // Memory interactions for input/output
        self.sha256_chip.set_memory_dependency(&self.memory_chip);
    }
}
```

## Message Padding Integration

### 1. SHA256 Padding Implementation

```rust
pub struct Sha256Padder;

impl Sha256Padder {
    /// Pad message according to SHA256 specification
    pub fn pad_message(message: &[u8]) -> Vec<[u8; 64]> {
        let mut padded = message.to_vec();
        
        // Add the '1' bit (0x80 byte)
        padded.push(0x80);
        
        // Add zero padding
        while (padded.len() % 64) != 56 {
            padded.push(0x00);
        }
        
        // Add length in bits as 64-bit big-endian
        let bit_length = (message.len() as u64) * 8;
        padded.extend_from_slice(&bit_length.to_be_bytes());
        
        // Convert to blocks
        padded.chunks_exact(64)
            .map(|chunk| chunk.try_into().unwrap())
            .collect()
    }
    
    pub fn create_records(message: &[u8]) -> Vec<([u8; 64], bool)> {
        let blocks = Self::pad_message(message);
        blocks.into_iter().enumerate().map(|(i, block)| {
            let is_last = i == blocks.len() - 1;
            (block, is_last)
        }).collect()
    }
}
```

### 2. Streaming Hash Interface

```rust
pub struct StreamingSha256 {
    sha256_air: Sha256Air,
    bitwise_chip: SharedBitwiseOperationLookupChip<8>,
    current_state: [u32; 8],
    blocks: Vec<([u8; 64], bool)>,
}

impl StreamingSha256 {
    pub fn new(sha256_air: Sha256Air, bitwise_chip: SharedBitwiseOperationLookupChip<8>) -> Self {
        Self {
            sha256_air,
            bitwise_chip,
            current_state: openvm_sha256_air::SHA256_H,
            blocks: Vec::new(),
        }
    }
    
    pub fn update(&mut self, data: &[u8]) {
        let records = Sha256Padder::create_records(data);
        self.blocks.extend(records);
    }
    
    pub fn finalize(self) -> ([u32; 8], RowMajorMatrix<impl Field>) {
        let trace = generate_trace(&self.sha256_air, self.bitwise_chip, self.blocks);
        
        // Calculate final hash
        let mut state = openvm_sha256_air::SHA256_H;
        for (block, _) in &self.blocks {
            state = Sha256Air::get_block_hash(&state, *block);
        }
        
        (state, trace)
    }
}
```

## Performance Optimization

### 1. Memory Management

```rust
pub struct Sha256MemoryOptimizer {
    block_buffer: Vec<[u8; 64]>,
    trace_buffer: Option<RowMajorMatrix<impl Field>>,
}

impl Sha256MemoryOptimizer {
    pub fn with_capacity(num_blocks: usize) -> Self {
        Self {
            block_buffer: Vec::with_capacity(num_blocks),
            trace_buffer: None,
        }
    }
    
    pub fn add_block(&mut self, block: [u8; 64], is_last: bool) {
        self.block_buffer.push((block, is_last));
    }
    
    pub fn generate_trace_batch(&mut self, sha256_air: &Sha256Air, bitwise_chip: SharedBitwiseOperationLookupChip<8>) {
        let records = std::mem::take(&mut self.block_buffer);
        self.trace_buffer = Some(generate_trace(sha256_air, bitwise_chip, records));
    }
}
```

### 2. Parallel Processing

```rust
use rayon::prelude::*;

pub fn process_multiple_messages_parallel(
    messages: Vec<Vec<u8>>,
    sha256_air: &Sha256Air,
    bitwise_chip: SharedBitwiseOperationLookupChip<8>,
) -> Vec<([u32; 8], RowMajorMatrix<impl Field>)> {
    messages.into_par_iter().map(|message| {
        let records = Sha256Padder::create_records(&message);
        let trace = generate_trace(sha256_air, bitwise_chip.clone(), records.clone());
        
        let mut hash = openvm_sha256_air::SHA256_H;
        for (block, _) in records {
            hash = Sha256Air::get_block_hash(&hash, block);
        }
        
        (hash, trace)
    }).collect()
}
```

## Testing Integration

### 1. Integration Test Framework

```rust
pub struct Sha256IntegrationTester {
    vm_tester: VmChipTestBuilder,
    known_test_vectors: Vec<(Vec<u8>, [u32; 8])>,
}

impl Sha256IntegrationTester {
    pub fn new() -> Self {
        let mut tester = Self {
            vm_tester: VmChipTestBuilder::default(),
            known_test_vectors: Vec::new(),
        };
        
        // Add standard test vectors
        tester.add_test_vector(b"", [
            0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
            0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
        ]);
        
        tester.add_test_vector(b"abc", [
            0xba7816bf, 0x8f01cfea, 0x414140de, 0x5dae2223,
            0xb00361a0, 0x96177a9c, 0xb410ff61, 0xf20015ad,
        ]);
        
        tester
    }
    
    pub fn test_all_vectors(&self) -> Result<(), TestError> {
        for (message, expected_hash) in &self.known_test_vectors {
            self.test_single_message(message, expected_hash)?;
        }
        Ok(())
    }
    
    fn test_single_message(&self, message: &[u8], expected: &[u32; 8]) -> Result<(), TestError> {
        let bitwise_bus = BitwiseOperationLookupBus::new(0);
        let bitwise_chip = SharedBitwiseOperationLookupChip::new(bitwise_bus);
        let sha256_air = Sha256Air::new(bitwise_bus, 1);
        
        let records = Sha256Padder::create_records(message);
        let chip = Sha256TestChip {
            air: sha256_air,
            bitwise_chip: bitwise_chip.clone(),
            records,
        };
        
        let tester = self.vm_tester.build().load(chip).load(bitwise_chip).finalize();
        tester.simple_test()?;
        
        // Verify hash computation
        let computed = compute_sha256_reference(message);
        assert_eq!(&computed, expected, "Hash mismatch for message: {:?}", message);
        
        Ok(())
    }
}
```

### 2. Fuzzing Integration

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn fuzz_sha256_integration(message in prop::collection::vec(any::<u8>(), 0..1000)) {
        let bitwise_bus = BitwiseOperationLookupBus::new(0);
        let bitwise_chip = SharedBitwiseOperationLookupChip::new(bitwise_bus);
        let sha256_air = Sha256Air::new(bitwise_bus, 1);
        
        let records = Sha256Padder::create_records(&message);
        let trace = generate_trace(&sha256_air, bitwise_chip.clone(), records);
        
        // Verify trace properties
        prop_assert!(trace.height() > 0);
        prop_assert!(trace.width() > 0);
        prop_assert_eq!(trace.height() % 17, 0); // Multiple of block size
        
        // Verify hash computation matches reference
        let our_hash = compute_sha256_with_air(&message, &sha256_air);
        let reference_hash = compute_sha256_reference(&message);
        prop_assert_eq!(our_hash, reference_hash);
    }
}
```

## Error Handling and Debugging

### 1. Comprehensive Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum Sha256IntegrationError {
    #[error("Invalid block size: expected 64 bytes, got {0}")]
    InvalidBlockSize(usize),
    
    #[error("Bus index conflict: {0} already in use")]
    BusIndexConflict(BusIndex),
    
    #[error("Trace generation failed: {0}")]
    TraceGenerationFailed(String),
    
    #[error("Constraint violation in row {row}: {constraint}")]
    ConstraintViolation { row: usize, constraint: String },
    
    #[error("Hash mismatch: expected {expected:?}, got {actual:?}")]
    HashMismatch { expected: [u32; 8], actual: [u32; 8] },
}
```

### 2. Debug Utilities

```rust
pub struct Sha256Debugger {
    trace: RowMajorMatrix<impl Field>,
    sha256_air: Sha256Air,
}

impl Sha256Debugger {
    pub fn validate_trace(&self) -> Result<(), Vec<ConstraintViolation>> {
        let mut violations = Vec::new();
        
        // Check row constraints
        for (row_idx, row) in self.trace.rows().enumerate() {
            if let Err(e) = self.validate_row(row_idx, row) {
                violations.push(e);
            }
        }
        
        // Check transition constraints
        for row_idx in 0..self.trace.height() - 1 {
            if let Err(e) = self.validate_transition(row_idx) {
                violations.push(e);
            }
        }
        
        if violations.is_empty() {
            Ok(())
        } else {
            Err(violations)
        }
    }
    
    pub fn print_trace_summary(&self) {
        println!("SHA256 Trace Summary:");
        println!("  Height: {}", self.trace.height());
        println!("  Width: {}", self.trace.width());
        println!("  Blocks: {}", self.trace.height() / 17);
    }
}
```

## Best Practices

### 1. Resource Management
- Always pair SHA256 AIR with bitwise lookup chip
- Manage bus indices carefully to avoid conflicts
- Pre-allocate trace buffers for known workloads
- Use streaming interfaces for large inputs

### 2. Security Considerations
- Validate all input block sizes
- Ensure proper message padding
- Test constraint violations with malformed inputs
- Use constant-time operations where applicable

### 3. Performance Tips
- Batch multiple messages when possible
- Use parallel trace generation for independent messages
- Cache intermediate states for incremental hashing
- Profile memory usage with large traces

### 4. Integration Testing
- Test with standard SHA256 test vectors
- Include edge cases (empty messages, single blocks)
- Verify cross-component interactions
- Use property-based testing for comprehensive coverage

This integration guide provides the foundation for successfully incorporating the SHA256 AIR component into larger systems while maintaining security, performance, and correctness.