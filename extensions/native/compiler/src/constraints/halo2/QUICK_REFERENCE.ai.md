# Halo2 Constraints Quick Reference

## Common Usage Patterns

### 1. Basic Constraint Compilation

```rust
use openvm_native_compiler::constraints::halo2::{
    Halo2ConstraintCompiler, Halo2State
};

// Create compiler
let compiler = Halo2ConstraintCompiler::<Config>::new(num_public_values);

// Initialize state
let mut state = Halo2State::default();

// Load witness values
state.load_witness(witness);

// Compile constraints
compiler.constrain_halo2(&mut state, operations);

// Get circuit builder
let circuit = state.builder;
```

### 2. Field Operations Reference

#### BabyBear (F) Operations
```rust
// Arithmetic
DslIr::AddF(result, a, b)     // result = a + b
DslIr::SubF(result, a, b)     // result = a - b  
DslIr::MulF(result, a, b)     // result = a * b
DslIr::DivF(result, a, b)     // result = a / b

// With constants
DslIr::AddFI(result, a, const) // result = a + const
DslIr::MulFI(result, a, const) // result = a * const
DslIr::DivFIN(result, const, a) // result = const / a

// Assertions
DslIr::AssertEqF(a, b)         // assert a == b
DslIr::AssertEqFI(a, const)    // assert a == const
```

#### Extension Field (E) Operations
```rust
// Arithmetic
DslIr::AddE(result, a, b)      // Extension field add
DslIr::MulE(result, a, b)      // Extension field multiply
DslIr::MulEF(result, ext, felt) // Scalar multiplication

// Conversions
DslIr::CircuitFelts2Ext(felts, ext)  // [F; 4] -> E
DslIr::CircuitExt2Felt(felts, ext)   // E -> [F; 4]
```

#### Native Field (V) Operations
```rust
// Used for Bn254Fr values
DslIr::AddV(result, a, b)
DslIr::MulV(result, a, b)
DslIr::CastFV(var, felt)      // F -> V conversion
```

### 3. Circuit-Specific Operations

#### Poseidon2 Permutation
```rust
// State array of 3 Bn254Fr elements
DslIr::CircuitPoseidon2Permute([state0, state1, state2])
```

#### Bit Decomposition
```rust
// Decompose felt to bits
DslIr::CircuitNum2BitsF(felt_value, output_bits)

// Convert var to 64-bit limbs  
DslIr::CircuitVarTo64BitsF(var_value, output_limbs)
```

#### Conditional Selection
```rust
// Select based on condition (0 or 1)
DslIr::CircuitSelectF(cond, true_val, false_val, result)
DslIr::CircuitSelectE(cond, true_val, false_val, result)
```

### 4. Witness Management

```rust
// Load witness values by ID
DslIr::WitnessVar(assigned_var, witness_id)
DslIr::WitnessFelt(assigned_felt, witness_id)  
DslIr::WitnessExt(assigned_ext, witness_id)

// In Halo2State:
state.vars.insert(id, bn254_value);
state.felts.insert(id, babybear_value);
state.exts.insert(id, ext_value);
```

### 5. Range Checking Patterns

```rust
// Explicit reduction
DslIr::CircuitFeltReduce(felt)  // Reduce to BabyBear range
DslIr::CircuitExtReduce(ext)    // Reduce extension field

// Comparison (includes range check)
DslIr::CircuitLessThan(a, b)    // Proves a < b
```

### 6. Public Values

```rust
// Publish value at specific index
DslIr::CircuitPublish(value, index)
// Note: total public values set in compiler constructor
```

## Type Mappings

| DSL Type | Halo2 Type | Rust Type | Bits |
|----------|------------|-----------|------|
| V | AssignedValue<Fr> | Bn254Fr | ~254 |
| F | AssignedBabyBear | BabyBear | 31 |
| E | AssignedBabyBearExt4 | BabyBearExt4 | 4×31 |

## Common Patterns

### Loading Constants
```rust
// Immediate values
DslIr::ImmV(var, Bn254Fr::from(42))
DslIr::ImmF(felt, BabyBear::from_canonical_u32(42))
DslIr::ImmE(ext, BabyBearExt4::from_base(42))
```

### Field Arithmetic Chain
```rust
// Compute: result = (a + b) * c - d
let t1 = DslIr::AddF(temp1, a, b);
let t2 = DslIr::MulF(temp2, temp1, c);
let result = DslIr::SubF(output, temp2, d);
```

### Conditional Logic
```rust
// if cond then a else b
DslIr::CircuitSelectF(cond, a, b, result)
```

### Working with Arrays
```rust
// Convert array of felts to extension
let felts = [felt0, felt1, felt2, felt3];
DslIr::CircuitFelts2Ext(felts, ext_result)
```

## Performance Tips

1. **Minimize Range Checks**: Use `reduce_max_bits` only when necessary
2. **Batch Operations**: Group similar operations together
3. **Reuse Intermediates**: Store frequently used values
4. **Constant Folding**: Compiler handles `is_zero()` and `is_one()` 
5. **Lazy Evaluation**: Range checks are deferred automatically

## Error Handling

Common errors and solutions:

| Error | Cause | Solution |
|-------|-------|----------|
| "unsupported operation" | Invalid DslIr variant | Check operation support |
| Range check failure | Value exceeds field size | Add explicit reduction |
| Witness not found | Missing witness ID | Ensure witness loaded |
| Circuit overflow | Too many constraints | Optimize operations |

## Debugging

```rust
// Print operations for debugging
DslIr::PrintV(var)   // Print Bn254Fr value
DslIr::PrintF(felt)  // Print BabyBear value
DslIr::PrintE(ext)   // Print extension field

// Enable metrics
let compiler = compiler.with_profiling();
```

## Integration Example

```rust
// Full example: Compute hash of two field elements
let mut ops = vec![];

// Load inputs
ops.push(DslIr::WitnessFelt(a, 0));
ops.push(DslIr::WitnessFelt(b, 1));

// Convert to native field
ops.push(DslIr::CastFV(a_var, a));
ops.push(DslIr::CastFV(b_var, b));

// Initialize Poseidon2 state [a, b, 0]
ops.push(DslIr::ImmV(zero, Bn254Fr::ZERO));
let state = [a_var, b_var, zero];

// Apply permutation
ops.push(DslIr::CircuitPoseidon2Permute(state));

// Output first element as public
ops.push(DslIr::CircuitPublish(state[0], 0));

// Compile
compiler.constrain_halo2(&mut halo2_state, ops.into());
```