# Keygen Implementation Guide

## Overview
This guide explains how to implement custom key generation logic or extend the existing keygen system in OpenVM.

## Core Concepts

### Proving Key Hierarchy
OpenVM uses a multi-level proof system where each level has its own proving key:

```
App VM → Leaf Verifier → Internal Verifier → Root Verifier → [Static Verifier]
```

Each arrow represents a proof verification relationship.

## Implementing Custom Key Generation

### Step 1: Define Your VM Configuration
```rust
use openvm_circuit::arch::VmConfig;

#[derive(Clone, Debug)]
pub struct MyVmConfig {
    // Your VM-specific configuration
    pub num_registers: usize,
    pub memory_size: usize,
}

impl VmConfig<F> for MyVmConfig {
    type Executor = MyExecutor;
    type Periphery = MyPeriphery;
    
    fn system(&self) -> &SystemConfig {
        // Return system configuration
    }
}
```

### Step 2: Create Proving Key Structure
```rust
use openvm_sdk::keygen::VmProvingKey;
use std::sync::Arc;

#[derive(Clone, Serialize, Deserialize)]
pub struct MyProvingKey {
    pub vm_pk: Arc<VmProvingKey<SC, MyVmConfig>>,
    pub committed_exe: Arc<VmCommittedExe<SC>>,
    pub custom_data: MyCustomData,
}
```

### Step 3: Implement Key Generation
```rust
impl MyProvingKey {
    pub fn keygen(config: MyConfig) -> Self {
        // 1. Create the engine
        let engine = BabyBearPoseidon2Engine::new(config.fri_params);
        
        // 2. Create and keygen the VM
        let vm = VirtualMachine::new(engine, config.vm_config.clone());
        let vm_pk = vm.keygen();
        
        // 3. Validate constraints
        assert!(
            vm_pk.max_constraint_degree <= config.fri_params.max_constraint_degree()
        );
        
        // 4. Check recursive verifier size if needed
        check_recursive_verifier_size(
            &vm_pk.get_vk(),
            config.fri_params,
            next_log_blowup,
        );
        
        // 5. Build and commit any programs
        let program = build_my_program(&config);
        let committed_exe = Arc::new(VmCommittedExe::commit(
            program.into(),
            engine.config.pcs(),
        ));
        
        // 6. Return the proving key
        Self {
            vm_pk: Arc::new(VmProvingKey {
                fri_params: config.fri_params,
                vm_config: config.vm_config,
                vm_pk,
            }),
            committed_exe,
            custom_data: generate_custom_data(&config),
        }
    }
}
```

## Extending AIR Permutation

### Custom Permutation Strategy
```rust
use openvm_sdk::keygen::perm::AirIdPermutation;

pub struct CustomPermutation {
    strategy: PermutationStrategy,
}

impl CustomPermutation {
    pub fn compute_custom(heights: &[usize], priorities: &[u32]) -> AirIdPermutation {
        let mut indexed: Vec<_> = heights.iter()
            .enumerate()
            .zip(priorities)
            .map(|((id, &height), &priority)| (id, height, priority))
            .collect();
        
        // Sort by custom criteria
        indexed.sort_by_key(|(_, h, p)| (Reverse(*p), Reverse(*h)));
        
        AirIdPermutation {
            perm: indexed.into_iter().map(|(id, _, _)| id).collect(),
        }
    }
}
```

## Implementing Dummy Proof Generation

### Basic Pattern
```rust
pub fn generate_dummy_proof<VC: VmConfig<F>>(
    vm_pk: Arc<VmProvingKey<SC, VC>>,
) -> Proof<SC>
where
    VC::Executor: Chip<SC>,
    VC::Periphery: Chip<SC>,
{
    // 1. Create minimal program
    let program = Program::from_instructions(&[
        Instruction::from_isize(TERMINATE.global_opcode(), 0, 0, 0, 0, 0)
    ]);
    
    // 2. Commit the program
    let engine = BabyBearPoseidon2Engine::new(vm_pk.fri_params);
    let exe = Arc::new(VmCommittedExe::commit(
        program.into(),
        engine.config.pcs(),
    ));
    
    // 3. Execute to get trace heights
    let executor = VmExecutor::new(vm_pk.vm_config.clone());
    let mut results = executor.execute_segments(exe.exe.clone(), vec![]).unwrap();
    let mut result = results.pop().unwrap();
    result.chip_complex.finalize_memory();
    
    // 4. Get and round heights
    let mut vm_heights = result.chip_complex.get_internal_trace_heights();
    vm_heights.round_to_next_power_of_two();
    
    // 5. Create prover with overridden heights
    let prover = VmLocalProver::new_with_overridden_trace_heights(
        vm_pk,
        exe,
        Some(vm_heights),
    );
    
    // 6. Generate proof
    SingleSegmentVmProver::prove(&prover, vec![])
}
```

## Optimizing Key Generation

