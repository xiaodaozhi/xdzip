#include "BrowserController.h"
#include "ImagePreviewProtocol.h"
#include "NativeDialogPolicy.h"
#include "PreviewImages.h"
#include "UiText.h"
#include "rust/cxx.h"
#include "xdzip-qt-bridge/src/lib.rs.h"
#include <QClipboard>
#include <QDate>
#include <QDateTime>
#include <QDesktopServices>
#include <QDir>
#include <QFileDialog>
#include <QFileInfo>
#include <QGuiApplication>
#include <QLocale>
#include <QPointer>
#include <QRegularExpression>
#include <QScopeGuard>
#include <QStandardPaths>
#include <QTime>
#include <QUrl>
#include <algorithm>
#include <limits>
#include <utility>
#include <vector>

int ArchiveTableModel::rowCount(const QModelIndex &parent) const {
    return parent.isValid() ? 0 : paged() ? m_pagedCount : static_cast<int>(m_rows.size());
}
int ArchiveTableModel::columnCount(const QModelIndex &parent) const {
    return parent.isValid() ? 0 : 5;
}
QVariant ArchiveTableModel::data(const QModelIndex &index, int role) const {
    if (!index.isValid() || index.row() < 0 || index.row() >= rowCount() || index.column() < 0 ||
        index.column() >= 5)
        return {};
    const auto *loaded = rowAt(index.row());
    if (!loaded) {
        if (paged()) {
            int offset = index.row() / 128 * 128;
            while (m_cache.contains(offset))
                ++offset;
            emit pageNeeded(offset);
        }
        if (role == Qt::UserRole + 2)
            return rangeSelected(index.row());
        if (role == Qt::UserRole + 6)
            return false;
        return {};
    }
    const auto &row = *loaded;
    if (role == Qt::DisplayRole)
        return row.cells.at(index.column());
    if (role == Qt::UserRole)
        return row.id;
    if (role == Qt::UserRole + 1)
        return row.directory;
    if (role == Qt::UserRole + 2)
        return paged() ? rangeSelected(index.row()) : m_selected.contains(row.id);
    if (role == Qt::UserRole + 3)
        return row.navigable;
    if (role == Qt::UserRole + 4)
        return row.status;
    if (role == Qt::UserRole + 6)
        return true;
    if (role == Qt::UserRole + 5)
        return row.attention;
    return {};
}
QHash<int, QByteArray> ArchiveTableModel::roleNames() const {
    return {{Qt::DisplayRole, "display"},      {Qt::UserRole, "entryId"},
            {Qt::UserRole + 1, "isDirectory"}, {Qt::UserRole + 2, "chosen"},
            {Qt::UserRole + 3, "canNavigate"}, {Qt::UserRole + 4, "entryStatus"},
            {Qt::UserRole + 5, "attention"},   {Qt::UserRole + 6, "canSelect"}};
}
void ArchiveTableModel::replace(QList<BrowserRow> rows) {
    beginResetModel();
    m_pagedCount = -1;
    m_cache.clear();
    m_cacheOrder.clear();
    m_ranges.clear();
    m_queryIdentity.clear();
    m_rows = std::move(rows);
    ++m_selectionRevision;
    m_selected.clear();
    endResetModel();
}

const BrowserRow *ArchiveTableModel::rowAt(int row) const {
    if (paged()) {
        auto found = m_cache.constFind(row);
        return found == m_cache.cend() ? nullptr : &found.value();
    }
    return row >= 0 && row < m_rows.size() ? &m_rows[row] : nullptr;
}
void ArchiveTableModel::select(int row) {
    const auto *item = rowAt(row);
    if (!item)
        return;
    if (paged()) {
        setRange(row, row, !rangeSelected(row));
        return;
    }
    ++m_selectionRevision;
    if (m_selected.contains(item->id))
        m_selected.remove(item->id);
    else
        m_selected.insert(item->id);
    emit dataChanged(index(row, 0), index(row, 4), {Qt::UserRole + 2});
}

bool ArchiveTableModel::isSelected(int row) const {
    const auto *item = rowAt(row);
    return paged() ? rangeSelected(row) : item && m_selected.contains(item->id);
}
void ArchiveTableModel::selectOnly(int row) {
    const auto *item = rowAt(row);
    if (!item)
        return;
    if (paged()) {
        m_ranges.clear();
        setRange(row, row, true);
        return;
    }
    ++m_selectionRevision;
    m_selected = {item->id};
    emit dataChanged(index(0, 0), index(rowCount() - 1, 4), {Qt::UserRole + 2});
}
void ArchiveTableModel::selectAll(bool selected) {
    if (paged()) {
        m_ranges.clear();
        setRange(0, m_pagedCount - 1, selected);
        return;
    }
    ++m_selectionRevision;
    m_selected.clear();
    if (selected)
        for (const auto &item : m_rows)
            m_selected.insert(item.id);
    if (!m_rows.empty())
        emit dataChanged(index(0, 0), index(rowCount() - 1, 4), {Qt::UserRole + 2});
}
void ArchiveTableModel::selectRange(int first, int last) {
    if (first < 0 || last < 0 || first >= rowCount() || last >= rowCount() ||
        (!paged() && (!rowAt(first) || !rowAt(last))))
        return;
    if (paged()) {
        m_ranges.clear();
        setRange(std::min(first, last), std::max(first, last), true);
        return;
    }
    ++m_selectionRevision;
    m_selected.clear();
    for (int row = std::min(first, last); row <= std::max(first, last); ++row)
        m_selected.insert(m_rows.at(row).id);
    emit dataChanged(index(0, 0), index(rowCount() - 1, 4), {Qt::UserRole + 2});
}

void ArchiveTableModel::replacePaged(int total, QList<BrowserRow> rows, const QString &identity) {
    replace({});
    beginResetModel();
    m_pagedCount = total;
    m_queryIdentity = identity;
    for (int i = 0; i < rows.size(); ++i) {
        m_cache.insert(i, std::move(rows[i]));
        m_cacheOrder.enqueue(i);
    }
    endResetModel();
}
void ArchiveTableModel::installPage(int offset, QList<BrowserRow> rows) {
    if (!paged() || offset < 0 || offset + rows.size() > m_pagedCount)
        return;
    for (int i = 0; i < rows.size(); ++i) {
        const int row = offset + i;
        if (!m_cache.contains(row))
            m_cacheOrder.enqueue(row);
        m_cache.insert(row, std::move(rows[i]));
    }
    while (m_cache.size() > 1024)
        m_cache.remove(m_cacheOrder.dequeue());
    if (m_pagedCount)
        emit dataChanged(index(0, 0), index(m_pagedCount - 1, 4));
}
QList<int> ArchiveTableModel::cachedRows() const {
    if (paged())
        return m_cache.keys();
    QList<int> result;
    result.reserve(m_rows.size());
    for (int row = 0; row < m_rows.size(); ++row)
        result.append(row);
    return result;
}
bool ArchiveTableModel::rangeSelected(int row) const {
    for (const auto &range : m_ranges) {
        if (range.first > row)
            return false;
        if (range.second >= row)
            return true;
    }
    return false;
}
void ArchiveTableModel::setRange(int first, int last, bool selected) {
    if (first < 0 || last < first || last >= m_pagedCount)
        return;
    QList<QPair<int, int>> ranges;
    for (const auto &range : m_ranges) {
        if (range.second < first || range.first > last)
            ranges.append(range);
        else {
            if (range.first < first)
                ranges.append({range.first, first - 1});
            if (range.second > last)
                ranges.append({last + 1, range.second});
        }
    }
    if (selected)
        ranges.append({first, last});
    std::sort(ranges.begin(), ranges.end());
    QList<QPair<int, int>> merged;
    for (const auto &range : ranges) {
        if (!merged.isEmpty() && range.first <= merged.last().second + 1)
            merged.last().second = std::max(merged.last().second, range.second);
        else
            merged.append(range);
    }
    if (merged.size() > 4096)
        return;
    m_ranges = std::move(merged);
    ++m_selectionRevision;
    emit dataChanged(index(0, 0), index(rowCount() - 1, 4), {Qt::UserRole + 2});
}
int ArchiveTableModel::selectedCount() const {
    if (!paged())
        return m_selected.size();
    int count = 0;
    for (const auto &range : m_ranges)
        count += range.second - range.first + 1;
    return count;
}
QStringList ArchiveTableModel::selectedIds() const {
    if (!paged())
        return m_selected.values();
    QStringList result;
    for (const auto &range : m_ranges)
        result.append(
            QStringLiteral("q%1:%2:%3").arg(m_queryIdentity).arg(range.first).arg(range.second));
    return result;
}
QStringList ArchiveTableModel::physicalSelectedIds() const {
    if (!paged())
        return selectedIds();
    QStringList ids;
    for (int row : cachedRows())
        if (rangeSelected(row)) {
            const auto *item = rowAt(row);
            if (!item->physical)
                return {};
            ids.append(item->id);
        }
    return ids.size() == selectedCount() ? ids : QStringList{};
}

int TaskListModel::rowCount(const QModelIndex &parent) const {
    return parent.isValid() ? 0 : static_cast<int>(m_rows.size());
}
QVariant TaskListModel::data(const QModelIndex &index, int role) const {
    if (!index.isValid() || index.row() < 0 || index.row() >= m_rows.size() ||
        index.column() != 0 || role != TaskDataRole)
        return {};
    return m_rows.at(index.row());
}
QHash<int, QByteArray> TaskListModel::roleNames() const {
    return {{TaskDataRole, "taskData"}};
}
void TaskListModel::sync(const QVariantList &snapshots) {
    QSet<QString> retained;
    for (const auto &value : snapshots)
        retained.insert(value.toMap().value("id").toString());
    for (int row = rowCount() - 1; row >= 0; --row) {
        if (retained.contains(m_rows.at(row).value("id").toString()))
            continue;
        beginRemoveRows({}, row, row);
        m_rows.removeAt(row);
        endRemoveRows();
    }
    for (int row = 0; row < snapshots.size(); ++row) {
        const auto snapshot = snapshots.at(row).toMap();
        const auto id = snapshot.value("id").toString();
        int previous = row;
        while (previous < rowCount() && m_rows.at(previous).value("id").toString() != id)
            ++previous;
        if (previous == rowCount()) {
            beginInsertRows({}, row, row);
            m_rows.insert(row, snapshot);
            endInsertRows();
        } else {
            if (previous != row) {
                beginMoveRows({}, previous, previous, {}, row);
                m_rows.move(previous, row);
                endMoveRows();
            }
            if (m_rows.at(row) != snapshot) {
                m_rows[row] = snapshot;
                emit dataChanged(index(row), index(row), {TaskDataRole});
            }
        }
    }
}

BrowserController::BrowserController(QObject *parent)
    : QObject(parent), m_model(this), m_taskModel(this), m_directories(this),
      m_connection(std::make_unique<RuntimeConnection>(this)) {
    connect(&m_model, &ArchiveTableModel::pageNeeded, this, [this](int offset) {
        if (m_desiredPage < 0)
            m_desiredPage = offset;
        QTimer::singleShot(0, this, &BrowserController::drainPage);
    });
    connect(this, &BrowserController::tasksChanged, this, [this] { m_taskModel.sync(tasks()); });
    m_directories.setItemRoleNames({{Qt::DisplayRole, "nodeName"},
                                    {Qt::UserRole, "nodeId"},
                                    {Qt::UserRole + 1, "entryCount"},
                                    {Qt::UserRole + 2, "fullPath"},
                                    {Qt::UserRole + 3, "continuationFlags"},
                                    {Qt::UserRole + 4, "lastSibling"},
                                    {Qt::UserRole + 5, "nodeDepth"}});
    m_admissionRetry.setSingleShot(true);
    connect(&m_admissionRetry, &QTimer::timeout, this, &BrowserController::attemptSubmission);
    m_connection->setConsumer([this](const xdzip::reader::Event &event) { receive(event); });
    connect(m_connection.get(), &RuntimeConnection::disconnected, this, [this](bool graceful) {
        m_busy = false;
        m_pending = 0;
        m_openCancellation = OpenCancellation::None;
        m_cancelledOpenSession = 0;
        m_deferredQuery.reset();
        m_waiting.reset();
        m_admissionRetry.stop();
        if (!graceful) {
            m_faulted = true;
            m_result = {};
            m_total = 0;
            m_offset = 0;
            m_session.clear();
            m_message = explain("runtime_unavailable");
            m_ready = false;
            m_model.replace({});
            replaceDirectories({});
            clearContentPreview();
            m_detail.clear();
            m_preflightReport.clear();
            for (auto &task : m_tasks) {
                if (task.value("terminal").toBool())
                    continue;
                task.insert("terminal", true);
                task.insert("status", "failed");
                task.insert("error", explain("task_abandoned"));
                task.insert("outcomeUncertain", true);
                task.insert("cleanupWarning", true);
                task.insert("promptId", QString{});
                task.insert("passwordPromptId", QString{});
            }
            m_openAfterTasks.clear();
            m_createdArchivesToOpen.clear();
            emit tasksChanged();
        }
        QPointer<BrowserController> alive(this);
        emit changed();
        if (alive && m_closing)
            finishCloseWhenTransfersDrain();
    });
}
BrowserController::~BrowserController() {
    PreviewImages::remove(m_contentPreview.value("image").toString());
    if (m_export) {
        m_export->cancelled = true;
        m_export->changed.notify_all();
        m_export->finish({}, "runtime_closed");
    }
    if (m_exportCopy)
        m_exportCopy->finish("runtime_closed");
}

