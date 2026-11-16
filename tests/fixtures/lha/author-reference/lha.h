/* Owned minimal test host; original decoder source stays a reference only. */
#include <limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#define THRESHOLD 3
#define MAXMATCH 256
#define N_CHAR (256 + 60 - THRESHOLD + 1)
#define TREESIZE_C (N_CHAR * 2)
#define TREESIZE_P (128 * 2)
#define TREESIZE (TREESIZE_C + TREESIZE_P)
#define ROOT_C 0
#define ROOT_P TREESIZE_C
extern int n_max, maxmatch, dicbit;
extern unsigned long decode_count;
extern unsigned short bitbuf;
unsigned short getbits(int);
void fillbuf(int);
void init_getbits(void);
void init_code_cache(void);
void init_putbits(void);
void putcode(int, unsigned int);
void putbits(int, unsigned int);
void start_c_dyn(void);
void decode_start_dyn(void);
unsigned short decode_c_dyn(void);
unsigned short decode_p_dyn(void);

void encode_p_st0(unsigned short);
