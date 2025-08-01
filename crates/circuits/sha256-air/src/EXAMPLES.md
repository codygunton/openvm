# SHA256 AIR Examples

## Basic Usage Examples

### 1. Creating a SHA256 AIR Instance

```rust
use openvm_circuit_primitives::bitwise_op_lookup::BitwiseOperationLookupBus;
use openvm_sha256_air::Sha256Air;
use openvm_stark_backend::interaction::BusIndex;

// Create the necessary bus connections
let bitwise_lookup_bus = BitwiseOperationLookupBus::new(0);
let self_bus_idx: BusIndex = 1;

// Create the SHA256 AIR instance
let sha256_air = Sha256Air::new(bitwise_lookup_bus, self_bus_idx);
```

### 2. Single Block Hash Computation

```rust
use openvm_sha256_air::{Sha256Air, SHA256_H, SHA256_BLOCK_U8S};

// Example: Hashing a single 512-bit block
let input_block: [u8; SHA256_BLOCK_U8S] = [
    // 64 bytes of input data (padded)
    0x61, 0x62, 0x63, 0x80, 0x00, 0x00, 0x00, 0x00,
    // ... (remaining bytes)
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18,
];

// Compute the hash using the reference implementation
let initial_hash = SHA256_H;
let final_hash = Sha256Air::get_block_hash(&initial_hash, input_block);

println!("Final hash: {:08x?}", final_hash);
```

### 3. Trace Generation for Testing

```rust
use openvm_circuit_primitives::bitwise_op_lookup::SharedBitwiseOperationLookupChip;
use openvm_sha256_air::{generate_trace, Sha256Air, SHA256_BLOCK_U8S};
use openvm_stark_backend::p3_field::extension::BinomialExtensionField;
use openvm_stark_backend::p3_baby_bear::BabyBear;

type F = BabyBear;

// Create test data
let records = vec![
    ([0u8; SHA256_BLOCK_U8S], true), // Single block, is_last_block = true
];

// Create required components
let bitwise_lookup_bus = BitwiseOperationLookupBus::new(0);
let bitwise_chip = SharedBitwiseOperationLookupChip::<8>::new(bitwise_lookup_bus);
let sha256_air = Sha256Air::new(bitwise_lookup_bus, 1);

// Generate the trace
let trace = generate_trace::<F>(&sha256_air, bitwise_chip, records);
```

## Advanced Examples

### 4. Multi-Block Message Processing

```rust
use openvm_sha256_air::{Sha256Air, SHA256_BLOCK_U8S, SHA256_H};

// Process multiple blocks (like a longer message)
let blocks = vec![
    ([0x61; SHA256_BLOCK_U8S], false), // First block, not last
    ([0x62; SHA256_BLOCK_U8S], false), // Middle block, not last  
    ([0x63; SHA256_BLOCK_U8S], true),  // Final block, is last
];

// Simulate the hash chain
let mut current_hash = SHA256_H;
for (block, is_last) in &blocks {
    let new_hash = Sha256Air::get_block_hash(&current_hash, *block);
    println!("Block hash: {:08x?}", new_hash);
    
    if !is_last {
        current_hash = new_hash;
    } else {
        println!("Final message hash: {:08x?}", new_hash);
    }
}
```

### 5. Custom Test Chip Implementation

```rust
use std::sync::Arc;
use openvm_circuit_primitives::bitwise_op_lookup::SharedBitwiseOperationLookupChip;
use openvm_sha256_air::{Sha256Air, SHA256_BLOCK_U8S};
use openvm_stark_backend::{
    config::StarkGenericConfig,
    prover::types::AirProofInput,
    Chip, AirRef,
};

pub struct CustomSha256Chip {
    pub air: Sha256Air,
    pub bitwise_lookup_chip: SharedBitwiseOperationLookupChip<8>,
    pub records: Vec<([u8; SHA256_BLOCK_U8S], bool)>,
}

impl<SC: StarkGenericConfig> Chip<SC> for CustomSha256Chip
where
    SC::Val: openvm_stark_backend::p3_field::PrimeField32,
{
    fn air(&self) -> AirRef<SC> {
        Arc::new(self.air.clone())
    }

    fn generate_air_proof_input(self) -> AirProofInput<SC> {
        let trace = openvm_sha256_air::generate_trace::<SC::Val>(
            &self.air,
            self.bitwise_lookup_chip.clone(),
            self.records,
        );
        AirProofInput::simple_no_pis(trace)
    }
}
```