QString BrowserController::explain(const QString &code) {
    if (code.isEmpty())
        return {};
    const QHash<QString, QString> taskErrors = {
        {"move_same_directory", qtTrId("move.same")},
        {"move_selected_directory", qtTrId("move.selected")},
        {"task_abandoned", qtTrId("ui.cc72ffea1705")},
        {"integrity_error", qtTrId("ui.6adbf73c75b1")},
        {"output_io", qtTrId("ui.7735b66ae246")},
        {"disk_full", qtTrId("ui.8e2070ff3086")},
        {"unsafe_target", qtTrId("ui.123399c0fb8b")},
        {"target_changed", qtTrId("ui.8c5b6e8cc7b3")},
        {"output_conflict", qtTrId("ui.370322f1d979")},
        {"unsupported_filesystem", qtTrId("ui.74b624993d20")},
        {"unsupported_link", qtTrId("error.unsupportedLink")},
        {"unsupported_entry", qtTrId("ui.491bda01a7fa")},
        {"empty_input", qtTrId("error.emptyArchiveInput")},
        {"unsafe_input", qtTrId("ui.2c68fedd118c")},
        {"input_collision", qtTrId("ui.96d580ed8b8a")},
        {"unsafe_name", qtTrId("ui.c9aa9ad002f7")},
        {"invalid_password", qtTrId("ui.a35179e5025f")},
        {"wrong_password", qtTrId("password.attemptLimit")},
        {"password_or_integrity", qtTrId("password.ambiguous")},
        {"password_required", qtTrId("password.waiting")},
        {"invalid_creation_options", qtTrId("ui.674e51a0a214")},
        {"split_overwrite_not_supported", qtTrId("ui.930e2ed5950e")},
        {"ambiguous_name", qtTrId("ui.1f3e5278a2e1")},
        {"high_ratio", qtTrId("ui.083aa56b5fb0")},
        {"file_quota", qtTrId("ui.026f5e2a7952")},
        {"task_quota", qtTrId("ui.08a85911314e")},
        {"protected_source", qtTrId("ui.f740b2508391")},
        {"task_busy", qtTrId("ui.159a5a9a826f")},
        {"cleanup_error", qtTrId("ui.3ca05aed62cc")},
        {"invalid_selection", qtTrId("ui.0541a7a33cf3")},
        {"unsafe_folder_name", qtTrId("ui.a811dae528ce")},
        {"rename_limit", qtTrId("ui.432c56670e32")}};
    if (taskErrors.contains(code))
        return taskErrors.value(code);
    if (code == "runtime_unavailable")
        return qtTrId("ui.4284b709fb50");
    if (code == "invalid_request")
        return qtTrId("ui.14387362d973");
    if (code == "identifier_exhausted")
        return qtTrId("ui.2b79855e1be0");
    if (code == "busy")
        return qtTrId("ui.9a27a22a22ff");
    if (code == "stale_session")
        return qtTrId("ui.a27358be0e89");
    if (code == "cancelled")
        return qtTrId("ui.a3562bc1ea59");
    if (code == "resource_limit")
        return qtTrId("ui.faf573ec7fbf");
    if (code == "unsupported_feature")
        return qtTrId("ui.48a1b719a41d");
    if (code == "stale_query")
        return qtTrId("ui.f4e564347a6a");
    if (code == "source_changed")
        return qtTrId("ui.b0783c10a988");
    if (code == "source_io")
        return qtTrId("ui.14df9062a079");
    if (code == "missing_volume")
        return qtTrId("error.missingVolume");
    if (code == "invalid_path")
        return qtTrId("ui.bcd73815bcd5");
    return qtTrId("ui.cd2a579ed2df");
}

