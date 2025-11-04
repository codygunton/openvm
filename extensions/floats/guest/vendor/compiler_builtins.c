// Compiler builtins needed by SoftFloat but not provided by libgcc for rv32 targets.
//
// NOTE: On rv32 targets, we must manually implement 64-bit comparison by splitting
// into 32-bit high/low parts. Using native 64-bit comparison would create a circular
// dependency where the compiler generates calls to __ucmpdi2 for the comparison itself.

typedef unsigned long long uint64_t;
typedef unsigned int uint32_t;

typedef union {
    uint64_t all;
    struct {
#if __BYTE_ORDER__ == __ORDER_LITTLE_ENDIAN__
        uint32_t low;
        uint32_t high;
#else
        uint32_t high;
        uint32_t low;
#endif
    } s;
} dwords;

// Compare two 64-bit unsigned integers
// Returns: 0 if a < b, 1 if a == b, 2 if a > b
int __ucmpdi2(uint64_t a, uint64_t b) {
    dwords au = {.all = a};
    dwords bu = {.all = b};

    if (au.s.high < bu.s.high)
        return 0;
    else if (au.s.high > bu.s.high)
        return 2;
    if (au.s.low < bu.s.low)
        return 0;
    else if (au.s.low > bu.s.low)
        return 2;
    return 1;  /* equal */
}
