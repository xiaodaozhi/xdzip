// apps/desktop/WindowActivation.h

#pragma once
#include <QCoreApplication>
#include <QEvent>
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
        if (window->windowStates().testFlag(Qt::WindowMinimized)) {
            auto state = window->windowStates() & ~Qt::WindowMinimized;
            if (state == Qt::WindowNoState && window->property("xdzipRestoreState").isValid())
                state = Qt::WindowStates(window->property("xdzipRestoreState").toInt());
            window->setWindowStates(state);
        }
        if (!window->isVisible())
            window->setVisible(true);
        window->raise();
        window->requestActivate();
    }

  protected:
    bool eventFilter(QObject *object, QEvent *event) override {
        auto *window = qobject_cast<QWindow *>(object);
        if (window &&
            (event->type() == QEvent::WindowStateChange || event->type() == QEvent::Hide)) {
            const auto state = window->windowStates();
            if (!state.testFlag(Qt::WindowMinimized))
                window->setProperty("xdzipRestoreState", int(state));
            else if (state.testFlag(Qt::WindowMaximized) || state.testFlag(Qt::WindowFullScreen))
                window->setProperty("xdzipRestoreState", int(state & ~Qt::WindowMinimized));
        }
        return false;
    }

  private:
    explicit WindowActivation(QObject *parent) : QObject(parent) {}
};
