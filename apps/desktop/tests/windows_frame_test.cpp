#include "../WindowsFrame.h"
#include <QGuiApplication>
#include <QWindow>
#include <cstdio>
#include <qt_windows.h>

int main(int argc, char **argv) {
    QGuiApplication app(argc, argv);
    // A private hidden window lets us inspect native messages without desktop input.
    QWindow window;
    window.resize(500, 300);
    WindowsFrame frame;
    frame.setWindow(&window);
    window.create();
    const auto diagnostic = frame.diagnostics();
    if (!diagnostic.value(QStringLiteral("attached")).toBool()) {
        std::fprintf(stderr, "Native frame was not attached\n");
        return 1;
    }
    if (!diagnostic.value(QStringLiteral("resizeStyle")).toBool() ||
        diagnostic.value(QStringLiteral("captionStyle")).toBool() !=
            diagnostic.value(QStringLiteral("expectedCaptionStyle")).toBool() ||
        (diagnostic.value(QStringLiteral("composition")).toBool() &&
         !diagnostic.value(QStringLiteral("nonClientRendering")).toBool())) {
        std::fprintf(stderr, "Unexpected native caption, resize, or DWM rendering policy\n");
        return 8;
    }
    const int topInset = diagnostic.value(QStringLiteral("topInsetPixels")).toInt();
    if (topInset != 1) {
        std::fprintf(stderr, "Unexpected native client top inset: %d\n", topInset);
        return 7;
    }
    const auto hwnd = reinterpret_cast<HWND>(window.winId());
    RECT outer{};
    if (!GetWindowRect(hwnd, &outer))
        return 2;
    for (const WPARAM validRects : {WPARAM(0), WPARAM(1)}) {
        WINDOWPOS pos{};
        NCCALCSIZE_PARAMS params{};
        params.rgrc[0] = outer;
        params.lppos = &pos;
        RECT simple = outer;
        NCCALCSIZE_PARAMS nativeParams = params;
        RECT nativeRect = outer;
        const auto nativePayload = validRects ? reinterpret_cast<LPARAM>(&nativeParams)
                                              : reinterpret_cast<LPARAM>(&nativeRect);
        DefWindowProcW(hwnd, WM_NCCALCSIZE, validRects, nativePayload);
        const RECT expected = validRects ? nativeParams.rgrc[0] : nativeRect;
        MSG message{};
        message.hwnd = hwnd;
        message.message = WM_NCCALCSIZE;
        message.wParam = validRects;
        message.lParam =
            validRects ? reinterpret_cast<LPARAM>(&params) : reinterpret_cast<LPARAM>(&simple);
        qintptr result = -1;
        if (!frame.nativeEventFilter(QByteArrayLiteral("windows_generic_MSG"), &message, &result)) {
            std::fprintf(stderr, "WM_NCCALCSIZE form %llu was not handled\n",
                         static_cast<unsigned long long>(validRects));
            return 3;
        }
        const RECT actual = validRects ? params.rgrc[0] : simple;
        if (result != 0 || actual.top != outer.top + 1 || actual.left != expected.left ||
            actual.right != expected.right || actual.bottom != expected.bottom) {
            std::fprintf(stderr, "WM_NCCALCSIZE form %llu returned unexpected bounds\n",
                         static_cast<unsigned long long>(validRects));
            return 4;
        }
    }
    MSG invalid{};
    invalid.hwnd = hwnd;
    invalid.message = WM_NCCALCSIZE;
    invalid.wParam = 1;
    qintptr invalidResult = -1;
    if (frame.nativeEventFilter(QByteArrayLiteral("windows_generic_MSG"), &invalid,
                                &invalidResult)) {
        std::fprintf(stderr, "Invalid payload was handled\n");
        return 5;
    }
    struct HitCase {
        int x;
        int y;
        int expected;
    };
    const int midX = outer.left + (outer.right - outer.left) / 2;
    const int midY = outer.top + (outer.bottom - outer.top) / 2;
    const HitCase cases[] = {
        {outer.left, outer.top, HTTOPLEFT},
        {midX, outer.top, HTTOP},
        {outer.right - 1, outer.top, HTTOPRIGHT},
        {outer.left, midY, HTLEFT},
        {outer.right - 1, midY, HTRIGHT},
        {outer.left, outer.bottom - 1, HTBOTTOMLEFT},
        {midX, outer.bottom - 1, HTBOTTOM},
        {outer.right - 1, outer.bottom - 1, HTBOTTOMRIGHT},
    };
    for (const auto &sample : cases) {
        MSG hit{};
        hit.hwnd = hwnd;
        hit.message = WM_NCHITTEST;
        hit.lParam = MAKELPARAM(sample.x, sample.y);
        qintptr hitResult = 0;
        if (!frame.nativeEventFilter(QByteArrayLiteral("windows_generic_MSG"), &hit, &hitResult) ||
            hitResult != sample.expected) {
            std::fprintf(stderr, "Resize hit test failed: expected %d, got %lld\n", sample.expected,
                         static_cast<long long>(hitResult));
            return 6;
        }
    }
    std::puts("Both WM_NCCALCSIZE forms, native policy, and resize hit tests passed");
    return 0;
}
