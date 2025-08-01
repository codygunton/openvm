# IsLessThan Component Integration Guide

## Architecture Overview

The `is_less_than` component integrates into the OpenVM zkVM architecture as a SubAir that provides fundamental comparison functionality. It works in conjunction with the variable range checker system to ensure sound arithmetic comparisons.

## Integration Requirements

### Dependencies
- **Required**: `var_range::VariableRangeCheckerChip` - Must be included in your chip set
- **Required**: `var_range::VariableRangeCheckerBus` - For range check interactions
- **Backend**: Compatible with `openvm_stark_backend` interaction system

### Soundness Prerequisites
1. Range checker must be properly configured and included
2. `max_bits ≤ 29` (enforced by constructor)
3. `count` parameter must be constrained boolean by caller
4. Range check interactions must be enabled

## Column Layout Planning

### Auxiliary Columns Required
```rust
// Calculate decomposition limbs needed
let decomp_limbs = max_bits.div_ceil(bus.range_max_bits);

// Your Air column layout should allocate:
// - Input columns: x, y  
// - Output column: out
// - Auxiliary columns: decomp_limbs worth
```

### Memory Layout Example
```rust
pub struct YourAirCols<T> {
    // Your existing columns
    pub existing_data: [T; N],
    
    // IsLessThan integration
    pub lt_x: T,
    pub lt_y: T, 
    pub lt_out: T,
    pub lt_decomp: [T; DECOMP_LIMBS], // Size determined at compile/runtime
}
```

## Integration Patterns

### Pattern 1: Single Comparison SubAir
```rust
#[derive(Clone, Copy)]
pub struct YourAir {
    pub is_lt_sub_air: IsLtSubAir,
    // ... other components
}

impl YourAir {
    pub fn new(bus: VariableRangeCheckerBus, max_bits: usize) -> Self {
        Self {
            is_lt_sub_air: IsLtSubAir::new(bus, max_bits),
        }
    }
}

impl<AB: InteractionBuilder> Air<AB> for YourAir {
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local = main.row_slice(0);
        
        // Extract your comparison columns
        let io = IsLessThanIo::new(
            local[LT_X_COL],
            local[LT_Y_COL], 
            local[LT_OUT_COL],
            AB::F::ONE  // Or your condition
        );
        
        self.is_lt_sub_air.eval(builder, (io, &local[LT_AUX_START..]));
    }
}
```

### Pattern 2: Multiple Comparisons
```rust
pub struct MultiComparisonAir {
    pub comparisons: Vec<IsLtSubAir>,
}

impl<AB: InteractionBuilder> Air<AB> for MultiComparisonAir {
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local = main.row_slice(0);
        
        let mut col_offset = 0;
        for (i, sub_air) in self.comparisons.iter().enumerate() {
            let io = IsLessThanIo::new(
                local[col_offset],     // x
                local[col_offset + 1], // y  
                local[col_offset + 2], // out
                AB::F::ONE
            );
            
            let aux_start = col_offset + 3;
            let aux_end = aux_start + sub_air.decomp_limbs;
            
            sub_air.eval(builder, (io, &local[aux_start..aux_end]));
            col_offset = aux_end;
        }
    }
}
```

### Pattern 3: Conditional Activation
```rust
impl<AB: InteractionBuilder> Air<AB> for ConditionalComparisonAir {
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local = main.row_slice(0);
        
        let is_comparison_row = local[0]; // Your condition logic
        let io = IsLessThanIo::new(
            local[1],
            local[2],
            local[3], 
            is_comparison_row  // Only active when condition is true
        );
        
        self.is_lt_sub_air.eval(builder, (io, &local[4..]));
    }
}
```

### Pattern 4: Transition Comparisons
```rust
impl<AB: InteractionBuilder> Air<AB> for TransitionComparisonAir {
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local = main.row_slice(0);
        let next = main.row_slice(1);
        
        // Compare current row value with next row value
        let io = IsLessThanIo::new(
            local[VALUE_COL],
            next[VALUE_COL],
            local[COMPARISON_OUT_COL],
            AB::F::ONE
        );
        
        self.is_lt_sub_air.when_transition().eval(builder, (io, &local[AUX_START..]));
    }
}
```

## Trace Generation Integration

### SubRow Generator Implementation
```rust
impl<F: PrimeField32> TraceSubRowGenerator<F> for YourChip {
    type TraceContext<'a> = (
        &'a VariableRangeCheckerChip,
        // ... your other context
    );
    
    type ColsMut<'a> = YourColsMut<'a, F>;
    
    fn generate_subrow<'a>(
        &'a self,
        (range_checker, /* your context */): Self::TraceContext<'a>,
        cols: Self::ColsMut<'a>,
    ) {
        // Your trace generation logic...
        let x_val = /* compute x */;
        let y_val = /* compute y */;
        
        // Set input columns
        *cols.lt_x = F::from_canonical_u32(x_val);
        *cols.lt_y = F::from_canonical_u32(y_val);
        
        // Generate comparison output and auxiliary data
        self.is_lt_sub_air.generate_subrow(
            (range_checker, x_val, y_val),
            (cols.lt_decomp, cols.lt_out)
        );
    }
}
```

