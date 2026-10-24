// apps/desktop/NativeExport.cpp

#include "NativeExport.h"
#include <QCoreApplication>
#include <QThread>

void NativeExportState::request() {
    std::function<void()> callback;
    {
        std::lock_guard<std::mutex> lock(mutex);
        if (requested || done)
            return;
        requested = true;
        callback = prepare;
    }
    if (callback)
        callback();
    else
        finish({}, QStringLiteral("source_changed"));
}
bool NativeExportState::wait() {
    request();
    // A synchronous native consumer cannot make the GUI wait for its own worker.
    if (QThread::currentThread() == QCoreApplication::instance()->thread())
        return false;
    std::unique_lock<std::mutex> lock(mutex);
    changed.wait(lock, [this] { return done || cancelled.load(); });
    return done && error.isEmpty() && !cancelled.load();
}
void NativeExportState::finish(const QString &path, const QString &failure) {
    {
        std::lock_guard<std::mutex> lock(mutex);
        if (done)
            return;
        directory = path;
        error = failure;
        done = true;
    }
    changed.notify_all();
}
#if !defined(Q_OS_WIN) && !defined(Q_OS_MACOS)
bool startNativeExport(QWindow *, const std::shared_ptr<NativeExportState> &) {
    return false;
}
#endif

#ifndef Q_OS_MACOS
void captureNativeExportGesture(QWindow *, const std::shared_ptr<NativeExportState> &) {}
#endif
