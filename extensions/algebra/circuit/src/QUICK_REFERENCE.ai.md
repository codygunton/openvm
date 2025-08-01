# Algebra Circuit Extension - Quick Reference

## Common Imports
```rust
use openvm_algebra_circuit::{
    ModularExtension, Fp2Extension,
    Rv32ModularConfig, Rv32ModularWithFp2Config,
    Fp2, modular_chip::*, fp2_chip::*
};
use openvm_mod_circuit_builder::{ExprBuilderConfig, FieldVariable};
use num_bigint::BigUint;
```

## Configuration Examples

### Basic Modular Arithmetic
```rust
// Single modulus
let config = Rv32ModularConfig::new(vec![
    BigUint::from_str("21888242871839275222246405745257275088696311157297823662689037894645226208583").unwrap()
]);

// Multiple moduli
let config = Rv32ModularConfig::new(vec![
    bn254_modulus.clone(),
    bls12_381_modulus.clone(),
]);
```

### With Fp2 Support
```rust
let config = Rv32ModularWithFp2Config::new(vec![
    ("Fq2".to_string(), bn254_modulus),
    ("Fq12".to_string(), bls12_381_modulus),
]);
```

## Guest Code Patterns

### Initialization (Auto-generated)
```rust
// In guest's build.rs
openvm_build_guest::build();

// In guest code
include!(concat!(env!("OUT_DIR"), "/init.rs"));
```

### Using Modular Arithmetic
```rust
use openvm_algebra_guest::{modular::*, IntMod};

// After init macro
type Fq = Modular<0>;  // First modulus
type Fr = Modular<1>;  // Second modulus

let a = Fq::from_u32(42);
let b = Fq::from_u32(13);
let c = a + b;
let d = a * b;
let e = a / b;  // Modular inverse
```

### Using Fp2
```rust
use openvm_algebra_guest::complex::Complex;

// After init macro
type Fq2 = Complex<Fq>;

let a = Fq2::new(Fq::from_u32(1), Fq::from_u32(2));
let b = Fq2::new(Fq::from_u32(3), Fq::from_u32(4));
let c = a + b;
let d = a * b;
```

## Chip Instantiation Patterns

### Modular Add/Sub Chip
```rust
let chip = ModularAddSubChip::<F, 1, 32>::new(
    adapter,
    ExprBuilderConfig {
        modulus: modulus.clone(),
        num_limbs: 32,
        limb_bits: 8,
    },
    opcode_offset,
    range_checker,
    offline_memory,
);
```

### Fp2 Mul/Div Chip
```rust
let chip = Fp2MulDivChip::<F, 2, 32>::new(
    adapter,
    config,
    opcode_offset,
    range_checker,
    offline_memory,
);
```

## Expression Builder Usage

### Basic Field Operations
```rust
let builder = Rc::new(RefCell::new(ExprBuilder::new(config, 1)));
let mut x = FieldVariable::new(builder.clone());
let mut y = FieldVariable::new(builder.clone());

let sum = &mut x + &mut y;
let prod = &mut x * &mut y;
let diff = &mut x - &mut y;
```

### Fp2 Operations
```rust
let mut a = Fp2::new(builder.clone());
let mut b = Fp2::new(builder.clone());

let sum = a.add(&mut b);
let prod = a.mul(&mut b);
let square = a.square();
let quotient = a.div(&mut b);  // Auto-saves
```

## Opcode Calculation
```rust
// For modular ops
let base_opcode = Rv32ModularArithmeticOpcode::ADD as usize;
let actual_opcode = base_opcode + 
    (modulus_index * Rv32ModularArithmeticOpcode::COUNT) + 
    Rv32ModularArithmeticOpcode::CLASS_OFFSET;

// For Fp2 ops
let base_opcode = Fp2Opcode::MUL as usize;
let actual_opcode = base_opcode + 
    (modulus_index * Fp2Opcode::COUNT) + 
    Fp2Opcode::CLASS_OFFSET;
```

## Common Size Configurations

| Field | Bytes | NUM_LANES | LANE_SIZE |
|-------|-------|-----------|-----------|
| BN254 | 32    | 1         | 32        |
| BLS12-381 | 48 | 3         | 16        |
| Ed25519 | 32   | 1         | 32        |
| secp256k1 | 32 | 1         | 32        |

## Phantom Hints

### Get Non-QR
```rust
// In guest
openvm_algebra_guest::modular::modulus_non_qr::<MODULUS_IDX>();
```

### Get Square Root
```rust
// In guest
use openvm_algebra_guest::modular::modulus_sqrt;

let x: Modular<0> = /* ... */;
let (is_square, sqrt) = modulus_sqrt(x);
if is_square {
    // sqrt is the square root of x
} else {
    // sqrt is the square root of x * non_qr
}
```

## Testing Helpers

### Generate Random Field Element
```rust
use rand::Rng;
let mut rng = rand::thread_rng();
let value = rng.gen_biguint_range(&BigUint::zero(), &modulus);
```

### Convert Between Representations
```rust
// BigUint to limbs
let limbs: Vec<u8> = value.to_bytes_le();

// Limbs to BigUint
let value = BigUint::from_bytes_le(&limbs);

// Pad to fixed size
let mut padded = limbs;
padded.resize(32, 0u8);
```

## Error Messages and Solutions

| Error | Likely Cause | Solution |
|-------|--------------|----------|
| "Modulus index out of range" | Wrong modulus index in guest | Check init macro ordering |
| "Modulus too large" | Field > 48 bytes | Add 64-byte chip variant |
| "Either x or x*non_qr should be square" | Bug in sqrt algorithm | Check non-QR computation |
| "Constraint violation" | Auto-save not triggered | Pre-save complex expressions |

## Performance Tips

1. **Batch Operations**: Process multiple field elements together
2. **Reuse Chips**: Clone existing chips instead of creating new
3. **Minimize Saves**: Let auto-save handle most cases
4. **Share Lookup Tables**: Use existing bitwise lookup chips

## Common Patterns

### Extension with Custom Modulus
```rust
impl MyExtension {
    fn build(&self, builder: &mut VmInventoryBuilder<F>) -> Result</*...*/> {
        // 1. Get system components
        let SystemPort { execution_bus, program_bus, memory_bridge } = 
            builder.system_port();
        
        // 2. Find or create shared resources
        let bitwise_chip = builder.find_chip::<SharedBitwiseOperationLookupChip<8>>()
            .first()
            .cloned()
            .unwrap_or_else(|| /* create new */);
        
        // 3. Create chips for each modulus
        for (i, modulus) in self.moduli.iter().enumerate() {
            let bytes = modulus.bits().div_ceil(8);
            let chip = match bytes {
                1..=32 => /* 32-byte variant */,
                33..=48 => /* 48-byte variant */,
                _ => panic!("Unsupported modulus size"),
            };
            
            // 4. Register with correct opcodes
            let opcodes = /* calculate opcode range */;
            inventory.add_executor(chip, opcodes)?;
        }
        
        Ok(inventory)
    }
}
```

### Guest Type Setup
```rust
// Define modular types
openvm_algebra_guest::moduli_macros::moduli_declare! {
    Bls12381Fq { modulus = "0x1a0111..." },
    Bls12381Fr { modulus = "0x73eda7..." },
}

// Define Fp2 types
openvm_algebra_guest::complex_macros::complex_declare! {
    Bls12381Fq2 { mod_type = Bls12381Fq },
}

// Use in main
moduli_init! { 
    "0x1a0111...",  // Fq 
    "0x73eda7...",  // Fr
}
complex_init! {
    Bls12381Fq2 { mod_idx = 0 },
}
```