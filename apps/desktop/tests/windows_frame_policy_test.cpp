#include "../WindowsFramePolicy.h"

static_assert(useWindows10ClientCaptionWorkaround(true, 10, 0, 14393));
static_assert(useWindows10ClientCaptionWorkaround(true, 10, 0, 19045));
static_assert(!useWindows10ClientCaptionWorkaround(true, 10, 0, 22000));
static_assert(!useWindows10ClientCaptionWorkaround(true, 10, 0, 26100));
static_assert(!useWindows10ClientCaptionWorkaround(false, 10, 0, 20348));
static_assert(!useWindows10ClientCaptionWorkaround(true, 6, 3, 9600));
static_assert(!useWindows10ClientCaptionWorkaround(true, 10, 0, 0));
static_assert(!useWindows10ClientCaptionWorkaround(true, 11, 0, 1));

int main() { return 0; }
