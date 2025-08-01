# IsLessThanArray Component Examples

## Basic Usage Examples

### Example 1: Simple Array Comparison

```rust
use openvm_circuit_primitives::is_less_than_array::{IsLtArraySubAir, IsLtArrayIo};
use openvm_circuit_primitives::var_range::{VariableRangeCheckerBus, VariableRangeCheckerChip};
use std::sync::Arc;

// Setup range checking infrastructure
let range_max_bits = 8;
let bus = VariableRangeCheckerBus::new(0, range_max_bits);
let range_checker = Arc::new(VariableRangeCheckerChip::new(bus));

// Create IsLtArraySubAir for arrays of length 2 with 16-bit elements
const ARRAY_LEN: usize = 2;
const MAX_BITS: usize = 16;
let lt_array_air = IsLtArraySubAir::<ARRAY_LEN>::new(range_checker.bus(), MAX_BITS);

// Example arrays for comparison
let x = [14321u32, 123u32];  // First array
let y = [26678u32, 233u32];  // Second array
// Result: x < y lexicographically (true, since 14321 < 26678)
```

### Example 2: Testing Framework Usage

```rust
use openvm_circuit_primitives::is_less_than_array::tests::{IsLtArrayChip, IsLtArrayTestAir};
use openvm_stark_sdk::config::baby_bear_poseidon2::BabyBearPoseidon2Engine;

const N: usize = 2;      // Array length
const LIMBS: usize = 2;  // Range check decomposition limbs

// Create test chip
let range_checker = get_tester_range_chip();
let mut chip = IsLtArrayChip::<N, LIMBS>::new(16, range_checker.clone());

// Add test cases
chip.pairs = vec![
    ([14321, 123], [26678, 233]),  // x < y (first element smaller)
    ([26678, 244], [14321, 233]),  // x > y (first element larger)  
    ([14321, 244], [14321, 244]),  // x == y (arrays identical)
    ([26678, 233], [14321, 244]),  // x > y (first element larger)
];

// Generate and verify proof
let trace = chip.generate_trace();
let range_checker_trace = range_checker.generate_trace();

BabyBearPoseidon2Engine::run_simple_test_no_pis_fast(
    any_rap_arc_vec![chip.air, range_checker.air],
    vec![trace, range_checker_trace],
).expect("Verification failed");
```

### Example 3: Trace Generation

```rust
use openvm_circuit_primitives::TraceSubRowGenerator;
use openvm_stark_backend::p3_field::Field;

// Generate trace for specific comparison
let x_values = [100u32, 200u32];
let y_values = [100u32, 250u32];

// Convert to field elements
let x_field: Vec<F> = x_values.iter().map(|&v| F::from_canonical_u32(v)).collect();
let y_field: Vec<F> = y_values.iter().map(|&v| F::from_canonical_u32(v)).collect();

// Generate subrow (this happens internally during trace generation)
lt_array_air.generate_subrow(
    (&range_checker, &x_field, &y_field),
    (aux_cols_mut, &mut output_field)
);

// Result: output_field = F::from_bool(true) since [100,200] < [100,250]
```

## Advanced Usage Patterns

### Example 4: Transition Constraints for Cross-Row Comparisons

```rust
// Use when comparing values between adjacent rows
let transition_air = lt_array_air.when_transition();

// This variant applies constraints only during transitions,
// useful for sorting verification or sequential ordering checks
```

### Example 5: Integration with Custom Air

```rust
use openvm_stark_backend::interaction::InteractionBuilder;
use openvm_circuit_primitives::SubAir;

struct MyCustomAir {
    lt_array: IsLtArraySubAir<4>, // Arrays of length 4
}

impl<AB: InteractionBuilder> Air<AB> for MyCustomAir {
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local = main.row_slice(0);
        
        // Extract arrays from your custom trace layout
        let x_array = [local[0], local[1], local[2], local[3]];
        let y_array = [local[4], local[5], local[6], local[7]];
        let output = local[8];
        let count = AB::Expr::ONE; // Always active
        
        let io = IsLtArrayIo {
            x: x_array.map(Into::into),
            y: y_array.map(Into::into),
            out: output.into(),
            count,
        };
        
        // Extract auxiliary columns for the comparison
        let aux_ref = /* construct from your trace layout */;
        
        // Apply lexicographic comparison constraints
        self.lt_array.eval(builder, (io, aux_ref));
    }
}
```

