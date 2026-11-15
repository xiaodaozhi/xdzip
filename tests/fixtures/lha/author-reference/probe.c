/* Owned controlled test host for the original author's public raw functions. */
#include "lha.h"
#include <fcntl.h>
#include <io.h>
int n_max, maxmatch, dicbit = 13;
unsigned long decode_count;
unsigned short bitbuf;
static unsigned char packed[1024 * 1024];
static size_t packed_size, bit_position;
static void peek(void) {
    bitbuf = 0;
    for (int i = 0; i < 16; i++) {
        size_t bit = bit_position + i;
        bitbuf = (bitbuf << 1) |
                 ((bit / 8 < packed_size) ? ((packed[bit / 8] >> (7 - bit % 8)) & 1) : 0);
    }
}
void fillbuf(int n) {
    bit_position += (size_t)n;
    peek();
}
unsigned short getbits(int n) {
    unsigned short result = bitbuf >> (16 - n);
    fillbuf(n);
    return result;
}
void init_getbits(void) {
    bit_position = 0;
    peek();
}
void init_code_cache(void) {}
void init_putbits(void) {}
void putcode(int n, unsigned int v) {
    (void)n;
    (void)v;
    abort();
}
void putbits(int n, unsigned int v) {
    (void)n;
    (void)v;
    abort();
}
void encode_p_st0(unsigned short v) {
    (void)v;
    abort();
}
int main(int argc, char **argv) {
    if (argc != 3)
        return 2;
    FILE *file = fopen(argv[1], "rb");
    if (!file)
        return 2;
    packed_size = fread(packed, 1, sizeof packed, file);
    if (ferror(file) || !feof(file))
        return 2;
    fclose(file);
    unsigned long size = strtoul(argv[2], NULL, 10);
    if (size > 2 * 1024 * 1024)
        return 2;
    unsigned char history[8192];
    memset(history, 32, sizeof history);
    _setmode(_fileno(stdout), _O_BINARY);
    decode_count = 0;
    decode_start_dyn();
    while (decode_count < size) {
        unsigned int code = decode_c_dyn();
        if (code < 256) {
            history[decode_count & 8191] = (unsigned char)code;
            if (fputc(code, stdout) == EOF)
                return 2;
            decode_count++;
        } else {
            unsigned int count = code - 253, distance = decode_p_dyn() + 1;
            if (count > size - decode_count || distance > 8192)
                return 3;
            for (unsigned int i = 0; i < count; i++) {
                unsigned char byte = history[(decode_count - distance) & 8191];
                history[decode_count & 8191] = byte;
                if (fputc(byte, stdout) == EOF)
                    return 2;
                decode_count++;
            }
        }
    }
    return fflush(stdout) ? 2 : 0;
}
