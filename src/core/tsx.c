/* SPDX-License-Identifier: BSD-3-Clause
 * Copyright(c) 2012,2013 Intel Corporation
 */

#include <stdint.h>

#define RTE_XBEGIN_STARTED      (~0u)
#define RTE_XABORT_EXPLICIT     (1 << 0)
#define RTE_XABORT_RETRY        (1 << 1)
#define RTE_XABORT_CONFLICT     (1 << 2)
#define RTE_XABORT_CAPACITY     (1 << 3)
#define RTE_XABORT_DEBUG        (1 << 4)
#define RTE_XABORT_NESTED       (1 << 5)
#define RTE_XABORT_CODE(x)      (((x) >> 24) & 0xff)

#define RTE_RTM_MAX_RETRIES (20)
#define RTE_XABORT_LOCK_BUSY (0xff)

#ifndef likely
#define likely(x) __builtin_expect(!!(x), 1)
#endif

#ifndef unlikely
#define unlikely(x) __builtin_expect(!!(x), 0)
#endif

static __attribute__((__always_inline__)) inline
unsigned int rte_xbegin(void)
{
    unsigned int ret = RTE_XBEGIN_STARTED;

    asm volatile(".byte 0xc7,0xf8 ; .long 0" : "+a" (ret) :: "memory");
    return ret;
}

void rte_xend(void)
{
     asm volatile(".byte 0x0f,0x01,0xd5" ::: "memory");
}

/* not an inline function to workaround a clang bug with -O0 */
#define rte_xabort(status) do { \
    asm volatile(".byte 0xc6,0xf8,%P0" :: "i" (status) : "memory"); \
} while (0)

static __attribute__((__always_inline__)) inline
int rte_xtest(void)
{
    unsigned char out;

    asm volatile(".byte 0x0f,0x01,0xd6 ; setnz %0" :
        "=r" (out) :: "memory");
    return out;
}

static inline uint64_t
rte_rdtsc(void)
{
    union {
        uint64_t tsc_64;
        struct {
            uint32_t lo_32;
            uint32_t hi_32;
        };
    } tsc;

    asm volatile("rdtsc" : "=a"(tsc.lo_32), "=d"(tsc.hi_32));
    return tsc.tsc_64;
}

int rte_try_tm(int32_t* lock)
{
    int retries = RTE_RTM_MAX_RETRIES;

    while (likely(retries--)) {
        const unsigned int status = rte_xbegin();
        if (likely(RTE_XBEGIN_STARTED == status)) {
            if (unlikely(*lock)) {
                rte_xabort(RTE_XABORT_LOCK_BUSY);
            }
            return 1;
        }

        while (*lock) {
            __builtin_ia32_pause();
        }

        if ((status & RTE_XABORT_CONFLICT) ||
            ((status & RTE_XABORT_EXPLICIT) &&
             (RTE_XABORT_CODE(status) == RTE_XABORT_LOCK_BUSY))) {
            const int try_count = RTE_RTM_MAX_RETRIES - retries;
            int pause_count = (rte_rdtsc() & 0x7) | 1;
            pause_count <<= try_count;
            for (int i = 0; i < pause_count; ++i) {
                __builtin_ia32_pause();
            }
            continue;
        }

        if ((status & RTE_XABORT_RETRY) == 0) {
            break;
        }
    }
    
    return 0;
}