void BrowserController::chooseArchive(QWindow *owner) {
    if (m_busy || !runtimeAvailable())
        return;
    QFileDialog dialog(nullptr, systemDialogText("ui.1b59353f1874"), {},
                       systemDialogText("ui.8f69a61727d2"));
    dialog.setFileMode(QFileDialog::ExistingFiles);
    if (execOwnedFileDialog(dialog, owner) && !dialog.selectedFiles().isEmpty())
        for (const auto &path : dialog.selectedFiles())
            requestOpen(path);
}
bool BrowserController::canOpenDroppedUrls(const QList<QUrl> &urls) const {
    if (m_busy || !runtimeAvailable() || urls.isEmpty() || urls.size() > 128 ||
        (!openIntent && urls.size() != 1))
        return false;
    for (const auto &url : urls) {
        if (!url.isLocalFile())
            return false;
        const QFileInfo file(url.toLocalFile());
        const auto name = file.fileName().toLower();
        static const QRegularExpression splitSuffix(
            QStringLiteral("(?:\\.(?:zip|7z)\\.[0-9]{3,4}|\\.z[0-9]{2,4}|\\.r[0-9]{2,4})$"));
        if (!file.isFile() ||
            !(name.endsWith(".zip") || name.endsWith(".tar") || name.endsWith(".tar.gz") ||
              name.endsWith(".tgz") || name.endsWith(".7z") || name.endsWith(".rar") ||
              name.endsWith(".cab") || name.endsWith(".iso") || name.endsWith(".udf") ||
              splitSuffix.match(name).hasMatch()))
            return false;
    }
    return true;
}
bool BrowserController::openDroppedUrls(const QList<QUrl> &urls) {
    if (m_busy || !runtimeAvailable())
        return false;
    if (!canOpenDroppedUrls(urls)) {
        m_message = qtTrId("ui.e827041f3805");
        emit changed();
        return false;
    }
    // This is an input hint only. The existing runtime probes content and applies
    // admission limits; a matching extension never asserts that an archive is valid.
    for (const auto &url : urls)
        requestOpen(url.toLocalFile());
    return true;
}
void BrowserController::openPath(const QString &path) {
    if (m_busy || m_closing || path.isEmpty())
        return;
    if (QString::fromUtf8(path.toUtf8()) != path || path.contains(QChar::Null)) {
        m_message = explain("invalid_path");
        emit changed();
        return;
    }
    // An explicit open can satisfy a queued creation result before its idle callback.
    // Do not reopen it later and invalidate a newly prepared extraction intent.
    const auto openKey = [](const QString &value) {
        const QFileInfo file(value);
        const auto canonical = file.canonicalFilePath();
        auto key = QDir::cleanPath(
            QDir::fromNativeSeparators(canonical.isEmpty() ? file.absoluteFilePath() : canonical));
#ifdef Q_OS_WIN
        key = key.toCaseFolded();
#endif
        return key;
    };
    // This comparison only deduplicates UI intents; it grants no output authority.
    const auto key = openKey(path);
    m_createdArchivesToOpen.removeIf([&](const QString &queued) { return openKey(queued) == key; });
    invalidateTarget();
    m_restoreAfterUpdate = false;
    m_committedQuery = {};
    m_directory.clear();
    m_location.clear();
    m_search.clear();
    m_sort = 0;
    m_navigationLimited = false;
    clearContentPreview();
    m_detail.clear();
    m_title = QFileInfo(path).fileName();
    m_sourcePath = path;
    m_ready = false;
    m_openPasswordPrompt.clear();
    emit tasksChanged();
    m_total = 0;
    m_offset = 0;
    m_session.clear();
    m_model.replace({});
    replaceDirectories({});
    start(path, 0);
}
void BrowserController::discardBrowsing() {
    if (m_browsingDetached)
        return;
    m_browsingDetached = true;
    m_deferredQuery.reset();
    invalidateTarget();
    invalidateCreationTarget();
    m_creationPreviewPending = false;
    m_creationPreviewPaths.clear();
    m_creationInputs.clear();
    const bool browseRequest = m_busy && !archiveMutationPending() && !m_result.extraction &&
                               !m_result.creating && !m_result.checking && !m_result.targetCheck &&
                               !m_result.inputPreview && !m_result.exportManifest;
    if (browseRequest)
        cancel();
    m_committedQuery = {};
    restoreQuery();
    m_updateLocation.clear();
    m_updateSort = 0;
    m_restoreAfterUpdate = false;
    m_ready = false;
    m_total = m_archiveTotal = m_offset = 0;
    m_navigationLimited = false;
    m_selectionAnchor.clear();
    m_selectionAnchorRow = -1;
    m_model.replace({});
    replaceDirectories({});
    clearContentPreview();
    m_detail.clear();
    m_preflightReport.clear();
    m_result.rows.clear();
    m_result.directories.clear();
    emit creationInputsChanged();
    emit changed();
}
void BrowserController::selectRow(int row) {
    if ((m_busy && !m_result.pageFetch && !m_result.contentPreview) || !m_ready)
        return;
    const auto *item = m_model.rowAt(row);
    if (!item)
        return;
    m_selectionAnchor = item->id;
    m_selectionAnchorRow = row;
    m_model.select(row);
    showRowDetail(row);
    m_preflightReport.clear();
    emit changed();
}
void BrowserController::showRowDetail(int row) {
    m_deferredDetailRow = -1;
    clearContentPreview();
    const auto *item = m_model.rowAt(row);
    if (!item || m_model.selectedCount() == 0) {
        m_detail.clear();
        return;
    }
    m_detail = {{"name", item->cells[0]},       {"size", item->cells[1]},
                {"packed", item->cells[2]},     {"type", item->cells[3]},
                {"modified", item->cells[4]},   {"method", item->method},
                {"status", item->status},       {"attention", item->attention},
                {"directory", item->directory}, {"id", item->id}};
}
void BrowserController::updateSelectionSummary() const {
    if (m_summaryRevision == m_model.selectionRevision())
        return;
    m_summaryRevision = m_model.selectionRevision();
    m_summaryPhysical = true;
    bool summaryKnown =
        !m_model.paged() || m_model.physicalSelectedIds().size() == m_model.selectedCount();
    if (!summaryKnown) {
        m_summaryPhysical = false;
        m_summaryBytes = QStringLiteral("—");
        return;
    }
    std::vector<std::uint64_t> sizes;
    sizes.reserve(static_cast<std::size_t>(m_model.selectedCount()));
    for (int row : m_model.cachedRows()) {
        if (!m_model.isSelected(row))
            continue;
        const auto *item = m_model.rowAt(row);
        if (!item->physical) {
            m_summaryPhysical = false;
            m_summaryBytes = QStringLiteral("—");
            return;
        }
        if (!item->sizeKnown) {
            summaryKnown = false;
            continue;
        }
        sizes.push_back(item->cells[1].toULongLong());
    }
    if (!summaryKnown) {
        m_summaryBytes = QStringLiteral("—");
        return;
    }
    const auto sum = xdzip::reader::sum_declared_sizes(
        rust::Slice<const std::uint64_t>(sizes.data(), sizes.size()));
    m_summaryBytes = QString::fromUtf8(sum.data(), static_cast<qsizetype>(sum.size()));
}
bool BrowserController::canPreflight() const {
    if (m_model.selectedCount() == 0 || m_model.selectedCount() > 128)
        return false;
    updateSelectionSummary();
    return m_summaryPhysical;
}
QString BrowserController::selectedBytes() const {
    updateSelectionSummary();
    return m_summaryBytes;
}
void BrowserController::closeArchive() {
    if (m_busy || !m_ready || !runtimeAvailable())
        return;
    xdzip::reader::Command command{};
    command.kind = xdzip::reader::CommandKind::CloseSession;
    command.session = m_session.toULongLong();
    submit(std::move(command));
}
void BrowserController::selectOnlyRow(int row) {
    if ((m_busy && !m_result.pageFetch && !m_result.contentPreview) || !m_ready ||
        !m_model.rowAt(row))
        return;
    m_selectionAnchor = m_model.rowAt(row)->id;
    m_selectionAnchorRow = row;
    m_model.selectOnly(row);
    showRowDetail(row);
    m_preflightReport.clear();
    emit changed();
}
void BrowserController::selectRangeTo(int row) {
    if ((m_busy && !m_result.pageFetch && !m_result.contentPreview) || !m_ready ||
        !m_model.rowAt(row))
        return;
    int anchor = m_model.paged() && m_selectionAnchorRow >= 0 ? m_selectionAnchorRow : row;
    bool foundAnchor = m_model.paged() && m_selectionAnchorRow >= 0;
    for (int index : m_model.cachedRows())
        if (!foundAnchor && m_model.rowAt(index)->id == m_selectionAnchor) {
            anchor = index;
            foundAnchor = true;
            break;
        }
    if (!foundAnchor)
        m_selectionAnchor = m_model.rowAt(row)->id;
    m_model.selectRange(anchor, row);
    showRowDetail(row);
    m_preflightReport.clear();
    emit changed();
}
void BrowserController::selectForContext(int row) {
    if ((m_busy && !m_result.pageFetch && !m_result.contentPreview) || !m_ready ||
        !m_model.rowAt(row))
        return;
    if (!m_model.isSelected(row)) {
        selectOnlyRow(row);
        return;
    }
    // Context menus preserve an existing batch selection; the clicked item owns
    // the detail focus, while the batch commands retain the complete ID set.
    showRowDetail(row);
    emit changed();
}
void BrowserController::selectAll(bool selected) {
    if ((m_busy && !m_result.pageFetch && !m_result.contentPreview) || !m_ready)
        return;
    m_model.selectAll(selected);
    m_selectionAnchor.clear();
    m_selectionAnchorRow = -1;
    m_deferredDetailRow = -1;
    if (selected && m_model.rowCount() > 0) {
        const int last = m_model.rowCount() - 1;
        showRowDetail(last);
        if (m_model.paged() && !m_model.rowAt(last)) {
            m_deferredDetailRow = last;
            m_desiredPage = last;
            QTimer::singleShot(0, this, &BrowserController::drainPage);
        }
    } else {
        clearContentPreview();
        m_detail.clear();
    }
    m_preflightReport.clear();
    emit changed();
}
void BrowserController::copySelection(bool includeArchivePath) {
    if (m_busy || !m_ready)
        return;
    QStringList names;
    for (int row : m_model.cachedRows()) {
        if (!m_model.isSelected(row))
            continue;
        const auto *item = m_model.rowAt(row);
        const auto display = item->path.isEmpty() ? item->cells[0] : item->path;
        QString name = display;
        while (name.endsWith('/'))
            name.chop(1);
        names.append(includeArchivePath ? display : name.section('/', -1));
    }
    if (m_model.paged() && names.size() != m_model.selectedCount()) {
        m_message = explain("resource_limit");
        emit changed();
        return;
    }
    if (!names.empty())
        QGuiApplication::clipboard()->setText(names.join('\n'));
}
void BrowserController::openContextEntry(const QString &session, const QString &entry) {
    if (session != m_session || m_busy || !m_ready)
        return;
    for (int row : m_model.cachedRows()) {
        const auto *item = m_model.rowAt(row);
        const auto identity = item->navigationId.isEmpty() ? item->id : item->navigationId;
        if (identity == entry) {
            enterDirectory(row);
            return;
        }
    }
}
void BrowserController::enterDirectory(int row) {
    if (m_updateActive || (m_busy && m_result.updating))
        return;
    if (m_busy || !m_ready)
        return;
    const auto *item = m_model.rowAt(row);
    if (!item)
        return;
    if (!item->navigable) {
        if (!item->physical || item->directory || taskStarting() || m_closing ||
            !runtimeAvailable())
            return;
        const auto cache =
            m_temporaryCacheOverride.isEmpty()
                ? QStandardPaths::writableLocation(QStandardPaths::CacheLocation) + "/opened-files"
                : m_temporaryCacheOverride;
        if (cache.isEmpty() || !QDir().mkpath(cache)) {
            m_message = explain("output_io");
            emit changed();
            return;
        }
        xdzip::reader::Command command{};
        command.kind = xdzip::reader::CommandKind::OpenEntry;
        command.session = m_session.toULongLong();
        command.offset = item->id.toULongLong();
        command.path = rust::String(cache.toStdString());
        m_extractionIntent.insert("archive", m_title);
        m_pendingOpenAfter = false;
        submit(std::move(command));
        return;
    }
    m_directory = item->navigationId.isEmpty() ? m_directoryPaths.value(item->path, item->id)
                                               : item->navigationId;
    m_location = item->path;
    start({}, 0);
}
QVariantList BrowserController::breadcrumbs() const {
    QVariantList result;
    auto *item = m_directoryItems.value(m_directory, nullptr);
    // The complete index is bounded to 64 levels. These are presentation names,
    // not host filesystem paths or additional archive queries.
    while (item) {
        result.prepend(
            QVariantMap{{"id", item->data(Qt::UserRole)},
                        {"label", item->text()},
                        {"current", item->data(Qt::UserRole).toString() == m_directory}});
        item = item->parent();
    }
    return result;
}
void BrowserController::replaceDirectories(const QList<DirectoryRow> &rows) {
    m_directoryItems.clear();
    m_directoryPaths.clear();
    m_directories.clear();
    m_directories.setColumnCount(1);
    m_navigationWidth = 160;
    QHash<QString, int> depths;
    // Rust emits parents before children, at most 32,768 nodes and 64 levels.
    for (const auto &row : rows) {
        auto *parent = row.parent.isEmpty() ? m_directories.invisibleRootItem()
                                            : m_directoryItems.value(row.parent, nullptr);
        if (!parent)
            continue;
        auto *item = new QStandardItem(row.name);
        item->setEditable(false);
        item->setData(row.id, Qt::UserRole);
        item->setData(row.count, Qt::UserRole + 1);
        item->setData(row.path, Qt::UserRole + 2);
        const int depth = depths.value(row.parent, -1) + 1;
        // Bind indentation to stable model identity, not TreeView's recycled row state.
        item->setData(depth, Qt::UserRole + 5);
        parent->appendRow(item);
        m_directoryItems.insert(row.id, item);
        m_directoryPaths.insert(row.path, row.id);
        depths.insert(row.id, depth);
        m_navigationWidth = qMax(m_navigationWidth, 160 + depth * 16);
    }
    // Presentation flags are derived only after the complete bounded tree exists.
    // They encode guide continuity, not format or output-path logic.
    for (auto *item : m_directoryItems) {
        const auto *parent = item->parent() ? item->parent() : m_directories.invisibleRootItem();
        item->setData(item->row() == parent->rowCount() - 1, Qt::UserRole + 4);
        QVariantList continuations;
        auto *ancestor = item->parent();
        while (ancestor && ancestor->parent()) {
            continuations.prepend(ancestor->row() < ancestor->parent()->rowCount() - 1);
            ancestor = ancestor->parent();
        }
        item->setData(continuations, Qt::UserRole + 3);
    }
}
QStringList BrowserController::directoryAncestors(const QString &id) const {
    QStringList ancestors;
    const auto *item = m_directoryItems.value(id, nullptr);
    while (item && item->parent()) {
        item = item->parent();
        ancestors.append(item->data(Qt::UserRole).toString());
    }
    return ancestors;
}
void BrowserController::navigateDirectory(const QString &id) {
    if (m_updateActive || (m_busy && m_result.updating))
        return;
    if (!m_ready || m_closing || m_result.preflight || m_result.extraction || m_result.targetCheck)
        return;
    const auto *item = m_directoryItems.value(id, nullptr);
    if (!item)
        return;
    m_directory = id;
    m_location = item->data(Qt::UserRole + 2).toString();
    start({}, 0);
}
void BrowserController::sortByColumn(int column) {
    if (m_updateActive || (m_busy && m_result.updating))
        return;
    if (!m_ready || m_closing || m_result.preflight || m_result.extraction ||
        m_result.targetCheck || column < -1 || column > 4)
        return;
    const int ascending = column < 3 ? column * 2 + 1 : column == 3 ? 9 : 11;
    m_sort = column == -1 ? 0 : (m_sort == ascending ? ascending + 1 : ascending);
    start({}, 0);
}
void BrowserController::showRoot() {
    if (m_updateActive || (m_busy && m_result.updating))
        return;
    if (!m_ready || m_closing || m_result.preflight || m_result.extraction || m_result.targetCheck)
        return;
    m_directory.clear();
    m_location.clear();
    start({}, 0);
}
void BrowserController::search(const QString &text) {
    if (m_updateActive || (m_busy && m_result.updating))
        return;
    if (!m_ready || m_closing || m_result.preflight || m_result.extraction ||
        m_result.targetCheck || text.toUtf8().size() > 1024)
        return;
    m_search = text;
    start({}, 0);
}
QVariantList BrowserController::tasks() const {
    QVariantList result;
    for (auto it = m_tasks.cend(); it != m_tasks.cbegin();) {
        --it;
        if (m_hiddenFinishedTasks.contains(it.key()) && it.value().value("terminal").toBool())
            continue;
        auto task = it.value();
        task.insert("cancelRequested",
                    !task.value("terminal").toBool() && m_cancellingTasks.contains(it.key()));
        result.append(task);
    }
    return result;
}
bool BrowserController::hasFinishedTasks() const {
    for (auto it = m_tasks.cbegin(); it != m_tasks.cend(); ++it)
        if (it.value().value("terminal").toBool() && !m_hiddenFinishedTasks.contains(it.key()))
            return true;
    return false;
}
void BrowserController::clearFinishedTasks() {
    bool changed = false;
    for (auto it = m_tasks.cbegin(); it != m_tasks.cend(); ++it)
        if (it.value().value("terminal").toBool() && !m_hiddenFinishedTasks.contains(it.key())) {
            m_hiddenFinishedTasks.insert(it.key());
            changed = true;
        }
    // Hide the history projection only. Backend state, pending acknowledgements,
    // passwords, cancellation and file lifetimes retain their original owner.
    if (changed)
        emit tasksChanged();
}
int BrowserController::activeTaskCount() const {
    const auto pendingTask =
        m_result.taskId.isEmpty() ? m_result.observedTask : m_result.taskId.toULongLong();
    const bool preparing = m_busy &&
                           (m_result.extraction || m_result.creating || m_result.checking) &&
                           (!pendingTask || !m_tasks.contains(pendingTask));
    const auto count = static_cast<int>(std::count_if(
                           m_tasks.cbegin(), m_tasks.cend(),
                           [](const auto &v) { return !v.value("terminal").toBool(); })) +
                       (preparing ? 1 : 0);
    return count == 0 && m_export && m_export->inOperation.load() ? 1 : count;
}
QVariantMap BrowserController::passwordPrompt() const {
    if (!runtimeAvailable())
        return {};
    if (!m_openPasswordPrompt.isEmpty())
        return m_openPasswordPrompt;
    for (auto it = m_tasks.cbegin(); it != m_tasks.cend(); ++it) {
        const auto &task = it.value();
        if (!m_cancellingTasks.contains(it.key()) && task.value("status") == "waiting_password" &&
            !task.value("passwordPromptId").toString().isEmpty())
            return {{"taskId", QString::number(it.key())},
                    {"promptId", task.value("passwordPromptId")},
                    {"name", task.value("passwordPromptName")},
                    {"kind", task.value("kind")},
                    {"retry", task.value("passwordRetry")}};
    }
    return {};
}
void BrowserController::prepareExtraction(bool all, const QStringList &selectors) {
    if (!m_ready || m_busy || !runtimeAvailable())
        return;
    m_extractionSelectors = selectors.isEmpty() ? m_model.selectedIds() : selectors;
    if (!all && m_extractionSelectors.isEmpty())
        return;
    m_extractionDirectory = selectors.isEmpty() ? m_directory : QString{};
    if (!all && std::any_of(m_extractionSelectors.cbegin(), m_extractionSelectors.cend(),
                            [](const QString &id) { return id.startsWith('q'); })) {
        xdzip::reader::Command command{};
        command.kind = xdzip::reader::CommandKind::FreezeSelection;
        command.session = m_session.toULongLong();
        for (const auto &id : m_extractionSelectors)
            command.selectors.push_back(rust::String(id.toStdString()));
        submit(std::move(command));
        return;
    }
    presentExtractionIntent(all);
}
void BrowserController::presentExtractionIntent(bool all) {
    QVariantList samples;
    for (int i : m_model.cachedRows()) {
        if (samples.size() >= 3)
            break;
        const auto *row = m_model.rowAt(i);
        if (all || m_extractionSelectors.contains(row->id))
            samples.append(
                QVariantMap{{"name", row->cells[0]},
                            {"size", row->directory ? QStringLiteral("—") : row->cells[1]},
                            {"directory", row->directory}});
    }
    m_extractionIntent = {{"session", m_session},
                          {"archive", m_title},
                          {"folder", m_archiveFolder},
                          {"all", all},
                          {"selectedCount", m_extractionSelectors.size()},
                          {"archiveCount", m_archiveTotal},
                          {"samples", samples},
                          {"target", QFileInfo(m_sourcePath).absolutePath()},
                          {"error", QString{}}};
    invalidateTarget();
    emit extractionChanged();
    emit extractionReady();
}
QString BrowserController::chooseTarget(const QString &initial, QWindow *owner) {
    QFileDialog dialog(nullptr, systemDialogText("ui.3fb3d26f0ffd"), initial);
    dialog.setFileMode(QFileDialog::Directory);
    dialog.setOptions(QFileDialog::ShowDirsOnly | QFileDialog::DontResolveSymlinks);
    return execOwnedFileDialog(dialog, owner) && !dialog.selectedFiles().isEmpty()
               ? dialog.selectedFiles().first()
               : QString{};
}
QStringList BrowserController::chooseCreationInputs(bool directories, QWindow *owner) {
    QFileDialog dialog(nullptr, directories ? systemDialogText("ui.45da2d9c85d1")
                                            : systemDialogText("ui.ad9daebe00dc"));
    dialog.setFileMode(directories ? QFileDialog::Directory : QFileDialog::ExistingFiles);
    dialog.setOptions(QFileDialog::DontResolveSymlinks |
                      (directories ? QFileDialog::ShowDirsOnly : QFileDialog::Options{}));
    return execOwnedFileDialog(dialog, owner) ? dialog.selectedFiles() : QStringList{};
}
QString BrowserController::defaultTargetDirectory() const {
    const auto path = QStandardPaths::writableLocation(QStandardPaths::DownloadLocation);
    return !path.isEmpty() && QDir(path).exists() ? path : QDir::homePath();
}
void BrowserController::inspectCreationInputs(const QStringList &paths) {
    const auto bounded = paths.mid(0, 256);
    if (bounded == m_creationPreviewPaths || m_closing || !runtimeAvailable())
        return;
    m_creationPreviewPaths = bounded;
    m_creationInputs.clear();
    m_creationPreviewPending =
        !bounded.isEmpty() &&
        std::all_of(bounded.cbegin(), bounded.cend(), [](const QString &path) {
            return !path.contains(QChar::Null) && QString::fromUtf8(path.toUtf8()) == path;
        });
    emit creationInputsChanged();
    QTimer::singleShot(0, this, &BrowserController::drainCreationUiWhenIdle);
}
void BrowserController::drainCreationUiWhenIdle() {
    if (m_busy || m_pending || m_closing || !runtimeAvailable())
        return;
    if (!m_createdArchivesToOpen.isEmpty()) {
        requestOpen(m_createdArchivesToOpen.takeFirst());
    } else if (m_pendingCreationTarget || m_pendingTarget) {
        const bool creation =
            !m_pendingTarget || (m_pendingCreationTarget &&
                                 m_pendingCreationTarget->sequence < m_pendingTarget->sequence);
        auto &pending = creation ? m_pendingCreationTarget : m_pendingTarget;
        m_activeTarget = std::exchange(pending, std::nullopt);
        if (!creation && (!m_ready || m_activeTarget->session != m_session ||
                          m_extractionIntent.value("session").toString() != m_session)) {
            finishTargetCheck(QStringLiteral("stale_session"));
            QTimer::singleShot(0, this, &BrowserController::drainCreationUiWhenIdle);
            return;
        }
        xdzip::reader::Command command{};
        command.kind = xdzip::reader::CommandKind::CheckTarget;
        command.session = creation ? 0 : m_activeTarget->session.toULongLong();
        command.path = rust::String(m_activeTarget->path.toStdString());
        submit(std::move(command));
    } else if (std::exchange(m_creationPreviewPending, false)) {
        xdzip::reader::Command command{};
        command.kind = xdzip::reader::CommandKind::InspectInputs;
        for (const auto &path : m_creationPreviewPaths)
            command.inputs.push_back(rust::String(path.toStdString()));
        submit(std::move(command));
        m_result.previewPaths = m_creationPreviewPaths;
    }
}
void BrowserController::checkCreationTarget(const QString &path) {
    queueTargetCheck(path, true);
}
void BrowserController::startCreation(const QStringList &inputs, const QString &target,
                                      const QString &name, int method, int level, bool openAfter,
                                      qulonglong volumeBytes, const QString &password, bool solid,
                                      bool encryptHeaders) {
    if (m_busy || !runtimeAvailable() || m_closing)
        return;
    const auto validText = [](const QString &value) {
        return !value.contains(QChar::Null) && QString::fromUtf8(value.toUtf8()) == value;
    };
    const bool sevenZip = name.endsWith(QStringLiteral(".7z"), Qt::CaseInsensitive);
    if ((method != 0 && method != (sevenZip ? 32 : 8)) ||
        (level != 1 && level != 6 && level != 9 &&
         !(name.endsWith(QStringLiteral(".tar.gz"), Qt::CaseInsensitive) && level == 0)) ||
        !validText(target) || !validText(name) ||
        std::any_of(inputs.cbegin(), inputs.cend(),
                    [&](const QString &path) { return !validText(path); })) {
        m_creationError = explain("invalid_path");
        emit extractionChanged();
        return;
    }
    if (volumeBytes > 24ULL * 1024 * 1024 * 1024 || !validText(password) ||
        password.toUtf8().size() > 128 || (!sevenZip && (solid || encryptHeaders)) ||
        (encryptHeaders && password.isEmpty()) ||
        std::any_of(password.cbegin(), password.cend(),
                    [](QChar c) { return c.category() == QChar::Other_Control; })) {
        m_creationError = explain("invalid_creation_options");
        emit extractionChanged();
        return;
    }
    xdzip::reader::Command command{};
    command.kind = xdzip::reader::CommandKind::Create;
    command.path = rust::String(QDir::cleanPath(target).toStdString());
    command.text = rust::String(name.toStdString());
    for (const auto &path : inputs)
        command.inputs.push_back(rust::String(QDir::cleanPath(path).toStdString()));
    command.method = static_cast<std::uint16_t>(method);
    command.level = static_cast<std::uint8_t>(level);
    command.volume_bytes = volumeBytes;
    command.encrypt = !password.isEmpty();
    command.solid = solid;
    command.encrypt_headers = encryptHeaders;
    command.password = rust::String(password.toUtf8().toStdString());
    command.conflict = 0;
    m_creationError.clear();
    // Creation auto-opens in this application; the legacy folder option is ignored.
    Q_UNUSED(openAfter)
    m_pendingOpenAfter = false;
    submit(std::move(command));
}
bool BrowserController::targetMatches(const QString &path) const {
    return m_targetCheck.value("valid").toBool() &&
           m_targetCheck.value("session").toString() == m_session &&
           m_extractionIntent.value("session").toString() == m_session &&
           QDir::cleanPath(path) == m_targetCheck.value("path");
}
bool BrowserController::creationTargetMatches(const QString &path) const {
    return m_creationTargetCheck.value("valid").toBool() &&
           QDir::cleanPath(path) == m_creationTargetCheck.value("path");
}
void BrowserController::invalidateTarget() {
    ++m_targetRevision;
    m_pendingTarget.reset();
    m_targetCheck.clear();
    emit extractionChanged();
}
void BrowserController::invalidateCreationTarget() {
    ++m_creationTargetRevision;
    m_pendingCreationTarget.reset();
    m_creationTargetCheck.clear();
    emit extractionChanged();
}
void BrowserController::checkTarget(const QString &path) {
    queueTargetCheck(path, false);
}
void BrowserController::queueTargetCheck(const QString &path, bool creation) {
    if (!runtimeAvailable() ||
        (!creation && (!m_ready || m_extractionIntent.value("session").toString() != m_session)))
        return;
    auto &revision = creation ? m_creationTargetRevision : m_targetRevision;
    auto &pending = creation ? m_pendingCreationTarget : m_pendingTarget;
    auto &state = creation ? m_creationTargetCheck : m_targetCheck;
    ++revision;
    pending.reset();
    if (path.isEmpty() || QString::fromUtf8(path.toUtf8()) != path || path.contains(QChar::Null)) {
        state = {
            {"valid", false}, {"errorCode", "invalid_path"}, {"error", explain("invalid_path")}};
    } else {
        TargetRequest request{QDir::cleanPath(path), creation ? QString{} : m_session, revision,
                              ++m_targetSequence, creation};
        pending = request;
        state = {{"path", request.path}, {"valid", false}, {"checking", true}};
    }
    emit extractionChanged();
    drainCreationUiWhenIdle();
}
void BrowserController::finishTargetCheck(const QString &error, quint64 leftovers) {
    const auto request = std::exchange(m_activeTarget, std::nullopt);
    if (!request)
        return;
    const auto currentRevision = request->creation ? m_creationTargetRevision : m_targetRevision;
    if (request->revision != currentRevision)
        return;
    auto &state = request->creation ? m_creationTargetCheck : m_targetCheck;
    if (!request->creation && (request->session != m_session ||
                               m_extractionIntent.value("session").toString() != m_session)) {
        state = {
            {"valid", false}, {"errorCode", "stale_session"}, {"error", explain("stale_session")}};
        return;
    }
    state = {{"path", request->path},
             {"session", request->session},
             {"valid", error.isEmpty()},
             {"errorCode", error},
             {"error", error.isEmpty() ? QString{} : explain(error)},
             {"leftovers", QString::number(leftovers)}};
}
void BrowserController::deferActiveTarget() {
    const auto request = std::exchange(m_activeTarget, std::nullopt);
    if (!request)
        return;
    const auto revision = request->creation ? m_creationTargetRevision : m_targetRevision;
    auto &pending = request->creation ? m_pendingCreationTarget : m_pendingTarget;
    if (request->revision == revision && !pending)
        pending = *request;
}
void BrowserController::startExtraction(const QString &target, bool all, bool createFolder,
                                        int conflict, bool allowHighRatio, bool openAfter,
                                        int shellLayout) {
    if (m_busy || !m_ready || !runtimeAvailable() ||
        m_extractionIntent.value("session") != m_session || !targetMatches(target) ||
        conflict < 0 || conflict > 3 || shellLayout < 0 || shellLayout > 2)
        return;
    xdzip::reader::Command command{};
    command.kind = xdzip::reader::CommandKind::Extract;
    command.session = m_session.toULongLong();
    command.path = rust::String(target.toStdString());
    command.directory = rust::String(m_extractionDirectory.toStdString());
    if (!all)
        for (const auto &id : m_extractionSelectors)
            command.selectors.push_back(rust::String(id.toStdString()));
    command.all = all;
    command.create_folder = createFolder;
    command.shell_layout = static_cast<std::uint8_t>(shellLayout);
    command.conflict = static_cast<std::uint8_t>(conflict);
    command.allow_high_ratio = allowHighRatio;
    m_pendingOpenAfter = openAfter;
    submit(std::move(command));
}
void BrowserController::startArchiveTest() {
    if (m_busy || !m_ready || !runtimeAvailable())
        return;
    m_pendingOpenAfter = false;
    xdzip::reader::Command command{};
    command.kind = xdzip::reader::CommandKind::Check;
    command.session = m_session.toULongLong();
    submit(std::move(command));
}
void BrowserController::cancelTask(const QString &id) {
    if (id == QStringLiteral("preview")) {
        clearContentPreview();
        emit changed();
        return;
    }
    if (id == QStringLiteral("open")) {
        m_openPasswordPrompt.clear();
        emit tasksChanged();
        return;
    }
    bool valid = false;
    const auto task = id.toULongLong(&valid);
    if (!valid || !task || m_cancellingTasks.contains(task) ||
        m_tasks.value(task).value("terminal").toBool())
        return;
    if (m_connection->cancelTask(task)) {
        m_cancellingTasks.insert(task);
        const auto state = m_tasks.value(task);
        if ((state.value("session").toString() == m_session &&
             (state.value("kind") == "update" || state.value("kind") == "move")) ||
            (m_result.updating && (m_result.taskId == id || m_result.observedTask == task)))
            m_archiveCancellationRequested = true;
        emit tasksChanged();
        emit changed();
    }
}
bool BrowserController::decideTask(const QString &id, const QString &prompt, int policy, bool all) {
    const auto task = m_tasks.constFind(id.toULongLong());
    return runtimeAvailable() && task != m_tasks.cend() &&
           !m_cancellingTasks.contains(task.key()) && task->value("status") == "waiting" &&
           !prompt.isEmpty() && task->value("promptId").toString() == prompt && policy >= 1 &&
           policy <= 3 &&
           m_connection->decideTask(id.toULongLong(), prompt.toULongLong(),
                                    static_cast<quint8>(policy), all);
}
bool BrowserController::submitPassword(const QString &id, const QString &prompt,
                                       const QString &password) {
    if (!runtimeAvailable() || password.size() > 1024)
        return false;
    if (id == QStringLiteral("preview")) {
        if (m_busy || !m_previewEntry || m_openPasswordPrompt.value("taskId") != "preview" ||
            prompt.isEmpty() || m_openPasswordPrompt.value("promptId") != prompt)
            return false;
        const quint64 entry = m_previewEntry;
        auto bytes = password.toUtf8();
        const auto clear = qScopeGuard([&] { bytes.fill('\0'); });
        if (bytes.size() > 1024 || QString::fromUtf8(bytes) != password)
            return false;
        m_openPasswordPrompt.clear();
        emit tasksChanged();
        startContentPreview(entry, password, true);
        return true;
    }
    if (id == QStringLiteral("open")) {
        if (m_busy || prompt.isEmpty() || m_openPasswordPrompt.value("promptId") != prompt)
            return false;
        auto bytes = password.toUtf8();
        const auto clear = qScopeGuard([&] { bytes.fill('\0'); });
        if (bytes.size() > 1024 || QString::fromUtf8(bytes) != password)
            return false;
        m_openPasswordPrompt.clear();
        emit tasksChanged();
        start(m_sourcePath, 0, password, true);
        return true;
    }
    const auto task = m_tasks.constFind(id.toULongLong());
    if (task == m_tasks.cend() || m_cancellingTasks.contains(task.key()) ||
        task->value("status") != "waiting_password" || prompt.isEmpty() ||
        task->value("passwordPromptId").toString() != prompt)
        return false;
    auto bytes = password.toUtf8();
    const auto clear = qScopeGuard([&] { bytes.fill('\0'); });
    if (bytes.size() > 1024 || QString::fromUtf8(bytes) != password)
        return false;
    return m_connection->submitPassword(id.toULongLong(), prompt.toULongLong(), std::move(bytes));
}
void BrowserController::openOutput(const QString &id) {
    const auto task = m_tasks.value(id.toULongLong());
    if (!task.value("openPath").toString().isEmpty()) {
        openTemporaryOutput(id.toULongLong());
        return;
    }
    if (!task.isEmpty())
        QDesktopServices::openUrl(
            QUrl::fromLocalFile(task.value("creating").toBool()
                                    ? QFileInfo(task.value("target").toString()).absolutePath()
                                    : task.value("target").toString()));
}
bool BrowserController::openTemporaryOutput(quint64 id) {
    auto it = m_tasks.find(id);
    if (it == m_tasks.end() || it->value("status") != "succeeded" ||
        it->value("committedFiles").toString() != "1" || it->value("openPath").toString().isEmpty())
        return false;
    const auto url = QUrl::fromLocalFile(it->value("openPath").toString());
    const QFileInfo file(url.toLocalFile());
    const bool available = file.isFile() && !file.isSymLink();
    const bool accepted =
        available && (m_fileOpener ? m_fileOpener(url) : QDesktopServices::openUrl(url));
    it->insert("openAccepted", accepted);
    const auto openError = accepted    ? QString{}
                           : available ? qtTrId("error.openExtractedFile")
                                       : qtTrId("error.temporaryFileExpired");
    it->insert("openError", openError);
    it->insert("error", openError);
    emit tasksChanged();
    return accepted;
}
void BrowserController::inspectSelection() {
    if (!canPreflight())
        return;
    if (m_busy || !m_ready || m_closing || m_model.selectedCount() == 0)
        return;
    xdzip::reader::Command command{};
    command.kind = xdzip::reader::CommandKind::Preflight;
    command.session = m_session.toULongLong();
    for (const auto &id : m_model.physicalSelectedIds())
        command.selected.push_back(id.toULongLong());
    submit(std::move(command), true);
}
void BrowserController::restoreQuery() {
    m_directory = m_committedQuery.directory;
    m_location = m_committedQuery.location;
    m_search = m_committedQuery.search;
    m_sort = m_committedQuery.sort;
}
void BrowserController::invalidateSnapshot() {
    m_archiveOperation.clear();
    m_ready = false;
    m_committedQuery = {};
    restoreQuery();
    m_total = m_archiveTotal = m_offset = 0;
    m_session.clear();
    m_navigationLimited = false;
    m_selectionAnchor.clear();
    m_selectionAnchorRow = -1;
    m_model.replace({});
    replaceDirectories({});
    clearContentPreview();
    m_detail.clear();
}
void BrowserController::cancel() {
    // Cleanup is part of the original cancellation and must not itself be cancelled.
    if (m_openCancellation != OpenCancellation::None)
        return;
    m_deferredQuery.reset();
    if (m_result.updating) {
        const auto task =
            m_result.taskId.isEmpty() ? m_result.observedTask : m_result.taskId.toULongLong();
        if (task) {
            cancelTask(QString::number(task));
            return;
        }
    }
    // An accepted edit outlives its registration request, including hidden conflict waits.
    if (m_updateActive)
        for (auto task = m_tasks.cbegin(); task != m_tasks.cend(); ++task)
            if (!task->value("terminal").toBool() &&
                task->value("session").toString() == m_session &&
                (task->value("kind") == "update" || task->value("kind") == "move")) {
                cancelTask(QString::number(task.key()));
                return;
            }
    if (m_waiting) {
        m_admissionRetry.stop();
        m_waiting.reset();
        if (m_result.opening) {
            beginCancelledOpenCleanup();
            return;
        }
        m_busy = false;
        m_archiveOperation.clear();
        restoreQuery();
        m_message = explain("cancelled");
        emit changed();
        return;
    }
    if (m_pending) {
        if (m_result.opening)
            m_openCancellation = OpenCancellation::AwaitingOpen;
        m_connection->cancel(m_pending);
    }
}
void BrowserController::beginCancelledOpenCleanup() {
    // Open can have installed a runtime session even when its payload was discarded.
    // Snapshot the connection after the opening request finishes, then close that session.
    m_openCancellation = OpenCancellation::Snapshot;
    m_cancelledOpenSession = 0;
    xdzip::reader::Command command{};
    command.kind = xdzip::reader::CommandKind::Snapshot;
    submit(std::move(command));
}
void BrowserController::finishCancelledOpenCleanup(const QString &error) {
    m_openCancellation = OpenCancellation::None;
    m_cancelledOpenSession = 0;
    m_result = {};
    m_hasPayload = false;
    m_busy = false;
    invalidateSnapshot();
    m_message = explain(error.isEmpty() ? QStringLiteral("cancelled") : error);
    QPointer<BrowserController> alive(this);
    emit changed();
    if (alive)
        QTimer::singleShot(0, this, &BrowserController::drainCreationUiWhenIdle);
}
bool BrowserController::requestClose() {
    setNativeDragActive(false);
    if (m_connection->stopped())
        return true;
    if (m_closing)
        return false;
    m_closing = true;
    if (m_export) {
        m_export->cancelled = true;
        m_export->changed.notify_all();
    }
    if (m_exportCopy)
        m_exportCopy->finish("runtime_closed");
    m_deferredQuery.reset();
    m_pendingCreationTarget.reset();
    m_pendingTarget.reset();
    m_openCancellation = OpenCancellation::None;
    m_cancelledOpenSession = 0;
    m_admissionRetry.stop();
    m_waiting.reset();
    m_preflightReport.clear();
    m_message = qtTrId("ui.0ae60dd37b14");
    m_connection->shutdown();
    emit tasksChanged(); // Hide password input immediately when shutdown starts.
    emit changed();
    return false;
}
void BrowserController::clearContentPreview() {
    PreviewImages::remove(m_contentPreview.value("image").toString());
    if (m_busy && m_result.contentPreview && m_pending)
        m_connection->cancel(m_pending);
    if (m_openPasswordPrompt.value("taskId") == "preview") {
        m_openPasswordPrompt.clear();
        emit tasksChanged();
    }
    m_previewEntry = 0;
    m_previewAttempts = 0;
    m_contentPreview.clear();
}
void BrowserController::requestPreview(const QString &id) {
    if (!canPreview() || id != m_detail.value("id").toString())
        return;
    clearContentPreview();
    startContentPreview(id.toULongLong());
}
void BrowserController::startContentPreview(quint64 entry, const QString &password, bool supplied) {
    if (!entry || m_busy || !m_ready || !runtimeAvailable())
        return;
    if (!supplied)
        m_previewAttempts = 0;
    m_previewEntry = entry;
    m_contentPreview = {{"state", "loading"}};
    xdzip::reader::Command command{};
    command.kind = xdzip::reader::CommandKind::Preview;
    command.session = m_session.toULongLong();
    command.offset = entry;
    command.all = supplied;
    auto secret = password.toUtf8();
    command.password = rust::String(secret.constData(), static_cast<std::size_t>(secret.size()));
    secret.fill('\0');
    submit(std::move(command));
}
void BrowserController::drainPage() {
    if (m_desiredPage < 0 || m_busy || !m_ready || !runtimeAvailable() || !m_model.paged())
        return;
    const int offset = std::exchange(m_desiredPage, -1);
    if (m_model.rowAt(offset))
        return;
    fetchPage(offset);
}
void BrowserController::fetchPage(int offset) {
    if (offset < 0 || offset >= m_model.rowCount())
        return;
    xdzip::reader::Command command{};
    command.kind = xdzip::reader::CommandKind::Browse;
    command.session = m_session.toULongLong();
    command.directory = rust::String(m_committedQuery.directory.toStdString());
    command.text = rust::String(m_committedQuery.search.toStdString());
    command.sort = static_cast<std::uint8_t>(m_committedQuery.sort);
    command.offset = static_cast<std::uint64_t>(offset);
    command.query = m_pageQuery;
    command.generation = m_pageGeneration;
    command.revision = m_pageRevision;
    submit(std::move(command));
    m_result.pageFetch = true;
    m_result.offset = offset;
    m_result.generation = m_pageGeneration;
    m_result.criteria = m_committedQuery;
    m_result.query = m_pageQuery;
    m_result.revision = m_pageRevision;
    m_result.session = m_session;
}
void BrowserController::start(const QString &path, int offset, const QString &password,
                              bool withPassword) {
    if (m_updateActive || (m_busy && m_result.updating))
        return;
    xdzip::reader::Command command{};
    command.kind = path.isEmpty() ? xdzip::reader::CommandKind::Browse
                   : withPassword ? xdzip::reader::CommandKind::OpenWithPassword
                                  : xdzip::reader::CommandKind::Open;
    if (withPassword) {
        auto secret = password.toUtf8();
        command.password =
            rust::String(secret.constData(), static_cast<std::size_t>(secret.size()));
        secret.fill('\0');
    }
    command.session = m_session.toULongLong();
    const auto bytes = path.toUtf8();
    command.path = rust::String(bytes.constData(), static_cast<std::size_t>(bytes.size()));
    command.directory = rust::String(m_directory.toStdString());
    command.text = rust::String(m_search.toStdString());
    command.offset = static_cast<std::uint64_t>(offset);
    command.sort = static_cast<std::uint8_t>(m_sort);
    if (!path.isEmpty())
        m_generation = 0;
    if (m_generation == std::numeric_limits<quint64>::max()) {
        m_message = explain("resource_limit");
        emit changed();
        return;
    }
    command.generation = ++m_generation;
    const QueryState criteria{m_directory, m_location, m_search, m_sort};
    if (m_busy) {
        if (m_result.targetCheck)
            deferActiveTarget();
        m_deferredQuery.emplace(DeferredQuery{std::move(command), criteria});
        if (m_waiting) {
            m_admissionRetry.stop();
            m_waiting.reset();
            auto latest = std::move(*m_deferredQuery);
            m_deferredQuery.reset();
            submit(std::move(latest.command));
            m_result.criteria = latest.criteria;
            m_result.generation = m_generation;
        } else if (m_pending)
            m_connection->cancel(m_pending);
        emit changed();
        return;
    }
    clearContentPreview();
    m_detail.clear();
    submit(std::move(command));
    m_result.criteria = criteria;
    m_result.generation = m_generation;
}
void BrowserController::submit(xdzip::reader::Command command, bool preflight) {
    // Only an explicit Extract request owns the optional folder-opening intent.
    // Native exports and temporary file opens must not inherit an earlier choice.
    if (command.kind != xdzip::reader::CommandKind::Extract)
        m_pendingOpenAfter = false;
    if (command.kind != xdzip::reader::CommandKind::CheckTarget &&
        command.kind != xdzip::reader::CommandKind::InspectInputs)
        m_preflightReport.clear();
    m_result = {};
    m_result.opening = command.kind == xdzip::reader::CommandKind::Open ||
                       command.kind == xdzip::reader::CommandKind::OpenWithPassword;
    m_result.preflight = preflight;
    m_result.freezeSelection = command.kind == xdzip::reader::CommandKind::FreezeSelection;
    m_result.contentPreview = command.kind == xdzip::reader::CommandKind::Preview;
    m_result.pageFetch =
        command.kind == xdzip::reader::CommandKind::Browse && command.query != 0 && m_model.paged();
    m_result.extraction = command.kind == xdzip::reader::CommandKind::Extract ||
                          command.kind == xdzip::reader::CommandKind::OpenEntry;
    m_result.updating = command.kind == xdzip::reader::CommandKind::Update ||
                        command.kind == xdzip::reader::CommandKind::Move;
    if (m_result.updating) {
        m_archiveOperation = command.kind == xdzip::reader::CommandKind::Move ? "move" : "add";
        m_archiveCancellationRequested = false;
    }
    m_result.exporting = command.kind == xdzip::reader::CommandKind::Export ||
                         command.kind == xdzip::reader::CommandKind::PublishExport;
    m_result.exportManifest = command.kind == xdzip::reader::CommandKind::ExportManifest;
    m_result.extraction |= m_result.exporting;
    m_result.creating = command.kind == xdzip::reader::CommandKind::Create || m_result.updating;
    m_result.checking = command.kind == xdzip::reader::CommandKind::Check;
    m_result.taskSession = command.session;
    m_result.inputPreview = command.kind == xdzip::reader::CommandKind::InspectInputs;
    m_result.targetCheck = command.kind == xdzip::reader::CommandKind::CheckTarget;
    m_result.closeSession = command.kind == xdzip::reader::CommandKind::CloseSession;
    m_hasPayload = false;
    m_busy = true;
    if (!m_result.targetCheck && !m_result.inputPreview)
        m_message = m_result.creating     ? qtTrId("ui.33495b6653c6")
                    : m_result.checking   ? qtTrId("integrity.starting")
                    : m_result.extraction ? qtTrId("ui.8cf09709f8f6")
                    : preflight           ? qtTrId("ui.d37d23d13df0")
                                          : qtTrId("ui.5126629edae6");
    m_waiting.emplace(std::move(command));
    m_admissionAttempts = 0;
    attemptSubmission();
}
void BrowserController::attemptSubmission() {
    if (!m_waiting)
        return;
    // One process-wide heavy operation, including preparation and cleanup. A
    // queued controller retains its bounded command and remains cancellable.
    static QPointer<BrowserController> heavyOwner;
    const bool heavy = m_waiting->kind == xdzip::reader::CommandKind::PublishExport ||
                       m_waiting->kind == xdzip::reader::CommandKind::Update ||
                       m_waiting->kind == xdzip::reader::CommandKind::Move ||
                       m_waiting->kind == xdzip::reader::CommandKind::Export ||
                       m_waiting->kind == xdzip::reader::CommandKind::Create ||
                       m_waiting->kind == xdzip::reader::CommandKind::OpenEntry ||
                       m_waiting->kind == xdzip::reader::CommandKind::Extract ||
                       m_waiting->kind == xdzip::reader::CommandKind::Check;
    if (heavy && heavyOwner && heavyOwner != this &&
        (heavyOwner->activeTaskCount() > 0 || heavyOwner->taskStarting())) {
        m_admissionRetry.start(100);
        emit changed();
        return;
    }
    if (heavy)
        heavyOwner = this;
    // A short try_lock collision is not a failed user operation. Keep only this bounded DTO
    // and retry on a later event-loop turn; persistent saturation still reports Busy.
    const auto result = m_connection->submit(*m_waiting);
    if (result.status == xdzip::reader::Admission::Busy && ++m_admissionAttempts < 32) {
        m_admissionRetry.start(1);
        emit changed();
        return;
    }
    m_waiting.reset();
    if (result.status != xdzip::reader::Admission::Accepted) {
        setNativeDragActive(false);
        m_busy = false;
        const auto code = result.status == xdzip::reader::Admission::Closed ? "runtime_unavailable"
                          : result.status == xdzip::reader::Admission::Invalid ? "invalid_request"
                          : result.status == xdzip::reader::Admission::Exhausted
                              ? "identifier_exhausted"
                              : "busy";
        if (m_result.targetCheck) {
            finishTargetCheck(QString::fromLatin1(code));
            m_result = {};
            emit extractionChanged();
            emit changed();
            QTimer::singleShot(0, this, &BrowserController::drainCreationUiWhenIdle);
            return;
        }
        if (m_result.inputPreview) {
            m_result = {};
            emit creationInputsChanged();
            emit changed();
            QTimer::singleShot(0, this, &BrowserController::drainCreationUiWhenIdle);
            return;
        }
        if (m_openCancellation == OpenCancellation::Snapshot ||
            m_openCancellation == OpenCancellation::Closing) {
            finishCancelledOpenCleanup(QString::fromLatin1(code));
            return;
        }
        restoreQuery();
        m_message = explain(code);
        if (m_result.creating)
            m_creationError = m_message;
        if (m_result.updating)
            m_archiveOperation.clear();
        if (m_result.exporting && m_export)
            m_export->finish({}, "runtime_unavailable");
        if (m_exportCopy) {
            m_exportCopy->finish("runtime_unavailable");
            m_exportCopy.reset();
        }
        emit changed();
        return;
    }
    m_pending = result.request_id;
    emit changed();
}
void BrowserController::receive(const xdzip::reader::Event &event) {
    using xdzip::reader::EventKind;
    const auto text = [](const rust::String &value) {
        return QString::fromUtf8(value.data(), static_cast<qsizetype>(value.size()));
    };
    if (event.kind == EventKind::RuntimeFault) {
        setNativeDragActive(false);
        m_admissionRetry.stop();
        m_waiting.reset();
        m_faulted = true;
        m_openCancellation = OpenCancellation::None;
        m_cancelledOpenSession = 0;
        m_result = {};
        m_activeTarget.reset();
        invalidateTarget();
        invalidateCreationTarget();
        m_total = 0;
        m_offset = 0;
        m_session.clear();
        m_pending = 0;
        m_busy = false;
        m_ready = false;
        m_model.replace({});
        replaceDirectories({});
        clearContentPreview();
        m_detail.clear();
        m_preflightReport.clear();
        m_openPasswordPrompt.clear();
        m_message = explain("runtime_unavailable");
        emit tasksChanged(); // Invalidate the password binding at the fault boundary.
        emit changed();
        return;
    }
    if (handleTransferEvent(event))
        return;
    if (event.kind == EventKind::TaskUpdate) {
        const auto &t = event.task;
        // The independent task lane can publish before TaskAccepted. Bind only
        // this pending request/session; the real acknowledgement still gates UI intent.
        if (taskStarting() && m_pending && event.request_id == m_pending && t.id &&
            t.session == m_result.taskSession && !m_result.observedTask)
            m_result.observedTask = t.id;
        handleTransferTask(t);
        const auto status = text(t.status);
        const bool terminal = status == "succeeded" || status == "failed" || status == "cancelled";
        if (m_tasks.value(t.id).value("terminal").toBool() && !terminal)
            return;
        if (terminal)
            m_cancellingTasks.remove(t.id);
        QVariantList details;
        for (const auto &entry : t.details)
            details.append(QVariantMap{{"name", text(entry.name)},
                                       {"size", QString::number(entry.size)},
                                       {"status", text(entry.status)},
                                       {"errorCode", text(entry.error)},
                                       {"error", explain(text(entry.error))}});
        QVariantMap value{
            {"openPath", text(t.open_path)},
            {"openAccepted", m_tasks.value(t.id).value("openAccepted", false)},
            {"openError", m_tasks.value(t.id).value("openError")},
            {"details", details},
            {"omittedDetails", QString::number(t.omitted_details)},
            {"externalReader", t.external_reader},
            {"outputParts", QString::number(t.output_parts)},
            {"creating", t.creating},
            {"kind", text(t.kind)},
            {"format", text(t.format)},
            {"source", text(t.source)},
            {"totalKnown", t.total_known},
            {"archive", t.creating
                            ? QVariant(QFileInfo(text(t.target)).fileName())
                            : m_tasks.value(t.id).value(
                                  "archive", !t.source.empty()
                                                 ? QVariant(QFileInfo(text(t.source)).fileName())
                                                 : m_extractionIntent.value("archive", m_title))},
            {"id", QString::number(t.id)},
            {"session", QString::number(t.session)},
            {"status", status},
            {"terminal", terminal},
            {"target", text(t.target)},
            {"current", text(t.current)},
            {"files", QString::number(t.files)},
            {"declaredBytes", QString::number(t.declared_bytes)},
            {"decodedBytes", QString::number(t.decoded_bytes)},
            {"readBytes", QString::number(t.read_bytes)},
            {"verifiedFiles", QString::number(t.verified_files)},
            {"verifiedBytes", QString::number(t.verified_bytes)},
            {"writtenBytes", QString::number(t.written_bytes)},
            {"committedBytes", QString::number(t.committed_bytes)},
            {"committedFiles", QString::number(t.committed_files)},
            {"skippedFiles", QString::number(t.skipped_files)},
            {"createdDirectories", QString::number(t.created_directories)},
            {"elapsedMs", QString::number(t.elapsed_ms)},
            {"errorCode", text(t.error)},
            {"error", t.error.empty() ? m_tasks.value(t.id).value("openError").toString()
                      : text(t.error) == "cancelled" ? qtTrId("ui.d7df27ca8d31")
                                                     : explain(text(t.error))},
            {"cleanupWarning", t.cleanup_warning},
            {"outcomeUncertain", t.outcome_uncertain},
            {"leftovers", QString::number(t.leftovers)},
            {"promptId", t.prompt_id ? QString::number(t.prompt_id) : QString{}},
            {"promptName", text(t.prompt_name)},
            {"passwordPromptId",
             t.password_prompt_id ? QString::number(t.password_prompt_id) : QString{}},
            {"passwordPromptName", text(t.password_prompt_name)},
            {"passwordRetry", t.password_retry},
            {"existingBytes", QString::number(t.existing_bytes)},
            {"incomingBytes", QString::number(t.incoming_bytes)},
            {"overwriteAllowed", t.overwrite_allowed}};
        const auto incoming = t.incoming_modified_dos;
        const QDate date(1980 + (incoming >> 25), (incoming >> 21) & 15, (incoming >> 16) & 31);
        const QTime time((incoming >> 11) & 31, (incoming >> 5) & 63, (incoming & 31) * 2);
        value.insert("incomingModified", date.isValid() && time.isValid()
                                             ? QLocale().toString(date, qtTrId("format.date")) +
                                                   " " +
                                                   QLocale().toString(time, qtTrId("format.time"))
                                             : QStringLiteral("—"));
        bool validTime = false;
        const auto existingTime = text(t.existing_modified).toLongLong(&validTime);
        value.insert("existingModified",
                     validTime ? QLocale().toString(QDateTime::fromSecsSinceEpoch(existingTime),
                                                    qtTrId("format.dateTime"))
                               : QStringLiteral("—"));
        const bool firstSuccess =
            status == "succeeded" && !m_tasks.value(t.id).value("terminal").toBool();
        m_tasks.insert(t.id, value);
        if (firstSuccess && !t.open_path.empty())
            openTemporaryOutput(t.id);
        if ((text(t.kind) == "update" || text(t.kind) == "move") && terminal &&
            !m_tasks.value(t.id).value("updateRefreshed").toBool()) {
            m_tasks[t.id].insert("updateRefreshed", true);
            const auto session = QString::number(t.session), source = text(t.source);
            if (session == m_session)
                m_updateActive = true;
            QTimer::singleShot(0, this,
                               [this, session, source] { refreshAfterUpdate(session, source); });
        }
        if (text(t.kind) == "create" && firstSuccess && !shellOperation) {
            m_createdArchivesToOpen.append(text(t.target));
            QTimer::singleShot(0, this, &BrowserController::drainCreationUiWhenIdle);
        }
        while (m_tasks.size() > 64) {
            auto oldest = std::find_if(m_tasks.begin(), m_tasks.end(),
                                       [](const auto &v) { return v.value("terminal").toBool(); });
            if (oldest == m_tasks.end())
                break;
            m_hiddenFinishedTasks.remove(oldest.key());
            m_tasks.erase(oldest);
        }
        if (text(t.error) == "source_changed" && QString::number(t.session) == m_session) {
            m_result.error = "source_changed";
            m_deferredQuery.reset();
            if (m_pending)
                m_connection->cancel(m_pending);
            invalidateSnapshot();
            m_message = explain("source_changed");
            emit changed();
        }
        if (terminal && m_openAfterTasks.remove(t.id) && status == "succeeded" && !t.creating)
            QDesktopServices::openUrl(QUrl::fromLocalFile(
                t.creating ? QFileInfo(text(t.target)).absolutePath() : text(t.target)));
        emit tasksChanged();
        return;
    }
    // Cancel acknowledgements, stale events and shutdown control never mutate an active page.
    if (!m_pending || event.request_id != m_pending || m_closing)
        return;
    const bool discardedBrowse =
        m_browsingDetached && m_openCancellation == OpenCancellation::None &&
        !m_result.extraction && !m_result.creating && !m_result.checking && !m_result.targetCheck &&
        !m_result.inputPreview && !m_result.exportManifest && !m_result.closeSession;
    if (discardedBrowse) {
        if (event.kind == EventKind::Finished) {
            m_pending = 0;
            m_busy = false;
            m_result = {};
            emit changed();
        }
        return;
    }
    if (m_openCancellation == OpenCancellation::AwaitingOpen && event.kind != EventKind::Finished)
        return;
    if (event.kind == EventKind::TaskAccepted) {
        m_result.taskId = QString::number(event.task.id);
        if (m_result.updating) {
            m_updateActive = true;
            m_ready = false;
        }
        m_hasPayload = true;
        if (m_pendingOpenAfter && m_result.extraction && !m_result.exporting) {
            m_pendingOpenAfter = false;
            const auto existing = m_tasks.value(event.task.id);
            if (existing.value("terminal").toBool()) {
                if (existing.value("status") == "succeeded")
                    QDesktopServices::openUrl(QUrl::fromLocalFile(
                        existing.value("creating").toBool()
                            ? QFileInfo(existing.value("target").toString()).absolutePath()
                            : existing.value("target").toString()));
            } else
                m_openAfterTasks.insert(event.task.id);
        }
    } else if (event.kind == EventKind::Inputs) {
        if (event.inputs.size() != static_cast<std::size_t>(m_result.previewPaths.size()))
            m_result.error = "resource_limit";
        else {
            for (std::size_t i = 0; i < event.inputs.size(); ++i) {
                const auto &row = event.inputs[i];
                m_result.inputInfo.append(
                    QVariantMap{{"path", m_result.previewPaths.at(static_cast<qsizetype>(i))},
                                {"known", row.known},
                                {"directory", row.directory},
                                {"sizeBytes", QString::number(row.bytes)}});
            }
        }
        m_hasPayload = true;
    } else if (event.kind == EventKind::TargetChecked) {
        m_result.leftovers = event.total;
        m_hasPayload = true;
    } else if (event.kind == EventKind::Selected) {
        if (!m_result.freezeSelection || event.selectors.size() > 100000) {
            m_result.error = "invalid_selection";
            return;
        }
        for (const auto &id : event.selectors)
            m_result.frozenSelectors.append(text(id));
        m_hasPayload = true;
    } else if (event.kind == EventKind::Preview) {
        const bool previewText = event.content_kind == 1;
        const bool previewImage = !ImagePreview::format(event.content_kind).isEmpty();
        const auto previewLimit = previewText ? qsizetype(256 * 1024) : ImagePreview::EncodedLimit;
        if (!m_result.contentPreview || event.session != m_session.toULongLong() ||
            event.content_entry != m_previewEntry || event.content.size() > size_t(previewLimit) ||
            (!previewText && !previewImage)) {
            m_result.error = "stale_query";
            return;
        }
        m_result.previewEntry = event.content_entry;
        m_result.previewKind = event.content_kind;
        m_result.previewBytes = QByteArray(reinterpret_cast<const char *>(event.content.data()),
                                           static_cast<qsizetype>(event.content.size()));
        m_hasPayload = true;
    } else if (event.kind == EventKind::Page) {
        if (m_deferredQuery)
            return; // superseded generation never adds rows or commits
        const auto limits = xdzip::reader::metadata_limits();
        if (event.query == 0 || event.revision == 0 || event.generation != m_result.generation ||
            (m_result.query &&
             (event.query != m_result.query || event.revision != m_result.revision ||
              QString::number(event.session) != m_result.session))) {
            m_result.error = "stale_query";
            return;
        }
        if (event.archive_total > limits.entries || event.total > limits.entries ||
            event.offset > event.total || event.rows.size() > limits.rows_per_batch ||
            event.directories.size() > limits.directories ||
            event.offset + event.rows.size() > event.total ||
            (event.rows.empty() && event.offset < event.total) ||
            event.offset !=
                static_cast<quint64>(m_result.pageFetch ? m_result.offset : m_result.rows.size())) {
            m_result.error = "resource_limit";
            return;
        }
        if (!m_result.query) {
            m_result.query = event.query;
            m_result.revision = event.revision;
            m_result.rows.reserve(
                static_cast<qsizetype>(std::min<std::uint64_t>(event.total, 1024)));
        }
        m_result.replaceDirectories = m_result.replaceDirectories || event.replace_directories;
        m_result.navigationLimited = event.navigation_limited;
        for (const auto &node : event.directories) {
            if (node.count > limits.entries) {
                m_result.error = "resource_limit";
                return;
            }
            m_result.directories.append({text(node.id), text(node.parent), text(node.name),
                                         text(node.path), static_cast<int>(node.count)});
        }
        m_result.total = static_cast<int>(event.total);
        m_result.archiveTotal = static_cast<int>(event.archive_total);
        m_format = text(event.format);
        m_updateAllowed = event.update_allowed;
        m_editRestriction = event.edit_restriction;
        m_archiveFolder = text(event.folder_name);
        m_sourceGroup = text(event.source_group);
        if (!m_result.pageFetch)
            m_result.offset = 0;
        m_result.session = QString::number(event.session);
        const bool nativeFormat =
            QStringList{QStringLiteral("7z"), QStringLiteral("RAR"), QStringLiteral("CAB"),
                        QStringLiteral("ISO"), QStringLiteral("UDF")}
                .contains(m_format, Qt::CaseInsensitive);
        for (const auto &row : event.rows) {
            const auto method = nativeFormat                        ? m_format
                                : m_format == QStringLiteral("TAR") ? qtTrId("ui.695e3ecda20b")
                                : row.method == 0                   ? QStringLiteral("Store")
                                : row.method == 8                   ? QStringLiteral("Deflate")
                                                                    : QString::number(row.method);
            QStringList status;
            if (row.directory)
                status << qtTrId("ui.52daa71ebc31");
            if (row.encrypted)
                status << qtTrId("ui.320c254572c4");
            if (row.warning)
                status << qtTrId("ui.ffcf372a457f");
            if (row.physical && !row.size_known)
                status << qtTrId("archive.unknownSize");
            const auto path = text(row.name);
            auto name = path;
            if (m_result.criteria.search.isEmpty()) {
                while (name.endsWith('/'))
                    name.chop(1);
                name = name.section('/', -1);
            }
            auto type =
                row.directory ? qtTrId("ui.7c7802d8adae") : QFileInfo(name).suffix().toUpper();
            if (type == QStringLiteral("JPG"))
                type = QStringLiteral("JPEG");
            if (type.isEmpty())
                type = qtTrId("ui.39932f24fe11");
            // DOS values describe archive-local wall time, with no timezone information.
            const auto packed = row.modified_dos;
            const QDate date(1980 + (packed >> 25), (packed >> 21) & 15, (packed >> 16) & 31);
            const QTime time((packed >> 11) & 31, (packed >> 5) & 63, (packed & 31) * 2);
            const auto modified = date.isValid() && time.isValid()
                                      ? QLocale().toString(date, qtTrId("format.date"))
                                      : QStringLiteral("—");
            m_result.rows.append(
                {text(row.id),
                 {name, row.physical && row.size_known ? QString::number(row.size) : QString{},
                  row.physical && m_format != QStringLiteral("TAR.GZ")
                      ? QString::number(row.packed_size)
                      : QString{},
                  type, modified},
                 row.directory,
                 row.directory && !row.warning,
                 row.physical ? method : QStringLiteral("—"),
                 status.isEmpty() ? qtTrId("ui.aeae1c799c94")
                                  : status.join(qtTrId("common.separator")),
                 row.warning || row.encrypted,
                 path,
                 text(row.navigation_id),
                 row.physical,
                 row.size_known});
        }
        m_hasPayload = true;
    } else if (event.kind == EventKind::Snapshot &&
               m_openCancellation == OpenCancellation::Snapshot) {
        m_cancelledOpenSession = event.session;
        m_hasPayload = true;
    } else if (event.kind == EventKind::Snapshot && m_result.closeSession) {
        m_hasPayload = event.session == 0;
    } else if (event.kind == EventKind::Preflight) {
        const auto &report = event.report;
        const QHash<QString, QString> reasons = {
            {"path", qtTrId("ui.10fee48a2a5d")},
            {"collision", qtTrId("ui.179c1af47cd2")},
            {"metadata", qtTrId("ui.5f62429e8f2b")},
            {"encrypted", qtTrId("ui.bd832c169421")},
            {"password_required", qtTrId("password.preflight")},
            {"method", qtTrId("ui.b3ac882336b4")},
            {"unknown_size", qtTrId("archive.unknownSize")},
            {"file_limit", qtTrId("ui.db4ca98ebb0e")},
            {"total_limit", qtTrId("ui.eca46ec574de")},
            {"high_ratio", qtTrId("ui.998c9646d2cd")}};
        QStringList findings;
        for (const auto &finding : report.findings) {
            const auto value = text(finding);
            const auto split = value.indexOf(':');
            findings << qtTrId("ui.a1881d3f7bb1")
                            .arg(value.left(split),
                                 reasons.value(value.mid(split + 1), qtTrId("ui.85ab227ee09d")));
        }
        m_result.report = {{"entries", QString::number(report.entries)},
                           {"bytes", text(report.declared_bytes)},
                           {"findingCount", QString::number(report.finding_count)},
                           {"findings", findings},
                           {"unicodeReview", report.unicode_review}};
        m_hasPayload = true;
    } else if (event.kind == EventKind::Finished) {
        m_busy = false;
        m_pending = 0;
        if (m_openCancellation == OpenCancellation::AwaitingOpen) {
            beginCancelledOpenCleanup();
            return;
        }
        if (m_openCancellation == OpenCancellation::Snapshot ||
            m_openCancellation == OpenCancellation::Closing) {
            auto error = text(event.error);
            if (error.isEmpty() && !m_hasPayload)
                error = QStringLiteral("runtime_unavailable");
            if (error.isEmpty() && m_openCancellation == OpenCancellation::Snapshot &&
                m_cancelledOpenSession) {
                xdzip::reader::Command command{};
                command.kind = xdzip::reader::CommandKind::CloseSession;
                command.session = m_cancelledOpenSession;
                m_openCancellation = OpenCancellation::Closing;
                submit(std::move(command));
                return;
            }
            finishCancelledOpenCleanup(error);
            return;
        }
        // Even a failed query must release a queued successful creation output.
        if (!m_createdArchivesToOpen.isEmpty())
            QTimer::singleShot(0, this, &BrowserController::drainCreationUiWhenIdle);
        if (m_deferredQuery && text(event.error) != "source_changed") {
            m_creationPreviewPending |= m_result.inputPreview;
            auto latest = std::move(*m_deferredQuery);
            m_deferredQuery.reset();
            submit(std::move(latest.command));
            m_result.criteria = latest.criteria;
            m_result.generation = m_generation;
            return;
        }
        m_deferredQuery.reset();
        if (!event.error.empty())
            m_result.error = text(event.error);
        if ((m_result.opening || m_result.contentPreview) &&
            (m_result.error == "password_required" || m_result.error == "wrong_password" ||
             m_result.error == "password_or_integrity")) {
            if (m_openPasswordGeneration == std::numeric_limits<quint64>::max() ||
                (m_result.contentPreview && ++m_previewAttempts > 10)) {
                m_result.error = "resource_limit";
            } else {
                m_openPasswordPrompt = {
                    {"taskId",
                     m_result.contentPreview ? QStringLiteral("preview") : QStringLiteral("open")},
                    {"promptId", QString::number(++m_openPasswordGeneration)},
                    {"name", m_result.contentPreview ? m_detail.value("name") : QVariant(m_title)},
                    {"kind",
                     m_result.contentPreview ? QStringLiteral("preview") : QStringLiteral("open")},
                    {"retry", m_result.error != "password_required"}};
                if (m_result.contentPreview)
                    m_contentPreview = {{"state", "password"}};
                m_result = {};
                m_message.clear();
                emit changed();
                emit tasksChanged();
                return;
            }
        }
        if (!m_hasPayload && m_result.error.isEmpty())
            m_result.error = "runtime_unavailable";
        if (m_result.freezeSelection) {
            if (m_result.error.isEmpty()) {
                m_extractionSelectors = std::move(m_result.frozenSelectors);
                m_result = {};
                presentExtractionIntent(false);
            } else {
                m_message = explain(m_result.error);
                m_result = {};
                emit changed();
            }
            return;
        }
        if (m_result.contentPreview) {
            if (m_previewEntry && m_detail.value("id").toString().toULongLong() == m_previewEntry) {
                if (!m_result.error.isEmpty()) {
                    const auto code = m_result.error;
                    const QString message =
                        code == "preview_limit" || code == "resource_limit"
                            ? qtTrId("preview.limit")
                        : code == "preview_encoding"    ? qtTrId("preview.encoding")
                        : code == "preview_unsupported" ? qtTrId("preview.unsupported")
                                                        : explain(code);
                    m_contentPreview = {{"state", "error"}, {"message", message}};
                } else if (m_result.previewKind == 1)
                    m_contentPreview = {{"state", "text"},
                                        {"text", QString::fromUtf8(m_result.previewBytes)}};
                else {
                    const QString url = PreviewImages::store(std::move(m_result.previewBytes),
                                                             m_result.previewKind);
                    m_contentPreview =
                        url.isEmpty()
                            ? QVariantMap{{"state", "error"}, {"message", qtTrId("preview.limit")}}
                            : QVariantMap{{"state", "image"}, {"image", url}};
                }
            }
            m_result = {};
            emit changed();
            QTimer::singleShot(0, this, &BrowserController::drainPage);
            return;
        }
        if (m_result.inputPreview) {
            if (m_result.previewPaths == m_creationPreviewPaths && m_result.error.isEmpty()) {
                m_creationInputs = m_result.inputInfo;
                emit creationInputsChanged();
            }
            m_result = {};
            emit changed();
            QTimer::singleShot(0, this, &BrowserController::drainCreationUiWhenIdle);
            return;
        }
        if (m_result.pageFetch) {
            if (m_result.error.isEmpty()) {
                m_model.installPage(m_result.offset, std::move(m_result.rows));
                m_summaryRevision = std::numeric_limits<quint64>::max();
                if (m_deferredDetailRow >= 0 && m_model.rowAt(m_deferredDetailRow)) {
                    showRowDetail(m_deferredDetailRow);
                    m_deferredDetailRow = -1;
                    emit changed();
                }
            } else if (m_result.error == "source_changed" || m_result.error == "stale_session")
                invalidateSnapshot();
            m_result = {};
            QTimer::singleShot(0, this, &BrowserController::drainPage);
            return;
        }
        // Fetch bounded transport batches until the current query is complete. The model
        // commits once, preserving row identity and avoiding visible paging or partial sorts.
        if (m_result.error.isEmpty() && !m_result.preflight && !m_result.closeSession &&
            !m_result.extraction && !m_result.creating && !m_result.checking &&
            !m_result.targetCheck && m_result.total <= 1024 &&
            m_result.rows.size() < m_result.total) {
            xdzip::reader::Command command{};
            command.kind = xdzip::reader::CommandKind::Browse;
            command.session = m_result.session.toULongLong();
            command.directory = rust::String(m_result.criteria.directory.toStdString());
            command.text = rust::String(m_result.criteria.search.toStdString());
            command.query = m_result.query;
            command.generation = m_result.generation;
            command.revision = m_result.revision;
            command.offset = static_cast<std::uint64_t>(m_result.rows.size());
            command.sort = static_cast<std::uint8_t>(m_result.criteria.sort);
            auto accumulated = std::move(m_result);
            submit(std::move(command));
            m_result = std::move(accumulated);
            return;
        }
        if (m_result.targetCheck) {
            const auto error = m_result.error;
            const bool currentSource = m_activeTarget && !m_activeTarget->creation &&
                                       m_activeTarget->revision == m_targetRevision &&
                                       m_activeTarget->session == m_session &&
                                       m_extractionIntent.value("session").toString() == m_session;
            finishTargetCheck(error, m_result.leftovers);
            if (currentSource && (error == "source_changed" || error == "stale_session")) {
                invalidateSnapshot();
                m_message = explain(error);
            }
            m_result = {};
            emit changed();
            emit extractionChanged();
            QTimer::singleShot(0, this, &BrowserController::drainCreationUiWhenIdle);
            return;
        }
        m_message = explain(m_result.error);
        if (m_result.extraction || m_result.creating || m_result.checking) {
            const bool extraction = m_result.extraction;
            const bool creating = m_result.creating;
            const bool updating = m_result.updating;
            const bool checking = m_result.checking;
            if (creating)
                m_creationError = m_result.error.isEmpty() ? QString{} : m_message;
            const QString taskId = m_result.taskId;
            if (updating && taskId.isEmpty())
                m_archiveOperation.clear();
            m_extractionIntent.insert("error", m_message);
            if (m_result.error == "source_changed" || m_result.error == "stale_session")
                invalidateSnapshot();
            m_result = {};
            emit changed();
            emit extractionChanged();
            emit tasksChanged();
            if (extraction && !taskId.isEmpty())
                emit extractionStarted(taskId);
            if (updating && !taskId.isEmpty())
                emit updateStarted(taskId);
            else if (creating && !taskId.isEmpty())
                emit creationStarted(taskId);
            if (checking && !taskId.isEmpty())
                emit archiveTestStarted(taskId);
            QTimer::singleShot(0, this, &BrowserController::drainCreationUiWhenIdle);
            return;
        }
        if (m_result.closeSession && m_result.error.isEmpty()) {
            m_ready = false;
            m_title.clear();
            m_session.clear();
            m_committedQuery = {};
            restoreQuery();
            m_total = m_archiveTotal = m_offset = 0;
            m_selectionAnchor.clear();
            m_selectionAnchorRow = -1;
            m_model.replace({});
            replaceDirectories({});
            clearContentPreview();
            m_detail.clear();
            m_preflightReport.clear();
            m_result = {};
            emit changed();
        } else if (m_result.preflight) {
            m_preflightReport =
                m_result.error.isEmpty() ? m_result.report : QVariantMap{{"error", m_message}};
            if (m_result.error == "source_changed" || m_result.error == "stale_session" ||
                m_result.error == "runtime_unavailable")
                invalidateSnapshot();
            m_result = {};
            QPointer<BrowserController> alive(this);
            emit changed();
            if (alive)
                emit preflightReady();
        } else {
            if (!m_result.error.isEmpty() && m_ready && !m_result.replaceDirectories &&
                m_result.error != "source_changed" && m_result.error != "stale_session" &&
                m_result.error != "runtime_unavailable") {
                restoreQuery();
                m_result = {};
                emit changed();
                return;
            }
            if (!m_result.error.isEmpty()) {
                invalidateSnapshot();
                m_result = {};
                emit changed();
                return;
            }
            m_ready = true;
            if (m_result.replaceDirectories)
                emit archiveOpened(m_sourcePath);
            m_committedQuery = m_result.criteria;
            restoreQuery();
            if (m_result.replaceDirectories)
                m_archiveTotal = m_ready ? m_result.archiveTotal : 0;
            if (!m_ready || m_result.replaceDirectories)
                replaceDirectories(m_ready ? m_result.directories : QList<DirectoryRow>{});
            m_navigationLimited = m_ready && m_result.navigationLimited;
            m_selectionAnchor.clear();
            m_selectionAnchorRow = -1;
            m_desiredPage = -1;
            m_deferredDetailRow = -1;
            m_selectionAnchorRow = -1;
            m_pageQuery = m_result.query;
            m_pageGeneration = m_result.generation;
            m_pageRevision = m_result.revision;
            if (m_result.total > 1024)
                m_model.replacePaged(m_result.total, std::move(m_result.rows),
                                     QStringLiteral("%1.%2.%3")
                                         .arg(m_pageQuery)
                                         .arg(m_pageGeneration)
                                         .arg(m_pageRevision));
            else
                m_model.replace(m_ready ? std::move(m_result.rows) : QList<BrowserRow>{});
            clearContentPreview();
            m_detail.clear();
            m_total = m_ready ? m_result.total : 0;
            m_offset = m_ready ? m_result.offset : 0;
            m_session = m_ready ? m_result.session : QString{};
            m_result = {};
            if (restoreUpdatedLocation())
                return;
            m_archiveOperation.clear();
            emit changed();
        }
        QTimer::singleShot(0, this, &BrowserController::drainCreationUiWhenIdle);
    }
}
