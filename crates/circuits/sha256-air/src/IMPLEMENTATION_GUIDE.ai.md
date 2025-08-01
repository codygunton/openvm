# SHA256-AIR Implementation Guide

## Architecture Deep Dive

### Trace Layout
The SHA256 computation is organized into a trace matrix with a specific structure:

```
Block 0:
  Row 0-15:  Round rows (4 rounds each, total 64 rounds)
  Row 16:    Digest row (finalization)
Block 1:
  Row 17-32: Round rows
  Row 33:    Digest row
...
Padding rows: Fill to power of 2
```

### Column Structure Design

#### Shared Fields (First 3 fields identical in both column types)
```rust
// Both Sha256RoundCols and Sha256DigestCols share:
1. flags: Sha256FlagsCols<T>
2. work_vars/hash: Sha256WorkVarsCols<T>  // Different names, same type
3. schedule_helper: Sha256MessageHelperCols<T>
```

This design enables polymorphic constraint application while maintaining type safety.

#### Work Variables Layout
```rust
Sha256WorkVarsCols {
    a: [[T; 32]; 4],      // 4 values of 'a', each 32 bits
    e: [[T; 32]; 4],      // 4 values of 'e', each 32 bits
    carry_a: [[T; 2]; 4], // Carries for 'a' computation (16-bit limbs)
    carry_e: [[T; 2]; 4], // Carries for 'e' computation (16-bit limbs)
}
```

### Constraint Implementation

#### 1. Message Schedule Constraints
The message schedule W[t] is computed as:
- W[0..15]: Input message words
- W[16..63]: W[t] = σ1(W[t-2]) + W[t-7] + σ0(W[t-15]) + W[t-16]

Implementation challenge: Can only constrain across 2 consecutive rows.
Solution: Use intermediate values (intermed_4, intermed_8, intermed_12) to propagate partial sums.

```rust
// Row propagation pattern
Row N:   compute intermed_4 = W[t-4] + σ0(W[t-3])
Row N+1: intermed_8 = prev.intermed_4
Row N+4: intermed_12 = prev.intermed_8
Row N+5: Use intermed_12 in final W[t] computation
```

#### 2. Working Variable Updates
Each round computes:
```rust
T1 = h + Σ1(e) + Ch(e,f,g) + K[t] + W[t]
T2 = Σ0(a) + Maj(a,b,c)
new_e = d + T1
new_a = T1 + T2
```

The constraint system verifies these using 16-bit limb arithmetic with carry tracking.

#### 3. Hash Chaining
Block-to-block hash chaining uses a self-interaction bus:
```rust
// Digest row sends: [hash_words..., next_block_idx]
self.bus.send(composed_hash.chain(next_global_idx), is_digest_row);

// Next block receives: [prev_hash_words..., current_block_idx]
self.bus.receive(prev_hash.chain(global_idx), is_digest_row);
```

### Bitwise Operations Integration

All bitwise operations use lookup tables for efficiency:
```rust
// Range checks for carries
bitwise_lookup_bus.send_range(carry_a, carry_e).eval(builder, is_round_row);

// Final hash byte validation
bitwise_lookup_bus.send_range(hash_byte1, hash_byte2).eval(builder, is_digest_row);
```

### Trace Generation Strategy

#### Two-Pass Generation
1. **First Pass**: Generate main computation trace
   - Compute message schedule
   - Update working variables
   - Track carries and intermediate values

2. **Second Pass**: Fill constraint-required dummy values
   - intermed_12 for rows 0-3 and 15-16
   - intermed_4 for row 0 and 16
   - carry_a/carry_e for digest rows

#### Dummy Value Computation
```rust
// Example: Generate carry values for digest row
fn generate_carry_ae(local: &RoundCols, next: &mut RoundCols) {
    // Compute in field to match constraint arithmetic
    let sum = d_limb + t1_limb_sum + prev_carry - cur_e_limb;
    let carry_e = sum * F::from_canonical_u32(1 << 16).inverse();
    next.carry_e[i][j] = carry_e;
}
```

### Performance Optimizations

1. **Parallel Block Processing**: Each block can be traced independently
2. **Precomputed Constants**: Invalid row carries are precomputed
3. **Batch Bitwise Lookups**: Request all lookups before constraint evaluation
4. **Efficient Flag Encoding**: Row indices use 5 bits with custom encoder

### Security Considerations

1. **Soundness**: Padding row after last digest ensures trace commitment
2. **Constraint Coverage**: All 64 rounds fully constrained
3. **Carry Bounds**: Although only checking carry < 256, arithmetic prevents overflow
4. **Hash Chaining**: Self-interaction bus ensures correct block sequencing

## Common Implementation Patterns

### Adding New Constraints
```rust
impl Sha256Air {
    fn eval_new_constraint<AB: InteractionBuilder>(&self, builder: &mut AB, ...) {
        // 1. Access columns with proper typing
        let cols: &Sha256RoundCols<AB::Var> = ...;
        
        // 2. Use conditional constraints
        builder.when(cols.flags.is_round_row)
            .assert_eq(computed_value, expected_value);
        
        // 3. Handle degree limitations
        // If constraint degree > 3, use intermediate values
    }
}
```

### Extending Column Layout
When adding new columns:
1. Update both `Sha256RoundCols` and `Sha256DigestCols` if shared
2. Implement `AlignedBorrow` trait derivation
3. Update `SHA256_ROUND_WIDTH` and `SHA256_DIGEST_WIDTH` calculations
4. Fill new columns in trace generation