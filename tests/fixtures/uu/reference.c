/* Owned harness: invoke only UUDeview's public stream encoder on owned input. */
#include "config.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "uudeview.h"
#ifdef _WIN32
/* The upstream POSIX temporary-file decoder is outside this harness. */
int getuid(void) { abort(); }
int geteuid(void) { abort(); }
int getgid(void) { abort(); }
int getegid(void) { abort(); }
int mkstemp(char *name) { (void)name; abort(); }
#endif
int main(int argc, char **argv) {
    if (argc != 6 || strcmp(argv[1], "encode")) return 64;
    FILE *in = fopen(argv[2], "rb");
    FILE *out = fopen(argv[3], "wb");
    if (!in || !out || UUInitialize() != UURET_OK) return 65;
    int result = UUEncodeToStream(out, in, NULL, atoi(argv[4]), argv[5], 0644);
    if (fclose(in) || fclose(out)) return 67;
    UUCleanUp();
    return result;
}
