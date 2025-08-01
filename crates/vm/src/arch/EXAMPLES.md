# OpenVM Virtual Machine Architecture - Examples

This document provides practical examples of using the OpenVM architecture component for building custom virtual machine extensions and instruction executors.

## Basic Setup

```rust
use openvm_circuit::arch::{
    VmCore, VmConfig, VmExtension, SystemPort, InstructionExecutor,
    MemoryController, ExecutionError
};
use openvm_stark_backend::p3_field::PrimeField;

// Basic VM configuration
let config = VmConfig {
    system: Default::default(),
    extensions: vec![],
};
```

## Example 1: Simple Arithmetic Executor

```rust
use openvm_instructions::{instruction::Instruction, riscv::RV32_REGISTER_NUM_LIMBS};

pub struct SimpleArithmeticExecutor;

impl<F: PrimeField> InstructionExecutor<F> for SimpleArithmeticExecutor {
    fn execute(
        &mut self,
        instruction: &Instruction,
        from_pc: u32,
    ) -> Result<(), ExecutionError> {
        // Get instruction components
        let opcode = instruction.opcode;
        let rs1 = instruction.c;
        let rs2 = instruction.b;
        let rd = instruction.a;
        
        // Access memory controller
        let memory = &mut self.memory_controller;
        
        // Read operands
        let val1 = memory.unsafe_read_cell(rs1, memory.timestamp())?;
        let val2 = memory.unsafe_read_cell(rs2, memory.timestamp())?;
        memory.increment_timestamp();
        
        // Perform operation based on opcode
        let result = match opcode {
            0x1000 => val1.wrapping_add(val2), // Custom ADD
            0x1001 => val1.wrapping_sub(val2), // Custom SUB
            0x1002 => val1.wrapping_mul(val2), // Custom MUL
            _ => return Err(ExecutionError::InvalidOpcode(opcode)),
        };
        
        // Write result
        memory.unsafe_write_cell(rd, result, memory.timestamp())?;
        memory.increment_timestamp();
        
        Ok(())
    }
    
    fn get_opcode(&self) -> u32 {
        // This executor handles multiple opcodes
        0x1000 // Base opcode
    }
}
```

## Example 2: Memory-Intensive Executor with Adapter Pattern

```rust
use openvm_circuit::arch::{VmChipWrapper, AdapterInterface};

pub struct MemoryIntensiveCore;
pub struct MemoryIntensiveAdapter;

// Core implementation (pure computation)
impl MemoryIntensiveCore {
    pub fn process_data(&self, data: &[u32]) -> Vec<u32> {
        // Complex computation without memory side effects
        data.iter().map(|x| x.wrapping_mul(2).wrapping_add(1)).collect()
    }
}

// Adapter implementation (memory access)
impl<F: PrimeField> AdapterInterface<F> for MemoryIntensiveAdapter {
    type Reads = Vec<u32>;
    type Writes = Vec<u32>;
    
    fn reads(&self, instruction: &Instruction) -> Self::Reads {
        // Define what memory locations to read
        let base_addr = instruction.b;
        let length = instruction.c;
        (0..length).map(|i| base_addr + i).collect()
    }
    
    fn writes(&self, instruction: &Instruction) -> Self::Writes {
        // Define what memory locations to write
        let base_addr = instruction.a;
        let length = instruction.c;
        (0..length).map(|i| base_addr + i).collect()
    }
    
    fn process_instruction(
        &mut self,
        instruction: &Instruction,
        memory_data: &[u32],
    ) -> Result<Vec<u32>, ExecutionError> {
        let core = MemoryIntensiveCore;
        Ok(core.process_data(memory_data))
    }
}

// Combine adapter and core
pub struct MemoryIntensiveExecutor;

impl<F: PrimeField> VmChipWrapper<F> for MemoryIntensiveExecutor {
    type Adapter = MemoryIntensiveAdapter;
    type Core = MemoryIntensiveCore;
    
    fn new(adapter: Self::Adapter, core: Self::Core) -> Self {
        MemoryIntensiveExecutor
    }
}
```

## Example 3: Creating a Custom VM Extension

