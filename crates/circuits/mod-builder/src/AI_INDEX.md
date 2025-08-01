# OpenVM Mod-Builder Component Index

## Component Structure

```
openvm-mod-circuit-builder/
├── src/
│   ├── lib.rs                    # Module exports and feature flags
│   ├── builder.rs                # Core expression builder implementation
│   ├── field_variable.rs         # Field element representation and operations
│   ├── symbolic_expr.rs          # Symbolic expression AST and evaluation
│   ├── core_chip.rs              # OpenVM chip integration and AIR implementation
│   ├── utils.rs                  # Utility functions for limb conversions
│   ├── tests.rs                  # Unit tests
│   └── test_utils/               # Testing utilities and examples
│       ├── mod.rs                # Common test setup and utilities
│       ├── bn254.rs              # BN254 curve test implementations
│       └── bls12381.rs           # BLS12-381 curve test implementations
└── Cargo.toml                    # Dependencies and features
```

## Key Types and Traits

### Core Types

| Type | Module | Purpose |
|------|--------|---------|
| `ExprBuilder` | `builder.rs` | Main builder for field expressions and constraints |
| `ExprBuilderConfig` | `builder.rs` | Configuration for modular arithmetic parameters |
| `FieldVariable` | `field_variable.rs` | Represents field elements with overflow tracking |
| `SymbolicExpr` | `symbolic_expr.rs` | AST for field arithmetic expressions |
| `FieldExpressionCoreAir` | `core_chip.rs` | AIR implementation for OpenVM integration |
| `FieldExpr` | `builder.rs` | Generated field expression with constraints |
| `FieldExprCols` | `builder.rs` | Trace columns for field expressions |

### Key Enums

| Enum | Module | Purpose |
|------|--------|---------|
| `SymbolicExpr` | `symbolic_expr.rs` | Expression variants (Add, Mul, Div, etc.) |

### Important Structs

| Struct | Module | Key Fields |
|--------|--------|------------|
| `ExprBuilder` | `builder.rs` | `prime`, `vars`, `constraints`, `range_checker` |
| `FieldVariable` | `field_variable.rs` | `expr`, `limb_max_abs`, `max_overflow_bits` |
| `ExprBuilderConfig` | `builder.rs` | `modulus`, `num_limbs`, `limb_bits` |
| `FieldExpressionCoreAir` | `core_chip.rs` | `expr`, `offset`, `local_opcode_idx` |

## Module Dependencies

### Internal Dependencies
- `openvm-circuit-primitives`: BigInt utilities, range checking
- `openvm-circuit`: Core circuit traits and adapter interfaces
- `openvm-stark-backend`: Constraint system and AIR builder
- `openvm-instructions`: Instruction definitions

### External Dependencies
- `num-bigint`: Large integer arithmetic
- `num-traits`: Numeric traits
- `itertools`: Iterator utilities
- `serde`: Serialization support

## Feature Flags

| Feature | Purpose |
|---------|---------|
| `parallel` | Enable parallel processing in STARK backend |
| `test-utils` | Export test utilities and curve implementations |

## Public API

### Main Exports (lib.rs)
```rust
pub use builder::*;
pub use core_chip::*;
pub use field_variable::*;
pub use symbolic_expr::*;
pub mod utils;
pub mod test_utils; // When test-utils feature enabled
```

### Key Functions

| Function | Module | Purpose |
|----------|--------|---------|
| `ExprBuilder::new()` | `builder.rs` | Create new expression builder |
| `ExprBuilder::new_var()` | `builder.rs` | Create new field variable |
| `ExprBuilder::constrain_eq()` | `builder.rs` | Add equality constraint |
| `ExprBuilder::select()` | `builder.rs` | Conditional selection |
| `FieldVariable::save()` | `field_variable.rs` | Save expression as variable |
| `SymbolicExpr::evaluate()` | `symbolic_expr.rs` | Evaluate to BigInt |

## Integration Points

### With OpenVM Core
- Implements `VmCoreChip` trait
- Provides `VmCoreAir` implementation
- Uses `AdapterInterface` for runtime execution

### With Range Checker
- Integrates with `VariableRangeCheckerChip`
- Uses range checking bus for constraint verification
- Configurable decomposition bits

### With Instruction Set
- Supports custom opcodes via `local_opcode_idx`
- Handles setup operations
- Flexible opcode configuration

## Common Usage Patterns

1. **Basic Setup**
   ```rust
   let config = ExprBuilderConfig { ... };
   let builder = ExprBuilder::new(config, range_bits);
   ```

2. **Variable Creation**
   ```rust
   let (idx, var) = builder.new_var();
   ```

3. **Expression Building**
   ```rust
   let expr = &var1 + &var2 * &var3;
   ```

4. **Constraint Generation**
   ```rust
   builder.constrain_eq(&expr1, &expr2);
   ```

5. **AIR Generation**
   ```rust
   let air = builder.generate_air(offset, opcodes);
   ```