// tests/fixtures/native/update_product.c

/* Owned updater protocol fixture. This is deliberately not the Qt application. */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#ifdef _WIN32
#include <windows.h>
#define pause_ms() Sleep(20)
#else
#include <fcntl.h>
#include <sys/file.h>
#include <unistd.h>
#define pause_ms() usleep(20000)
#endif
#ifndef FIXTURE_VERSION
#error A fixture identity is required
#endif

static int exists(const char *path) {
    FILE *file = fopen(path, "rb");
    if (!file)
        return 0;
    fclose(file);
    return 1;
}
int main(int argc, char **argv) {
    if (argc == 2 && strcmp(argv[1], "--version-json") == 0) {
        puts("{\"product\":\"xdzip\",\"version\":\"" FIXTURE_VERSION "\",\"build\":" FIXTURE_BUILD
             ",\"revision\":\"" FIXTURE_REVISION "\",\"dirty\":false}");
        return 0;
    }
#ifdef _WIN32
    HANDLE gate = OpenMutexW(SYNCHRONIZE, FALSE, L"Local\\xdzip.desktop.updating");
    if (gate) {
        CloseHandle(gate);
        return 11;
    }
    HANDLE running = CreateMutexW(NULL, FALSE, L"Local\\xdzip.desktop.running");
    if (!running)
        return 12;
    const char *home = getenv("USERPROFILE");
    unsigned long uid = 0;
#else
    if (exists("/run/xdzip/updating") || exists("/run/xdzip/installing"))
        return 11;
    int running = open("/run/xdzip/usage.lock", O_RDONLY | O_NOFOLLOW);
    if (running < 0 || flock(running, LOCK_SH | LOCK_NB) != 0)
        return 12;
    if (exists("/run/xdzip/updating") || exists("/run/xdzip/installing"))
        return 11;
    const char *home = getenv("HOME");
    unsigned long uid = (unsigned long)getuid();
#endif
    if (argc == 3 && strcmp(argv[1], "--fixture-parent") == 0) {
        while (!exists(argv[2]))
            pause_ms();
    } else if (argc == 2 && strcmp(argv[1], "--gate-probe") == 0) {
        return 0;
    } else {
        char path[4096];
        if (!home || snprintf(path, sizeof(path), "%s/xdzip-update-fixture-restarted.json", home) >=
                         (int)sizeof(path))
            return 13;
        FILE *file = fopen(path, "wb");
        if (!file)
            return 14;
        fprintf(file, "{\"version\":\"%s\",\"build\":%s,\"revision\":\"%s\",\"uid\":%lu}\n",
                FIXTURE_VERSION, FIXTURE_BUILD, FIXTURE_REVISION, uid);
        if (fclose(file))
            return 15;
    }
    return 0;
}
