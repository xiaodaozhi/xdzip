// apps/desktop/WindowActivation.h

#pragma once
#include <QCoreApplication>
#include <QEvent>
#include <QGuiApplication>
#include <QTimer>
#include <QVariant>
#include <QWindow>

#ifdef Q_OS_MACOS
#include <QPlatformSurfaceEvent>
bool watchCocoaActivation(QWindow *window);
void unwatchCocoaActivation(QWindow *window);
bool cocoaActivationInFlight(QWindow *window);
bool cocoaActivationIsMiniaturized(QWindow *window);
bool cocoaActivationIsVisible(QWindow *window);
#endif

// Showing an already visible QWindow chooses the platform default state. Keep
// the last non-minimized state instead; activation must never normalize it first.
class WindowActivation final : public QObject {
  public:
    static void install(QCoreApplication *application) {
        if (application->property("xdzipActivationInstalled").toBool())
            return;
        application->setProperty("xdzipActivationInstalled", true);
        application->installEventFilter(new WindowActivation(application));
    }
    static void reveal(QWindow *window) {
        if (!window)
            return;
        bool nativeTransitionInFlight = false;
#ifdef Q_OS_MACOS
        const bool cocoa = QGuiApplication::platformName() == QStringLiteral("cocoa");
        const bool cocoaObserved = cocoa && watchCocoaActivation(window);
        nativeTransitionInFlight = cocoaObserved && cocoaActivationInFlight(window);
#endif
        const bool minimized = window->windowStates().testFlag(Qt::WindowMinimized);
        if (!minimized && window->isExposed() && window->isActive() &&
            int(window->windowStates()) == window->property("xdzipRestoreState").toInt() &&
            !nativeTransitionInFlight) {
            window->setProperty("xdzipWasMinimized", false);
            window->setProperty("xdzipWakePending", false);
        }
        // Cocoa finishes leaving fullscreen before it miniaturizes. Keep this
        // activation pending until the native minimized event is confirmed,
        // instead of cancelling a transition halfway through it. A fullscreen
        // Space can become unexposed before the window is actually minimized.
        if (QGuiApplication::platformName() == QStringLiteral("cocoa") && window->isVisible() &&
            (minimized || window->property("xdzipWasMinimized").toBool()) &&
            !window->property("xdzipNativeMinimized").toBool()) {
            window->setProperty("xdzipWakePending", true);
#ifdef Q_OS_MACOS
            if (cocoaObserved && cocoaActivationIsMiniaturized(window)) {
                if (!window->windowStates().testFlag(Qt::WindowMinimized))
                    window->setWindowStates(window->windowStates() | Qt::WindowMinimized);
                window->setProperty("xdzipNativeMinimized", true);
            } else if (cocoaObserved && !nativeTransitionInFlight && window->isExposed() &&
                       cocoaActivationIsVisible(window)) {
                // Qt may optimistically report Minimized even when AppKit did
                // not start a transition. Cancel that request only when the
                // observed native window is visible and no transition is active.
                const auto state = Qt::WindowStates(window->property("xdzipRestoreState").toInt());
                window->setWindowStates(state);
                window->setProperty("xdzipWasMinimized", false);
                window->setProperty("xdzipWakePending", false);
                window->raise();
                window->requestActivate();
                return;
            } else {
                return;
            }
#else
            return;
#endif
        }
        if (window->windowStates().testFlag(Qt::WindowMinimized)) {
            auto state = window->windowStates() & ~Qt::WindowMinimized;
            // Cocoa may retain the older zoomed bit after leaving fullscreen
            // to miniaturize. The saved pre-minimize target is authoritative.
            if (window->property("xdzipRestoreState").isValid())
                state = Qt::WindowStates(window->property("xdzipRestoreState").toInt());
            // showMinimized() replaces the previous state bits. Prepare the
            // saved target while still minimized, then remove only Minimized;
            // this avoids entering fullscreen from a native normal restore.
            if ((window->windowStates() & ~Qt::WindowMinimized) != state)
                window->setWindowStates(state | Qt::WindowMinimized);
            window->setWindowStates(state);
            window->setProperty("xdzipWasMinimized", false);
            window->setProperty("xdzipRestoreState", int(state));
        }
        window->setProperty("xdzipWakePending", false);
        if (!window->isVisible())
            window->setVisible(true);
        window->raise();
        window->requestActivate();
    }

#ifdef Q_OS_MACOS
    static void cocoaNativeMinimized(QWindow *window) {
        if (window->property("xdzipWakePending").toBool() &&
            !window->windowStates().testFlag(Qt::WindowMinimized))
            window->setWindowStates(window->windowStates() | Qt::WindowMinimized);
        window->setProperty("xdzipNativeMinimized", true);
        queueWake(window);
    }
    static void cocoaTransitionFinished(QWindow *window) {
        if (window->property("xdzipWakePending").toBool())
            QTimer::singleShot(0, window, [window] {
                if (window->property("xdzipWakePending").toBool())
                    reveal(window);
            });
    }
#endif

