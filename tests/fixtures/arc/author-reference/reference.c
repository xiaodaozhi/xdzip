/* Project-owned test adapter for unchanged, external ARC GPL reference sources.
 * Never linked into or deployed with the product. */
#include "source/arc.h"
#include "source/proto.h"
#include <stdarg.h>
#include <stdio.h>
#include <string.h>
#include <sys/types.h>
extern unsigned char *outbuf, *outbeg, *outend;
unsigned char *pinbuf, *pakbuf;
void pack(FILE *, FILE *, struct heads *);
int unpack(FILE *, FILE *, struct heads *);
void init_cm(unsigned char *);
long pred_cm(FILE *);
void flsh_cm(FILE *);
unsigned int ncr_buf(unsigned int);
extern long stdlen;
extern short crcval;
extern unsigned char state;
void arcdie(char *message, ...) {
    va_list args;
    va_start(args, message);
    vfprintf(stderr, message, args);
    va_end(args);
    exit(20);
}
int crcbuf(int crc, unsigned int count, unsigned char *bytes) {
    unsigned int value = (unsigned short)crc;
    while (count--) {
        int bit;
        value ^= *bytes++;
        for (bit = 0; bit < 8; ++bit)
            value = (value >> 1) ^ ((value & 1) ? 0xa001 : 0);
    }
    return (unsigned short)value;
}
static unsigned long number(const char *s) {
    return strtoul(s, NULL, 10);
}
int main(int argc, char **argv) {
    struct heads header = {0};
    FILE *input, *output;
    int result = 0;
    if (argc != 5 && argc != 8)
        return 2;
    pinbuf = calloc(MYBUF * 4, 1);
    pakbuf = calloc(MYBUF * 4, 1);
    outbuf = calloc(MYBUF * 4, 1);
    if (!pinbuf || !pakbuf || !outbuf)
        return 3;
    outbeg = outbuf;
    outend = outbuf + MYBUF - 1;
    input = fopen(argv[2], "rb");
    output = fopen(argv[3], "wb+");
    if (!input || !output)
        return 4;
    note = 0;
    warn = 1;
    image = 1;
    if (!strcmp(argv[1], "lzw")) {
        unsigned int count = getbuf(input);
        if (!count)
            return 7;
        dosquash = number(argv[4]) == 9;
        state = 0;
        init_cm(pinbuf);
        for (;;) {
            unsigned int encoded = ncr_buf(count);
            lzw_buf(dosquash ? pinbuf : pakbuf, dosquash ? count : encoded, output);
            if (count < MYBUF)
                break;
            count = getbuf(input);
        }
        header.size = pred_cm(output);
        flsh_cm(output);
        header.length = stdlen;
        header.crc = crcval;
        hdrver = dosquash ? 9 : 8;
        printf("{\"method\":%u,\"packed\":%lu,\"decoded\":%lu,\"crc\":%u}\n", (unsigned char)hdrver,
               (unsigned long)header.size, (unsigned long)header.length,
               (unsigned short)header.crc);
    } else if (!strcmp(argv[1], "pack")) {
        dosquash = number(argv[4]) == 9;
        nocomp = number(argv[4]) == 2;
        pack(input, output, &header);
        printf("{\"method\":%u,\"packed\":%lu,\"decoded\":%lu,\"crc\":%u}\n", (unsigned char)hdrver,
               (unsigned long)header.size, (unsigned long)header.length,
               (unsigned short)header.crc);
    } else if (!strcmp(argv[1], "unpack") && argc == 8) {
        hdrver = number(argv[4]);
        header.size = number(argv[5]);
        header.length = number(argv[6]);
        header.crc = number(argv[7]);
        result = unpack(input, output, &header);
        printf("{\"method\":%u,\"written\":%ld,\"result\":%d}\n", (unsigned char)hdrver,
               ftell(output), result);
    } else
        return 5;
    fclose(input);
    if (fclose(output))
        return 6;
    return result;
}
