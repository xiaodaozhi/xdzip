// Owned test-only wrapper. Include the checksum-verified author's header from build/toolchain.
#define XXH_STATIC_LINKING_ONLY
#define XXH_IMPLEMENTATION
#include "xxhash.h"
#include <stdio.h>

int main(void) {
    unsigned char data[65536];
    const unsigned lengths[] = {0,  1,  3,  4,  7,  8,  15,  16,   17,
                                31, 32, 33, 63, 64, 65, 255, 4096, 65536};
    for (unsigned i = 0; i < 65536; ++i)
        data[i] = (unsigned char)((i * 73) ^ (i >> 8));
    for (unsigned i = 0; i < sizeof(lengths) / sizeof(lengths[0]); ++i)
        printf("%u %016llx\n", lengths[i], (unsigned long long)XXH64(data, lengths[i], 0));
    return 0;
}
