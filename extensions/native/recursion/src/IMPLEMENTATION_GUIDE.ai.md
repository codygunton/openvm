# OpenVM Native Recursion Extension - Implementation Guide

## Overview

The recursion extension implements STARK proof verification within the OpenVM zkVM, enabling recursive proof composition. This guide covers the technical implementation details.

## Core Architecture

### 1. Verification Flow

The verification process follows these steps:

```rust
// 1. Build verification program
let program = VerifierProgram::<InnerConfig>::build(advice, &fri_params);

// 2. Program reads proof from input
let proof: StarkProofVariable<_> = builder.uninit();
Proof::witness(&proof, &mut builder);

// 3. Initialize PCS configuration
let pcs = TwoAdicFriPcsVariable { config: const_fri_config(...) };

// 4. Run verification
StarkVerifier::verify(&mut builder, &pcs, &advice, &proof);
```

### 2. Configuration System

The extension supports two main configurations:

#### Inner Configuration (BabyBear)
```rust
pub type InnerConfig = AsmConfig<InnerVal, InnerChallenge>;
// Where:
// InnerVal = BabyBear (31-bit prime field)
// InnerChallenge = BinomialExtensionField<BabyBear, 4>
```

#### Outer Configuration (Bn254)
```rust
impl Config for OuterConfig {
    type N = Bn254Fr;  // Native field for EVM
    type F = BabyBear; // Computation field
    type EF = BinomialExtensionField<BabyBear, 4>;
}
```

### 3. Proof Structure

The proof is organized hierarchically:

```rust
pub struct StarkProofVariable<C: Config> {
    pub commitments: CommitmentsVariable<C>,
    pub opening: OpeningProofVariable<C>,
    pub per_air: Array<C, AirProofDataVariable<C>>,
    pub air_perm_by_height: Array<C, Usize<C::N>>,
    pub log_up_pow_witness: Felt<C::F>,
}
```

### 4. Multi-STARK Verification

The verifier supports multiple AIRs with different trace heights:

```rust
// Verify each AIR's constraints
for (i, air_const) in m_advice.per_air.iter().enumerate() {
    StarkVerifier::verify_single_rap_constraints(
        builder,
        air_const,
        preprocessed_values,
        &partitioned_main_values,
        quotient_chunks,
        // ... other parameters
    );
}
```

## Key Implementation Details

### 1. Challenge Generation

The challenger uses a duplex sponge construction:

```rust
impl<C: Config> DuplexChallengerVariable<C> {
    pub fn new(builder: &mut Builder<C>) -> Self {
        let mut sponge_state = builder.array(PERMUTATION_WIDTH);
        // Initialize with zeros
        // ...
        Self { sponge_state, input_buffer, output_buffer }
    }
    
    pub fn observe(&mut self, builder: &mut Builder<C>, value: Felt<C::F>) {
        // Absorb value into sponge
    }
    
    pub fn sample_ext(&mut self, builder: &mut Builder<C>) -> Ext<C::F, C::EF> {
        // Squeeze challenge from sponge
    }
}
```

### 2. FRI Protocol

The FRI implementation uses two-adic domains:

```rust
impl<C: Config> TwoAdicMultiplicativeCosetVariable<C> {
    pub fn next_point(&self, builder: &mut Builder<C>, x: Ext<C::F, C::EF>) -> Ext<C::F, C::EF> {
        // Compute x * g where g is the generator
        builder.eval(x * self.g)
    }
    
    pub fn selectors_at_point(&self, builder: &mut Builder<C>, zeta: Ext<C::F, C::EF>) -> LagrangeSelectors<Ext<C::F, C::EF>> {
        // Compute Lagrange selectors for constraint evaluation
    }
}
```

### 3. Constraint Evaluation

Constraints are evaluated using a folding approach:

