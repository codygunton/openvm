# OpenVM Poseidon2 AIR - Usage Examples

## Basic Usage

### Creating a Poseidon2SubChip with Default Configuration

```rust
use openvm_poseidon2_air::{Poseidon2Config, Poseidon2SubChip};
use openvm_stark_sdk::p3_baby_bear::BabyBear;
use std::sync::Arc;

// Create a default configuration (uses standard BabyBear round constants)
let config = Poseidon2Config::<BabyBear>::default();

// Initialize the Poseidon2SubChip with 0 SBOX registers
let subchip = Arc::new(Poseidon2SubChip::<BabyBear, 0>::new(config.constants));

println!("Poseidon2SubChip initialized with width: {}", 16);
```

### Performing Hash Operations

```rust
use openvm_poseidon2_air::{Poseidon2Config, Poseidon2SubChip, POSEIDON2_WIDTH};
use openvm_stark_sdk::p3_baby_bear::BabyBear;
use openvm_stark_backend::p3_field::Field;

let config = Poseidon2Config::<BabyBear>::default();
let subchip = Poseidon2SubChip::<BabyBear, 0>::new(config.constants);

// Create input state (16 BabyBear elements)
let input_state: [BabyBear; POSEIDON2_WIDTH] = [
    BabyBear::from_canonical_u32(1),
    BabyBear::from_canonical_u32(2), 
    BabyBear::from_canonical_u32(3),
    BabyBear::ZERO, BabyBear::ZERO, BabyBear::ZERO, BabyBear::ZERO,
    BabyBear::ZERO, BabyBear::ZERO, BabyBear::ZERO, BabyBear::ZERO,
    BabyBear::ZERO, BabyBear::ZERO, BabyBear::ZERO, BabyBear::ZERO,
    BabyBear::ZERO,
];

// Perform permutation (immutable)
let output_state = subchip.permute(input_state);
println!("Hash output: {:?}", output_state[0]); // First element is typically the hash

// Perform permutation (mutable)
let mut mutable_state = input_state;
subchip.permute_mut(&mut mutable_state);
assert_eq!(output_state, mutable_state);
```

### Generating Execution Traces for Proving

```rust
use openvm_poseidon2_air::{Poseidon2Config, Poseidon2SubChip, POSEIDON2_WIDTH};
use openvm_stark_sdk::p3_baby_bear::BabyBear;
use openvm_stark_backend::p3_field::Field;

let config = Poseidon2Config::<BabyBear>::default();
let subchip = Poseidon2SubChip::<BabyBear, 0>::new(config.constants);

// Prepare multiple inputs for batch processing
let inputs = vec![
    [BabyBear::from_canonical_u32(1); POSEIDON2_WIDTH],
    [BabyBear::from_canonical_u32(2); POSEIDON2_WIDTH],
    [BabyBear::from_canonical_u32(3); POSEIDON2_WIDTH],
];

// Generate trace for proving system
let trace = subchip.generate_trace(inputs);
println!("Generated trace with {} rows and {} columns", 
         trace.height(), trace.width());
```

## Advanced Usage

### Custom Round Constants

```rust
use openvm_poseidon2_air::{
    Poseidon2Constants, Poseidon2SubChip, 
    BABY_BEAR_POSEIDON2_HALF_FULL_ROUNDS, BABY_BEAR_POSEIDON2_PARTIAL_ROUNDS,
    POSEIDON2_WIDTH
};
use openvm_stark_sdk::p3_baby_bear::BabyBear;
use p3_poseidon2::ExternalLayerConstants;
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::array::from_fn;

// Create custom round constants for testing
let mut rng = StdRng::seed_from_u64(42);

let external_constants = ExternalLayerConstants::new_from_rng(
    2 * BABY_BEAR_POSEIDON2_HALF_FULL_ROUNDS, 
    &mut rng
);

let beginning_full_round_constants = from_fn(|i| 
    external_constants.get_initial_constants()[i]
);
let ending_full_round_constants = from_fn(|i| 
    external_constants.get_terminal_constants()[i]
);
let partial_round_constants = from_fn(|_| 
    BabyBear::from_wrapped_u32(rng.next_u32())
);

let custom_constants = Poseidon2Constants {
    beginning_full_round_constants,
    partial_round_constants,
    ending_full_round_constants,
};

let subchip = Poseidon2SubChip::<BabyBear, 0>::new(custom_constants);
```

### Integration with STARK Proving System

