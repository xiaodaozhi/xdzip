// apps/desktop/tests/native_dialog_mac.mm

#import <AppKit/AppKit.h>
#include <QDebug>
#include <QGuiApplication>
#include <QPointer>
#include <QVariant>
#include <QWindow>

bool watchNativeWindowTransitions(QWindow *window) {
    if (!window || QGuiApplication::platformName() != QStringLiteral("cocoa"))
        return false;
    if (window->property("xdzipNativeStateWatched").toBool())
        return true;
    NSView *view = reinterpret_cast<NSView *>(window->winId());
    NSWindow *native = view.window;
    if (!native)
        return false;
    window->setProperty("xdzipNativeStateWatched", true);
    window->setProperty("xdzipNativeWatchedIdentity",
                        QVariant::fromValue<qulonglong>(reinterpret_cast<quintptr>(native)));
    const QPointer<QWindow> guarded(window);
    NSMutableArray *tokens = [[NSMutableArray alloc] init];
    for (NSNotificationName name in @[
             NSWindowWillEnterFullScreenNotification, NSWindowWillExitFullScreenNotification,
             NSWindowWillMiniaturizeNotification
         ]) {
        [tokens addObject:[[NSNotificationCenter defaultCenter]
                              addObserverForName:name
                                          object:native
                                           queue:nil
                                      usingBlock:^(NSNotification *notification) {
                                        if (guarded &&
                                            guarded->property("xdzipTraceActivation").toBool())
                                            qInfo() << "activation native notification"
                                                    << QString::fromNSString(notification.name)
                                                    << "states" << guarded->windowStates()
                                                    << "miniaturized" << native.miniaturized
                                                    << "pending"
                                                    << guarded->property("xdzipWakePending");
                                        if (guarded)
                                            guarded->setProperty("xdzipNativeTransition", true);
                                      }]];
    }
    for (NSNotificationName name in @[
             NSWindowDidEnterFullScreenNotification, NSWindowDidExitFullScreenNotification,
             NSWindowDidMiniaturizeNotification, NSWindowDidDeminiaturizeNotification
         ]) {
        [tokens addObject:[[NSNotificationCenter defaultCenter]
                              addObserverForName:name
                                          object:native
                                           queue:nil
                                      usingBlock:^(NSNotification *notification) {
                                        if (guarded &&
                                            guarded->property("xdzipTraceActivation").toBool())
                                            qInfo() << "activation native notification"
                                                    << QString::fromNSString(notification.name)
                                                    << "states" << guarded->windowStates()
                                                    << "miniaturized" << native.miniaturized
                                                    << "pending"
                                                    << guarded->property("xdzipWakePending");
                                        if (guarded)
                                            guarded->setProperty("xdzipNativeTransition", false);
                                      }]];
    }
    QObject::connect(window, &QObject::destroyed, [tokens] {
        for (id token in tokens)
            [[NSNotificationCenter defaultCenter] removeObserver:token];
        [tokens release];
    });
    return true;
}

void traceNativeWindowState(QWindow *window) {
    if (!window || QGuiApplication::platformName() != QStringLiteral("cocoa"))
        return;
    NSView *view = reinterpret_cast<NSView *>(window->winId());
    NSWindow *native = view.window;
    qInfo() << "activation native snapshot" << "hasWindow" << bool(native) << "sameObservedWindow"
            << (native && reinterpret_cast<quintptr>(native) ==
                              window->property("xdzipNativeWatchedIdentity").toULongLong())
            << "miniaturized" << (native && native.miniaturized) << "visible"
            << (native && native.visible) << "transition"
            << window->property("xdzipNativeTransition") << "qtStates" << window->windowStates()
            << "pending" << window->property("xdzipWakePending");
}

bool nativeWindowMatchesState(QWindow *window, Qt::WindowState state) {
    NSView *view = reinterpret_cast<NSView *>(window->winId());
    NSWindow *native = view.window;
    if (!native || window->property("xdzipNativeTransition").toBool())
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
