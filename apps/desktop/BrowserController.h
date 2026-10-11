#pragma once
#include "RuntimeConnection.h"
#include <QAbstractTableModel>
#include <QObject>
#include <QSet>
#include <QStandardItemModel>
#include <QStringList>
#include <QTimer>
#include <QVariantMap>
#include <QtQml/qqmlregistration.h>
#include <memory>

// Only one bounded page crosses to Qt. The complete metadata index remains in Rust.
struct BrowserRow {
    QString id;
    QStringList cells;
    bool directory = false;
    bool navigable = false;
};
class ArchiveTableModel final : public QAbstractTableModel {
    Q_OBJECT
  public:
    explicit ArchiveTableModel(QObject *parent = nullptr) : QAbstractTableModel(parent) {}
    int rowCount(const QModelIndex &parent = {}) const override;
    int columnCount(const QModelIndex &parent = {}) const override;
    QVariant data(const QModelIndex &index, int role) const override;
    QHash<int, QByteArray> roleNames() const override;
    void replace(QList<BrowserRow> rows);
    const BrowserRow *rowAt(int row) const;
    void select(int row);
    QStringList selectedIds() const {
        return m_selected.values();
    }
    int selectedCount() const {
        return m_selected.size();
    }

  private:
    QList<BrowserRow> m_rows;
    QSet<QString> m_selected;
};

class BrowserController : public QObject {
    Q_OBJECT
    QML_ELEMENT
    Q_PROPERTY(QAbstractItemModel *entries READ entries CONSTANT)
    Q_PROPERTY(QAbstractItemModel *directories READ directories CONSTANT)
    Q_PROPERTY(QString directoryId READ directoryId NOTIFY changed)
    Q_PROPERTY(bool navigationLimited READ navigationLimited NOTIFY changed)
    Q_PROPERTY(int navigationWidth READ navigationWidth NOTIFY changed)
    Q_PROPERTY(int sortOrder READ sortOrder NOTIFY changed)
    Q_PROPERTY(QVariantMap preflightReport READ preflightReport NOTIFY changed)
    Q_PROPERTY(QVariantMap detail READ detail NOTIFY changed)
    Q_PROPERTY(int selectedCount READ selectedCount NOTIFY changed)
    Q_PROPERTY(QString location READ location NOTIFY changed)
    Q_PROPERTY(bool runtimeAvailable READ runtimeAvailable NOTIFY changed)
    Q_PROPERTY(bool busy READ busy NOTIFY changed)
    Q_PROPERTY(bool ready READ ready NOTIFY changed)
    Q_PROPERTY(QString title READ title NOTIFY changed)
    Q_PROPERTY(QString message READ message NOTIFY changed)
    Q_PROPERTY(int total READ total NOTIFY changed)
    Q_PROPERTY(int offset READ offset NOTIFY changed)
    Q_PROPERTY(QString sessionId READ sessionId NOTIFY changed)
  public:
    explicit BrowserController(QObject *parent = nullptr);
    ~BrowserController() override;
    QAbstractItemModel *entries() {
        return &m_model;
    }
    QAbstractItemModel *directories() {
        return &m_directories;
    }
    QString directoryId() const {
        return m_directory;
    }
    bool navigationLimited() const {
        return m_navigationLimited;
    }
    int navigationWidth() const {
        return m_navigationWidth;
    }
    int sortOrder() const {
        return m_sort;
    }
    Q_INVOKABLE void navigateDirectory(const QString &id);
    // Columns 0..3 toggle ascending/descending; -1 restores archive order.
    Q_INVOKABLE void sortByColumn(int column);
    bool runtimeAvailable() const {
        return !m_closing && !m_faulted && !m_connection->stopped();
    }
    bool busy() const {
        return m_busy;
    }
    bool ready() const {
        return m_ready;
    }
    QString title() const {
        return m_title;
    }
    QString message() const {
        return m_message;
    }
    int total() const {
        return m_total;
    }
    int offset() const {
        return m_offset;
    }
    QString sessionId() const {
        return m_session;
    }
    QVariantMap detail() const {
        return m_detail;
    }
    int selectedCount() const {
        return m_model.selectedCount();
    }
    QString location() const {
        return m_location;
    }
    QVariantMap preflightReport() const {
        return m_preflightReport;
    }
    Q_INVOKABLE void inspectSelection();
    Q_INVOKABLE void selectRow(int row);
    Q_INVOKABLE void enterDirectory(int row);
    Q_INVOKABLE void showAll();
    Q_INVOKABLE void search(const QString &text);
    Q_INVOKABLE void openPath(const QString &path);
    Q_INVOKABLE void chooseArchive();
    Q_INVOKABLE void fetchPage(int offset);
    Q_INVOKABLE void cancel();
    Q_INVOKABLE bool requestClose();
  signals:
    void changed();
    void readyToClose();
    void preflightReady();

  private:
    struct DirectoryRow {
        QString id, parent, name, path;
        int count = 0;
    };
    struct Result {
        bool preflight = false;
        QVariantMap report;
        QList<BrowserRow> rows;
        QList<DirectoryRow> directories;
        bool replaceDirectories = false;
        bool navigationLimited = false;
        QString error;
        QString session;
        int total = 0;
        int offset = 0;
    };
    struct QueryState {
        QString directory, location, search;
        int sort = 0;
    };
    QueryState m_committedQuery;
    void restoreQuery();
    void start(const QString &path, int offset);
    static QString explain(const QString &code);
    void replaceDirectories(const QList<DirectoryRow> &rows);
    ArchiveTableModel m_model;
    QStandardItemModel m_directories;
    QHash<QString, QStandardItem *> m_directoryItems;
    QHash<QString, QString> m_directoryPaths;
    bool m_navigationLimited = false;
    int m_navigationWidth = 160;
    int m_sort = 0;
    std::unique_ptr<RuntimeConnection> m_connection;
    quint64 m_pending = 0;
    // One bounded command may await admission; retries never occupy the GUI thread.
    std::optional<xdzip::reader::Command> m_waiting;
    QTimer m_admissionRetry;
    int m_admissionAttempts = 0;
    Result m_result;
    bool m_hasPayload = false;
    void receive(const xdzip::reader::Event &event);
    void submit(xdzip::reader::Command command, bool preflight = false);
    void attemptSubmission();
    bool m_busy = false;
    bool m_ready = false;
    bool m_closing = false;
    bool m_faulted = false;
    QVariantMap m_preflightReport;
    QVariantMap m_detail;
    QString m_directory;
    QString m_location;
    QString m_search;
    QString m_title;
    QString m_message;
    QString m_session;
    int m_total = 0;
    int m_offset = 0;
};
