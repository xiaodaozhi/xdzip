#include "BrowserController.h"
#include "rust/cxx.h"
#include "xdzip-qt-bridge/src/lib.rs.h"
#include <QFileDialog>
#include <QFileInfo>
#include <QtConcurrent/QtConcurrentRun>
#include <optional>

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

struct BrowserController::Store {
    // Worker exclusively owns the archive; only the atomic cancellation supports shared calls.
    std::optional<rust::Box<xdzip::reader::Archive>> archive;
    std::shared_ptr<rust::Box<xdzip::reader::Cancellation>> control;
};

BrowserController::BrowserController(QObject *parent)
    : QObject(parent), m_model(this), m_store(std::make_shared<Store>()) {
    connect(&m_watcher, &QFutureWatcher<Result>::finished, this, [this] {
        const auto result = m_watcher.result();
        m_busy = false;
        if (m_closing) {
            emit changed();
            emit readyToClose();
            return;
        }
        m_message = explain(result.error);
        m_ready = result.error.isEmpty();
        m_model.replace(result.rows);
        m_detail.clear();
        m_total = result.total;
        m_offset = result.offset;
        m_session = result.session;
        emit changed();
    });
}
BrowserController::~BrowserController() {
    cancel();
}

QString BrowserController::explain(const QString &code) {
    if (code.isEmpty())
        return {};
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
void BrowserController::cancel() {
    if (m_store->control)
        (*m_store->control)->cancel();
}
bool BrowserController::requestClose() {
    if (!m_busy)
        return true;
    m_closing = true;
    cancel();
    m_message = tr("正在停止读取…");
    emit changed();
    return false;
}
void BrowserController::start(const QString &path, int offset) {
    m_busy = true;
    m_detail.clear();
    m_message = tr("正在读取归档目录…");
    m_store->control =
        std::make_shared<rust::Box<xdzip::reader::Cancellation>>(xdzip::reader::new_cancellation());
    auto store = m_store;
    const auto bytes = path.toUtf8();
    const auto directory = m_directory.toUtf8();
    const auto query = m_search.toUtf8();
    emit changed();
    // Exactly one outstanding job. Result delivery is QObject-bound; no worker captures this.
    m_watcher.setFuture(QtConcurrent::run([store, bytes, offset, directory, query] {
        Result result;
        try {
            if (!bytes.isEmpty()) {
                store->archive.reset();
                store->archive.emplace(xdzip::reader::open_archive(
                    rust::Str(bytes.constData(), static_cast<std::size_t>(bytes.size())),
                    **store->control));
            }
            if (!store->archive) {
                result.error = "source_io";
                return result;
            }
            const auto &archive = **store->archive;
            const auto count = xdzip::reader::count(archive);
            if (count > 10000) {
                result.error = "resource_limit";
                return result;
            }
            result.total = static_cast<int>(count);
            result.offset = offset;
            const auto session = xdzip::reader::session_id(archive);
            result.session =
                QString::fromUtf8(session.data(), static_cast<qsizetype>(session.size()));
            const auto str = [](const QByteArray &value) {
                return rust::Str(value.constData(), static_cast<std::size_t>(value.size()));
            };
            auto page = xdzip::reader::browse(archive, str(directory), str(query),
                                              static_cast<std::uint64_t>(offset), **store->control);
            result.total = static_cast<int>(page.total);
            for (const auto &row : page.rows) {
                const auto text = [](const rust::String &value) {
                    return QString::fromUtf8(value.data(), static_cast<qsizetype>(value.size()));
                };
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
                result.rows.append(
                    {text(row.id),
                     {text(row.name), QString::number(row.size), QString::number(row.packed_size),
                      method, status.isEmpty() ? tr("元数据") : status.join(" · ")},
                     row.directory,
                     row.directory && !row.warning});
            }
        } catch (const rust::Error &error) {
            result.error = QString::fromUtf8(error.what());
        } catch (const std::exception &) {
            result.error = QStringLiteral("source_io");
        }
        return result;
    }));
}