## Test Case Scenarios

### Example 6: Edge Cases

```rust
// Test identical arrays
let identical_case = ([12345, 67890], [12345, 67890]);
// Expected result: false (not less than)

// Test single element difference at start
let early_diff = ([100, 200], [200, 100]); 
// Expected result: true (100 < 200 at index 0)

// Test single element difference at end
let late_diff = ([100, 200], [100, 300]);
// Expected result: true (same at index 0, 200 < 300 at index 1)

// Test maximum bit values
let max_vals = ([0xFFFF, 0], [0xFFFF, 1]);
// Expected result: true (same at index 0, 0 < 1 at index 1)

chip.pairs = vec![identical_case, early_diff, late_diff, max_vals];
```

### Example 7: Error Testing

```rust
// Test verification failure with wrong output
let mut chip = IsLtArrayChip::<N, LIMBS>::new(16, range_checker.clone());
chip.pairs = vec![([14321, 123], [26678, 233])];

let mut trace = chip.generate_trace(); 
let range_checker_trace = range_checker.generate_trace();

// Corrupt the output to test constraint enforcement
trace.values[2] = F::ZERO; // Should be F::ONE for this comparison

// This should fail verification
let result = BabyBearPoseidon2Engine::run_simple_test_no_pis_fast(
    any_rap_arc_vec![chip.air, range_checker.air],
    vec![trace, range_checker_trace]
);
assert!(result.is_err(), "Expected verification to fail");
```

## Performance Optimization Examples

### Example 8: Batch Processing

```rust
// Process multiple comparisons efficiently
let mut chip = IsLtArrayChip::<4, 4>::new(20, range_checker.clone());

// Add many test cases (must be power of 2 for trace generation)
let mut test_pairs = Vec::new();
for i in 0..1024 {
    let x = [i, i+1, i+2, i+3];
    let y = [i+1, i+2, i+3, i+4];
    test_pairs.push((x, y));
}
chip.pairs = test_pairs;

// Generate trace for all comparisons at once
let trace = chip.generate_trace();
```

### Example 9: Range Checker Optimization

```rust
// Share range checker across multiple components
let shared_range_checker = Arc::new(VariableRangeCheckerChip::new(bus));

let lt_array_1 = IsLtArraySubAir::<2>::new(shared_range_checker.bus(), 16);
let lt_array_2 = IsLtArraySubAir::<3>::new(shared_range_checker.bus(), 16);
let lt_array_3 = IsLtArraySubAir::<4>::new(shared_range_checker.bus(), 16);

// All components share the same range checking infrastructure
// This reduces overall proof size and verification time
```

## Integration Testing

### Example 10: Full Integration Test

```rust
#[test]
fn comprehensive_lexicographic_test() {
    let range_checker = get_tester_range_chip();
    let mut chip = IsLtArrayChip::<3, 3>::new(16, range_checker.clone());
    
    chip.pairs = vec![
        // Various lexicographic orderings
        ([1, 2, 3], [1, 2, 4]),    // true: differs at last position
        ([1, 2, 3], [1, 3, 2]),    // true: differs at middle position  
        ([2, 1, 3], [1, 2, 3]),    // false: differs at first position
        ([5, 5, 5], [5, 5, 5]),    // false: identical arrays
        ([0, 0, 1], [0, 1, 0]),    // true: middle position determines
        ([65535, 0, 0], [65535, 0, 1]), // true: max values at boundaries
        ([32767, 32767, 32767], [32768, 0, 0]), // true: first diff at start
        ([100, 200, 300], [99, 999, 999]),      // false: first element larger
    ];

    let trace = chip.generate_trace();
    let range_checker_trace = range_checker.generate_trace();

    // Verify all test cases pass
    BabyBearPoseidon2Engine::run_simple_test_no_pis_fast(
        any_rap_arc_vec![chip.air, range_checker.air],
        vec![trace, range_checker_trace],
    ).expect("Comprehensive test failed");
}
```