## Multi-Chip Setup

### Chip Collection Setup
```rust
pub fn create_chip_collection() -> Vec<Arc<dyn YourChipTrait>> {
    // Create range checker first
    let bus = VariableRangeCheckerBus::new(0, 8);
    let range_checker = Arc::new(VariableRangeCheckerChip::new(bus));
    
    // Create your comparison chips
    let comparison_chip = Arc::new(YourComparisonChip::new(bus, 16));
    
    vec![
        range_checker,
        comparison_chip,
        // ... other chips
    ]
}
```

### Air Collection Setup
```rust
pub fn create_air_collection() -> Vec<Arc<dyn YourAirTrait>> {
    let bus = VariableRangeCheckerBus::new(0, 8);
    
    vec![
        Arc::new(YourComparisonAir::new(bus, 16)),
        Arc::new(VariableRangeCheckerAir::new(bus)),
        // ... other airs
    ]
}
```

## Performance Considerations

### Column Count Optimization
- Each comparison requires `3 + max_bits.div_ceil(range_max_bits)` columns
- Choose `range_max_bits` to balance limb count vs range checker cost
- Consider sharing auxiliary columns when comparisons don't overlap

### Constraint Degree Management
- Comparison constraint degree: `deg(count) + max(1, deg(x), deg(y))`
- Keep input expressions simple to minimize total degree
- Consider using intermediate columns for complex expressions

### Interaction Cost
- Each active comparison sends `decomp_limbs` range check interactions
- Factor this into your range checker sizing
- Consider conditional activation to reduce unnecessary interactions

## Common Integration Pitfalls

### 1. Missing Range Checker
```rust
// ❌ WRONG: Using IsLtSubAir without range checker
let airs = vec![your_comparison_air]; // Missing range checker!

// ✅ CORRECT: Include range checker
let airs = vec![your_comparison_air, range_checker_air];
```

### 2. Incorrect Column Layout
```rust
// ❌ WRONG: Not allocating enough auxiliary columns
pub struct BadCols<T> {
    pub x: T,
    pub y: T, 
    pub out: T,
    pub aux: [T; 2], // Not enough for most decompositions!
}

// ✅ CORRECT: Proper auxiliary column allocation
pub struct GoodCols<T> {
    pub x: T,
    pub y: T,
    pub out: T, 
    pub aux: [T; CALCULATED_DECOMP_LIMBS],
}
```

### 3. Forgetting Boolean Constraint on Count
```rust
// ❌ WRONG: Not constraining count to be boolean
let io = IsLessThanIo::new(x, y, out, some_expression);

// ✅ CORRECT: Ensure count is constrained boolean elsewhere
builder.assert_bool(some_expression);
let io = IsLessThanIo::new(x, y, out, some_expression);
```

### 4. Exceeding Max Bits
```rust
// ❌ WRONG: Values exceeding max_bits
let x_val = 1 << 20; // Too large for max_bits = 16
sub_air.generate_subrow((range_checker, x_val, y_val), cols);

// ✅ CORRECT: Validate inputs
assert!(x_val < (1 << max_bits));
assert!(y_val < (1 << max_bits));
```

## Testing Integration

### Integration Test Structure
```rust
#[test]
fn test_integrated_comparison() {
    // Setup chips
    let range_checker = Arc::new(VariableRangeCheckerChip::new(bus));
    let comparison_chip = YourComparisonChip::new(bus, 16);
    
    // Generate traces
    let comparison_trace = comparison_chip.generate_trace();
    let range_trace = range_checker.generate_trace();
    
    // Create AIRs
    let airs = vec![
        Arc::new(comparison_chip.air) as Arc<dyn YourAirTrait>,
        Arc::new(range_checker.air) as Arc<dyn YourAirTrait>,
    ];
    
    // Run integrated proof
    YourEngine::run_simple_test_no_pis_fast(
        airs, 
        vec![comparison_trace, range_trace]
    ).expect("Integration test failed");
}
```

## Version Compatibility

This integration guide is compatible with OpenVM version 1.3.0 and later. Key compatibility notes:

- Uses `InteractionBuilder` trait from `openvm_stark_backend`
- Compatible with the modular AIR architecture
- Works with the current range checker interface
- Supports both compile-time and runtime configuration patterns