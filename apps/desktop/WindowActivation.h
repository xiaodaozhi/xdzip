// apps/desktop/WindowActivation.h

#pragma once
#include <QCoreApplication>
#include <QEvent>
#include <QGuiApplication>
#include <QTimer>
#include <QVariant>
#include <QWindow>

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
        const bool minimized = window->windowStates().testFlag(Qt::WindowMinimized);
        if (!minimized && window->isExposed() && window->isActive() &&
            int(window->windowStates()) == window->property("xdzipRestoreState").toInt()) {
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
            return;
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

  protected:
    bool eventFilter(QObject *object, QEvent *event) override {
        auto *window = qobject_cast<QWindow *>(object);
        if (!window)
            return false;
        if (!window->property("xdzipActivationTracked").toBool()) {
            window->setProperty("xdzipActivationTracked", true);
            connect(window, &QWindow::windowStateChanged, window,
                    [window](Qt::WindowState) { remember(window); });
            remember(window);
        }
        if (event->type() == QEvent::FocusIn && window->isExposed() &&
            !window->windowStates().testFlag(Qt::WindowMinimized) &&
            (!window->property("xdzipWakePending").toBool() ||
             int(window->windowStates()) == window->property("xdzipRestoreState").toInt())) {
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
        if (event->type() == QEvent::Expose)
            queueWake(window);
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
