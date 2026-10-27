// apps/desktop/NativeExport.h

#pragma once
#include <QList>
#include <QImage>
#include <QPointer>
#include <QString>
#include <QWindow>
#include <atomic>
#include <condition_variable>
#include <functional>
#include <memory>
#include <mutex>

struct ExportItem {
    QString name;
    quint64 size = 0;
    bool directory = false;
};
// Shared lifetime crosses the native drag session, background consumers and the
// source page. Only validated private copies become readable when done is true.
struct NativeExportCopy {
    QString source, target, error;
    std::mutex mutex;
    std::condition_variable changed;
    bool done = false;
    void finish(const QString &failure) {
        {
            std::lock_guard<std::mutex> lock(mutex);
            error = failure;
            done = true;
        }
        changed.notify_all();
    }
};
struct NativeExportState {
    QList<ExportItem> entries;
    std::shared_ptr<void> gesture;
    std::mutex mutex;
    std::condition_variable changed;
    QString directory, error;
    bool requested = false, done = false;
    std::atomic<bool> cancelled{false};
    std::atomic<bool> inOperation{false};
    std::atomic<int> pendingPromises{0};
    std::function<void()> prepare, releaseLease;
    std::function<void()> dragFinished;
    ~NativeExportState() {
        if (releaseLease)
            releaseLease();
    }
    std::function<void(std::shared_ptr<NativeExportCopy>)> publish;
    void request();
    bool wait();
    void finish(const QString &path, const QString &failure);
};
void captureNativeExportGesture(QWindow *owner, const std::shared_ptr<NativeExportState> &state);
bool startNativeExport(QWindow *owner, const std::shared_ptr<NativeExportState> &state);
QImage nativeExportPreview(const QList<ExportItem> &entries, qreal devicePixelRatio);

#ifdef Q_OS_WIN
struct IDataObject;
IDataObject *nativeExportData(const std::shared_ptr<NativeExportState> &state);
bool initializeNativeExportImage(IDataObject *data, const QImage &preview, QString *failure = nullptr);
#endif
