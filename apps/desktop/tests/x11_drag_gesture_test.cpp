#include "NativeDragGesture.h"

#include <QElapsedTimer>
#include <QGuiApplication>
#include <QThread>
#include <QWindow>
#include <X11/Xlib.h>
#include <dlfcn.h>

int main(int argc, char **argv) {
    QGuiApplication application(argc, argv);
    auto *xcb = qGuiApp->nativeInterface<QNativeInterface::QX11Application>();
    if (!xcb || !xcb->display())
        return 1;
    auto *display = xcb->display();
    void *xtest = dlopen("libXtst.so.6", RTLD_NOW | RTLD_LOCAL);
    if (!xtest)
        return 2;
    // POSIX XTest exports this ABI; a missing symbol fails the test.
    using FakeButton = int (*)(Display *, unsigned int, int, unsigned long);
    auto fakeButton = reinterpret_cast<FakeButton>(dlsym(xtest, "XTestFakeButtonEvent"));
    if (!fakeButton || !fakeButton(display, 1, True, CurrentTime))
        return 3;
    XSync(display, False);
    // Qt has not processed a button event, but X11 reports the real press.
    const bool pressed = nativeDragGestureHeld();
    const bool qtUnchanged = !QGuiApplication::mouseButtons().testFlag(Qt::LeftButton);
    const bool releasedEvent = fakeButton(display, 1, False, CurrentTime);
    XSync(display, False);
    const bool released = !nativeDragGestureHeld();
    if (!pressed || !qtUnchanged || !releasedEvent || !released)
        return 4;

    if (ScreenCount(display) > 1) {
        XWarpPointer(display, None, RootWindow(display, 1), 0, 0, 0, 0, 20, 20);
        XSync(display, False);
        if (!fakeButton(display, 1, True, CurrentTime))
            return 8;
        XSync(display, False);
        const bool secondScreenPress = nativeDragGestureHeld();
        const bool secondScreenReleaseEvent = fakeButton(display, 1, False, CurrentTime);
        XSync(display, False);
        const bool secondScreenRelease = !nativeDragGestureHeld();
        if (!secondScreenPress || !secondScreenReleaseEvent || !secondScreenRelease)
            return 9;
        XWarpPointer(display, None, RootWindow(display, 0), 0, 0, 0, 0, 20, 20);
        XSync(display, False);
    }

    QWindow window;
    window.resize(100, 100);
    window.show();
    XSync(display, False);
    XWarpPointer(display, None, window.winId(), 0, 0, 0, 0, 20, 20);
    XSync(display, False);
    QGuiApplication::processEvents();
    if (!fakeButton(display, 1, True, CurrentTime))
        return 5;
    XSync(display, False);
    QElapsedTimer timer;
    timer.start();
    while (!QGuiApplication::mouseButtons().testFlag(Qt::LeftButton) && timer.elapsed() < 1000) {
        QGuiApplication::processEvents();
        QThread::msleep(1);
    }
    const bool receivedPress = QGuiApplication::mouseButtons().testFlag(Qt::LeftButton);
    if (!fakeButton(display, 1, False, CurrentTime))
        return 6;
    XSync(display, False);
    // Do not process the release: Qt still sees the press but X11 sees the release.
    const bool qtStillPressed = QGuiApplication::mouseButtons().testFlag(Qt::LeftButton);
    const bool actualRelease = !nativeDragGestureHeld();
    // Xlib retains extension callbacks until the display closes during Qt teardown.
    // Keep the library loaded for the remaining lifetime of this test process.
    return receivedPress && qtStillPressed && actualRelease ? 0 : 7;
}
