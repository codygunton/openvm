# OpenVM Modular Arithmetic Circuit Builder - Integration Guide

This document provides comprehensive guidelines for integrating the `openvm-mod-circuit-builder` with other OpenVM components and external systems.

## Core Integration Architecture

### With OpenVM Core System

The mod-builder integrates with OpenVM's core architecture through several key interfaces:

#### 1. VmCoreAir Integration

```rust
use openvm_circuit::arch::{VmCoreAir, VmCoreChip};
use openvm_mod_circuit_builder::*;

// Create a chip that uses modular arithmetic
pub struct MyModularChip {
    pub air: FieldExpressionCoreAir,
    // Other chip-specific data
}

impl VmCoreChip for MyModularChip {
    type Air = FieldExpressionCoreAir;
    type Interface = MyModularInterface;

    fn air(&self) -> &Self::Air {
        &self.air
    }
}
```

#### 2. Instruction Integration

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModularArithmeticInstruction {
    pub opcode: ModularOpcode,
    pub operands: Vec<u32>,
}

impl Instruction for ModularArithmeticInstruction {
    const NUM_PHANTOM: usize = 0;
    // Implementation details...
}
```

### With Range Checker Component

The mod-builder requires tight integration with the range checker for overflow validation:

#### Setup and Configuration

```rust
use openvm_circuit_primitives::var_range::{
    VariableRangeCheckerBus, VariableRangeCheckerChip
};

// Coordinate range checker configuration
let range_bus_id = 1; // Must be unique across all components
let range_decomp_bits = 17; // Must accommodate max overflow bits

let range_checker = Arc::new(VariableRangeCheckerChip::new(
    VariableRangeCheckerBus::new(range_bus_id, range_decomp_bits)
));

// Builder must use compatible range bits
let builder = ExprBuilder::new(config, range_checker.range_max_bits());
```

#### Trace Coordination

```rust
// Generate traces in coordination
let main_trace = expr.generate_trace(inputs, flags);
let range_trace = range_checker.generate_trace();

// Both traces must be included in proving
let traces = vec![main_trace, range_trace];
```

## Memory Management Integration

### Limb Layout Coordination

When integrating with memory-intensive operations, coordinate limb layouts:

```rust
pub struct SharedLimbConfig {
    pub limb_bits: usize,      // Usually 8 bits
    pub num_limbs: usize,      // Field-dependent
    pub limb_alignment: usize, // Memory alignment requirements
}

// Ensure all components use compatible limb sizes
impl SharedLimbConfig {
    pub fn for_field(modulus: &BigUint) -> Self {
        let limb_bits = 8; // Standard choice
        let num_limbs = (modulus.bits() as usize + limb_bits - 1) / limb_bits;
        Self {
            limb_bits,
            num_limbs,
            limb_alignment: 4, // Word alignment
        }
    }
}
```

### Memory Trace Integration

```rust
use openvm_circuit::system::memory::MemoryController;

pub struct MemoryIntegratedModChip {
    pub mod_air: FieldExpressionCoreAir,
    pub memory_controller: MemoryController,
}

impl MemoryIntegratedModChip {
    pub fn execute_with_memory(&self, instruction: &ModularInstruction) {
        // Load operands from memory
        let operands = self.memory_controller.load_operands(&instruction.addresses);
        
        // Perform modular arithmetic
        let result = self.mod_air.execute(operands);
        
        // Store result back to memory
        self.memory_controller.store_result(&instruction.result_address, result);
    }
}
```

## Bus System Integration

### InteractionBuilder Usage

```rust
use openvm_stark_backend::interaction::InteractionBuilder;

impl<AB: InteractionBuilder> SubAir<AB> for FieldExpressionCoreAir {
    fn eval(&self, builder: &mut AB) {
        // Evaluate modular arithmetic constraints
        self.eval_constraints(builder);
        
        // Interface with other components through buses
        self.range_checker_bus.eval(builder, &self.range_values);
        
        // Optional: Interface with memory bus
        if let Some(memory_bus) = &self.memory_bus {
            memory_bus.eval(builder, &self.memory_interactions);
        }
    }
}
```

### Custom Bus Interfaces

```rust
pub struct ModularArithmeticBus {
    pub bus_id: usize,
    pub input_limbs: usize,
    pub output_limbs: usize,
}

impl ModularArithmeticBus {
    pub fn send_operation<AB: InteractionBuilder>(
        &self,
        builder: &mut AB,
        opcode: AB::Expr,
        inputs: &[AB::Expr],
        outputs: &[AB::Expr],
    ) {
        let interaction = [vec![opcode], inputs.to_vec(), outputs.to_vec()].concat();
        builder.push_send(self.bus_id, interaction, AB::Expr::one());
    }
    
