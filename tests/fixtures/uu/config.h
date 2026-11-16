#define PROTOTYPES 1
#define HAVE_STDIO 1
#define HAVE_ERRNO_H 1
#define HAVE_FCNTL_H 1
#define HAVE_MALLOC_H 1
#define HAVE_MEMORY_H 1
#define VERSION "0.5.20"
#define PATCH "7640bc56"
#ifdef _WIN32
#define HAVE_IO_H 1
#define SYSTEM_DOS 1
#define _CRT_SECURE_NO_WARNINGS 1
#define unlink _unlink
#define tempnam _tempnam
#define strdup _strdup
#define strcasecmp _stricmp
#define strncasecmp _strnicmp
#include <io.h>
#else
#define HAVE_UNISTD_H 1
#endif