```rust
use openvm_poseidon2_air::{Poseidon2Config, Poseidon2SubChip, POSEIDON2_WIDTH};
use openvm_stark_sdk::{
    config::{
        baby_bear_poseidon2::BabyBearPoseidon2Engine,
        fri_params::standard_fri_params_with_100_bits_conjectured_security,
    },
    engine::StarkFriEngine,
    p3_baby_bear::BabyBear,
    utils::create_seeded_rng,
};
use openvm_stark_backend::p3_field::Field;
use std::sync::Arc;

// Setup
let config = Poseidon2Config::<BabyBear>::default();
let subchip = Arc::new(Poseidon2SubChip::<BabyBear, 0>::new(config.constants));

// Generate some test data
let mut rng = create_seeded_rng();
let num_permutations = 16;
let inputs: Vec<[BabyBear; POSEIDON2_WIDTH]> = (0..num_permutations)
    .map(|_| {
        let vec: Vec<BabyBear> = (0..POSEIDON2_WIDTH)
            .map(|_| BabyBear::from_canonical_u32(rng.next_u32() % (1 << 30)))
            .collect();
        vec.try_into().unwrap()
    })
    .collect();

// Generate execution trace
let trace = subchip.generate_trace(inputs);

// Setup STARK proving parameters
let fri_params = standard_fri_params_with_100_bits_conjectured_security(3);
let engine = BabyBearPoseidon2Engine::new(fri_params);

// Run the STARK prover and verifier
let result = engine.run_simple_test_impl(
    vec![subchip.air.clone()],
    vec![trace],
    vec![vec![]], // No public values
);

match result {
    Ok(_) => println!("STARK proof verification successful!"),
    Err(e) => println!("STARK proof verification failed: {:?}", e),
}
```

### Different SBOX Register Configurations

```rust
use openvm_poseidon2_air::{Poseidon2Config, Poseidon2SubChip};
use openvm_stark_sdk::p3_baby_bear::BabyBear;

let config = Poseidon2Config::<BabyBear>::default();

// SBOX_REGISTERS affects the constraint degree
// 0 registers: higher degree, fewer columns
let subchip_0 = Poseidon2SubChip::<BabyBear, 0>::new(config.constants);

// 1 register: lower degree, more columns  
let subchip_1 = Poseidon2SubChip::<BabyBear, 1>::new(config.constants);

println!("0 SBOX registers - AIR width: {}", subchip_0.air.width());
println!("1 SBOX register - AIR width: {}", subchip_1.air.width());
```

## Testing Examples

### Basic Functionality Test

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use openvm_poseidon2_air::{Poseidon2Config, Poseidon2SubChip, POSEIDON2_WIDTH};
    use openvm_stark_sdk::p3_baby_bear::BabyBear;
    use openvm_stark_backend::p3_field::Field;

    #[test]
    fn test_poseidon2_deterministic() {
        let config = Poseidon2Config::<BabyBear>::default();
        let subchip = Poseidon2SubChip::<BabyBear, 0>::new(config.constants);
        
        let input = [BabyBear::ONE; POSEIDON2_WIDTH];
        let output1 = subchip.permute(input);
        let output2 = subchip.permute(input);
        
        // Same input should produce same output
        assert_eq!(output1, output2);
        
        // Output should be different from input (non-identity)
        assert_ne!(input, output1);
    }

    #[test]
    fn test_mutable_vs_immutable_permute() {
        let config = Poseidon2Config::<BabyBear>::default();
        let subchip = Poseidon2SubChip::<BabyBear, 0>::new(config.constants);
        
        let input = [BabyBear::from_canonical_u32(42); POSEIDON2_WIDTH];
        let output_immutable = subchip.permute(input);
        
        let mut input_mutable = input;
        subchip.permute_mut(&mut input_mutable);
        
        assert_eq!(output_immutable, input_mutable);
    }
}
```

## Performance Considerations

### Batch Processing

```rust
use openvm_poseidon2_air::{Poseidon2Config, Poseidon2SubChip, POSEIDON2_WIDTH};
use openvm_stark_sdk::p3_baby_bear::BabyBear;
use std::time::Instant;

let config = Poseidon2Config::<BabyBear>::default();
let subchip = Poseidon2SubChip::<BabyBear, 0>::new(config.constants);

// Batch processing is more efficient for trace generation
let batch_size = 1000;
let inputs: Vec<[BabyBear; POSEIDON2_WIDTH]> = 
    vec![[BabyBear::ONE; POSEIDON2_WIDTH]; batch_size];

let start = Instant::now();
let _trace = subchip.generate_trace(inputs);
let duration = start.elapsed();

println!("Processed {} permutations in {:?}", batch_size, duration);
```

### Memory Usage Optimization

```rust
use openvm_poseidon2_air::{Poseidon2Config, Poseidon2SubChip, POSEIDON2_WIDTH};
use openvm_stark_sdk::p3_baby_bear::BabyBear;
use std::sync::Arc;

// Use Arc to share subchip across threads without cloning
let config = Poseidon2Config::<BabyBear>::default();
let subchip = Arc::new(Poseidon2SubChip::<BabyBear, 0>::new(config.constants));

// Clone Arc (cheap) instead of the whole subchip
let subchip_clone = Arc::clone(&subchip);

// Both references point to the same underlying data
let input = [BabyBear::ZERO; POSEIDON2_WIDTH];
let result1 = subchip.permute(input);
let result2 = subchip_clone.permute(input);
assert_eq!(result1, result2);
```