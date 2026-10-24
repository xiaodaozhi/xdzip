// apps/desktop/BrowserTransfers.cpp

#include "BrowserController.h"
#include "xdzip-qt-bridge/src/lib.rs.h"
#include <QDir>
#include <QDrag>
#include <QFileInfo>
#include <QGuiApplication>
#include <QMimeData>
#include <QQuickItem>
#include <QQuickWindow>
#include <QStandardPaths>

QString BrowserController::addDropMessage() const {
    return !m_search.isEmpty()  ? qtTrId("update.search")
           : canUpdateArchive() ? qtTrId("update.drop")
                                : qtTrId("update.unavailable");
}
bool BrowserController::canAddDroppedUrls(const QList<QUrl> &urls) const {
    if (!canUpdateArchive() || urls.isEmpty() || urls.size() > 100000)
        return false;
    qsizetype bytes = 0;
    for (const auto &url : urls) {
        if (!url.isLocalFile())
            return false;
        const auto path = url.toLocalFile();
        const QFileInfo info(path);
        bytes += path.toUtf8().size();
        if (!info.isAbsolute() || info.isSymLink() || (!info.isFile() && !info.isDir()) ||
            bytes > 4 * 1024 * 1024)
            return false;
    }
    return true;
}
void BrowserController::addDroppedUrls(const QList<QUrl> &urls) {
    if (!canAddDroppedUrls(urls))
        return;
    QStringList paths;
    for (const auto &url : urls)
        paths.append(url.toLocalFile());
    emit additionRequested(paths);
}
void BrowserController::startUpdate(const QStringList &inputs, const QString &session,
                                    const QString &directory, int conflict) {
    if (!canUpdateArchive() || m_busy || session != m_session || directory != m_directory ||
        inputs.isEmpty() || conflict < 0 || conflict > 3)
        return;
    xdzip::reader::Command command{};
    command.kind = xdzip::reader::CommandKind::Update;
    m_updateLocation = m_location;
    m_updateSort = m_sort;
    command.session = session.toULongLong();
    command.directory = directory.toStdString();
    command.conflict = static_cast<std::uint8_t>(conflict);
    for (const auto &path : inputs)
        command.inputs.emplace_back(path.toStdString());
    m_extractionIntent = {{"archive", m_title}, {"target", m_sourcePath}};
    submit(std::move(command));
}
void BrowserController::beginExport(int row, QWindow *owner) {
    if (!m_ready || m_busy || m_closing || m_updateActive || !owner || !m_model.rowAt(row))
        return;
    selectForContext(row);
    auto selectors = m_model.selectedIds();
    selectors.sort();
    const auto identity =
        m_session + QChar(0x0a) + m_directory + QChar(0x0a) + selectors.join(QChar(','));
    if (m_export && (m_export->inOperation.load() || (m_export->requested && !m_export->done)))
        return;
    if (m_export && m_exportKey == identity && !m_export->cancelled.load()) {
        if (m_export->done && m_export->error.isEmpty())
            startPreparedDrag();
        else if (!m_export->requested && !m_export->entries.isEmpty()) {
            captureNativeExportGesture(owner, m_export);
            launchExternalExport();
        }
        return;
    }
    if (m_export && m_export->inOperation.load())
        return;
    m_exportTask = 0;
    m_export = std::make_shared<NativeExportState>();
    m_exportOwner = owner;
    captureNativeExportGesture(owner, m_export);
    m_exportSelectors = selectors;
    m_exportSession = m_session;
    m_exportDirectory = m_directory;
    m_exportKey = identity;
    m_exportCache =
        QStandardPaths::writableLocation(QStandardPaths::CacheLocation) + "/exported-files";
    if (!QDir().mkpath(m_exportCache)) {
        m_export->finish({}, "output_io");
        return;
    }
    const QPointer<BrowserController> guard(this);
    m_export->prepare = [guard] {
        if (guard)
            QMetaObject::invokeMethod(
                guard,
                [guard] {
                    if (guard)
                        guard->prepareExternalExport();
                },
                Qt::QueuedConnection);
    };
    m_export->publish = [guard](std::shared_ptr<NativeExportCopy> request) {
        if (!guard) {
            request->finish("runtime_closed");
            return;
        }
        QMetaObject::invokeMethod(
            guard,
            [guard, request] {
                if (guard)
                    guard->publishExternalExport(request);
                else
                    request->finish("runtime_closed");
            },
            Qt::QueuedConnection);
    };
    xdzip::reader::Command command{};
    command.kind = xdzip::reader::CommandKind::ExportManifest;
    command.session = m_session.toULongLong();
    command.directory = m_directory.toStdString();
    command.path = m_exportCache.toStdString();
    for (const auto &id : selectors)
        command.selectors.emplace_back(id.toStdString());
    submit(std::move(command));
}
void BrowserController::prepareExternalExport() {
    if (!m_export || m_export->done || m_export->cancelled.load())
        return;
    if (m_busy) {
        QTimer::singleShot(20, this, &BrowserController::prepareExternalExport);
        return;
    }
    if (m_session != m_exportSession || m_closing) {
        m_export->finish({}, "source_changed");
        return;
    }
    xdzip::reader::Command command{};
    command.kind = xdzip::reader::CommandKind::Export;
    command.session = m_exportSession.toULongLong();
    command.directory = m_exportDirectory.toStdString();
    command.path = m_exportCache.toStdString();
    for (const auto &id : m_exportSelectors)
        command.selectors.emplace_back(id.toStdString());
    m_extractionIntent = {{"archive", m_title}};
    submit(std::move(command));
}
void BrowserController::publishExternalExport(const std::shared_ptr<NativeExportCopy> &request) {
    if (m_closing || !runtimeAvailable()) {
        request->finish("runtime_closed");
        return;
    }
    if (m_busy || m_exportCopy) {
        QTimer::singleShot(20, this, [this, request] { publishExternalExport(request); });
        return;
    }
    m_exportCopy = request;
    xdzip::reader::Command command{};
    command.kind = xdzip::reader::CommandKind::PublishExport;
    command.text = request->source.toStdString();
    command.path = request->target.toStdString();
    submit(std::move(command));
}
void BrowserController::startPreparedDrag() {
    if (!m_export || !m_exportOwner || !(QGuiApplication::mouseButtons() & Qt::LeftButton))
        return;
    if (!m_export->done || !m_export->error.isEmpty())
        return;
    QList<QUrl> urls;
    for (const auto &entry : m_export->entries)
        if (!entry.name.contains('/'))
            urls.append(QUrl::fromLocalFile(QDir(m_export->directory).filePath(entry.name)));
    if (urls.isEmpty())
        return;
    auto *mime = new QMimeData;
    mime->setUrls(urls);
    mime->setData("application/x-xdzip-export", "1");
    if (auto *window = qobject_cast<QQuickWindow *>(m_exportOwner.data()))
        if (auto *grabber = window->mouseGrabberItem())
            grabber->ungrabMouse();
    auto *drag = new QDrag(m_exportOwner);
    drag->setMimeData(mime);
    drag->exec(Qt::CopyAction, Qt::CopyAction);
    drag->deleteLater();
}
bool BrowserController::handleTransferEvent(const xdzip::reader::Event &event) {
    using xdzip::reader::EventKind;
    if (!m_pending || event.request_id != m_pending)
        return false;
    if (event.kind == EventKind::ExportManifest) {
        if (m_export)
            for (const auto &row : event.rows)
                m_export->entries.append(
                    {QString::fromUtf8(row.name.data(), qsizetype(row.name.size())), row.size,
                     row.directory});
        m_hasPayload = true;
        return true;
    }
    if (event.kind == EventKind::TaskAccepted && m_result.exporting && !m_exportCopy)
        m_exportTask = event.task.id;
    if (event.kind == EventKind::TaskAccepted && m_exportCopy)
        m_exportCopyTask = event.task.id;
    if (event.kind == EventKind::Finished && m_exportCopy && !event.error.empty()) {
        m_exportCopy->finish("export_failed");
        m_exportCopy.reset();
        m_exportCopyTask = 0;
    }
    if (event.kind != EventKind::Finished || !m_result.exportManifest)
        return false;
    m_pending = 0;
    m_busy = false;
    m_result = {};
    if (!event.error.empty() || !m_hasPayload || !m_export) {
        if (m_export)
            m_export->finish({}, "unsupported_feature");
        m_message = qtTrId("export.failed");
        emit changed();
        return true;
    }
    emit changed();
    QTimer::singleShot(0, this, &BrowserController::launchExternalExport);
    return true;
}
void BrowserController::launchExternalExport() {
    if (!m_export || !m_exportOwner || !(QGuiApplication::mouseButtons() & Qt::LeftButton))
        return;
#if defined(Q_OS_WIN) || defined(Q_OS_MACOS)
    if (auto *window = qobject_cast<QQuickWindow *>(m_exportOwner.data()))
        if (auto *grabber = window->mouseGrabberItem())
            grabber->ungrabMouse();
    if (!startNativeExport(m_exportOwner, m_export))
        m_message = qtTrId("export.failed");
    pollExternalExport();
#else
    m_export->request();
#endif
}
void BrowserController::pollExternalExport() {
    if (m_export && m_export->cancelled.load() && m_exportTask)
        cancelTask(QString::number(m_exportTask));
    emit changed();
    emit tasksChanged();
    if (m_export && m_export->inOperation.load())
        QTimer::singleShot(50, this, &BrowserController::pollExternalExport);
}
void BrowserController::refreshAfterUpdate(const QString &session, const QString &source) {
    if (m_closing || m_session != session)
        return;
    if (m_busy) {
        QTimer::singleShot(20, this,
                           [this, session, source] { refreshAfterUpdate(session, source); });
        return;
    }
    m_updateActive = false;
    m_restoreAfterUpdate = true;
    m_directory.clear();
    m_location.clear();
    m_search.clear();
    m_sort = 0;
    start(source, 0);
}
bool BrowserController::restoreUpdatedLocation() {
    if (!m_restoreAfterUpdate)
        return false;
    m_restoreAfterUpdate = false;
    m_directory = m_directoryPaths.value(m_updateLocation);
    m_location = m_directory.isEmpty() ? QString{} : m_updateLocation;
    m_sort = m_updateSort;
    if (!m_directory.isEmpty() || m_sort != 0) {
        start({}, 0);
        return true;
    }
    return false;
}
void BrowserController::finishCloseWhenTransfersDrain() {
    if (m_export && m_export->inOperation.load()) {
        QTimer::singleShot(20, this, &BrowserController::finishCloseWhenTransfersDrain);
        return;
    }
    if (m_closing && m_connection->stopped())
        emit readyToClose();
}
void BrowserController::handleTransferTask(const xdzip::reader::TaskSnapshot &task) {
    const auto kind = QString::fromUtf8(task.kind.data(), qsizetype(task.kind.size()));
    // The task lane may publish a terminal snapshot before metadata admission
    // reaches Qt. Bind the first matching task before resolving its terminal.
    if (kind == "export" && m_export && m_result.exporting && !m_exportCopy && m_exportTask == 0)
        m_exportTask = task.id;
    if (kind == "export-copy" && m_exportCopy && m_exportCopyTask == 0)
        m_exportCopyTask = task.id;
    if (m_exportCopy && task.id == m_exportCopyTask) {
        const auto status = QString::fromUtf8(task.status.data(), qsizetype(task.status.size()));
        if (status == "succeeded" || status == "failed" || status == "cancelled") {
            m_exportCopy->finish(status == "succeeded" ? QString{}
                                                       : QStringLiteral("export_failed"));
            m_exportCopy.reset();
            m_exportCopyTask = 0;
        }
    }
    if (!m_export || task.id != m_exportTask)
        return;
    const auto status = QString::fromUtf8(task.status.data(), qsizetype(task.status.size()));
    if (status == "succeeded") {
        const auto id = task.id;
        const QPointer<RuntimeConnection> connection(m_connection.get());
        m_export->releaseLease = [connection, id] {
            if (connection)
                QMetaObject::invokeMethod(
                    connection,
                    [connection, id] {
                        if (connection)
                            connection->releaseExport(id);
                    },
                    Qt::QueuedConnection);
        };
        m_export->finish(QString::fromUtf8(task.target.data(), qsizetype(task.target.size())), {});
#if !defined(Q_OS_WIN) && !defined(Q_OS_MACOS)
        m_message = qtTrId("export.ready");
        QTimer::singleShot(0, this, &BrowserController::startPreparedDrag);
#endif
    } else if (status == "failed" || status == "cancelled")
        m_export->finish({}, "export_failed");
}