    pub fn receive_operation<AB: InteractionBuilder>(
        &self,
        builder: &mut AB,
        opcode: AB::Expr,
        inputs: &[AB::Expr],
        outputs: &[AB::Expr],
    ) {
        let interaction = [vec![opcode], inputs.to_vec(), outputs.to_vec()].concat();
        builder.push_receive(self.bus_id, interaction, AB::Expr::one());
    }
}
```

## Elliptic Curve Component Integration

### Point Representation Coordination

```rust
pub struct EcPointLimbs<const NUM_LIMBS: usize> {
    pub x: [u32; NUM_LIMBS],
    pub y: [u32; NUM_LIMBS],
    pub is_infinity: bool,
}

// Ensure consistent point representation across EC operations
pub trait EcPointProvider {
    const LIMB_BITS: usize;
    const NUM_LIMBS: usize;
    
    fn to_limbs(&self, point: &EcPoint) -> EcPointLimbs<{ Self::NUM_LIMBS }>;
    fn from_limbs(&self, limbs: &EcPointLimbs<{ Self::NUM_LIMBS }>) -> EcPoint;
}
```

### Coordinated Field Operations

```rust
pub struct CurveFieldCoordinator {
    pub base_field_builder: Rc<RefCell<ExprBuilder>>, // For coordinate field
    pub scalar_field_builder: Rc<RefCell<ExprBuilder>>, // For scalar field
}

impl CurveFieldCoordinator {
    pub fn new(curve_params: &CurveParameters) -> Self {
        let base_config = ExprBuilderConfig {
            modulus: curve_params.base_field_modulus.clone(),
            limb_bits: 8,
            num_limbs: 32,
        };
        
        let scalar_config = ExprBuilderConfig {
            modulus: curve_params.scalar_field_modulus.clone(),
            limb_bits: 8,
            num_limbs: 32,
        };
        
        Self {
            base_field_builder: Rc::new(RefCell::new(ExprBuilder::new(base_config, 17))),
            scalar_field_builder: Rc::new(RefCell::new(ExprBuilder::new(scalar_config, 17))),
        }
    }
}
```

## Pairing Component Integration

### Tower Field Extensions

```rust
pub struct TowerFieldBuilder {
    pub base_builder: Rc<RefCell<ExprBuilder>>,    // Fq
    pub extension_builders: Vec<Rc<RefCell<ExprBuilder>>>, // Fq2, Fq6, Fq12
}

impl TowerFieldBuilder {
    pub fn for_bn254() -> Self {
        let fq_modulus = bn254_fq_prime();
        let base_config = ExprBuilderConfig {
            modulus: fq_modulus,
            limb_bits: 8,
            num_limbs: 32,
        };
        
        let base_builder = Rc::new(RefCell::new(ExprBuilder::new(base_config.clone(), 17)));
        
        // Extension fields use same limb configuration but different constraints
        let extension_builders = vec![
            base_builder.clone(), // Fq2 coordinates are Fq elements
            base_builder.clone(), // Fq6 coordinates are Fq elements  
            base_builder.clone(), // Fq12 coordinates are Fq elements
        ];
        
        Self { base_builder, extension_builders }
    }
}
```

### Miller Loop Integration

```rust
pub struct MillerLoopAir {
    pub line_evaluation_builder: Rc<RefCell<ExprBuilder>>,
    pub accumulator_update_builder: Rc<RefCell<ExprBuilder>>,
    pub final_exp_builder: Rc<RefCell<ExprBuilder>>,
}

impl MillerLoopAir {
    pub fn coordinate_builders(&mut self) {
        // Ensure all builders use compatible configurations
        let config = self.line_evaluation_builder.borrow().config.clone();
        
        *self.accumulator_update_builder.borrow_mut() = 
            ExprBuilder::new(config.clone(), 17);
        *self.final_exp_builder.borrow_mut() = 
            ExprBuilder::new(config, 17);
    }
}
```

## Testing Integration Patterns

### Multi-Component Test Setup

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    fn setup_integrated_test() -> IntegratedTestEnvironment {
        let range_checker = Arc::new(VariableRangeCheckerChip::new(
            VariableRangeCheckerBus::new(1, 17)
        ));
        
        let memory_controller = MemoryController::new();
        let ec_chip = EcChip::new(range_checker.clone());
        let mod_chip = ModularChip::new(range_checker.clone());
        
        IntegratedTestEnvironment {
            range_checker,
            memory_controller,
            ec_chip,
            mod_chip,
        }
    }
    
    #[test]
    fn test_ec_scalar_multiplication() {
        let env = setup_integrated_test();
        // Test coordinated EC and modular arithmetic operations
    }
}
```

### Trace Verification

