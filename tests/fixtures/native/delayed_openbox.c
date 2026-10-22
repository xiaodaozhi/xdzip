// tests/fixtures/native/delayed_openbox.c

#define _GNU_SOURCE
#include <X11/Xlib.h>
#include <dlfcn.h>
#include <stdlib.h>
#include <time.h>
#include <unistd.h>

// Test-only POSIX interposition, loaded into our single-threaded Openbox child.
// Match Xlib's ABI, forward every write, and fail with 125 if resolution fails.
// The delay starts after both EWMH identity writes, before Openbox initialization ends.
int XChangeProperty(Display *display, Window window, Atom property, Atom type, int format, int mode,
                    const unsigned char *data, int count) {
    typedef int (*ChangeProperty)(Display *, Window, Atom, Atom, int, int, const unsigned char *,
                                  int);
    static ChangeProperty original;
    static Atom identity;
    static int writes;
    if (!original)
        original = (ChangeProperty)dlsym(RTLD_NEXT, "XChangeProperty");
    if (!original)
        _exit(125);
    if (!identity)
        identity = XInternAtom(display, "_NET_SUPPORTING_WM_CHECK", False);
    int result = original(display, window, property, type, format, mode, data, count);
    if (property == identity && ++writes == 2) {
        const char *setting = getenv("XDZIP_TEST_WM_DELAY_MS");
        char *end;
        unsigned long milliseconds = setting ? strtoul(setting, &end, 10) : 0;
        if (setting && (end == setting || *end || milliseconds > 8000))
            _exit(125);
        if (milliseconds) {
            struct timespec delay = {(time_t)(milliseconds / 1000),
                                     (long)(milliseconds % 1000) * 1000000};
            XFlush(display);
            while (nanosleep(&delay, &delay) != 0) {
            }
        }
    }
    return result;
}
