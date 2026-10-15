#pragma once
#include <QFileDialog>
#include <QGuiApplication>
#include <QPointer>
#include <QTimer>
#include <QWindow>

// Keep the native selector independent, but give it the window that initiated it.
// A guarded pointer prevents a queued focus restoration from reviving a closed owner.
inline bool execOwnedFileDialog(QFileDialog &dialog, QWindow *caller) {
    const QPointer<QWindow> owner(caller ? caller : QGuiApplication::focusWindow());
    dialog.setWindowModality(Qt::ApplicationModal);
    dialog.winId();
    if (owner)
        dialog.windowHandle()->setTransientParent(owner);
    const bool accepted = dialog.exec() == QDialog::Accepted;
    QTimer::singleShot(0, qApp, [owner] {
        if (owner && owner->isVisible()) {
            owner->raise();
            owner->requestActivate();
        }
    });
    return accepted;
}
