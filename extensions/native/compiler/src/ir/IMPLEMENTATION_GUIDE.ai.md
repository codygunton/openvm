# OpenVM Native Compiler IR - Implementation Guide

## Creating a Custom Config

To use the IR, first define a Config that specifies your field types:

```rust
use openvm_stark_backend::p3_field::{PrimeField32, TwoAdicField, ExtensionField};
use openvm_native_compiler::ir::Config;

#[derive(Clone, Default)]
struct MyConfig;

impl Config for MyConfig {
    type N = BabyBear;           // Native field (31-bit prime)
    type F = BN254Fr;            // Target field (254-bit prime)  
    type EF = BN254Ext;          // Quadratic extension of F
}
```

## Building a Simple Program

### Basic Setup
```rust
use openvm_native_compiler::ir::{Builder, Felt, Var};

fn build_program<C: Config>() {
    let mut builder = Builder::<C>::default();
    
    // Create variables
    let x: Var<C::N> = builder.eval(42);
    let y: Felt<C::F> = builder.eval(C::F::from_canonical_u32(100));
    
    // Perform operations
    let z = builder.eval(x + 10);
    builder.print_v(z);
}
```

### Implementing Field Arithmetic
```rust
fn field_arithmetic<C: Config>(builder: &mut Builder<C>) {
    // Input values
    let a: Felt<C::F> = builder.eval(C::F::from_canonical_u32(7));
    let b: Felt<C::F> = builder.eval(C::F::from_canonical_u32(13));
    
    // Basic operations
    let sum = builder.eval(a + b);
    let product = builder.eval(a * b);
    let quotient = builder.eval(a / b);
    
    // More complex expression
    let result = builder.eval((a + b) * (a - b) / (a * b));
    
    builder.print_f(result);
}
```

## Working with Arrays

### Static Arrays
```rust
fn static_array_example<C: Config>(builder: &mut Builder<C>) {
    // Create array with known values
    let values = vec![
        builder.eval(1),
        builder.eval(2),
        builder.eval(3),
    ];
    let array = builder.array(values);
    
    // Access elements
    let first = builder.get(&array, 0);
    let last = builder.get(&array, 2);
    
    // Iterate over array
    builder.range(0, 3).for_each(|i, builder| {
        let elem = builder.get(&array, i);
        builder.print_v(elem);
    });
}
```

### Dynamic Arrays
```rust
fn dynamic_array_example<C: Config>(builder: &mut Builder<C>) {
    // Runtime size
    let size: Var<C::N> = builder.eval(10);
    
    // Allocate memory
    let ptr = builder.alloc(size);
    
    // Initialize array
    builder.range(0, size).for_each(|i, builder| {
        builder.store(ptr, i, i);
    });
    
    // Sum elements
    let mut sum = builder.eval(0);
    builder.range(0, size).for_each(|i, builder| {
        let value = builder.load(ptr, i);
        builder.assign(&mut sum, sum + value);
    });
    
    builder.print_v(sum);
}
```

## Control Flow Patterns

### Conditional Logic
```rust
fn conditional_pattern<C: Config>(builder: &mut Builder<C>) {
    let x = builder.eval(5);
    let y = builder.eval(10);
    
    // Simple if-then
    builder.if_eq(x, 5).then(|builder| {
        builder.print_v(x);
    });
    
    // If-then-else with return value
    let max = builder.if_ne(x, y).then_value(
        builder.if_ne(x - y, 0).then_value(x).else_value(y)
    ).else_value(x);
    
    // Nested conditions
    builder.if_eq(x, 5).then(|builder| {
        builder.if_eq(y, 10).then(|builder| {
            builder.print_v(x + y);
        });
    });
}
```

### Loop Patterns
```rust
fn loop_patterns<C: Config>(builder: &mut Builder<C>) {
    // Find first occurrence
    let array = builder.array(vec![1, 5, 3, 7, 5].map(|v| builder.eval(v)));
    let target = builder.eval(5);
    let mut found_index = builder.eval(-1);
    
    builder.range(0, 5).for_each(|i, builder| {
        let elem = builder.get(&array, i);
        builder.if_eq(elem, target).then(|builder| {
            builder.if_eq(found_index, -1).then(|builder| {
                builder.assign(&mut found_index, i);
                builder.break_loop();
            });
        });
    });
    
    // Nested loops
    let n = builder.eval(3);
    builder.range(0, n).for_each(|i, builder| {
        builder.range(0, n).for_each(|j, builder| {
            let value = builder.eval(i * n + j);
            builder.print_v(value);
        });
    });
}
```

## Implementing Algorithms

### Binary Search
```rust
fn binary_search<C: Config>(
    builder: &mut Builder<C>,
    array: &Array<C, Var<C::N>>,
    target: Var<C::N>,
) -> Var<C::N> {
    let mut left = builder.eval(0);
    let mut right = builder.eval(array.len() - 1);
    let mut result = builder.eval(-1);
    
    builder.range(0, 32).for_each(|_, builder| { // Max 32 iterations
        builder.if_ne(left, right + 1).then(|builder| {
            let mid = builder.eval((left + right) / 2);
            let mid_val = builder.get(array, mid);
            
            builder.if_eq(mid_val, target).then(|builder| {
                builder.assign(&mut result, mid);
                builder.break_loop();
            }).else_then(|builder| {
                builder.if_ne(mid_val - target, 0).then(|builder| {
                    // mid_val > target
                    builder.assign(&mut right, mid - 1);
                }).else_then(|builder| {
                    builder.assign(&mut left, mid + 1);
                });
            });
        });
    });
    
    result
}
```

