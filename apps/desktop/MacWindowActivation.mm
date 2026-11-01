#include "WindowActivation.h"
#import <AppKit/AppKit.h>
#include <QPointer>
#include <QVariant>

namespace {
NSWindow *nativeWindow(QWindow *window) {
    if (!window || !window->handle())
        return nil;
    NSView *view = reinterpret_cast<NSView *>(window->winId());
    return view.window;
}

class CocoaActivationObserver final : public QObject {
  public:
    explicit CocoaActivationObserver(QWindow *window) : QObject(window), m_window(window) {
        m_tokens = [[NSMutableArray alloc] init];
    }
    ~CocoaActivationObserver() override {
        clear();
        [m_tokens release];
    }
    void watch(NSWindow *native) {
        if (m_native == native)
            return;
        clear();
        m_native = native;
        if (!native || !m_window)
            return;
        m_window->setProperty("xdzipCocoaMiniTransition", false);
        m_window->setProperty("xdzipCocoaFullTransition", false);
        m_window->setProperty("xdzipNativeMinimized", bool(native.miniaturized));
        const QPointer<QWindow> guarded(m_window);
        for (NSNotificationName name in @[
                 NSWindowWillMiniaturizeNotification, NSWindowDidMiniaturizeNotification,
                 NSWindowDidDeminiaturizeNotification, NSWindowWillEnterFullScreenNotification,
                 NSWindowDidEnterFullScreenNotification, NSWindowWillExitFullScreenNotification,
                 NSWindowDidExitFullScreenNotification
             ]) {
            [m_tokens
                addObject:
                    [[NSNotificationCenter defaultCenter]
                        addObserverForName:name
                                    object:native
                                     queue:nil
                                usingBlock:^(NSNotification *notification) {
                                  QWindow *window = guarded.data();
                                  if (!window)
                                      return;
                                  NSString *event = notification.name;
                                  if ([event isEqualToString:NSWindowWillMiniaturizeNotification]) {
                                      window->setProperty("xdzipCocoaMiniTransition", true);
                                  } else if ([event isEqualToString:
                                                        NSWindowDidMiniaturizeNotification]) {
                                      window->setProperty("xdzipCocoaMiniTransition", false);
                                      WindowActivation::cocoaNativeMinimized(window);
                                  } else if ([event isEqualToString:
                                                        NSWindowDidDeminiaturizeNotification]) {
                                      window->setProperty("xdzipCocoaMiniTransition", false);
                                      window->setProperty("xdzipNativeMinimized", false);
                                  } else if ([event isEqualToString:
                                                        NSWindowWillEnterFullScreenNotification] ||
                                             [event isEqualToString:
                                                        NSWindowWillExitFullScreenNotification]) {
                                      window->setProperty("xdzipCocoaFullTransition", true);
                                  } else {
                                      window->setProperty("xdzipCocoaFullTransition", false);
                                      WindowActivation::cocoaTransitionFinished(window);
                                  }
                                }]];
        }
    }

  private:
    void clear() {
        for (id token in m_tokens)
            [[NSNotificationCenter defaultCenter] removeObserver:token];
        [m_tokens removeAllObjects];
        m_native = nil;
    }
    QPointer<QWindow> m_window;
    NSMutableArray *m_tokens;
    NSWindow *m_native = nil;
};
} // namespace

bool watchCocoaActivation(QWindow *window) {
    NSWindow *native = nativeWindow(window);
    if (!native)
        return false;
    auto *base = window->property("xdzipCocoaActivationObserver").value<QObject *>();
    if (base && base->parent() != window)
        return false;
    auto *observer = static_cast<CocoaActivationObserver *>(base);
    if (!observer) {
        observer = new CocoaActivationObserver(window);
        window->setProperty("xdzipCocoaActivationObserver",
                            QVariant::fromValue<QObject *>(observer));
    }
    observer->watch(native);
    return true;
}

void unwatchCocoaActivation(QWindow *window) {
    auto *base = window->property("xdzipCocoaActivationObserver").value<QObject *>();
    if (base && base->parent() == window)
        static_cast<CocoaActivationObserver *>(base)->watch(nil);
}

bool cocoaActivationInFlight(QWindow *window) {
    return window->property("xdzipCocoaMiniTransition").toBool() ||
           window->property("xdzipCocoaFullTransition").toBool();
}

bool cocoaActivationIsMiniaturized(QWindow *window) {
    NSWindow *native = nativeWindow(window);
    return native && native.miniaturized;
}

bool cocoaActivationIsVisible(QWindow *window) {
    NSWindow *native = nativeWindow(window);
    return native && native.visible;
}
