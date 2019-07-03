#include <stdint.h>

uint64_t __dpdk_rdtsc() {
    union {
        uint64_t tsc_64;
        struct {
            uint32_t lo_32;
            uint32_t hi_32;
        };
    } tsc;

    __asm__ __volatile__("rdtsc" : "=a"(tsc.lo_32), "=d"(tsc.hi_32));
    return tsc.tsc_64;
}
