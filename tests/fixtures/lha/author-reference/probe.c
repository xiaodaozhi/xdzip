/* Owned test host; all original Huffman state stays in the pinned reference. */
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
static unsigned int output_byte, output_count;
void putbits(int n, unsigned int v) {
    if (n < 0 || n > 24)
        abort();
    for (int i = n - 1; i >= 0; i--) {
        output_byte = (output_byte << 1) | ((v >> i) & 1);
        if (++output_count == 8) {
            if (fputc((int)output_byte, stdout) == EOF)
                abort();
            output_byte = output_count = 0;
        }
    }
}
void putcode(int n, unsigned int v) {
    if (n < 0 || n > 16)
        abort();
    putbits(n, v >> (16 - n));
}
void encode_p_st0(unsigned short v) {
    (void)v;
    abort();
}
/* Source is unmodified and remains outside Git and every production target.
 * Inclusion exposes its test-only internal encoder/growth helpers to this host. */
#include "dhuf.c"
static int encode_short(unsigned long matches) {
    if (matches == 0 || matches > 200000)
        return 2;
    decode_count = 0;
    decode_start_dyn();
    for (unsigned int i = 0; i < 64; i++) {
        encode_c_dyn(i);
        decode_count++;
    }
    unsigned int random_state = 0x6d2b79f5U;
    for (unsigned long i = 0; i < matches; i++) {
        encode_c_dyn(256);
        while (decode_count > nextcount && nextcount < (unsigned long)nn) {
            make_new_node((int)(nextcount / 64));
            nextcount += 64;
        }
        random_state ^= random_state << 13;
        random_state ^= random_state >> 17;
        random_state ^= random_state << 5;
        unsigned int available = decode_count < 8192 ? (unsigned int)decode_count : 8192;
        unsigned int offset = random_state % available;
        unsigned int position_symbol = offset >> 6;
        unsigned char path[512];
        unsigned int count = 0;
        int node = s_node[N_CHAR + position_symbol];
        while (node != ROOT_P) {
            if (count == sizeof path)
                return 2;
            int above = parent[node];
            path[count++] = child[above] == node ? 0 : 1;
            node = above;
        }
        while (count != 0)
            putbits(1, path[--count]);
        update_p((int)position_symbol);
        putbits(6, offset & 63);
        decode_count += 3;
    }
    if (output_count != 0)
        putbits((int)(8 - output_count), 0);
    return fflush(stdout) ? 2 : 0;
}
int main(int argc, char **argv) {
    if (argc != 3 && argc != 4)
        return 2;
    _setmode(_fileno(stdout), _O_BINARY);
    if (strcmp(argv[1], "--encode-short") == 0)
        return encode_short(strtoul(argv[2], NULL, 10));
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
    decode_count = 0;
    decode_start_dyn();
    while (decode_count < size) {
        unsigned int code = decode_c_dyn();
        if (code < 256) {
            if (argc == 4)
                fprintf(stderr, "%lu %u -1\n", decode_count, code);
            history[decode_count & 8191] = (unsigned char)code;
            if (fputc(code, stdout) == EOF)
                return 2;
            decode_count++;
        } else {
            unsigned int count = code - 253, distance = decode_p_dyn() + 1;
            if (argc == 4)
                fprintf(stderr, "%lu %u %u\n", decode_count, code, distance - 1);
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
