// Compiler builtins needed by SoftFloat but not provided by Rust's compiler-builtins
// for cross-language linkage
// TODO: why are these needed?

typedef unsigned long long uint64_t;

// Compare two 64-bit unsigned integers
// Returns: 0 if a < b, 1 if a == b, 2 if a > b
int __ucmpdi2(uint64_t a, uint64_t b) {
    if (a < b) return 0;
    if (a > b) return 2;
    return 1;
}
