#include "BrowserController.h"
#include "rust/cxx.h"
#include "xdzip-qt-bridge/src/lib.rs.h"
#include <QFileDialog>
#include <QFileInfo>

int ArchiveTableModel::rowCount(const QModelIndex &parent) const {
    return parent.isValid() ? 0 : static_cast<int>(m_rows.size()); // bounded by 128
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
    return {};
}
QHash<int, QByteArray> ArchiveTableModel::roleNames() const {
    return {{Qt::DisplayRole, "display"},
            {Qt::UserRole, "entryId"},
            {Qt::UserRole + 1, "isDirectory"},
            {Qt::UserRole + 2, "chosen"}};
}
void ArchiveTableModel::replace(QList<BrowserRow> rows) {
    beginResetModel();
    m_rows = std::move(rows);
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
    if (m_selected.contains(item->id))
        m_selected.remove(item->id);
    else
        m_selected.insert(item->id);
    emit dataChanged(index(row, 0), index(row, 4), {Qt::UserRole + 2});
}

BrowserController::BrowserController(QObject *parent)
    : QObject(parent), m_model(this), m_connection(std::make_unique<RuntimeConnection>(this)) {
    m_connection->setConsumer([this](const xdzip::reader::Event &event) { receive(event); });
    connect(m_connection.get(), &RuntimeConnection::disconnected, this, [this](bool graceful) {
        m_busy = false;
        m_pending = 0;
        if (!graceful) {
            m_message = explain("runtime_unavailable");
            m_ready = false;
            m_model.replace({});
            m_detail.clear();
            m_preflightReport.clear();
        }
        emit changed();
        if (m_closing)
            emit readyToClose();
    });
}
BrowserController::~BrowserController() = default;

QString BrowserController::explain(const QString &code) {
    if (code.isEmpty())
        return {};
    if (code == "runtime_unavailable")
        return tr("运行时已断开，请关闭并重新启动应用。");
    if (code == "busy")
        return tr("请求暂未接收，请稍后重试。");
    if (code == "stale_session")
        return tr("归档会话已变化，请重新打开。");
    if (code == "cancelled")
        return tr("读取已取消。");
    if (code == "resource_limit")
        return tr("归档元数据超过本阶段的读取限额。");
    if (code == "unsupported_feature")
        return tr("暂不支持此归档的分卷或 ZIP64 索引。");
    if (code == "source_changed")
        return tr("读取期间源文件发生变化，请重新打开。");
    if (code == "source_io")
        return tr("无法读取所选文件，请检查文件类型和权限。");
    if (code == "invalid_path")
        return tr("当前版本无法无损表示这个源文件路径。");
    return tr("无法读取 ZIP 目录，文件可能损坏或格式尚不支持。");
}

