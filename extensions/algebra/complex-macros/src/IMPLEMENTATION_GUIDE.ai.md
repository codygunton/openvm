# Complex Macros - Implementation Guide

## Adding a New Complex Field Operation

### Step 1: Define the Operation in Algebra Transpiler
First, add the new operation to the `ComplexExtFieldBaseFunct7` enum in `openvm-algebra-transpiler`:

```rust
pub enum ComplexExtFieldBaseFunct7 {
    Add = 0,
    Sub = 1,
    Mul = 2,
    Div = 3,
    Setup = 4,
    NewOp = 5,  // Your new operation
    // ...
}
```

### Step 2: Add to complex_declare! Macro
In the `complex_declare!` macro, add the new operation:

1. **Create extern function identifier**:
```rust
create_extern_func!(complex_newop_extern_func);
```

2. **Add extern declaration**:
```rust
extern "C" {
    // ... existing functions ...
    fn #complex_newop_extern_func(rd: usize, rs1: usize, rs2: usize);
}
```

3. **Implement the operation method**:
```rust
impl #struct_name {
    #[inline(always)]
    fn newop_assign_impl(&mut self, other: &Self) {
        #[cfg(not(target_os = "zkvm"))]
        {
            // Native implementation
            // Example: component-wise operation
            self.c0.newop_assign(&other.c0);
            self.c1.newop_assign(&other.c1);
        }
        #[cfg(target_os = "zkvm")]
        {
            Self::set_up_once();
            unsafe {
                #complex_newop_extern_func(
                    self as *mut Self as usize,
                    self as *const Self as usize,
                    other as *const Self as usize
                );
            }
        }
    }
}
```

4. **Add trait implementations** if needed:
```rust
pub trait NewOp<Rhs = Self> {
    type Output;
    fn newop(self, rhs: Rhs) -> Self::Output;
}

impl NewOp for #struct_name {
    type Output = Self;
    fn newop(mut self, other: Self) -> Self::Output {
        self.newop_assign_impl(&other);
        self
    }
}
```

### Step 3: Add to complex_init! Macro
In the `complex_init!` macro, generate the FFI function:

```rust
// In the loop generating extern functions
for op_type in ["add", "sub", "mul", "div", "newop"] {
    // ... existing code ...
}
```

## Extending Complex Field Types

### Supporting Different Base Fields
The current implementation assumes any field implementing `IntMod`. To add constraints:

```rust
// In complex_declare!, add bounds checking
let result = TokenStream::from(quote::quote_spanned! { span.into() =>
    #[derive(Clone, PartialEq, Eq)]
    pub struct #struct_name 
    where 
        #intmod_type: openvm_algebra_guest::IntMod + SpecialTrait
    {
        pub c0: #intmod_type,
        pub c1: #intmod_type,
    }
});
```

### Supporting Different Irreducible Polynomials
Currently hardcoded to `X² + 1`. To make configurable:

1. **Add parameter to macro**:
```rust
complex_declare! {
    ComplexType { 
        mod_type = Fq,
        irreducible = "X^2 + X + 1"  // New parameter
    }
}
```

2. **Parse the parameter**:
```rust
let mut irreducible: Option<String> = None;
for param in item.params {
    match param.name.to_string().as_str() {
        "irreducible" => {
            if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(s), ..
            }) = param.value {
                irreducible = Some(s.value());
            }
        }
        // ... other parameters
    }
}
```

3. **Adjust arithmetic based on polynomial**:
```rust
// For X² + X + 1, multiplication becomes:
// (a + bX)(c + dX) = ac + (ad + bc)X + bdX²
//                   = ac + (ad + bc)X + bd(-X - 1)
//                   = (ac - bd) + (ad + bc - bd)X
```

## Performance Optimization Techniques

### 1. Minimizing Memory Allocations
Use `MaybeUninit` for all temporary values:

```rust
fn optimized_operation(&self, other: &Self) -> Self {
    let mut uninit: core::mem::MaybeUninit<Self> = 
        core::mem::MaybeUninit::uninit();
    
    unsafe {
        // Perform operation directly into uninitialized memory
        complex_op_extern_func(
            uninit.as_mut_ptr() as usize,
            self as *const Self as usize,
            other as *const Self as usize
        );
        uninit.assume_init()
    }
}
```

### 2. Avoiding Redundant Setup Calls
Cache setup state more efficiently:

```rust
// Instead of OnceBool, use OnceCell for richer state
use openvm_algebra_guest::once_cell::sync::OnceCell;

static SETUP_STATE: OnceCell<SetupInfo> = OnceCell::new();

fn set_up_once() {
    SETUP_STATE.get_or_init(|| {
        unsafe { 
            #complex_setup_extern_func();
            SetupInfo { /* cached data */ }
        }
    });
}
```

### 3. Batched Operations
For multiple operations on the same types:

```rust
impl #struct_name {
    pub fn batch_mul(pairs: &[(Self, Self)]) -> Vec<Self> {
        Self::set_up_once();
        
        #[cfg(target_os = "zkvm")]
        unsafe {
            // Single setup, multiple operations
            pairs.iter().map(|(a, b)| {
                let mut uninit = MaybeUninit::uninit();
                #complex_mul_extern_func(
                    uninit.as_mut_ptr() as usize,
                    a as *const Self as usize,
                    b as *const Self as usize
                );
                uninit.assume_init()
            }).collect()
        }
        
        #[cfg(not(target_os = "zkvm"))]
        pairs.iter().map(|(a, b)| a * b).collect()
    }
}
```