### 6. Working with Specific SHA256 Functions

```rust
use openvm_sha256_air::{big_sig0, big_sig1, ch, maj, small_sig0, small_sig1};

// Example: Using individual SHA256 functions
let x = 0x12345678u32;
let y = 0x9abcdef0u32;
let z = 0xfedcba98u32;

// Apply SHA256 functions
let ch_result = ch(x, y, z);
let maj_result = maj(x, y, z);
let big_sig0_result = big_sig0(x);
let big_sig1_result = big_sig1(x);
let small_sig0_result = small_sig0(x);
let small_sig1_result = small_sig1(x);

println!("Ch(x,y,z) = 0x{:08x}", ch_result);
println!("Maj(x,y,z) = 0x{:08x}", maj_result);
```

### 7. Random Message Generation for Testing

```rust
use openvm_sha256_air::get_random_message;
use openvm_stark_sdk::utils::create_seeded_rng;

// Generate random test data
let mut rng = create_seeded_rng();
let message_length = 1000; // bytes
let random_message = get_random_message(&mut rng, message_length);

// Convert to blocks (you would need proper SHA256 padding)
// This is just for demonstration
let num_blocks = (message_length + 63) / 64;
let mut records = Vec::new();

for i in 0..num_blocks {
    let mut block = [0u8; 64];
    let start = i * 64;
    let end = std::cmp::min(start + 64, message_length);
    
    if end > start {
        block[..end-start].copy_from_slice(&random_message[start..end]);
    }
    
    // Mark the last block
    let is_last = i == num_blocks - 1;
    records.push((block, is_last));
}
```

## Testing Examples

### 8. Basic Constraint Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use openvm_circuit::arch::testing::VmChipTestBuilder;
    
    #[test]
    fn test_single_block_sha256() {
        let tester = VmChipTestBuilder::default();
        let bitwise_bus = BitwiseOperationLookupBus::new(0);
        let bitwise_chip = SharedBitwiseOperationLookupChip::<8>::new(bitwise_bus);
        
        // Test data: "abc" padded to 512 bits
        let abc_block = [
            0x61, 0x62, 0x63, 0x80, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18,
        ];
        
        let chip = CustomSha256Chip {
            air: Sha256Air::new(bitwise_bus, 1),
            bitwise_lookup_chip: bitwise_chip.clone(),
            records: vec![(abc_block, true)],
        };

        let tester = tester.build().load(chip).load(bitwise_chip).finalize();
        tester.simple_test().expect("Verification failed");
    }
}
```

### 9. Constraint Violation Testing

```rust
#[test]
#[should_panic]
fn test_invalid_final_hash_should_fail() {
    // This example shows how to test that invalid traces are rejected
    // The test chip would modify the final_hash to an incorrect value
    // and the proof should fail
    
    let tester = VmChipTestBuilder::default();
    let bitwise_bus = BitwiseOperationLookupBus::new(0);
    let bitwise_chip = SharedBitwiseOperationLookupChip::<8>::new(bitwise_bus);
    
    // Use a malicious chip that violates constraints
    let chip = MaliciousSha256Chip { /* ... */ };
    
    let tester = tester.build().load(chip).load(bitwise_chip).finalize();
    tester.simple_test().expect("This should panic due to constraint violation");
}
```

## Common Patterns

### 10. Utility Functions for Development

```rust
use openvm_sha256_air::{u32_into_limbs, limbs_into_u32, SHA256_WORD_U16S};

// Converting between u32 and limb representations
let value = 0x12345678u32;
let limbs = u32_into_limbs::<SHA256_WORD_U16S>(value); // [0x5678, 0x1234]
let reconstructed = limbs_into_u32(limbs); // 0x12345678

assert_eq!(value, reconstructed);
```

### 11. Field Element Operations

```rust
use openvm_sha256_air::{compose, big_sig0_field};
use openvm_stark_backend::p3_baby_bear::BabyBear;

type F = BabyBear;

// Working with field elements for constraint checking
let bits: [F; 32] = [F::ZERO; 32]; // Example bit representation
let composed = compose::<F>(&bits[0..16], 1); // Compose first 16 bits

// Apply SHA256 operations in field arithmetic
let sig0_result = big_sig0_field::<F>(&bits);
```

These examples demonstrate the key usage patterns for the SHA256 AIR component, from basic hash computation to advanced constraint testing and integration with the OpenVM framework.