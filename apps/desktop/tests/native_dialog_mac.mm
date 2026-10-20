// apps/desktop/tests/native_dialog_mac.mm

#import <AppKit/AppKit.h>
#include <QWindow>

bool nativeWindowMatchesState(QWindow *window, Qt::WindowState state) {
    NSView *view = reinterpret_cast<NSView *>(window->winId());
    NSWindow *native = view.window;
    if (!native)
        return false;
    if (state == Qt::WindowMinimized)
        return native.miniaturized;
    if (state == Qt::WindowFullScreen)
        return !native.miniaturized && (native.styleMask & NSWindowStyleMaskFullScreen);
    if (state == Qt::WindowMaximized)
        return !native.miniaturized && native.zoomed;
    return !native.miniaturized && !(native.styleMask & NSWindowStyleMaskFullScreen) &&
           !native.zoomed;
}

// Cocoa's modal file panel runs its own loop. Cancel the actual panel instead
// of relying on a Qt timer firing before that panel has finished opening.
void scheduleNativeFileDialogCancellation() {
    __block int attempts = 0;
    NSTimer *timer = [NSTimer
        timerWithTimeInterval:0.1
                      repeats:YES
                        block:^(NSTimer *current) {
                          for (NSWindow *window in NSApp.windows) {
                              if ([window isKindOfClass:[NSSavePanel class]] && window.visible) {
                                  [(NSSavePanel *)window cancel:nil];
                                  [current invalidate];
                                  return;
                              }
                          }
                          if (++attempts >= 100)
                              [current invalidate];
                        }];
    [[NSRunLoop mainRunLoop] addTimer:timer forMode:NSModalPanelRunLoopMode];
    [[NSRunLoop mainRunLoop] addTimer:timer forMode:NSRunLoopCommonModes];
}
