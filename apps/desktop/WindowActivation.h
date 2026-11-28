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
#include <QPointer>
bool watchCocoaActivation(QWindow *window);
void unwatchCocoaActivation(QWindow *window);
bool cocoaActivationInFlight(QWindow *window);
bool cocoaActivationIsMiniaturized(QWindow *window);
bool cocoaActivationIsVisible(QWindow *window);
bool cocoaFullscreenRetryReady(QWindow *window);
void cocoaRetryFullscreenState(QWindow *window, Qt::WindowState state);
void cocoaRetryMinimizeAfterFullscreenExit(QWindow *window);
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
#ifdef Q_OS_MACOS
        if (QGuiApplication::platformName() == QStringLiteral("cocoa")) {
            window->setProperty("xdzipCocoaRevealSerial",
                                window->property("xdzipCocoaRevealSerial").toULongLong() + 1);
            window->setProperty("xdzipCocoaPendingMinimizeGeneration", QVariant());
            window->setProperty("xdzipCocoaStateCandidateGeneration", QVariant());
        }
#endif
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
        if (window->windowState() == Qt::WindowMinimized)
            noteCocoaMinimizeIntent(window);
        queueCocoaMinimizeRetry(window);
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
            connect(window, &QWindow::windowStateChanged, window, [window](Qt::WindowState) {
                remember(window);
#ifdef Q_OS_MACOS
                noteCocoaMinimizeIntent(window);
                queueCocoaMinimizeRetry(window);
#endif
            });
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
#ifdef Q_OS_MACOS
            // A native Qt state signal is followed by this spontaneous event;
            // it must not supersede an explicit request made during entry.
            if (QGuiApplication::platformName() == QStringLiteral("cocoa"))
                window->setProperty("xdzipCocoaStateCandidateGeneration", QVariant());