## Debugging Complex Macro Issues

### 1. Macro Expansion Debugging
Use `cargo expand` to see generated code:

```bash
cargo expand --package my-guest-program
```

### 2. Add Debug Prints During Generation
```rust
// In complex_declare!
println!("Generating complex type: {} with base: {:?}", 
         struct_name, intmod_type);

// This prints during compilation
```

### 3. Runtime Debugging in zkVM
Add debug functions that work in zkVM:

```rust
impl #struct_name {
    #[cfg(target_os = "zkvm")]
    pub fn debug_print(&self) {
        // Use openvm's debugging facilities
        openvm::println!("Complex({:?}, {:?})", self.c0, self.c1);
    }
}
```

## Integration with Circuit Layer

### 1. Opcode Routing
Ensure opcodes match between macro and circuit:

```rust
// In macro
let funct7 = ComplexExtFieldBaseFunct7::Add as usize
    + complex_idx * ComplexExtFieldBaseFunct7::COMPLEX_EXT_FIELD_MAX_KINDS;

// In circuit
let opcode_base = ComplexExtFieldBaseFunct7::Add as usize
    + field_idx * ComplexExtFieldBaseFunct7::COMPLEX_EXT_FIELD_MAX_KINDS;
```

### 2. Memory Layout Compatibility
Ensure `#[repr(C)]` for circuit compatibility:

```rust
#[repr(C)]  // Critical for FFI
pub struct #struct_name {
    pub c0: #intmod_type,
    pub c1: #intmod_type,
}
```

### 3. Instruction Encoding
The encoding must match circuit expectations:

```rust
// Macro generates:
openvm::platform::custom_insn_r!(
    opcode = OPCODE,           // Fixed algebra opcode
    funct3 = FUNCT3,          // Complex field discriminator
    funct7 = computed_value,  // Operation + field index
    rd = destination,
    rs1 = source1,
    rs2 = source2
);
```

## Common Pitfalls and Solutions

### 1. Modulus Index Confusion
**Problem**: `mod_idx` in `complex_init!` doesn't match `moduli_init!` order

**Solution**: Add validation:
```rust
// Generate assertion in init code
assert!(#mod_idx < moduli_count, 
        "mod_idx {} exceeds moduli count", #mod_idx);
```

### 2. Missing Trait Bounds
**Problem**: Generated code fails to compile due to missing traits

**Solution**: Add comprehensive bounds:
```rust
impl #struct_name 
where 
    #intmod_type: IntMod + Clone + Debug + Serialize
{
    // methods
}
```

### 3. Namespace Collisions
**Problem**: Multiple complex types generate conflicting function names

**Solution**: Use fully qualified names:
```rust
let func_name = syn::Ident::new(
    &format!("{}_{}_{}_{}", 
             module_path!().replace("::", "_"),
             "complex",
             op_type,
             struct_name),
    span.into(),
);
```

## Testing Macro-Generated Code

### 1. Unit Test Template
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    complex_declare! {
        TestComplex { mod_type = TestMod }
    }
    
    #[test]
    fn test_complex_arithmetic() {
        // Mock the IntMod type
        struct TestMod(u32);
        impl IntMod for TestMod { /* ... */ }
        
        let a = TestComplex::new(TestMod(1), TestMod(2));
        let b = TestComplex::new(TestMod(3), TestMod(4));
        
        // Test will use native implementation
        let c = a + b;
        assert_eq!(c.c0.0, 4);
        assert_eq!(c.c1.0, 6);
    }
}
```

### 2. Integration Test Pattern
```rust
// In tests/integration_test.rs
#[test]
fn test_complex_with_real_modulus() {
    openvm_algebra_moduli_macros::moduli_declare! {
        Fq { modulus = "0x30644e..." }
    }
    
    openvm_algebra_complex_macros::complex_declare! {
        Fq2 { mod_type = Fq }
    }
    
    // Test both native and simulated zkVM execution
}
```

## Advanced Customization

### 1. Custom Serialization
Override default serde implementation:

```rust
// In macro, make serde optional
#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct #struct_name { /* ... */ }

// Add custom implementation
#[cfg(feature = "serde")]
impl serde::Serialize for #struct_name {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        // Custom format
    }
}
```

### 2. Const Evaluation Support
Enable const operations where possible:

```rust
impl #struct_name {
    pub const fn const_new(c0: #intmod_type, c1: #intmod_type) -> Self {
        Self { c0, c1 }
    }
    
    pub const fn const_zero() -> Self {
        Self::const_new(
            <#intmod_type as IntMod>::ZERO,
            <#intmod_type as IntMod>::ZERO
        )
    }
}
```

### 3. Generic Programming Support
Make complex types work in generic contexts:

```rust
// Generate additional trait
impl<T> From<T> for #struct_name 
where 
    #intmod_type: From<T>
{
    fn from(value: T) -> Self {
        Self::new(
            #intmod_type::from(value),
            <#intmod_type as IntMod>::ZERO
        )
    }
}
```

This implementation guide provides the foundation for extending and customizing the complex field macros for various use cases in the OpenVM ecosystem.