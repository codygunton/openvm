/* Provide __ucmpdi2 for rv32 targets */
/* This function compares two 64-bit unsigned integers */

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
