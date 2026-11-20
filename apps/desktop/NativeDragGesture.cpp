#include "NativeDragGesture.h"

#include <QDrag>
#include <QGuiApplication>

#ifdef Q_OS_LINUX
#include <X11/Xlib.h>
#endif

void executeNativeFileDrag(QDrag *drag) {
    // Qt's X11 backend retains the drag until the recipient finishes its
    // selection request, which can happen after exec() returns.
    drag->exec(Qt::CopyAction, Qt::CopyAction);
}

bool nativeDragGestureHeld() {
#ifdef Q_OS_LINUX
    if (auto *xcb = qGuiApp->nativeInterface<QNativeInterface::QX11Application>()) {
        auto *display = xcb->display();
        if (!display)
            return false;
        Window root = 0;
        Window child = 0;
        int rootX = 0;
        int rootY = 0;
        int windowX = 0;
        int windowY = 0;
        unsigned int buttons = 0;
        // Preparation is asynchronous; Qt's last received mouse state can be
        // stale if the release happened outside the application window.
        for (int screen = 0; screen < ScreenCount(display); ++screen) {
            if (XQueryPointer(display, RootWindow(display, screen), &root, &child, &rootX, &rootY,
                              &windowX, &windowY, &buttons))
                return buttons & Button1Mask;
        }
        return false;
    }
#endif
    return QGuiApplication::mouseButtons().testFlag(Qt::LeftButton);
}
