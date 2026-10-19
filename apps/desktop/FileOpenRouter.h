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
        if (!m_controller || !m_window || m_controller->busy())
            return;
        if (!m_controller->runtimeAvailable() || m_pending.isEmpty()) {
            m_window->setProperty("externalFileOpening", false);
            return;
        }
        const auto path = m_pending.takeFirst();
        // Publish the intent before openPath clears ready, including replacement
        // opens. A hidden startup window can submit without exposing empty UI.
        m_window->setProperty("externalFileOpening", true);
        m_controller->openPath(path);
        if (!m_controller->browsingBusy())
            m_window->setProperty("externalFileOpening", false);
        if (m_window->isVisible()) {
            m_window->raise();
            m_window->requestActivate();
        }
    }

  public:
    void enqueue(const QString &path) {
        if (path.isEmpty() || path.contains(QChar::Null) || m_pending.size() >= 128)
            return;
        if (!m_pending.contains(path))
            m_pending.append(path);
        pump();
    }
    void attach(BrowserController *controller, QQuickWindow *window) {
        m_controller = controller;
        m_window = window;
        connect(
            controller, &BrowserController::changed, this, [this] { pump(); },
            Qt::QueuedConnection);
        pump();
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
