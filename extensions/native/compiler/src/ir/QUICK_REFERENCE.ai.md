# OpenVM Native Compiler IR - Quick Reference

## Variable Types

```rust
// Native field element
let x: Var<N> = builder.eval(123);

// Emulated field element  
let y: Felt<F> = builder.eval(456);

// Extension field element
let z: Ext<F, EF> = builder.eval(ext_value);

// Compile-time or runtime size
let size: Usize<N> = 100.into(); // Static
let size: Usize<N> = builder.eval(n); // Dynamic
```

## Basic Operations

### Arithmetic
```rust
// Addition
let sum = builder.eval(x + y);
let sum = builder.eval(x + 5); // With constant

// Subtraction  
let diff = builder.eval(x - y);

// Multiplication
let prod = builder.eval(x * y);

// Division (field elements only)
let quot = builder.eval(x / y);

// Negation
let neg = builder.eval(-x);
```

### Memory Operations
```rust
// Allocate array
let arr: Ptr<N> = builder.alloc(size);

// Load from memory
let value = builder.load(arr, index);

// Store to memory
builder.store(arr, index, value);

// Array construction
let array = builder.array(vec![a, b, c]);
```

## Control Flow

### Conditionals
```rust
// If-then
builder.if_eq(x, y).then(|builder| {
    // True branch
});

// If-then-else
builder.if_ne(x, y).then(|builder| {
    // True branch
}).else_then(|builder| {
    // False branch  
});

// Conditional assignment
let result = builder.select(condition, true_val, false_val);
```

### Loops
```rust
// Static range
builder.range(0, 10).for_each(|i, builder| {
    // Loop body with index i
});

// Dynamic range
let start = builder.eval(0);
let end = builder.eval(n);
builder.range(start, end).for_each(|i, builder| {
    // Loop body
});

// Break/continue
builder.break_loop();
builder.continue_loop();

// Conditional break
builder.break_if(condition);
```

## Builder Methods

### Variable Creation
```rust
// From constant
let x = builder.eval(42);
let x = builder.constant(42);

// Uninitialized
let x: Var<N> = builder.uninit();

// From witness
let x = builder.witness(witness_ref);
```

### Debugging
```rust
// Print values
builder.print_v(x);
builder.print_f(felt);
builder.print_e(ext);

// Assertions
builder.assert_eq(x, y);
builder.assert_ne(x, y);
```

### Advanced Operations
```rust
// Poseidon permutation
let result = builder.poseidon_permute(input_array);

// Cycle tracking
builder.cycle_tracker_start("operation");
// ... operations ...
builder.cycle_tracker_end("operation");

// Hint for non-deterministic values
let value = builder.hint(witness_ref);
```

## Arrays and Collections

```rust
// Fixed array (compile-time size)
let arr = builder.array(vec![a, b, c]);

// Dynamic array
let arr = builder.vec(values);

// Array operations
let elem = builder.get(&arr, index);
builder.set(&mut arr, index, value);
let len = arr.len();

// Memory-backed array
let ptr = builder.alloc(size);
let arr = Array::Dyn(ptr, size);
```

## Type Conversions

```rust
// Usize conversions
let static_size: Usize<N> = 100.into();
let dynamic_size: Usize<N> = var.into();

// To SymbolicVar
let sym: SymbolicVar<N> = x.into();
let sym = SymbolicVar::from(x);

// From RVar
let var = builder.eval(rvar);
```

## Common Patterns

### Loop with accumulator
```rust
let mut sum = builder.eval(0);
builder.range(0, n).for_each(|i, builder| {
    let value = builder.get(&array, i);
    builder.assign(&mut sum, sum + value);
});
```

### Conditional update
```rust
let result = builder.if_eq(flag, 1)
    .then_value(new_value)
    .else_value(old_value);
```

### Array initialization
```rust
let mut arr = builder.vec(vec![builder.eval(0); size]);
builder.range(0, size).for_each(|i, builder| {
    builder.set(&mut arr, i, builder.eval(i));
});
```

### Early exit pattern
```rust
builder.range(0, n).for_each(|i, builder| {
    let value = builder.get(&array, i);
    builder.if_eq(value, target).then(|builder| {
        builder.break_loop();
    });
});
```

## Config Types

```rust
// Common configs
type BabyBearConfig = /* ... */;
type BN254Config = /* ... */;

// Access field types
type NativeField = <C as Config>::N;
type TargetField = <C as Config>::F;
type ExtField = <C as Config>::EF;
```

## Memory Safety

```rust
// Bounds checking (debug mode)
let value = builder.load(ptr, index); // Panics if out of bounds

// Safe access pattern
builder.if_ne(index, array.len()).then(|builder| {
    let value = builder.get(&array, index);
    // Use value
});
```

## Performance Tips

1. Use static sizes when possible
2. Batch similar operations
3. Minimize memory allocations
4. Use symbolic operations for complex expressions
5. Avoid unnecessary type conversions

## Common Errors

- **Type mismatch**: Mixing Var, Felt, and Ext types
- **Uninitialized variables**: Using variables before assignment
- **Out of bounds**: Array access beyond allocated size
- **Break outside loop**: Using break_loop() outside loop context
- **Symbolic evaluation**: Not calling builder.eval() on symbolic expressions