```rust
pub fn verify_integrated_traces(
    main_traces: Vec<RowMajorMatrix<BabyBear>>,
    range_trace: RowMajorMatrix<BabyBear>,
    memory_trace: Option<RowMajorMatrix<BabyBear>>,
) -> Result<(), VerificationError> {
    let mut airs = vec![];
    let mut traces = vec![];
    
    // Add all component AIRs and traces
    for (air, trace) in main_traces {
        airs.push(air);
        traces.push(trace);
    }
    
    // Always include range checker
    airs.push(range_checker.air);
    traces.push(range_trace);
    
    // Include memory if present
    if let Some(mem_trace) = memory_trace {
        airs.push(memory_controller.air);
        traces.push(mem_trace);
    }
    
    BabyBearBlake3Engine::run_simple_test_no_pis_fast(airs, traces)
}
```

## Performance Optimization Integration

### Batch Processing

```rust
pub struct BatchedModularOperations {
    pub builder: Rc<RefCell<ExprBuilder>>,
    pub batch_size: usize,
}

impl BatchedModularOperations {
    pub fn execute_batch(&self, operations: &[ModularOperation]) -> Vec<FieldVariable> {
        let mut results = Vec::new();
        
        for batch in operations.chunks(self.batch_size) {
            // Process operations in batches to optimize constraint generation
            let batch_results = self.process_batch(batch);
            results.extend(batch_results);
        }
        
        results
    }
    
    fn process_batch(&self, batch: &[ModularOperation]) -> Vec<FieldVariable> {
        // Batch processing logic with shared subexpression elimination
        let mut builder = self.builder.borrow_mut();
        
        // Identify common subexpressions across batch
        let common_exprs = self.find_common_expressions(batch);
        
        // Execute with reuse
        batch.iter().map(|op| self.execute_with_reuse(op, &common_exprs)).collect()
    }
}
```

### Memory Layout Optimization

```rust
pub struct OptimizedLimbLayout {
    pub cache_line_size: usize,
    pub limbs_per_cache_line: usize,
    pub padding: usize,
}

impl OptimizedLimbLayout {
    pub fn optimize_for_target() -> Self {
        let cache_line_size = 64; // bytes
        let limb_size = 4; // bytes per limb (u32)
        let limbs_per_cache_line = cache_line_size / limb_size;
        
        Self {
            cache_line_size,
            limbs_per_cache_line,
            padding: 0, // Add padding if needed for alignment
        }
    }
    
    pub fn layout_field_elements(&self, num_elements: usize) -> MemoryLayout {
        // Optimize memory layout for cache efficiency
        MemoryLayout::new(num_elements, self.limbs_per_cache_line)
    }
}
```

## Error Handling Integration

### Coordinated Error Reporting

```rust
#[derive(Debug, thiserror::Error)]
pub enum IntegrationError {
    #[error("Range checker configuration mismatch: expected {expected} bits, got {actual}")]
    RangeCheckerMismatch { expected: usize, actual: usize },
    
    #[error("Limb configuration incompatible: {component1} uses {limbs1} limbs, {component2} uses {limbs2}")]
    LimbConfigMismatch {
        component1: String,
        component2: String,
        limbs1: usize,
        limbs2: usize,
    },
    
    #[error("Bus ID conflict: bus {bus_id} already in use")]
    BusIdConflict { bus_id: usize },
    
    #[error("Memory alignment violation: address {address} not aligned to {alignment} bytes")]
    MemoryAlignmentError { address: usize, alignment: usize },
}

pub trait IntegrationValidation {
    fn validate_compatibility(&self, other: &Self) -> Result<(), IntegrationError>;
}
```

### Recovery Strategies

```rust
pub struct IntegrationRecovery {
    pub fallback_configs: Vec<ExprBuilderConfig>,
    pub retry_strategies: Vec<RetryStrategy>,
}

impl IntegrationRecovery {
    pub fn attempt_recovery(&self, error: &IntegrationError) -> Result<ExprBuilder, IntegrationError> {
        match error {
            IntegrationError::RangeCheckerMismatch { .. } => {
                // Try fallback configurations
                for config in &self.fallback_configs {
                    if let Ok(builder) = self.try_config(config) {
                        return Ok(builder);
                    }
                }
                Err(error.clone())
            }
            _ => Err(error.clone()),
        }
    }
}
```

## Best Practices for Integration

### 1. Configuration Management

- Use shared configuration objects to ensure consistency
- Validate compatibility before integration
- Provide clear error messages for misconfigurations

### 2. Resource Coordination

- Coordinate bus IDs to avoid conflicts
- Share range checkers where possible
- Align memory layouts for cache efficiency

### 3. Testing Strategy

- Test individual components before integration
- Use property-based testing for interaction verification
- Include performance benchmarks in integration tests

### 4. Documentation

- Document all integration points clearly
- Provide working examples for common integration patterns
- Maintain compatibility matrices for different versions

### 5. Versioning

- Use semantic versioning for integration APIs
- Provide migration guides for breaking changes
- Maintain backward compatibility where possible

This integration guide ensures that the mod-builder component works seamlessly with the broader OpenVM ecosystem while maintaining performance, correctness, and usability.