#endif
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
#ifdef Q_OS_MACOS
    static void noteCocoaMinimizeIntent(QWindow *window) {
        if (QGuiApplication::platformName() != QStringLiteral("cocoa") ||
            window->property("xdzipCocoaSendingMinimizeRetry").toBool())
            return;
        const bool entering = window->property("xdzipCocoaFullEntering").toBool();
        const auto generation = window->property("xdzipCocoaSurfaceGeneration").toULongLong();
        if (!generation)
            return;
        const bool pending =
            window->property("xdzipCocoaPendingMinimizeGeneration").toULongLong() == generation;
        const auto state = window->windowState();
        const bool unconfirmed = state == Qt::WindowMinimized &&
                                 !cocoaActivationIsMiniaturized(window) &&
                                 (window->property("xdzipCocoaFullTransition").toBool() ||
                                  cocoaFullscreenRetryReady(window));
        if (!entering && !window->property("xdzipCocoaJustEnteredFullscreen").toBool() &&
            !pending && !unconfirmed)
            return;
        if (state == Qt::WindowMinimized) {
            window->setProperty("xdzipCocoaPendingMinimizeGeneration", generation);
            window->setProperty("xdzipCocoaPendingState", int(state));
            window->setProperty("xdzipCocoaStateCandidateGeneration", QVariant());
            if (entering)
                window->setProperty("xdzipRestoreState", int(Qt::WindowFullScreen));
        } else if (pending) {
            // Direct Qt state setters emit a signal without a spontaneous
            // event. Defer the decision so later explicit intent wins over
            // the intermediate fullscreen notification from the platform.
            window->setProperty("xdzipCocoaStateCandidateGeneration", generation);
            window->setProperty("xdzipCocoaStateCandidate", int(state));
            QTimer::singleShot(0, window, [window, generation] {
                if (window->property("xdzipCocoaStateCandidateGeneration").toULongLong() ==
                        generation &&
                    window->property("xdzipCocoaSurfaceGeneration").toULongLong() == generation) {
                    window->setProperty("xdzipCocoaPendingState",
                                        window->property("xdzipCocoaStateCandidate"));
                    window->setProperty("xdzipCocoaStateCandidateGeneration", QVariant());
                    queueCocoaMinimizeRetry(window);
                }
            });
        }
    }
    // AppKit can ignore a minimize before its fullscreen Space becomes ready.
    static void queueCocoaMinimizeRetry(QWindow *window) {
        if (QGuiApplication::platformName() != QStringLiteral("cocoa") ||
            window->property("xdzipCocoaFullEntering").toBool())
            return;
        const auto generation = window->property("xdzipCocoaSurfaceGeneration").toULongLong();
        if (!generation ||
            window->property("xdzipCocoaPendingMinimizeGeneration").toULongLong() != generation)
            return;
        awaitCocoaFullscreenRetry(window, generation, 20);
    }
    static void awaitCocoaFullscreenRetry(QWindow *window, qulonglong generation,
                                          int remainingChecks) {
        if (window->property("xdzipCocoaMinimizeRetryGeneration").toULongLong() == generation)
            return;
        window->setProperty("xdzipCocoaMinimizeRetryGeneration", generation);
        QTimer::singleShot(50, window, [window, generation, remainingChecks] {
            if (window->property("xdzipCocoaMinimizeRetryGeneration").toULongLong() != generation)
                return;
            window->setProperty("xdzipCocoaMinimizeRetryGeneration", QVariant());
            if (window->property("xdzipCocoaPendingMinimizeGeneration").toULongLong() != generation)
                return;
            if (window->property("xdzipCocoaStateCandidateGeneration").toULongLong() ==
                generation) {
                window->setProperty("xdzipCocoaPendingState",
                                    window->property("xdzipCocoaStateCandidate"));
                window->setProperty("xdzipCocoaStateCandidateGeneration", QVariant());
            }
            const auto state = Qt::WindowState(window->property("xdzipCocoaPendingState").toInt());
            if (window->property("xdzipCocoaSurfaceGeneration").toULongLong() != generation ||
                !window->handle() || !window->isVisible() || state == Qt::WindowFullScreen ||
                window->property("xdzipWakePending").toBool() ||
                cocoaActivationIsMiniaturized(window)) {
                window->setProperty("xdzipCocoaPendingMinimizeGeneration", QVariant());
                return;
            }
            if (cocoaActivationInFlight(window) ||
                (!cocoaFullscreenRetryReady(window) &&
                 !(state == Qt::WindowMinimized && cocoaActivationIsVisible(window)))) {
                // A fullscreen completion notification can precede the next
                // visible Space. Wait briefly for the observed native state.
                if (remainingChecks > 1)
                    awaitCocoaFullscreenRetry(window, generation, remainingChecks - 1);
                else if (!cocoaActivationInFlight(window))
                    window->setProperty("xdzipCocoaPendingMinimizeGeneration", QVariant());
                return;
            }
            if (state != Qt::WindowMinimized)
                window->setProperty("xdzipCocoaPendingMinimizeGeneration", QVariant());
            const QPointer<QWindow> guarded(window);
            const auto reveal = window->property("xdzipCocoaRevealSerial").toULongLong();
            window->setProperty("xdzipCocoaSendingMinimizeRetry", true);
            if (cocoaFullscreenRetryReady(window))
                cocoaRetryFullscreenState(window, state);
            else
                cocoaRetryMinimizeAfterFullscreenExit(window);
            if (!guarded)
                return;
            guarded->setProperty("xdzipCocoaSendingMinimizeRetry", false);
            // A dispatched request may still have been ignored by AppKit.
            // Retry only if the same surface still shows the same unconfirmed intent.
            if (remainingChecks > 1 && state == Qt::WindowMinimized &&
                guarded->property("xdzipCocoaSurfaceGeneration").toULongLong() == generation &&
                guarded->property("xdzipCocoaRevealSerial").toULongLong() == reveal &&
                guarded->property("xdzipCocoaPendingMinimizeGeneration").toULongLong() ==
                    generation &&
                guarded->property("xdzipCocoaPendingState").toInt() == int(state)) {
                awaitCocoaFullscreenRetry(guarded, generation, remainingChecks - 1);
            } else if (remainingChecks <= 1 && !cocoaActivationInFlight(guarded) &&
                       guarded->property("xdzipCocoaPendingMinimizeGeneration").toULongLong() ==
                           generation) {
                guarded->setProperty("xdzipCocoaPendingMinimizeGeneration", QVariant());
            }
        });
    }
#endif
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
