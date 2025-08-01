# OpenVM Poseidon2 AIR - Integration Guide

## API Reference

### Core Types

#### Poseidon2SubChip<F, const SBOX_REGISTERS: usize>
The main interface for Poseidon2 operations within OpenVM.

**Generic Parameters:**
- `F: Field` - The finite field type (currently must be BabyBear)
- `SBOX_REGISTERS: usize` - Number of registers for S-box computation (affects constraint degree)

**Key Methods:**
```rust
// Constructor
pub fn new(constants: Poseidon2Constants<F>) -> Self

// Hash operations
pub fn permute(&self, input_state: [F; POSEIDON2_WIDTH]) -> [F; POSEIDON2_WIDTH]
pub fn permute_mut(&self, input_state: &mut [F; POSEIDON2_WIDTH])

// Trace generation for proving
pub fn generate_trace(&self, inputs: Vec<[F; POSEIDON2_WIDTH]>) -> RowMajorMatrix<F>
```

#### Poseidon2Config<F>
Configuration container for Poseidon2 parameters.

```rust
pub struct Poseidon2Config<F> {
    pub constants: Poseidon2Constants<F>,
}

impl<F: PrimeField32> Default for Poseidon2Config<F>
```

#### Poseidon2Constants<F>
Round constants structure for cryptographic operations.

```rust
pub struct Poseidon2Constants<F> {
    pub beginning_full_round_constants: [[F; POSEIDON2_WIDTH]; BABY_BEAR_POSEIDON2_HALF_FULL_ROUNDS],
    pub partial_round_constants: [F; BABY_BEAR_POSAIDON2_PARTIAL_ROUNDS],
    pub ending_full_round_constants: [[F; POSEIDON2_WIDTH]; BABY_BEAR_POSEIDON2_HALF_FULL_ROUNDS],
}
```

### AIR Integration

#### Poseidon2SubAir<F, const SBOX_REGISTERS: usize>
Arithmetic Intermediate Representation for constraint system integration.

**Trait Implementations:**
- `BaseAir<F>` - Core AIR functionality
- `BaseAirWithPublicValues<F>` - Public value handling
- `PartitionedBaseAir<F>` - Partitioned constraint evaluation
- `Air<AB>` - Constraint evaluation with AirBuilder

## Integration Patterns

### 1. Standalone Hash Function

```rust
use openvm_poseidon2_air::{Poseidon2Config, Poseidon2SubChip, POSEIDON2_WIDTH};
use openvm_stark_sdk::p3_baby_bear::BabyBear;

fn hash_data(data: &[u32]) -> BabyBear {
    let config = Poseidon2Config::<BabyBear>::default();
    let hasher = Poseidon2SubChip::<BabyBear, 0>::new(config.constants);
    
    // Prepare input state (pad to 16 elements)
    let mut state = [BabyBear::ZERO; POSEIDON2_WIDTH];
    for (i, &value) in data.iter().take(POSEIDON2_WIDTH).enumerate() {
        state[i] = BabyBear::from_canonical_u32(value);
    }
    
    // Hash and return first element
    hasher.permute(state)[0]
}
```

### 2. STARK Circuit Integration

```rust
use openvm_poseidon2_air::{Poseidon2Config, Poseidon2SubChip};
use openvm_stark_backend::{
    p3_field::Field,
    rap::{AnyRap, BaseAirWithPublicValues, PartitionedBaseAir},
};
use openvm_stark_sdk::p3_baby_bear::BabyBear;
use std::sync::Arc;

pub struct MyCircuit<F: Field> {
    poseidon2_chip: Arc<Poseidon2SubChip<F, 0>>,
    // ... other circuit components
}

impl<F: Field> MyCircuit<F> {
    pub fn new() -> Self {
        let config = Poseidon2Config::<F>::default();
        Self {
            poseidon2_chip: Arc::new(Poseidon2SubChip::new(config.constants)),
        }
    }
    
    pub fn get_airs(&self) -> Vec<Arc<dyn AnyRap<F>>> {
        vec![
            self.poseidon2_chip.air.clone() as Arc<dyn AnyRap<F>>,
            // ... other AIRs
        ]
    }
}
```

### 3. Batch Hash Processing

```rust
use openvm_poseidon2_air::{Poseidon2Config, Poseidon2SubChip, POSEIDON2_WIDTH};
use openvm_stark_sdk::p3_baby_bear::BabyBear;
use openvm_stark_backend::p3_matrix::dense::RowMajorMatrix;

pub struct BatchHasher {
    chip: Poseidon2SubChip<BabyBear, 0>,
}

impl BatchHasher {
    pub fn new() -> Self {
        let config = Poseidon2Config::<BabyBear>::default();
        Self {
            chip: Poseidon2SubChip::new(config.constants),
        }
    }
    
    pub fn hash_batch(&self, inputs: Vec<[u32; POSEIDON2_WIDTH]>) -> Vec<BabyBear> {
        let field_inputs: Vec<[BabyBear; POSEIDON2_WIDTH]> = inputs
            .into_iter()
            .map(|input| input.map(BabyBear::from_canonical_u32))
            .collect();
            
        field_inputs
            .into_iter()
            .map(|state| self.chip.permute(state)[0])
            .collect()
    }
    
    pub fn generate_proof_trace(&self, inputs: Vec<[BabyBear; POSEIDON2_WIDTH]>) -> RowMajorMatrix<BabyBear> {
        self.chip.generate_trace(inputs)
    }
}
```

## Configuration Guidelines

### SBOX Register Selection

The `SBOX_REGISTERS` parameter affects the trade-off between constraint degree and trace width:

