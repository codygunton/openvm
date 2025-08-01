# SHA256-AIR Quick Reference

## Core Usage Pattern
```rust
use openvm_sha256_air::{Sha256Air, generate_trace};
use openvm_circuit_primitives::bitwise_op_lookup::{SharedBitwiseOperationLookupChip, BitwiseOperationLookupBus};
use openvm_stark_backend::interaction::InteractionBuilder;

// Create SHA256 AIR with bitwise lookup integration
let bitwise_bus = BitwiseOperationLookupBus::new(bus_index);
let sha256_air = Sha256Air::new(bitwise_bus, self_bus_idx);

// Generate trace for SHA256 blocks
let blocks: Vec<([u8; 64], bool)> = vec![
    (message_block, false),  // Not last block
    (final_block, true),     // Last block of message
];
let trace = generate_trace(&sha256_air, bitwise_chip, blocks);
```

## Key Constants
```rust
// Block and word sizes
SHA256_BLOCK_U8S = 64         // 512 bits = 64 bytes per block
SHA256_WORD_BITS = 32         // Standard SHA256 word size
SHA256_HASH_WORDS = 8         // Output hash is 8 words

// Trace dimensions
SHA256_ROWS_PER_BLOCK = 17    // 16 round rows + 1 digest row
SHA256_ROUNDS_PER_ROW = 4     // Process 4 rounds per row
SHA256_WIDTH = max(round_width, digest_width)  // Dynamic column width
```

## Column Access Patterns
```rust
// Round row columns (rows 0-15)
let round_cols: &Sha256RoundCols<F> = &row[start_col..start_col + SHA256_ROUND_WIDTH];
round_cols.flags.is_round_row       // Should be 1
round_cols.work_vars.a[i]           // Working variable a for round i
round_cols.message_schedule.w[i]    // Message word for round i

// Digest row columns (row 16)
let digest_cols: &Sha256DigestCols<F> = &row[start_col..start_col + SHA256_DIGEST_WIDTH];
digest_cols.final_hash[i]           // Final hash word i
digest_cols.prev_hash[i]            // Previous block's hash
```

## Constraint Helpers
```rust
// Bitwise operations with field elements
ch_field(&e, &f, &g)              // Ch(e,f,g) = (e AND f) XOR (NOT e AND g)
maj_field(&a, &b, &c)             // Maj(a,b,c) = (a AND b) XOR (a AND c) XOR (b AND c)
big_sig0_field(&x)                // Σ0(x) = ROTR(2,x) XOR ROTR(13,x) XOR ROTR(22,x)
big_sig1_field(&x)                // Σ1(x) = ROTR(6,x) XOR ROTR(11,x) XOR ROTR(25,x)
small_sig0_field(&x)              // σ0(x) = ROTR(7,x) XOR ROTR(18,x) XOR SHR(3,x)
small_sig1_field(&x)              // σ1(x) = ROTR(17,x) XOR ROTR(19,x) XOR SHR(10,x)

// Word addition with carry tracking
constraint_word_addition(
    builder,
    &[&term1_bits, &term2_bits],   // Terms in bit representation
    &[&term3_limbs],                // Terms in 16-bit limbs
    &expected_sum,                  // Expected sum in bits
    &carries                        // Carry values for verification
);
```

## Trace Generation Flow
```rust
// 1. First pass: Generate block traces
for (input, is_last_block) in blocks {
    sha256_air.generate_block_trace(
        trace_slice,
        width,
        start_col,
        &input_words,
        bitwise_chip.clone(),
        &prev_hash,
        is_last_block,
        global_idx,
        local_idx,
        &buffer_values
    );
}

// 2. Second pass: Fill missing cells
sha256_air.generate_missing_cells(trace, width, start_col);

// 3. Padding rows (if needed)
sha256_air.generate_default_row(padding_row);
```

## Common Patterns

### Multi-block Message Processing
```rust
let mut prev_hash = SHA256_H;  // Initial hash values
let mut local_idx = 0;

for (block, is_last) in message_blocks {
    // Process block
    let new_hash = if is_last {
        SHA256_H  // Reset for next message
    } else {
        Sha256Air::get_block_hash(&prev_hash, block)
    };
    
    // Update indices
    local_idx = if is_last { 0 } else { local_idx + 1 };
    prev_hash = new_hash;
}
```

### Flag Checking
```rust
// Check row type
if flags.is_round_row { /* Round row logic */ }
if flags.is_digest_row { /* Digest row logic */ }
if flags.is_first_4_rows { /* Special handling for rows 0-3 */ }

// Check block position
if flags.is_last_block { /* Last block of message */ }
if flags.is_padding_row() { /* Padding row (trace padding) */ }
```