/* Test-only: model a lost Xvfb readiness notification in an owned process. */
#define _GNU_SOURCE
#include <dlfcn.h>
#include <fcntl.h>
#include <signal.h>
#include <stdlib.h>
#include <string.h>
#include <sys/types.h>
#include <unistd.h>

static int is_xvfb_process(void) {
    char exe[4096] = {0};
    ssize_t n = readlink("/proc/self/exe", exe, sizeof(exe) - 1);
    const char *base = strrchr(exe, '/');
    return n > 0 && base && strcmp(base, "/Xvfb") == 0;
}

__attribute__((constructor)) static void trace_xvfb_loaded(void) {
    const char *stages = getenv("XDZIP_TEST_XVFB_STAGES");
    if (stages && is_xvfb_process()) {
        int fd = open(stages, O_WRONLY | O_CREAT | O_APPEND, 0600);
        if (fd >= 0) {
            (void)write(fd, "Xvfb preload loaded\n", 20);
            close(fd);
        }
    }
}

int kill(pid_t pid, int sig) {
    if (sig == SIGUSR1 && is_xvfb_process()) {
        const char *trace = getenv("XDZIP_TEST_XVFB_READY_TRACE");
        if (trace) {
            int fd = open(trace, O_WRONLY | O_CREAT | O_APPEND, 0600);
            if (fd >= 0) {
                (void)write(fd, "suppressed\n", 11);
                close(fd);
            }
        }
        return 0;
    }
    typedef int (*KillFunction)(pid_t, int);
    KillFunction original = (KillFunction)dlsym(RTLD_NEXT, "kill");
    if (!original)
        _exit(125);
    return original(pid, sig);
}