```rust
// High degree, narrow trace (faster proving for small traces)
type FastConfig<F> = Poseidon2SubChip<F, 0>;

// Lower degree, wider trace (better for large traces)  
type WideConfig<F> = Poseidon2SubChip<F, 1>;

// Even lower degree, even wider trace
type ExtraWideConfig<F> = Poseidon2SubChip<F, 2>;
```

**Guidelines:**
- Use `SBOX_REGISTERS = 0` for small traces (< 1000 rows)
- Use `SBOX_REGISTERS = 1` for medium traces (1000-10000 rows)
- Use `SBOX_REGISTERS = 2+` for large traces (> 10000 rows)

### Performance Optimization

#### Memory Management
```rust
use std::sync::Arc;

// Share chips across multiple components
let shared_chip = Arc::new(Poseidon2SubChip::<BabyBear, 0>::new(constants));

// Clone Arc, not the chip itself
let chip_ref1 = Arc::clone(&shared_chip);
let chip_ref2 = Arc::clone(&shared_chip);
```

#### Batch Processing
```rust
// Efficient: Process multiple inputs in one trace generation
let inputs = vec![state1, state2, state3, /* ... */];
let trace = chip.generate_trace(inputs);

// Inefficient: Generate separate traces
// let trace1 = chip.generate_trace(vec![state1]);
// let trace2 = chip.generate_trace(vec![state2]);
```

## Error Handling

### Common Issues and Solutions

#### Field Type Mismatch
```rust
// ❌ Wrong: Using non-BabyBear field
// let chip = Poseidon2SubChip::<SomeOtherField, 0>::new(constants);

// ✅ Correct: Using BabyBear field
let chip = Poseidon2SubChip::<BabyBear, 0>::new(constants);
```

#### Invalid Input Size
```rust
use openvm_poseidon2_air::POSEIDON2_WIDTH;

fn safe_hash(data: &[u32]) -> Result<BabyBear, &'static str> {
    if data.len() > POSEIDON2_WIDTH {
        return Err("Input data too large for Poseidon2 width");
    }
    
    let mut state = [BabyBear::ZERO; POSEIDON2_WIDTH];
    for (i, &value) in data.iter().enumerate() {
        state[i] = BabyBear::from_canonical_u32(value);
    }
    
    let chip = Poseidon2SubChip::<BabyBear, 0>::new(
        Poseidon2Config::default().constants
    );
    Ok(chip.permute(state)[0])
}
```

### Verification Failures

When STARK verification fails, check:

1. **Trace consistency**: Ensure trace matches the execution
2. **Constraint satisfaction**: All AIR constraints must be satisfied
3. **Field arithmetic**: Verify no overflow/underflow in field operations
4. **Round constants**: Ensure correct constants are used

```rust
use openvm_stark_backend::verifier::VerificationError;

match verification_result {
    Err(VerificationError::OodEvaluationMismatch) => {
        println!("Constraint evaluation mismatch - check trace generation");
    }
    Err(VerificationError::InvalidProof) => {
        println!("Invalid proof structure - check proving setup");
    }
    Ok(_) => println!("Verification successful"),
}
```

## Dependencies and Versioning

### Required Dependencies

Add to your `Cargo.toml`:

```toml
[dependencies]
openvm-poseidon2-air = { workspace = true }
openvm-stark-backend = { workspace = true }  
openvm-stark-sdk = { workspace = true }

# Optional: for custom round constants
p3-poseidon2 = { workspace = true }
p3-symmetric = { workspace = true }
```

### Version Compatibility

- **OpenVM Version**: 1.3.0+
- **Rust MSRV**: 1.82+
- **Plonky3**: Compatible with workspace version
- **Field Support**: BabyBear only (p = 2^31 - 2^27 + 1)

## Security Considerations

### Cryptographic Safety

1. **Round Constants**: Always use standard constants or properly generated random constants
2. **Field Operations**: Ensure no wraparound or reduction errors
3. **Input Validation**: Validate all inputs are within field bounds
4. **Constant-Time**: Be aware of timing attack considerations

### Best Practices

```rust
// ✅ Good: Use default constants for production
let config = Poseidon2Config::<BabyBear>::default();

// ⚠️ Caution: Only use custom constants for testing
let custom_constants = generate_test_constants();

// ✅ Good: Validate field elements
fn safe_from_u32(value: u32) -> Option<BabyBear> {
    if value < BabyBear::ORDER_U32 {
        Some(BabyBear::from_canonical_u32(value))
    } else {
        None
    }
}
```

## Advanced Integration

### Custom Linear Layers
For future field extensions, implement the `GenericPoseidon2LinearLayers` trait:

```rust
use p3_poseidon2::GenericPoseidon2LinearLayers;

impl<FA: FieldAlgebra> GenericPoseidon2LinearLayers<FA, WIDTH> for CustomLinearLayers {
    fn internal_linear_layer(state: &mut [FA; WIDTH]) {
        // Custom internal linear transformation
    }
    
    fn external_linear_layer(state: &mut [FA; WIDTH]) {
        // Custom external linear transformation  
    }
}
```

### Multi-Field Support
Prepare for future multi-field support:

```rust
trait HashProvider<F: Field> {
    fn hash(&self, input: [F; POSEIDON2_WIDTH]) -> [F; POSEIDON2_WIDTH];
}

impl HashProvider<BabyBear> for Poseidon2SubChip<BabyBear, 0> {
    fn hash(&self, input: [BabyBear; POSEIDON2_WIDTH]) -> [BabyBear; POSEIDON2_WIDTH] {
        self.permute(input)
    }
}
```

This design pattern will facilitate adding support for additional fields in the future while maintaining API compatibility.