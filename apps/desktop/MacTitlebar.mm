#import <AppKit/AppKit.h>
#include <QGuiApplication>
#include <QQuickWindow>

// Called on the GUI thread after QML creates the window. Qt's Cocoa WId is an
// NSView owned by Qt; neither it nor its NSWindow may be retained or destroyed here.
void integrateMacTitlebar(QQuickWindow *window) {
    if (!window || QGuiApplication::platformName() != QStringLiteral("cocoa"))
        return;
    NSView *view = reinterpret_cast<NSView *>(window->winId());
    NSWindow *native = view.window;
    if (!native)
        return;
    native.titleVisibility = NSWindowTitleHidden;
    native.titlebarAppearsTransparent = YES;
    native.styleMask |= NSWindowStyleMaskFullSizeContentView;
    // QML owns background dragging. AppKit retains traffic lights, resizing,
    // fullscreen, accessibility and the system window menu.
    native.movableByWindowBackground = NO;
    window->setProperty("integratedTitlebar", true);
}