```rust
use openvm_circuit::arch::{VmExtension, VmExtensionOutput, SystemPort};

pub struct CustomArithmeticExtension;

impl<F: PrimeField> VmExtension<F> for CustomArithmeticExtension {
    fn build(
        &self,
        system: &dyn SystemPort<F>,
    ) -> Result<VmExtensionOutput<F>, Box<dyn std::error::Error>> {
        // Create executors
        let arithmetic_executor = SimpleArithmeticExecutor::new();
        let memory_executor = MemoryIntensiveExecutor::new(
            MemoryIntensiveAdapter,
            MemoryIntensiveCore,
        );
        
        // Register executors with their opcodes
        let mut executors = Vec::new();
        
        // Register multiple opcodes for arithmetic executor
        for opcode in 0x1000..=0x1002 {
            executors.push((opcode, Box::new(arithmetic_executor.clone())));
        }
        
        // Register memory-intensive operations
        executors.push((0x2000, Box::new(memory_executor)));
        
        Ok(VmExtensionOutput {
            executors,
            periphery_chips: vec![], // No additional chips needed
        })
    }
}
```

## Example 4: VM Setup and Execution

```rust
use openvm_circuit::arch::{VmCore, VmConfig};

fn setup_custom_vm() -> Result<VmCore<F>, Box<dyn std::error::Error>> {
    // Create VM configuration
    let mut config = VmConfig::default();
    
    // Add custom extensions
    config.extensions.push(Box::new(CustomArithmeticExtension));
    
    // You might also add standard extensions
    config.extensions.push(Box::new(RV32IExtension));
    config.extensions.push(Box::new(SystemExtension));
    
    // Build the VM
    let vm = VmCore::build(config)?;
    Ok(vm)
}

fn execute_program(vm: &mut VmCore<F>, program: &[Instruction]) -> Result<(), ExecutionError> {
    // Load program into memory
    vm.load_program(program)?;
    
    // Execute until completion or error
    loop {
        match vm.step() {
            Ok(true) => continue,   // Continue execution
            Ok(false) => break,     // Program completed
            Err(e) => return Err(e), // Execution error
        }
    }
    
    Ok(())
}
```

## Example 5: Advanced Memory Access Patterns

```rust
pub struct BatchMemoryExecutor {
    memory_controller: MemoryController,
}

impl<F: PrimeField> InstructionExecutor<F> for BatchMemoryExecutor {
    fn execute(
        &mut self,
        instruction: &Instruction,
        from_pc: u32,
    ) -> Result<(), ExecutionError> {
        let base_addr = instruction.b;
        let count = instruction.c as usize;
        let output_addr = instruction.a;
        
        // Batch read multiple values
        let mut values = Vec::with_capacity(count);
        for i in 0..count {
            let addr = base_addr + i as u32;
            let value = self.memory_controller.unsafe_read_cell(
                addr, 
                self.memory_controller.timestamp()
            )?;
            values.push(value);
            self.memory_controller.increment_timestamp();
        }
        
        // Process values (example: sum them)
        let sum = values.iter().fold(0u32, |acc, &x| acc.wrapping_add(x));
        
        // Write result
        self.memory_controller.unsafe_write_cell(
            output_addr,
            sum,
            self.memory_controller.timestamp()
        )?;
        self.memory_controller.increment_timestamp();
        
        Ok(())
    }
    
    fn get_opcode(&self) -> u32 {
        0x3000
    }
}
```

## Example 6: Cross-Extension Communication via Buses

```rust
use openvm_circuit::arch::{ExecutionBus, ProgramBus};

pub struct CommunicatingExecutor {
    execution_bus: ExecutionBus,
    program_bus: ProgramBus,
}

impl<F: PrimeField> InstructionExecutor<F> for CommunicatingExecutor {
    fn execute(
        &mut self,
        instruction: &Instruction,
        from_pc: u32,
    ) -> Result<(), ExecutionError> {
        // Read from execution bus (data from other executors)
        if let Some(bus_data) = self.execution_bus.try_read() {
            // Process data from other components
            self.process_cross_component_data(bus_data)?;
        }
        
        // Perform local computation
        let result = self.local_computation(instruction)?;
        
        // Write to execution bus for other components
        self.execution_bus.write(result)?;
        
        // Update program counter if needed
        if instruction.opcode == 0x4001 { // Conditional jump
            let new_pc = self.calculate_jump_target(instruction)?;
            self.program_bus.set_pc(new_pc)?;
        }
        
        Ok(())
    }
    
    fn get_opcode(&self) -> u32 {
        0x4000
    }
}
```

