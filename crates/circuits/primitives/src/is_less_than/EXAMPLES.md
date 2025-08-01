# IsLessThan Component Examples

## Basic Usage Example

```rust
use openvm_circuits_primitives::{
    is_less_than::{IsLtSubAir, IsLessThanIo},
    var_range::{VariableRangeCheckerBus, VariableRangeCheckerChip},
};

// Setup range checker and bus
let decomp_bits = 8;  // Range check limb size
let bus = VariableRangeCheckerBus::new(0, decomp_bits);
let range_checker = VariableRangeCheckerChip::new(bus);

// Create the SubAir
let max_bits = 16;  // Maximum bits for x, y values
let sub_air = IsLtSubAir::new(bus, max_bits);

// In your Air evaluation:
impl<AB: InteractionBuilder> Air<AB> for YourAir {
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local = main.row_slice(0);
        
        // Extract columns (assuming your layout)
        let x = local[0];  // First input
        let y = local[1];  // Second input
        let out = local[2]; // Output boolean
        let aux_cols = &local[3..]; // Auxiliary columns for decomposition
        
        // Create IO structure
        let io = IsLessThanIo::new(x, y, out, AB::F::ONE);
        
        // Apply constraints
        sub_air.eval(builder, (io, aux_cols));
    }
}
```

## Conditional Usage Example

```rust
// Use the comparison only when a condition is met
let condition = some_boolean_expression;
let io = IsLessThanIo::new(x, y, out, condition);
sub_air.eval(builder, (io, aux_cols));
```

## Transition Air Example

```rust
// For comparing values between adjacent rows
let transition_air = sub_air.when_transition();

impl<AB: InteractionBuilder> Air<AB> for YourTransitionAir {
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local = main.row_slice(0);
        let next = main.row_slice(1);
        
        // Compare current row's x with next row's y
        let x_curr = local[0];
        let y_next = next[1];
        let out = local[2];
        let aux_cols = &local[3..];
        
        let io = IsLessThanIo::new(x_curr, y_next, out, AB::F::ONE);
        transition_air.eval(builder, (io, aux_cols));
    }
}
```

## Trace Generation Example

```rust
use openvm_stark_backend::p3_field::PrimeField32;

impl<F: PrimeField32> TraceSubRowGenerator<F> for YourChip {
    type TraceContext<'a> = (&'a VariableRangeCheckerChip, Vec<(u32, u32)>);
    type ColsMut<'a> = YourColsMut<'a, F>;

    fn generate_subrow<'a>(
        &'a self,
        (range_checker, pairs): Self::TraceContext<'a>,
        cols: Self::ColsMut<'a>,
    ) {
        for (x, y) in pairs {
            // Set input values
            *cols.x = F::from_canonical_u32(x);
            *cols.y = F::from_canonical_u32(y);
            
            // Generate the comparison result and auxiliary data
            self.sub_air.generate_subrow(
                (range_checker, x, y),
                (cols.lower_decomp, cols.out)
            );
        }
    }
}
```

## Complete Chip Example

```rust
use std::sync::Arc;
use openvm_stark_backend::{
    p3_field::PrimeField32,
    p3_matrix::{dense::RowMajorMatrix, Matrix},
};

pub struct MyComparisonChip {
    pub sub_air: IsLtSubAir,
    pub range_checker: Arc<VariableRangeCheckerChip>,
    pub comparisons: Vec<(u32, u32)>,
}

impl MyComparisonChip {
    pub fn new(max_bits: usize) -> Self {
        let bus = VariableRangeCheckerBus::new(0, 8);
        let range_checker = Arc::new(VariableRangeCheckerChip::new(bus));
        let sub_air = IsLtSubAir::new(bus, max_bits);
        
        Self {
            sub_air,
            range_checker,
            comparisons: vec![],
        }
    }
    
    pub fn add_comparison(&mut self, x: u32, y: u32) {
        self.comparisons.push((x, y));
    }
    
    pub fn generate_trace<F: PrimeField32>(self) -> RowMajorMatrix<F> {
        let width = 3 + self.sub_air.decomp_limbs; // x, y, out + aux
        let height = self.comparisons.len().next_power_of_two();
        
        let mut rows = F::zero_vec(width * height);
        
        for (row_idx, (x, y)) in self.comparisons.into_iter().enumerate() {
            let row = &mut rows[row_idx * width..(row_idx + 1) * width];
            
            // Set x, y values
            row[0] = F::from_canonical_u32(x);
            row[1] = F::from_canonical_u32(y);
            
            // Generate output and aux columns
            let (out_slice, aux_slice) = row[2..].split_at_mut(1);
            self.sub_air.generate_subrow(
                (&self.range_checker, x, y),
                (aux_slice, &mut out_slice[0])
            );
        }
        
        RowMajorMatrix::new(rows, width)
    }
}
```

## Test Examples

```rust
#[test]
fn test_basic_comparisons() {
    let mut chip = MyComparisonChip::new(16);
    
    // Add various test cases
    chip.add_comparison(10, 20);   // true case
    chip.add_comparison(20, 10);   // false case  
    chip.add_comparison(15, 15);   // equal case (false)
    chip.add_comparison(0, 1);     // edge case
    
    let trace = chip.generate_trace();
    let range_trace = chip.range_checker.generate_trace();
    
    // Verify with your prover/verifier setup
    // ...
}

#[test]
fn test_max_values() {
    let max_bits = 16;
    let mut chip = MyComparisonChip::new(max_bits);
    
    let max_val = (1 << max_bits) - 1;
    chip.add_comparison(max_val - 1, max_val);  // Should be true
    chip.add_comparison(max_val, max_val - 1);  // Should be false
    
    // Test trace generation and verification
    // ...
}
```

## Integration with Custom AIR Example

```rust
#[derive(Clone, Copy)]
pub struct MyCustomAir {
    pub is_lt_sub_air: IsLtSubAir,
    // ... other sub-airs
}

impl<AB: InteractionBuilder> Air<AB> for MyCustomAir {
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local = main.row_slice(0);
        
        // Your custom logic here...
        let custom_x = local[0] + local[1];  // Some computed value
        let custom_y = local[2] * AB::F::from_canonical_u32(3);
        let comparison_out = local[10];
        let aux_start = 11;
        
        // Use the is_less_than sub-air as part of your computation
        let io = IsLessThanIo::new(
            custom_x,
            custom_y, 
            comparison_out,
            AB::F::ONE  // Always active
        );
        
        self.is_lt_sub_air.eval(
            builder,
            (io, &local[aux_start..aux_start + self.is_lt_sub_air.decomp_limbs])
        );
        
        // Continue with other constraints that might depend on comparison_out
        // ...
    }
}
```

## Error Handling Example

```rust
// Example of proper bounds checking before using the SubAir
fn safe_comparison(x: u32, y: u32, max_bits: usize) -> Result<bool, &'static str> {
    if x >= (1 << max_bits) {
        return Err("x exceeds max_bits");
    }
    if y >= (1 << max_bits) {
        return Err("y exceeds max_bits");
    }
    
    // Safe to use in SubAir now
    Ok(x < y)
}
```