#include "BrowserController.h"
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
#include <QPointer>
#include <QStandardPaths>
#include <QTime>
#include <QUrl>
#include <algorithm>
#include <limits>
#include <utility>
#include <vector>

int ArchiveTableModel::rowCount(const QModelIndex &parent) const {
    return parent.isValid()
               ? 0
               : static_cast<int>(m_rows.size()); // bounded by the admitted 100,000-row snapshot
}
int ArchiveTableModel::columnCount(const QModelIndex &parent) const {
    return parent.isValid() ? 0 : 5;
}
QVariant ArchiveTableModel::data(const QModelIndex &index, int role) const {
    if (!index.isValid() || index.row() < 0 || index.row() >= m_rows.size() || index.column() < 0 ||
        index.column() >= 5)
        return {};
    const auto &row = m_rows.at(index.row());
    if (role == Qt::DisplayRole)
        return row.cells.at(index.column());
    if (role == Qt::UserRole)
        return row.id;
    if (role == Qt::UserRole + 1)
        return row.directory;
    if (role == Qt::UserRole + 2)
        return m_selected.contains(row.id);
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
    m_rows = std::move(rows);
    ++m_selectionRevision;
    m_selected.clear();
    endResetModel();
}

const BrowserRow *ArchiveTableModel::rowAt(int row) const {
    return row >= 0 && row < m_rows.size() ? &m_rows[row] : nullptr;
}
void ArchiveTableModel::select(int row) {
    const auto *item = rowAt(row);
    if (!item)
        return;
    ++m_selectionRevision;
    if (m_selected.contains(item->id))
        m_selected.remove(item->id);
    else
        m_selected.insert(item->id);
    emit dataChanged(index(row, 0), index(row, 4), {Qt::UserRole + 2});
}

bool ArchiveTableModel::isSelected(int row) const {
    const auto *item = rowAt(row);
    return item && m_selected.contains(item->id);
}
void ArchiveTableModel::selectOnly(int row) {
    const auto *item = rowAt(row);
    if (!item)
        return;
    ++m_selectionRevision;
    m_selected = {item->id};
    emit dataChanged(index(0, 0), index(rowCount() - 1, 4), {Qt::UserRole + 2});
}
void ArchiveTableModel::selectAll(bool selected) {
    ++m_selectionRevision;
    m_selected.clear();
    if (selected)
        for (const auto &item : m_rows)
            m_selected.insert(item.id);
    if (!m_rows.empty())
        emit dataChanged(index(0, 0), index(rowCount() - 1, 4), {Qt::UserRole + 2});
}
void ArchiveTableModel::selectRange(int first, int last) {
    if (!rowAt(first) || !rowAt(last))
        return;
    ++m_selectionRevision;
    m_selected.clear();
    for (int row = std::min(first, last); row <= std::max(first, last); ++row)
        m_selected.insert(m_rows.at(row).id);
    emit dataChanged(index(0, 0), index(rowCount() - 1, 4), {Qt::UserRole + 2});
}

BrowserController::BrowserController(QObject *parent)
    : QObject(parent), m_model(this), m_directories(this),
      m_connection(std::make_unique<RuntimeConnection>(this)) {
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
            }
            m_openAfterTasks.clear();
            emit tasksChanged();
        }
        QPointer<BrowserController> alive(this);
        emit changed();
        if (alive && m_closing)
            emit readyToClose();
    });
}
BrowserController::~BrowserController() = default;