## Example 7: Debugging and Profiling Integration

```rust
use openvm_circuit::arch::ExecutionTrace;

pub struct DebuggableExecutor {
    trace: ExecutionTrace,
    debug_enabled: bool,
}

impl<F: PrimeField> InstructionExecutor<F> for DebuggableExecutor {
    fn execute(
        &mut self,
        instruction: &Instruction,
        from_pc: u32,
    ) -> Result<(), ExecutionError> {
        if self.debug_enabled {
            // Log execution details
            self.trace.log_instruction(instruction, from_pc);
            
            // Capture memory state before execution
            let pre_state = self.memory_controller.snapshot();
            
            // Execute instruction
            let result = self.execute_internal(instruction, from_pc);
            
            // Capture memory state after execution
            let post_state = self.memory_controller.snapshot();
            
            // Log memory changes
            self.trace.log_memory_diff(&pre_state, &post_state);
            
            result
        } else {
            // Fast path without debugging
            self.execute_internal(instruction, from_pc)
        }
    }
    
    fn get_opcode(&self) -> u32 {
        0x5000
    }
}
```

## Example 8: Segmented Execution for Large Programs

```rust
use openvm_circuit::arch::{ExecutionSegment, SegmentationStrategy};

pub struct SegmentedVm<F: PrimeField> {
    vm_core: VmCore<F>,
    segmentation_strategy: SegmentationStrategy,
}

impl<F: PrimeField> SegmentedVm<F> {
    pub fn execute_with_segmentation(
        &mut self,
        program: &[Instruction],
        max_segment_size: usize,
    ) -> Result<Vec<ExecutionSegment>, ExecutionError> {
        let mut segments = Vec::new();
        let mut current_pc = 0;
        
        while current_pc < program.len() {
            // Create new segment
            let segment_end = std::cmp::min(
                current_pc + max_segment_size,
                program.len()
            );
            
            // Execute segment
            let segment = self.vm_core.execute_segment(
                &program[current_pc..segment_end],
                current_pc as u32,
            )?;
            
            segments.push(segment);
            current_pc = segment_end;
        }
        
        Ok(segments)
    }
}
```

## Testing Examples

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_arithmetic_executor() {
        let mut executor = SimpleArithmeticExecutor::new();
        let instruction = Instruction {
            opcode: 0x1000, // ADD
            a: 10,          // rd = 10
            b: 20,          // rs1 = 20
            c: 30,          // rs2 = 30
            d: 0,           // unused
            e: 0,           // unused
        };
        
        // Setup memory with initial values
        executor.memory_controller.unsafe_write_cell(20, 5, 0).unwrap();
        executor.memory_controller.unsafe_write_cell(30, 3, 1).unwrap();
        
        // Execute instruction
        executor.execute(&instruction, 0).unwrap();
        
        // Check result
        let result = executor.memory_controller.unsafe_read_cell(10, 2).unwrap();
        assert_eq!(result, 8); // 5 + 3 = 8
    }
    
    #[test]
    fn test_custom_vm_extension() {
        let extension = CustomArithmeticExtension;
        let system = MockSystemPort::new();
        
        let output = extension.build(&system).unwrap();
        assert_eq!(output.executors.len(), 4); // 3 arithmetic + 1 memory
    }
}
```

These examples demonstrate the key patterns for working with the OpenVM architecture:

1. **Simple executors** for straightforward operations
2. **Adapter-core pattern** for complex memory access
3. **Extension creation** for modularity
4. **VM setup and execution** workflow
5. **Advanced memory patterns** for optimization
6. **Cross-component communication** via buses
7. **Debugging integration** for development
8. **Segmentation strategies** for large programs
9. **Comprehensive testing** approaches