#pragma once
#include "BrowserController.h"
#include <QEvent>
#include <QPointer>
#include <QQuickWindow>

// Native Quit and window close share the QML task decision, then the asynchronous join barrier.
class AppQuitGuard final : public QObject {
  public:
    QPointer<BrowserController> controller;
    QPointer<QQuickWindow> window;
    bool eventFilter(QObject *, QEvent *event) override {
        if (event->type() != QEvent::Quit || !controller)
            return false;
        if (window && window->isVisible()) {
            window->close();
            return true; // The QML closing handler may show a decision or wait for shutdown.
        }
        return !controller->requestClose();
    }
};