```rust
fn eval_constraints(
    builder: &mut Builder<C>,
    constraints: &SymbolicExpressionDag<C::F>,
    // ... trace values
) -> Ext<C::F, C::EF> {
    let mut folder = RecursiveVerifierConstraintFolder { /* ... */ };
    folder.eval_constraints(constraints);
    builder.eval(folder.accumulator)
}
```

### 4. Memory Management

The verifier uses sub-builders for memory efficiency:

```rust
if !builder.flags.static_only {
    // Create sub-builder to recycle stack space
    let mut tmp_builder = builder.create_sub_builder();
    
    // Save heap pointer to recycle heap space
    let old_heap_ptr = tmp_builder.load_heap_ptr();
    
    // Run verification
    // ...
    
    // Restore heap pointer
    tmp_builder.store_heap_ptr(old_heap_ptr);
}
```

## Optimization Techniques

### 1. Witness Hints

The extension uses hints to optimize witness generation:

```rust
impl<C: Config> Hintable<C> for StarkProof<SC> {
    type HintVariable = StarkProofHint<C>;
    
    fn read(builder: &mut Builder<C>, hint: Self::HintVariable) -> Self {
        // Efficiently read proof from hint data
    }
}
```

### 2. Parallel Domain Evaluation

Multiple domains can be evaluated in parallel:

```rust
let domains = builder.array(num_airs);
builder.range(0, num_airs).for_each(|i_vec, builder| {
    let domain = pcs.natural_domain_for_log_degree(builder, log_degree);
    builder.set_value(&domains, i, domain);
});
```

### 3. Batched Opening Verification

FRI openings are batched for efficiency:

```rust
let rounds = builder.array::<TwoAdicPcsRoundVariable<_>>(total_rounds);
// Build all rounds
// ...
pcs.verify(builder, rounds, opening.proof, log_max_height, challenger);
```

## Security Considerations

### 1. Proof of Work

The verifier includes proof-of-work verification:

```rust
challenger.check_witness(builder, m_advice.log_up_pow_bits, log_up_pow_witness);
```

### 2. Constraint Validation

All constraints are validated before evaluation:

```rust
// Check trace height constraints
StarkVerifier::check_trace_height_constraints(
    builder,
    &m_advice_var.trace_height_constraint_system,
    air_proofs,
);
```

### 3. Domain Separation

Different proof components use separate challenge rounds:

```rust
// 1. Observe preprocessed commitments
// 2. Observe main trace commitments  
// 3. Sample challenges
// 4. Observe after-challenge commitments
// 5. Sample alpha for constraint folding
// 6. Observe quotient commitment
// 7. Sample zeta for opening point
```

## Integration with OpenVM

### 1. Compiler Integration

The recursion extension uses the native compiler's IR:

```rust
use openvm_native_compiler::ir::{Builder, Config, DslIr, Ext, Felt};

let mut builder = Builder::<InnerConfig>::default();
// Build verification logic
let program = builder.compile_isa_with_options(options);
```

### 2. Cycle Tracking

Performance monitoring is built-in:

```rust
builder.cycle_tracker_start("VerifierProgram");
// ... verification logic
builder.cycle_tracker_end("VerifierProgram");
```

### 3. Feature Flags

Optional features can be enabled:

```toml
[features]
static-verifier = ["snark-verifier-sdk"]
evm-prove = ["static-verifier"]
test-utils = ["openvm-circuit/test-utils"]
```

## Common Patterns

### 1. Creating a Verifier Program

```rust
let advice = new_from_inner_multi_vk(&vk);
let program = VerifierProgram::<InnerConfig>::build(advice, &fri_params);
```

### 2. Custom Verification Logic

```rust
impl<C: Config> StarkVerifier<C> {
    pub fn verify_custom(
        builder: &mut Builder<C>,
        // ... parameters
    ) {
        // Custom verification logic
    }
}
```

### 3. Integrating with Halo2

```rust
#[cfg(feature = "static-verifier")]
use crate::halo2::AggregationCircuit;

let circuit = AggregationCircuit::new(&params, &stark_vk, &proof);
```