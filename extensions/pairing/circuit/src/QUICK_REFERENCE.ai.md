# OpenVM Pairing Circuit Extension - Quick Reference

## Installation

Add to your `Cargo.toml`:
```toml
[dependencies]
openvm-pairing-circuit = { workspace = true }
```

## Basic Usage

### Creating a Pairing Configuration

```rust
use openvm_pairing_circuit::{PairingCurve, Rv32PairingConfig};

// Configure for BN254 and BLS12-381
let curves = vec![PairingCurve::Bn254, PairingCurve::Bls12_381];
let complex_names = vec!["Bn254Fp2".to_string(), "Bls12_381Fp2".to_string()];
let config = Rv32PairingConfig::new(curves, complex_names);
```

### Building the Extension

```rust
use openvm_pairing_circuit::PairingExtension;

let pairing_ext = PairingExtension::new(vec![
    PairingCurve::Bn254,
    PairingCurve::Bls12_381,
]);
```

## Supported Operations

### Miller Loop Operations

| Operation | Opcode | Input | Output |
|-----------|---------|--------|---------|
| Miller Double Step | `MILLER_DOUBLE_STEP` | G2 point (4 Fp) | Updated point + line (8 Fp) |
| Miller Double-and-Add | `MILLER_DOUBLE_AND_ADD_STEP` | G2 points | Updated point + line |
| Evaluate Line | `EVALUATE_LINE` | Line + eval point | Evaluated line (4 Fp) |

### Fp12 Arithmetic

| Operation | Opcode | Description |
|-----------|---------|-------------|
| Add | `FP12_ADD` | Addition in Fp12 |
| Sub | `FP12_SUB` | Subtraction in Fp12 |
| Mul | `FP12_MUL` | Full multiplication in Fp12 |

### Specialized Multiplications

| Operation | Type | Description |
|-----------|------|-------------|
| `mul_013_by_013` | D-type | Multiply sparse Fp12 elements |
| `mul_by_01234` | D-type | Multiply by 5-sparse element |
| `mul_023_by_023` | M-type | Multiply sparse Fp12 elements |
| `mul_by_02345` | M-type | Multiply by 5-sparse element |

## Curve Parameters

### BN254
- **Modulus size**: 256 bits
- **Limbs**: 32 × 8-bit
- **Block size**: 32
- **Xi parameter**: [9, 1]

### BLS12-381
- **Modulus size**: 384 bits  
- **Limbs**: 48 × 8-bit
- **Block size**: 16
- **Xi parameter**: [1, 1]

## Code Examples

### Using Fp12 Operations

```rust
use openvm_pairing_circuit::Fp12;
use openvm_mod_circuit_builder::ExprBuilder;
use std::{cell::RefCell, rc::Rc};

// Create builder
let builder = Rc::new(RefCell::new(ExprBuilder::new(config, max_bits)));

// Create Fp12 elements
let mut a = Fp12::new(builder.clone());
let mut b = Fp12::new(builder.clone());

// Perform operations
let mut c = a.add(&mut b);
let mut d = a.mul(&mut b, xi);

// Save results
let indices = c.save();
```

### Creating a Miller Loop Chip

```rust
use openvm_pairing_circuit::MillerDoubleStepChip;

let chip = MillerDoubleStepChip::<F, 4, 8, 32>::new(
    adapter,
    config,
    offset,
    range_checker,
    offline_memory,
);
```

### Configuring for Guest Programs

```rust
// In your build script
let config = Rv32PairingConfig::new(
    vec![PairingCurve::Bn254],
    vec!["ComplexBn254".to_string()],
);

// Generate initialization file
let init_contents = config.generate_init_file_contents();
```

## Memory Layout

### G1 Point (BN254)
```
[x: 32 bytes][y: 32 bytes]
```

### G2 Point (BN254)
```
[x.c0: 32 bytes][x.c1: 32 bytes][y.c0: 32 bytes][y.c1: 32 bytes]
```

### Fp12 Element (BN254)
```
[c0.c0: 32 bytes][c0.c1: 32 bytes]...[c5.c0: 32 bytes][c5.c1: 32 bytes]
```

## Common Patterns

### Multi-Pairing Computation

```rust
// Guest side
let p_ptr = /* pointer to G1 points */;
let q_ptr = /* pointer to G2 points */;
let len = /* number of pairs */;

// Phantom will compute final exponentiation hint
// Circuit verifies the computation
```

### Sparse Element Multiplication

```rust
// For D-type curves (e.g., BN254)
let result = fp12.mul_by_01234(&mut x0, &mut x1, &mut x2, &mut x3, &mut x4, xi);

// For M-type curves (e.g., BLS12-381)  
let result = fp12.mul_by_02345(&mut x0, &mut x2, &mut x3, &mut x4, &mut x5, xi);
```

## Testing

### Unit Test Pattern

```rust
#[test]
fn test_pairing_operation() {
    let config = ExprBuilderConfig {
        modulus: BN254_MODULUS.clone(),
        limb_bits: BN254_LIMB_BITS,
        num_limbs: BN254_NUM_LIMBS,
    };
    
    // Set up test infrastructure
    let mut tester = VmChipTestBuilder::<F>::default();
    
    // Create and test chip
    let chip = create_chip(&mut tester, config);
    
    // Execute and verify
    tester.simple_test().expect("Verification failed");
}
```

## Performance Tips

1. **Block Size Selection**
   - BN254: Use 32-byte blocks
   - BLS12-381: Use 16-byte blocks

2. **Batching Operations**
   - Process multiple pairings together
   - Reuse intermediate values

3. **Memory Alignment**
   - Align data to block boundaries
   - Minimize cross-block reads

## Troubleshooting

### Common Issues

1. **Invalid Curve Parameter**
   - Ensure curve is in `supported_curves`
   - Check curve configuration matches

2. **Memory Alignment**
   - Verify pointers are correctly aligned
   - Check block size configuration

3. **Constraint Failures**
   - Verify input points are on curve
   - Check field element bounds

### Debug Helpers

```rust
// Enable debug logging
RUST_LOG=debug cargo test

// Check constraint counts
let usage = chip.get_chip_usage();
println!("Constraints: {:?}", usage);
```

## Integration Checklist

- [ ] Add pairing extension to VM config
- [ ] Configure supported curves
- [ ] Set up complex field names
- [ ] Generate init file
- [ ] Configure memory layout
- [ ] Test with example pairings
- [ ] Benchmark performance