#pragma once
#include "BrowserController.h"
#include <QFileOpenEvent>
#include <QPointer>
#include <QQuickWindow>
#include <QTimer>

// Capture macOS events before QML is ready; serialize intents through the
// existing controller idle barrier instead of dropping opens during a task.
class FileOpenRouter final : public QObject {
    QStringList m_pending;
    QPointer<BrowserController> m_controller;
    QPointer<QQuickWindow> m_window;
    void pump() {
        if (!m_controller || !m_controller->runtimeAvailable() || !m_window ||
            !m_window->isVisible() || m_controller->busy() || m_pending.isEmpty())
            return;
        const auto path = m_pending.takeFirst();
        m_controller->openPath(path);
        m_window->raise();
        m_window->requestActivate();
    }

  public:
    void enqueue(const QString &path) {
        if (path.isEmpty() || path.contains(QChar::Null) || m_pending.size() >= 128)
            return;
        if (!m_pending.contains(path))
            m_pending.append(path);
        QTimer::singleShot(0, this, [this] { pump(); });
    }
    void attach(BrowserController *controller, QQuickWindow *window) {
        m_controller = controller;
        m_window = window;
        connect(
            controller, &BrowserController::changed, this, [this] { pump(); },
            Qt::QueuedConnection);
        QTimer::singleShot(0, this, [this] { pump(); });
    }
    bool eventFilter(QObject *, QEvent *event) override {
        if (event->type() != QEvent::FileOpen)
            return false;
        const auto *open = static_cast<QFileOpenEvent *>(event);
        if (open->url().isLocalFile())
            enqueue(open->url().toLocalFile());
        event->accept();
        return true;
    }
};