void BrowserController::chooseArchive() {
    if (m_busy)
        return;
    // No QWidget parent: the native chooser is independent, not an attached macOS sheet.
    // Native per-platform modality and ownership still require desktop acceptance testing.
    const auto path = QFileDialog::getOpenFileName(nullptr, tr("打开归档"), {},
                                                   tr("ZIP 归档 (*.zip);;所有文件 (*)"));
    if (!path.isEmpty())
        openPath(path);
}
void BrowserController::openPath(const QString &path) {
    if (m_busy || m_closing || path.isEmpty())
        return;
    if (QString::fromUtf8(path.toUtf8()) != path || path.contains(QChar::Null)) {
        m_message = explain("invalid_path");
        emit changed();
        return;
    }
    m_directory.clear();
    m_location.clear();
    m_search.clear();
    m_detail.clear();
    m_title = QFileInfo(path).fileName();
    m_ready = false;
    m_total = 0;
    m_offset = 0;
    m_session.clear();
    m_model.replace({});
    start(path, 0);
}
void BrowserController::fetchPage(int offset) {
    if (m_busy || !m_ready || m_closing || offset < 0 || offset >= m_total)
        return;
    start({}, offset);
}
void BrowserController::selectRow(int row) {
    if (m_busy || !m_ready)
        return;
    const auto *item = m_model.rowAt(row);
    if (!item)
        return;
    m_detail = {{"name", item->cells[0]},
                {"size", item->cells[1]},
                {"packed", item->cells[2]},
                {"method", item->cells[3]},
                {"status", item->cells[4]},
                {"directory", item->directory},
                {"id", item->id}};
    m_model.select(row);
    m_preflightReport.clear();
    emit changed();
}
void BrowserController::enterDirectory(int row) {
    if (m_busy || !m_ready)
        return;
    const auto *item = m_model.rowAt(row);
    if (!item || !item->navigable)
        return;
    m_directory = item->id;
    m_location = item->cells[0];
    start({}, 0);
}
void BrowserController::showAll() {
    if (m_busy || !m_ready)
        return;
    m_directory.clear();
    m_location.clear();
    start({}, 0);
}
void BrowserController::search(const QString &text) {
    if (m_busy || !m_ready || text.toUtf8().size() > 1024)
        return;
    m_search = text;
    start({}, 0);
}
void BrowserController::inspectSelection() {
    if (m_busy || !m_ready || m_closing || m_model.selectedCount() == 0)
        return;
    xdzip::reader::Command command{};
    command.kind = xdzip::reader::CommandKind::Preflight;
    command.session = m_session.toULongLong();
    for (const auto &id : m_model.selectedIds())
        command.selected.push_back(id.toULongLong());
    submit(std::move(command), true);
}
void BrowserController::cancel() {
    if (m_pending)
        m_connection->cancel(m_pending);
}
bool BrowserController::requestClose() {
    if (m_connection->stopped())
        return true;
    m_closing = true;
    m_preflightReport.clear();
    m_message = tr("正在关闭运行时…");
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
    m_detail.clear();
    submit(std::move(command));
}
void BrowserController::submit(xdzip::reader::Command command, bool preflight) {
    m_preflightReport.clear();
    const auto result = m_connection->submit(std::move(command));
    if (result.status != xdzip::reader::Admission::Accepted) {
        m_message = explain(
            result.status == xdzip::reader::Admission::Closed ? "runtime_unavailable" : "busy");
        emit changed();
        return;
    }
    m_pending = result.request_id;
    m_result = {};
    m_result.preflight = preflight;
    m_hasPayload = false;
    m_busy = true;
    m_message = preflight ? tr("正在检查所选条目…") : tr("正在读取归档目录…");
    emit changed();
}
void BrowserController::receive(const xdzip::reader::Event &event) {
    using xdzip::reader::EventKind;
    const auto text = [](const rust::String &value) {
        return QString::fromUtf8(value.data(), static_cast<qsizetype>(value.size()));
    };
    if (event.kind == EventKind::RuntimeFault) {
        m_pending = 0;
        m_busy = false;
        m_ready = false;
        m_model.replace({});
        m_detail.clear();
        m_preflightReport.clear();
        m_message = explain("runtime_unavailable");
        emit changed();
        return;
    }
    // Cancel acknowledgements, stale events and shutdown control never mutate an active page.
    if (!m_pending || event.request_id != m_pending || m_closing)
        return;
    if (event.kind == EventKind::Page) {
        if (event.total > 10000 || event.offset > event.total || event.rows.size() > 128) {
            m_result.error = "resource_limit";
            return;
        }
        m_result.total = static_cast<int>(event.total);
        m_result.offset = static_cast<int>(event.offset);
        m_result.session = QString::number(event.session);
        for (const auto &row : event.rows) {
            const auto method = row.method == 0   ? QStringLiteral("Store")
                                : row.method == 8 ? QStringLiteral("Deflate")
                                                  : QString::number(row.method);
            QStringList status;
            if (row.directory)
                status << tr("目录");
            if (row.encrypted)
                status << tr("加密");
            if (row.warning)
                status << tr("需检查");
            m_result.rows.append(
                {text(row.id),
                 {text(row.name), QString::number(row.size), QString::number(row.packed_size),
                  method, status.isEmpty() ? tr("元数据") : status.join(" · ")},
                 row.directory,
                 row.directory && !row.warning});
        }
        m_hasPayload = true;
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
        if (!event.error.empty())
            m_result.error = text(event.error);
        if (!m_hasPayload && m_result.error.isEmpty())
            m_result.error = "runtime_unavailable";
        m_message = explain(m_result.error);
        if (m_result.preflight) {
            m_preflightReport =
                m_result.error.isEmpty() ? m_result.report : QVariantMap{{"error", m_message}};
            emit changed();
            emit preflightReady();
        } else {
            m_ready = m_result.error.isEmpty();
            m_model.replace(m_ready ? std::move(m_result.rows) : QList<BrowserRow>{});
            m_detail.clear();
            m_total = m_ready ? m_result.total : 0;
            m_offset = m_ready ? m_result.offset : 0;
            m_session = m_ready ? m_result.session : QString{};
            emit changed();
        }
        m_result = {};
    }
}