### Memory-Efficient Generation
```rust
pub struct StreamingKeyGenerator {
    config: KeyGenConfig,
    buffer_size: usize,
}

impl StreamingKeyGenerator {
    pub fn generate_keys(&self) -> Result<()> {
        // 1. Generate keys in chunks
        for chunk in self.config.air_configs.chunks(self.buffer_size) {
            let chunk_keys = self.generate_chunk(chunk)?;
            self.write_to_disk(&chunk_keys)?;
        }
        
        // 2. Merge chunks
        self.merge_chunks()
    }
    
    fn generate_chunk(&self, configs: &[AirConfig]) -> Result<Vec<AirProvingKey>> {
        configs.par_iter()
            .map(|config| self.generate_single_air_key(config))
            .collect()
    }
}
```

### Parallel Key Generation
```rust
use rayon::prelude::*;

pub fn parallel_keygen(configs: Vec<VmConfig>) -> Vec<ProvingKey> {
    configs.into_par_iter()
        .map(|config| {
            // Each thread generates its own keys
            let engine = BabyBearPoseidon2Engine::new(config.fri_params);
            let vm = VirtualMachine::new(engine, config);
            vm.keygen()
        })
        .collect()
}
```

## Handling Special Cases

### Variable Trace Heights
```rust
pub fn handle_variable_heights(
    vm_config: &VmConfig,
    exe: &VmExe<F>,
) -> VmComplexTraceHeights {
    // 1. Run multiple executions with different inputs
    let test_inputs = generate_test_inputs();
    let mut max_heights = VmComplexTraceHeights::default();
    
    for input in test_inputs {
        let executor = VmExecutor::new(vm_config.clone());
        let results = executor.execute_segments(exe.clone(), input).unwrap();
        
        for result in results {
            let heights = result.chip_complex.get_internal_trace_heights();
            max_heights.merge_max(&heights);
        }
    }
    
    // 2. Add safety margin and round
    max_heights.scale_by(1.2);
    max_heights.round_to_next_power_of_two();
    
    max_heights
}
```

### Custom Assembly Generation
```rust
pub fn custom_asm_generator(program: Program<F>) -> String {
    let mut output = String::new();
    
    // 1. Add custom header
    output.push_str(".section .text\n");
    output.push_str(".global _start\n");
    output.push_str("_start:\n");
    
    // 2. Convert instructions
    for (idx, instruction) in program.instructions.iter().enumerate() {
        let asm_line = convert_instruction_to_asm(instruction, idx);
        output.push_str(&asm_line);
        output.push('\n');
    }
    
    // 3. Add custom footer
    output.push_str("\n.section .data\n");
    
    output
}
```

## Testing Key Generation

### Unit Test Template
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_key_generation() {
        // 1. Create minimal config
        let config = TestConfig::minimal();
        
        // 2. Generate keys
        let pk = MyProvingKey::keygen(config);
        
        // 3. Verify key properties
        assert!(pk.vm_pk.vm_pk.per_air.len() > 0);
        assert_eq!(
            pk.vm_pk.fri_params.max_constraint_degree(),
            expected_degree
        );
        
        // 4. Test serialization
        let serialized = bincode::serialize(&pk).unwrap();
        let deserialized: MyProvingKey = bincode::deserialize(&serialized).unwrap();
        
        // 5. Verify consistency
        assert_eq!(
            pk.committed_exe.get_program_commit(),
            deserialized.committed_exe.get_program_commit()
        );
    }
}
```

## Common Patterns

### Lazy Key Loading
```rust
pub struct LazyProvingKey {
    path: PathBuf,
    cached: OnceCell<Arc<ProvingKey>>,
}

impl LazyProvingKey {
    pub fn get(&self) -> Result<Arc<ProvingKey>> {
        self.cached.get_or_try_init(|| {
            let bytes = std::fs::read(&self.path)?;
            let pk = bincode::deserialize(&bytes)?;
            Ok(Arc::new(pk))
        }).cloned()
    }
}
```

### Key Validation
```rust
pub fn validate_proving_key(pk: &ProvingKey) -> Result<()> {
    // 1. Check constraint degrees
    for air_pk in &pk.vm_pk.per_air {
        if air_pk.preprocessed_degree > MAX_DEGREE {
            return Err("Preprocessed degree too high");
        }
    }
    
    // 2. Verify commitments
    let recomputed = compute_commitment(&pk.program);
    if recomputed != pk.commitment {
        return Err("Commitment mismatch");
    }
    
    // 3. Check FRI parameters
    validate_fri_params(&pk.fri_params)?;
    
    Ok(())
}
```

## Best Practices

1. **Always validate constraint degrees** against FRI parameters
2. **Use Arc for large structures** to enable sharing without copying
3. **Generate dummy proofs** to validate configurations
4. **Round trace heights** to powers of 2 for efficiency
5. **Check recursive verifier sizes** to prevent soundness issues
6. **Cache generated keys** to avoid regeneration
7. **Use parallel generation** when generating multiple keys
8. **Implement proper error handling** for memory allocation failures