QString BrowserController::explain(const QString &code) {
    if (code.isEmpty())
        return {};
    const QHash<QString, QString> taskErrors = {
        {"task_abandoned", tr("任务异常中断，提交计数可能不完整，请检查目标目录并重新启动应用。")},
        {"integrity_error", tr("文件内容校验失败，未提交的文件已丢弃。")},
        {"output_io", tr("无法写入目标目录，请检查权限和文件系统。")},
        {"disk_full", tr("目标磁盘空间不足，已停止写入。")},
        {"unsafe_target", tr("目标包含链接、特殊对象或不可信的共享权限，请选择个人目录。")},
        {"target_changed", tr("目标目录或已有文件发生变化，请重新检查后重试。")},
        {"output_conflict", tr("提交时目标文件被其他操作创建，请重新解压并选择冲突处理方式。")},
        {"unsupported_filesystem", tr("目标文件系统不支持安全提交，请选择本地磁盘目录。")},
        {"unsupported_entry", tr("所选条目包含不支持的方法、加密、编码或特殊对象。")},
        {"unsafe_input", tr("输入含链接、特殊对象，或输出与输入相同，请重新选择。")},
        {"input_collision", tr("输入名称在大小写或 Unicode 规范化后重名，请调整名称。")},
        {"unsafe_name", tr("所选文件名不满足安全输出规则。")},
        {"invalid_password", tr("密码必须为 1–128 个 UTF-8 字节，且不含控制字符。")},
        {"invalid_creation_options", tr("分卷大小或密码无效，请重新检查创建选项。")},
        {"split_overwrite_not_supported", tr("已有分卷不能原地覆盖，请整组改名或选择其他目录。")},
        {"ambiguous_name", tr("归档中的文件与目录名称存在歧义，无法安全解压。")},
        {"high_ratio", tr("压缩比超过 1000:1；确认内容可信后可在选项中允许高压缩比。")},
        {"file_quota", tr("单文件超过 8 GiB 解压限额。")},
        {"task_quota", tr("任务超过 20 GiB 解压限额。")},
        {"protected_source", tr("不能覆盖正在解压的源归档。")},
        {"task_busy", tr("任务队列已满，请等待已有任务结束。")},
        {"cleanup_error", tr("暂存目录清理失败，请检查目标目录中的恢复材料。")},
        {"invalid_selection", tr("所选文件与当前目录不匹配，请重新选择。")},
        {"unsafe_folder_name", tr("归档名称不能用作输出文件夹，请取消创建同名文件夹。")},
        {"rename_limit", tr("可用的自动重命名次数已耗尽。")}};
    if (taskErrors.contains(code))
        return taskErrors.value(code);
    if (code == "runtime_unavailable")
        return tr("应用无法继续处理任务，请关闭并重新启动。");
    if (code == "invalid_request")
        return tr("请求参数超出当前支持范围。");
    if (code == "identifier_exhausted")
        return tr("请求编号已耗尽，请重新启动应用。");
    if (code == "busy")
        return tr("请求暂未接收，请稍后重试。");
    if (code == "stale_session")
        return tr("归档会话已变化，请重新打开。");
    if (code == "cancelled")
        return tr("读取已取消。");
    if (code == "resource_limit")
        return tr("操作超过内存、记录或字节限额。");
    if (code == "unsupported_feature")
        return tr("暂不支持此归档的分卷、加密或压缩方法。");
    if (code == "stale_query")
        return tr("查询结果已过期，请重试");
    if (code == "source_changed")
        return tr("读取期间源文件发生变化，请重新打开。");
    if (code == "source_io")
        return tr("无法读取所选文件，请检查文件类型和权限。");
    if (code == "invalid_path")
        return tr("当前版本无法无损表示这个源文件路径。");
    return tr("无法读取归档目录，文件可能损坏、超过资源上限或格式尚不支持。");
}