  protected:
    bool eventFilter(QObject *object, QEvent *event) override {
        auto *window = qobject_cast<QWindow *>(object);
        if (!window)
            return false;
        bool nativeTransitionInFlight = false;
#ifdef Q_OS_MACOS
        if (QGuiApplication::platformName() == QStringLiteral("cocoa") &&
            event->type() == QEvent::PlatformSurface &&
            static_cast<QPlatformSurfaceEvent *>(event)->surfaceEventType() ==
                QPlatformSurfaceEvent::SurfaceAboutToBeDestroyed)
            unwatchCocoaActivation(window);
        if (QGuiApplication::platformName() == QStringLiteral("cocoa") &&
            (event->type() == QEvent::Show || event->type() == QEvent::Expose))
            watchCocoaActivation(window);
        nativeTransitionInFlight = QGuiApplication::platformName() == QStringLiteral("cocoa") &&
                                   cocoaActivationInFlight(window);
#endif
        if (!window->property("xdzipActivationTracked").toBool()) {
            window->setProperty("xdzipActivationTracked", true);
            connect(window, &QWindow::windowStateChanged, window,
                    [window](Qt::WindowState) { remember(window); });
            remember(window);
        }
        if (event->type() == QEvent::FocusIn && window->isExposed() &&
            !window->windowStates().testFlag(Qt::WindowMinimized) &&
            (!window->property("xdzipWakePending").toBool() ||
             (int(window->windowStates()) == window->property("xdzipRestoreState").toInt() &&
              !nativeTransitionInFlight))) {
            window->setProperty("xdzipWasMinimized", false);
            window->setProperty("xdzipWakePending", false);
            remember(window);
        }
        if (event->type() == QEvent::WindowStateChange && event->spontaneous()) {
            remember(window);
            window->setProperty("xdzipNativeMinimized",
                                window->windowStates().testFlag(Qt::WindowMinimized));
            queueWake(window);
        }
        if (event->type() == QEvent::Expose) {
            queueWake(window);
#ifdef Q_OS_MACOS
            if (QGuiApplication::platformName() == QStringLiteral("cocoa"))
                cocoaTransitionFinished(window);
#endif
        }
        if (event->type() == QEvent::Hide)
            remember(window);
        return false;
    }

  private:
    static void queueWake(QWindow *window) {
        if (window->property("xdzipWakePending").toBool() &&
            window->property("xdzipNativeMinimized").toBool() &&
            window->windowStates().testFlag(Qt::WindowMinimized))
            QTimer::singleShot(0, window, [window] {
                if (window->property("xdzipWakePending").toBool())
                    reveal(window);
            });
    }
    static void remember(QWindow *window) {
        const auto state = window->windowStates();
        if (state.testFlag(Qt::WindowMinimized)) {
            if (!window->property("xdzipWasMinimized").toBool())
                window->setProperty("xdzipNativeMinimized", false);
            if (!window->property("xdzipWasMinimized").toBool() &&
                (state.testFlag(Qt::WindowMaximized) || state.testFlag(Qt::WindowFullScreen)))
                window->setProperty("xdzipRestoreState", int(state & ~Qt::WindowMinimized));
            window->setProperty("xdzipWasMinimized", true);
        } else if (!window->property("xdzipWasMinimized").toBool() ||
                   state.testFlag(Qt::WindowFullScreen) ||
                   (state.testFlag(Qt::WindowMaximized) &&
                    !(window->property("xdzipRestoreState").toInt() & Qt::WindowFullScreen)) ||
                   int(state) == window->property("xdzipRestoreState").toInt()) {
            window->setProperty("xdzipWasMinimized", false);
            window->setProperty("xdzipRestoreState", int(state));
        }
        // Observe requests and native acknowledgements. X11 can remove the
        // maximized state before iconifying; Cocoa first leaves fullscreen.
        // Neither intermediate state replaces the pre-minimize target.
        queueWake(window);
    }
    explicit WindowActivation(QObject *parent) : QObject(parent) {}
};
