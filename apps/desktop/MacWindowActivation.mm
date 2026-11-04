#include "WindowActivation.h"
#import <AppKit/AppKit.h>
#include <QPointer>
#include <QVariant>

namespace {
NSWindow *nativeWindow(QWindow *window) {
    if (!window || !window->handle() || QGuiApplication::platformName() != QStringLiteral("cocoa"))
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
        if (m_window)
            m_window->setProperty("xdzipCocoaSurfaceGeneration",
                                  m_window->property("xdzipCocoaSurfaceGeneration").toULongLong() +
                                      1);
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
                                      window->setProperty("xdzipCocoaPendingMinimizeGeneration",
                                                          QVariant());
                                      window->setProperty("xdzipCocoaStateCandidateGeneration",
                                                          QVariant());
                                      WindowActivation::cocoaNativeMinimized(window);
                                  } else if ([event isEqualToString:
                                                        NSWindowDidDeminiaturizeNotification]) {
                                      window->setProperty("xdzipCocoaMiniTransition", false);
                                      window->setProperty("xdzipNativeMinimized", false);
                                  } else if ([event isEqualToString:
                                                        NSWindowWillEnterFullScreenNotification] ||
                                             [event isEqualToString:
                                                        NSWindowWillExitFullScreenNotification]) {
                                      const bool entering = [event
                                          isEqualToString:NSWindowWillEnterFullScreenNotification];
                                      window->setProperty("xdzipCocoaFullEntering", entering);
                                      if (!entering) {
                                          window->setProperty("xdzipCocoaPendingMinimizeGeneration",
                                                              QVariant());
                                          window->setProperty("xdzipCocoaStateCandidateGeneration",
                                                              QVariant());
                                      }
                                      window->setProperty("xdzipCocoaFullTransition", true);
                                  } else {
                                      window->setProperty("xdzipCocoaFullEntering", false);
                                      if ([event isEqualToString:
                                                     NSWindowDidEnterFullScreenNotification]) {
                                          window->setProperty("xdzipCocoaJustEnteredFullscreen",
                                                              true);
                                          const auto generation =
                                              window->property("xdzipCocoaSurfaceGeneration")
                                                  .toULongLong();
                                          QTimer::singleShot(0, window, [window, generation] {
                                              if (window->property("xdzipCocoaSurfaceGeneration")
                                                      .toULongLong() == generation)
                                                  window->setProperty(
                                                      "xdzipCocoaJustEnteredFullscreen", false);
                                          });
                                      } else
                                          window->setProperty("xdzipCocoaPendingMinimizeGeneration",
                                                              QVariant());
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
        if (m_window) {
            m_window->setProperty("xdzipCocoaJustEnteredFullscreen", false);
            m_window->setProperty("xdzipCocoaFullEntering", false);
            m_window->setProperty("xdzipCocoaPendingMinimizeGeneration", QVariant());
            m_window->setProperty("xdzipCocoaStateCandidateGeneration", QVariant());
        }
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

void cocoaRetryFullscreenState(QWindow *window, Qt::WindowState state) {
    NSWindow *native = nativeWindow(window);
    if (native && native.visible && !native.miniaturized &&
        (native.styleMask & NSWindowStyleMaskFullScreen)) {
        if (state != Qt::WindowMinimized) {
            window->setWindowState(state);
            return;
        }
        const QPointer<QWindow> guarded(window);
        const auto revealSerial = window->property("xdzipCocoaRevealSerial").toULongLong();
        const auto surfaceGeneration =
            window->property("xdzipCocoaSurfaceGeneration").toULongLong();
        // Qt must first match the actual fullscreen state before it can forward
        // the same minimize request again. A reentrant reveal supersedes it.
        window->setWindowState(Qt::WindowFullScreen);
        if (guarded && guarded->property("xdzipCocoaRevealSerial").toULongLong() == revealSerial &&
            guarded->property("xdzipCocoaSurfaceGeneration").toULongLong() == surfaceGeneration &&
            guarded->handle() && guarded->isVisible() &&
            guarded->windowState() == Qt::WindowFullScreen &&
            !guarded->property("xdzipWakePending").toBool())
            guarded->showMinimized();
    }
}