### Matrix Operations
```rust
fn matrix_multiply<C: Config>(
    builder: &mut Builder<C>,
    a: &Array<C, Felt<C::F>>, // n x m matrix
    b: &Array<C, Felt<C::F>>, // m x p matrix
    n: usize, m: usize, p: usize,
) -> Array<C, Felt<C::F>> {
    let mut result = Vec::new();
    
    for i in 0..n {
        for j in 0..p {
            let mut sum = builder.eval(C::F::ZERO);
            
            for k in 0..m {
                let a_elem = builder.get(a, i * m + k);
                let b_elem = builder.get(b, k * p + j);
                let prod = builder.eval(a_elem * b_elem);
                builder.assign(&mut sum, sum + prod);
            }
            
            result.push(sum);
        }
    }
    
    builder.array(result)
}
```

## Advanced Techniques

### Symbolic Execution
```rust
fn symbolic_example<C: Config>(builder: &mut Builder<C>) {
    let x = builder.eval(5);
    let y = builder.eval(10);
    
    // Build symbolic expression
    let expr = SymbolicVar::from(x) + SymbolicVar::from(y) * 2;
    
    // Evaluate when needed
    let result: Var<C::N> = builder.eval(expr);
    
    // Symbolic expressions optimize automatically
    let zero_expr = SymbolicVar::from(x) - SymbolicVar::from(x);
    let zero = builder.eval(zero_expr); // Optimized to constant 0
}
```

### Custom Instructions
```rust
use openvm_native_compiler::ir::DslIr;

// Extend DslIr for custom operations
fn emit_custom_instruction<C: Config>(
    builder: &mut Builder<C>,
    input: Var<C::N>,
) -> Var<C::N> {
    let output = builder.uninit();
    
    // Emit custom instruction
    builder.operations.vec.push(DslIr::Comment(
        format!("Custom operation on {:?}", input)
    ));
    
    // Simulate the operation
    builder.assign(&output, input * 2);
    
    output
}
```

### Memory Management
```rust
fn memory_patterns<C: Config>(builder: &mut Builder<C>) {
    // Stack allocation pattern
    let size = builder.eval(100);
    let stack_ptr = builder.alloc(size);
    let mut stack_top = builder.eval(0);
    
    // Push operation
    let value = builder.eval(42);
    builder.store(stack_ptr, stack_top, value);
    builder.assign(&mut stack_top, stack_top + 1);
    
    // Pop operation
    builder.if_ne(stack_top, 0).then(|builder| {
        builder.assign(&mut stack_top, stack_top - 1);
        let value = builder.load(stack_ptr, stack_top);
        builder.print_v(value);
    });
}
```

## Integration with Backend

### Generating Constraints
```rust
fn to_constraints<C: Config>(builder: Builder<C>) -> ConstraintSystem {
    // Process operations
    for op in builder.operations.vec {
        match op {
            DslIr::AddV(out, a, b) => {
                // Generate constraint: out = a + b
            }
            DslIr::MulF(out, a, b) => {
                // Generate constraint: out = a * b in field F
            }
            // ... handle other operations
        }
    }
    
    // Return constraint system
    ConstraintSystem::new()
}
```

### Witness Generation
```rust
fn generate_witness<C: Config>(
    builder: &Builder<C>,
    inputs: &[C::N],
) -> Witness<C> {
    let mut witness = Witness::default();
    
    // Initialize input variables
    for (i, &input) in inputs.iter().enumerate() {
        witness.vars.push(input);
    }
    
    // Execute operations
    for op in &builder.operations.vec {
        execute_operation(op, &mut witness);
    }
    
    witness
}
```

## Best Practices

### Error Handling
```rust
fn safe_division<C: Config>(
    builder: &mut Builder<C>,
    numerator: Felt<C::F>,
    denominator: Felt<C::F>,
) -> Felt<C::F> {
    // Check for zero denominator
    let is_zero = builder.eval(denominator) == builder.eval(C::F::ZERO);
    
    builder.if_eq(is_zero, 1).then(|builder| {
        panic!("Division by zero!");
    });
    
    builder.eval(numerator / denominator)
}
```

### Performance Optimization
```rust
fn optimized_sum<C: Config>(
    builder: &mut Builder<C>,
    array: &Array<C, Var<C::N>>,
) -> Var<C::N> {
    // Use symbolic execution for better optimization
    let mut sum = SymbolicVar::from(builder.eval(0));
    
    for i in 0..array.len() {
        let elem = builder.get(array, i);
        sum = sum + SymbolicVar::from(elem);
    }
    
    // Single evaluation at the end
    builder.eval(sum)
}
```

### Testing IR Programs
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_arithmetic() {
        let mut builder = Builder::<TestConfig>::default();
        
        let a = builder.eval(5);
        let b = builder.eval(3);
        let sum = builder.eval(a + b);
        
        // Verify witness
        let witness = builder.witness();
        assert_eq!(witness.vars[sum.0 as usize], 8);
    }
}
```

## Debugging Tips

1. Use `builder.print_*` liberally during development
2. Enable backtrace collection with `BuilderFlags`
3. Add assertions to verify invariants
4. Use cycle trackers to measure performance
5. Test with small inputs before scaling up