void BrowserController::chooseArchive() {
    if (m_busy || !runtimeAvailable())
        return;
    // No QWidget parent: the native chooser is independent, not an attached macOS sheet.
    // Native per-platform modality and ownership still require desktop acceptance testing.
    const auto path = QFileDialog::getOpenFileName(
        nullptr, tr("打开归档"), {}, tr("压缩文件 (*.zip *.tar *.tar.gz *.tgz);;所有文件 (*)"));
    if (!path.isEmpty())
        openPath(path);
}
bool BrowserController::canOpenDroppedUrls(const QList<QUrl> &urls) const {
    if (m_busy || !runtimeAvailable() || urls.size() != 1 || !urls.first().isLocalFile())
        return false;
    const QFileInfo file(urls.first().toLocalFile());
    const auto name = file.fileName().toLower();
    return file.isFile() && (name.endsWith(".zip") || name.endsWith(".tar") ||
                             name.endsWith(".tar.gz") || name.endsWith(".tgz"));
}
bool BrowserController::openDroppedUrls(const QList<QUrl> &urls) {
    if (m_busy || !runtimeAvailable())
        return false;
    if (!canOpenDroppedUrls(urls)) {
        m_message = tr("请拖入单个本地 ZIP、TAR 或 TAR.GZ 压缩文件。");
        emit changed();
        return false;
    }
    // This is an input hint only. The existing runtime probes content and applies
    // admission limits; a matching extension never asserts that an archive is valid.
    openPath(urls.first().toLocalFile());
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
    m_committedQuery = {};
    m_directory.clear();
    m_location.clear();
    m_search.clear();
    m_sort = 0;
    m_navigationLimited = false;
    m_detail.clear();
    m_title = QFileInfo(path).fileName();
    m_sourcePath = path;
    m_ready = false;
    m_total = 0;
    m_offset = 0;
    m_session.clear();
    m_model.replace({});
    replaceDirectories({});
    start(path, 0);
}
void BrowserController::selectRow(int row) {
    if (m_busy || !m_ready)
        return;
    const auto *item = m_model.rowAt(row);
    if (!item)
        return;
    m_selectionAnchor = item->id;
    m_model.select(row);
    showRowDetail(row);
    m_preflightReport.clear();
    emit changed();
}
void BrowserController::showRowDetail(int row) {
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
    std::vector<std::uint64_t> sizes;
    sizes.reserve(static_cast<std::size_t>(m_model.selectedCount()));
    for (int row = 0; row < m_model.rowCount(); ++row) {
        if (!m_model.isSelected(row))
            continue;
        const auto *item = m_model.rowAt(row);
        if (!item->physical) {
            m_summaryPhysical = false;
            m_summaryBytes = QStringLiteral("—");
            return;
        }
        sizes.push_back(item->cells[1].toULongLong());
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
    if (m_busy || !m_ready || !m_model.rowAt(row))
        return;
    m_selectionAnchor = m_model.rowAt(row)->id;
    m_model.selectOnly(row);
    showRowDetail(row);
    m_preflightReport.clear();
    emit changed();
}
void BrowserController::selectRangeTo(int row) {
    if (m_busy || !m_ready || !m_model.rowAt(row))
        return;
    int anchor = row;
    bool foundAnchor = false;
    for (int index = 0; index < m_model.rowCount(); ++index)
        if (m_model.rowAt(index)->id == m_selectionAnchor) {
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
    if (m_busy || !m_ready || !m_model.rowAt(row))
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
    if (m_busy || !m_ready)
        return;
    m_model.selectAll(selected);
    m_selectionAnchor.clear();
    if (selected && m_model.rowCount() > 0)
        showRowDetail(m_model.rowCount() - 1);
    else
        m_detail.clear();
    m_preflightReport.clear();
    emit changed();
}
void BrowserController::copySelection(bool includeArchivePath) {
    if (m_busy || !m_ready)
        return;
    QStringList names;
    for (int row = 0; row < m_model.rowCount(); ++row) {
        if (!m_model.isSelected(row))
            continue;
        const auto *item = m_model.rowAt(row);
        const auto display = item->path.isEmpty() ? item->cells[0] : item->path;
        QString name = display;
        while (name.endsWith('/'))
            name.chop(1);
        names.append(includeArchivePath ? display : name.section('/', -1));
    }
    if (!names.empty())
        QGuiApplication::clipboard()->setText(names.join('\n'));
}
void BrowserController::enterDirectory(int row) {
    if (m_busy || !m_ready)
        return;
    const auto *item = m_model.rowAt(row);
    if (!item || !item->navigable)
        return;
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
    if (!m_ready || m_closing || m_result.preflight || m_result.extraction ||
        m_result.targetCheck || column < -1 || column > 4)
        return;
    const int ascending = column < 3 ? column * 2 + 1 : column == 3 ? 9 : 11;
    m_sort = column == -1 ? 0 : (m_sort == ascending ? ascending + 1 : ascending);
    start({}, 0);
}
void BrowserController::showRoot() {
    if (!m_ready || m_closing || m_result.preflight || m_result.extraction || m_result.targetCheck)
        return;
    m_directory.clear();
    m_location.clear();
    start({}, 0);
}
void BrowserController::search(const QString &text) {
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
        result.append(it.value());
    }
    return result;
}
int BrowserController::activeTaskCount() const {
    const bool preparing =
        m_busy && m_result.extraction &&
        (m_result.taskId.isEmpty() || !m_tasks.contains(m_result.taskId.toULongLong()));
    return static_cast<int>(
               std::count_if(m_tasks.cbegin(), m_tasks.cend(),
                             [](const auto &v) { return !v.value("terminal").toBool(); })) +
           (preparing ? 1 : 0);
}
void BrowserController::prepareExtraction(bool all, const QStringList &selectors) {
    if (!m_ready || m_busy || !runtimeAvailable())
        return;
    m_extractionSelectors = selectors.isEmpty() ? m_model.selectedIds() : selectors;
    if (!all && m_extractionSelectors.isEmpty())
        return;
    m_extractionDirectory = selectors.isEmpty() ? m_directory : QString{};
    QVariantList samples;
    for (int i = 0; i < m_model.rowCount() && samples.size() < 3; ++i) {
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
    m_targetCheck.clear();
    emit extractionChanged();
    emit extractionReady();
}
QString BrowserController::chooseTarget(const QString &initial) {
    return QFileDialog::getExistingDirectory(nullptr, tr("选择目标目录"), initial,
                                             QFileDialog::ShowDirsOnly |
                                                 QFileDialog::DontResolveSymlinks);
}
QStringList BrowserController::chooseCreationInputs(bool directories) {
    if (directories) {
        const auto path = QFileDialog::getExistingDirectory(nullptr, tr("添加文件夹"), {},
                                                            QFileDialog::ShowDirsOnly |
                                                                QFileDialog::DontResolveSymlinks);
        return path.isEmpty() ? QStringList{} : QStringList{path};
    }
    return QFileDialog::getOpenFileNames(nullptr, tr("添加文件"), {}, {}, nullptr,
                                         QFileDialog::DontResolveSymlinks);
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
    if (!m_pendingCreationTarget.isEmpty()) {
        const auto path = std::exchange(m_pendingCreationTarget, QString{});
        checkCreationTarget(path);
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
    if (!runtimeAvailable() || m_closing)
        return;
    if (m_busy) {
        m_pendingCreationTarget = path;
        return;
    }
    if (QString::fromUtf8(path.toUtf8()) != path || path.contains(QChar::Null)) {
        m_targetCheck = {
            {"valid", false}, {"errorCode", "invalid_path"}, {"error", explain("invalid_path")}};
        emit extractionChanged();
        return;
    }
    m_checkedTarget = QDir::cleanPath(path);
    m_targetCheck = {{"path", m_checkedTarget}, {"valid", false}, {"checking", true}};
    emit extractionChanged();
    xdzip::reader::Command command{};
    command.kind = xdzip::reader::CommandKind::CheckTarget;
    command.session = 0;
    command.path = rust::String(m_checkedTarget.toStdString());
    submit(std::move(command));
}
void BrowserController::startCreation(const QStringList &inputs, const QString &target,
                                      const QString &name, int method, int level, bool openAfter,
                                      qulonglong volumeBytes, const QString &password) {
    if (m_busy || !runtimeAvailable() || m_closing)
        return;
    const auto validText = [](const QString &value) {
        return !value.contains(QChar::Null) && QString::fromUtf8(value.toUtf8()) == value;
    };
    if ((method != 0 && method != 8) ||
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
        password.toUtf8().size() > 128 ||
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
    command.password = rust::String(password.toUtf8().toStdString());
    command.conflict = 0;
    m_creationError.clear();
    m_pendingOpenAfter = openAfter;
    submit(std::move(command));
}
bool BrowserController::targetMatches(const QString &path) const {
    return m_targetCheck.value("valid").toBool() &&
           QDir::cleanPath(path) == m_targetCheck.value("path");
}
void BrowserController::checkTarget(const QString &path) {
    if (m_busy || !m_ready || !runtimeAvailable() ||
        m_extractionIntent.value("session") != m_session)
        return;
    if (QString::fromUtf8(path.toUtf8()) != path || path.contains(QChar::Null)) {
        m_targetCheck = {
            {"valid", false}, {"errorCode", "invalid_path"}, {"error", explain("invalid_path")}};
        emit extractionChanged();
        return;
    }
    m_checkedTarget = QDir::cleanPath(path);
    m_targetCheck = {{"path", m_checkedTarget}, {"valid", false}, {"checking", true}};
    emit extractionChanged();
    xdzip::reader::Command command{};
    command.kind = xdzip::reader::CommandKind::CheckTarget;
    command.session = m_session.toULongLong();
    command.path = rust::String(m_checkedTarget.toStdString());
    submit(std::move(command));
}
void BrowserController::startExtraction(const QString &target, bool all, bool createFolder,
                                        int conflict, bool allowHighRatio, bool openAfter) {
    if (m_busy || !m_ready || !runtimeAvailable() ||
        m_extractionIntent.value("session") != m_session ||
        !m_targetCheck.value("valid").toBool() ||
        QDir::cleanPath(target) != m_targetCheck.value("path") || conflict < 0 || conflict > 3)
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
    command.conflict = static_cast<std::uint8_t>(conflict);
    command.allow_high_ratio = allowHighRatio;
    m_pendingOpenAfter = openAfter;
    submit(std::move(command));
}
void BrowserController::cancelTask(const QString &id) {
    m_connection->cancelTask(id.toULongLong());
}
bool BrowserController::decideTask(const QString &id, const QString &prompt, int policy, bool all) {
    return policy >= 1 && policy <= 3 &&
           m_connection->decideTask(id.toULongLong(), prompt.toULongLong(),
                                    static_cast<quint8>(policy), all);
}
void BrowserController::openOutput(const QString &id) {
    const auto task = m_tasks.value(id.toULongLong());
    if (!task.isEmpty())
        QDesktopServices::openUrl(
            QUrl::fromLocalFile(task.value("creating").toBool()
                                    ? QFileInfo(task.value("target").toString()).absolutePath()
                                    : task.value("target").toString()));
}
void BrowserController::inspectSelection() {
    if (!canPreflight())
        return;
    if (m_busy || !m_ready || m_closing || m_model.selectedCount() == 0)
        return;
    xdzip::reader::Command command{};
    command.kind = xdzip::reader::CommandKind::Preflight;
    command.session = m_session.toULongLong();
    for (const auto &id : m_model.selectedIds())
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
    m_ready = false;
    m_committedQuery = {};
    restoreQuery();
    m_total = m_archiveTotal = m_offset = 0;
    m_session.clear();
    m_navigationLimited = false;
    m_selectionAnchor.clear();
    m_model.replace({});
    replaceDirectories({});
    m_detail.clear();
}
void BrowserController::cancel() {
    m_deferredQuery.reset();
    if (m_waiting) {
        m_admissionRetry.stop();
        m_waiting.reset();
        m_busy = false;
        restoreQuery();
        m_message = explain("cancelled");
        emit changed();
        return;
    }
    if (m_pending)
        m_connection->cancel(m_pending);
}
bool BrowserController::requestClose() {
    if (m_connection->stopped())
        return true;
    m_closing = true;
    m_deferredQuery.reset();
    m_admissionRetry.stop();
    m_waiting.reset();
    m_preflightReport.clear();
    m_message = tr("正在退出…");
    m_connection->shutdown();
    emit changed();
    return false;
}
void BrowserController::start(const QString &path, int offset) {
    xdzip::reader::Command command{};
    command.kind =
        path.isEmpty() ? xdzip::reader::CommandKind::Browse : xdzip::reader::CommandKind::Open;
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
    m_detail.clear();
    submit(std::move(command));
    m_result.criteria = criteria;
    m_result.generation = m_generation;
}
void BrowserController::submit(xdzip::reader::Command command, bool preflight) {
    m_preflightReport.clear();
    m_result = {};
    m_result.preflight = preflight;
    m_result.extraction = command.kind == xdzip::reader::CommandKind::Extract;
    m_result.creating = command.kind == xdzip::reader::CommandKind::Create;
    m_result.inputPreview = command.kind == xdzip::reader::CommandKind::InspectInputs;
    m_result.targetCheck = command.kind == xdzip::reader::CommandKind::CheckTarget;
    m_result.closeSession = command.kind == xdzip::reader::CommandKind::CloseSession;
    m_hasPayload = false;
    m_busy = true;
    m_message = m_result.inputPreview  ? tr("正在读取选中文件大小…")
                : m_result.creating    ? tr("正在准备压缩任务…")
                : m_result.extraction  ? tr("正在准备解压任务…")
                : m_result.targetCheck ? tr("正在检查目标目录…")
                : preflight            ? tr("正在检查所选条目…")
                                       : tr("正在读取归档目录…");
    m_waiting.emplace(std::move(command));
    m_admissionAttempts = 0;
    attemptSubmission();
}
void BrowserController::attemptSubmission() {
    if (!m_waiting)
        return;
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
        m_busy = false;
        const auto code = result.status == xdzip::reader::Admission::Closed ? "runtime_unavailable"
                          : result.status == xdzip::reader::Admission::Invalid ? "invalid_request"
                          : result.status == xdzip::reader::Admission::Exhausted
                              ? "identifier_exhausted"
                              : "busy";
        restoreQuery();
        m_message = explain(code);
        if (m_result.creating)
            m_creationError = m_message;
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
        m_admissionRetry.stop();
        m_waiting.reset();
        m_faulted = true;
        m_result = {};
        m_total = 0;
        m_offset = 0;
        m_session.clear();
        m_pending = 0;
        m_busy = false;
        m_ready = false;
        m_model.replace({});
        replaceDirectories({});
        m_detail.clear();
        m_preflightReport.clear();
        m_message = explain("runtime_unavailable");
        emit changed();
        return;
    }
    if (event.kind == EventKind::TaskUpdate) {
        const auto &t = event.task;
        const auto status = text(t.status);
        const bool terminal = status == "succeeded" || status == "failed" || status == "cancelled";
        QVariantList details;
        for (const auto &entry : t.details)
            details.append(QVariantMap{{"name", text(entry.name)},
                                       {"size", QString::number(entry.size)},
                                       {"status", text(entry.status)},
                                       {"errorCode", text(entry.error)},
                                       {"error", explain(text(entry.error))}});
        QVariantMap value{
            {"details", details},
            {"omittedDetails", QString::number(t.omitted_details)},
            {"externalReader", t.external_reader},
            {"outputParts", QString::number(t.output_parts)},
            {"creating", t.creating},
            {"totalKnown", t.total_known},
            {"archive", t.creating ? QVariant(QFileInfo(text(t.target)).fileName())
                                   : m_tasks.value(t.id).value(
                                         "archive", m_extractionIntent.value("archive"))},
            {"id", QString::number(t.id)},
            {"session", QString::number(t.session)},
            {"status", status},
            {"terminal", terminal},
            {"target", text(t.target)},
            {"current", text(t.current)},
            {"files", QString::number(t.files)},
            {"declaredBytes", QString::number(t.declared_bytes)},
            {"decodedBytes", QString::number(t.decoded_bytes)},
            {"writtenBytes", QString::number(t.written_bytes)},
            {"committedBytes", QString::number(t.committed_bytes)},
            {"committedFiles", QString::number(t.committed_files)},
            {"skippedFiles", QString::number(t.skipped_files)},
            {"createdDirectories", QString::number(t.created_directories)},
            {"elapsedMs", QString::number(t.elapsed_ms)},
            {"errorCode", text(t.error)},
            {"error", text(t.error) == "cancelled" ? tr("任务已取消。") : explain(text(t.error))},
            {"cleanupWarning", t.cleanup_warning},
            {"outcomeUncertain", t.outcome_uncertain},
            {"leftovers", QString::number(t.leftovers)},
            {"promptId", t.prompt_id ? QString::number(t.prompt_id) : QString{}},
            {"promptName", text(t.prompt_name)},
            {"existingBytes", QString::number(t.existing_bytes)},
            {"incomingBytes", QString::number(t.incoming_bytes)},
            {"overwriteAllowed", t.overwrite_allowed}};
        const auto incoming = t.incoming_modified_dos;
        const QDate date(1980 + (incoming >> 25), (incoming >> 21) & 15, (incoming >> 16) & 31);
        const QTime time((incoming >> 11) & 31, (incoming >> 5) & 63, (incoming & 31) * 2);
        value.insert("incomingModified",
                     date.isValid() && time.isValid()
                         ? date.toString("yyyy/MM/dd") + " " + time.toString("HH:mm")
                         : QStringLiteral("—"));
        bool validTime = false;
        const auto existingTime = text(t.existing_modified).toLongLong(&validTime);
        value.insert("existingModified",
                     validTime
                         ? QDateTime::fromSecsSinceEpoch(existingTime).toString("yyyy/MM/dd HH:mm")
                         : QStringLiteral("—"));
        m_tasks.insert(t.id, value);
        while (m_tasks.size() > 64) {
            auto oldest = std::find_if(m_tasks.begin(), m_tasks.end(),
                                       [](const auto &v) { return v.value("terminal").toBool(); });
            if (oldest == m_tasks.end())
                break;
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
        if (terminal && m_openAfterTasks.remove(t.id) && status == "succeeded")
            QDesktopServices::openUrl(QUrl::fromLocalFile(
                t.creating ? QFileInfo(text(t.target)).absolutePath() : text(t.target)));
        emit tasksChanged();
        return;
    }
    // Cancel acknowledgements, stale events and shutdown control never mutate an active page.
    if (!m_pending || event.request_id != m_pending || m_closing)
        return;
    if (event.kind == EventKind::TaskAccepted) {
        m_result.taskId = QString::number(event.task.id);
        m_hasPayload = true;
        if (m_pendingOpenAfter) {
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
            event.offset != static_cast<quint64>(m_result.rows.size())) {
            m_result.error = "resource_limit";
            return;
        }
        if (!m_result.query) {
            m_result.query = event.query;
            m_result.revision = event.revision;
            m_result.rows.reserve(static_cast<qsizetype>(event.total));
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
        m_archiveFolder = text(event.folder_name);
        m_result.offset = 0;
        m_result.session = QString::number(event.session);
        for (const auto &row : event.rows) {
            const auto method = m_format == QStringLiteral("TAR") ? tr("无压缩")
                                : row.method == 0                 ? QStringLiteral("Store")
                                : row.method == 8                 ? QStringLiteral("Deflate")
                                                                  : QString::number(row.method);
            QStringList status;
            if (row.directory)
                status << tr("目录");
            if (row.encrypted)
                status << tr("加密");
            if (row.warning)
                status << tr("需检查");
            const auto path = text(row.name);
            auto name = path;
            if (m_result.criteria.search.isEmpty()) {
                while (name.endsWith('/'))
                    name.chop(1);
                name = name.section('/', -1);
            }
            auto type = row.directory ? tr("文件夹") : QFileInfo(name).suffix().toUpper();
            if (type == QStringLiteral("JPG"))
                type = QStringLiteral("JPEG");
            if (type.isEmpty())
                type = tr("文件");
            // DOS values describe archive-local wall time, with no timezone information.
            const auto packed = row.modified_dos;
            const QDate date(1980 + (packed >> 25), (packed >> 21) & 15, (packed >> 16) & 31);
            const QTime time((packed >> 11) & 31, (packed >> 5) & 63, (packed & 31) * 2);
            const auto modified = date.isValid() && time.isValid() ? date.toString("yyyy/MM/dd")
                                                                   : QStringLiteral("—");
            m_result.rows.append({text(row.id),
                                  {name, row.physical ? QString::number(row.size) : QString{},
                                   row.physical && m_format != QStringLiteral("TAR.GZ")
                                       ? QString::number(row.packed_size)
                                       : QString{},
                                   type, modified},
                                  row.directory,
                                  row.directory && !row.warning,
                                  row.physical ? method : QStringLiteral("—"),
                                  status.isEmpty() ? tr("元数据") : status.join("，"),
                                  row.warning || row.encrypted,
                                  path,
                                  text(row.navigation_id),
                                  row.physical});
        }
        m_hasPayload = true;
    } else if (event.kind == EventKind::Snapshot && m_result.closeSession) {
        m_hasPayload = event.session == 0;
    } else if (event.kind == EventKind::Preflight) {
        const auto &report = event.report;
        const QHash<QString, QString> reasons = {
            {"path", tr("名称包含不支持的路径、保留名或非法组件")},
            {"collision", tr("输出名称重名，或文件与目录冲突")},
            {"metadata", tr("编码、链接或特殊条目需要检查")},
            {"encrypted", tr("加密条目尚不支持")},
            {"method", tr("压缩方法未纳入首版解码范围")},
            {"file_limit", tr("声明大小超过单文件 8 GiB 限额")},
            {"total_limit", tr("声明总量超过 20 GiB 或计数溢出")},
            {"high_ratio", tr("声明压缩比超过 1000:1，需要复核")}};
        QStringList findings;
        for (const auto &finding : report.findings) {
            const auto value = text(finding);
            const auto split = value.indexOf(':');
            findings << tr("记录 %1：%2")
                            .arg(value.left(split),
                                 reasons.value(value.mid(split + 1), tr("需要检查")));
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
        if (!m_hasPayload && m_result.error.isEmpty())
            m_result.error = "runtime_unavailable";
        if (m_result.inputPreview) {
            if (m_result.previewPaths == m_creationPreviewPaths && m_result.error.isEmpty()) {
                m_creationInputs = m_result.inputInfo;
                emit creationInputsChanged();
            }
            m_message.clear();
            m_result = {};
            emit changed();
            QTimer::singleShot(0, this, &BrowserController::drainCreationUiWhenIdle);
            return;
        }
        // Fetch bounded transport batches until the current query is complete. The model
        // commits once, preserving row identity and avoiding visible paging or partial sorts.
        if (m_result.error.isEmpty() && !m_result.preflight && !m_result.closeSession &&
            !m_result.extraction && !m_result.creating && !m_result.targetCheck &&
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
        m_message = explain(m_result.error);
        if (m_result.extraction || m_result.creating || m_result.targetCheck) {
            const bool extraction = m_result.extraction;
            const bool creating = m_result.creating;
            if (creating)
                m_creationError = m_result.error.isEmpty() ? QString{} : m_message;
            const QString taskId = m_result.taskId;
            if (m_result.targetCheck)
                m_targetCheck = {{"path", m_checkedTarget},
                                 {"valid", m_result.error.isEmpty()},
                                 {"errorCode", m_result.error},
                                 {"error", m_message},
                                 {"leftovers", QString::number(m_result.leftovers)}};
            else
                m_extractionIntent.insert("error", m_message);
            if (m_result.error == "source_changed" || m_result.error == "stale_session")
                invalidateSnapshot();
            m_result = {};
            emit changed();
            emit extractionChanged();
            emit tasksChanged();
            if (extraction && !taskId.isEmpty())
                emit extractionStarted(taskId);
            if (creating && !taskId.isEmpty())
                emit creationStarted(taskId);
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
            m_model.replace({});
            replaceDirectories({});
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
            m_committedQuery = m_result.criteria;
            restoreQuery();
            if (m_result.replaceDirectories)
                m_archiveTotal = m_ready ? m_result.archiveTotal : 0;
            if (!m_ready || m_result.replaceDirectories)
                replaceDirectories(m_ready ? m_result.directories : QList<DirectoryRow>{});
            m_navigationLimited = m_ready && m_result.navigationLimited;
            m_selectionAnchor.clear();
            m_model.replace(m_ready ? std::move(m_result.rows) : QList<BrowserRow>{});
            m_detail.clear();
            m_total = m_ready ? m_result.total : 0;
            m_offset = m_ready ? m_result.offset : 0;
            m_session = m_ready ? m_result.session : QString{};
            m_result = {};
            emit changed();
        }
        QTimer::singleShot(0, this, &BrowserController::drainCreationUiWhenIdle);
    }
}
