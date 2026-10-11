#pragma once
#include <QAbstractTableModel>
#include <QFutureWatcher>
#include <QObject>
#include <QSet>
#include <QStringList>
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
    Q_PROPERTY(QVariantMap preflightReport READ preflightReport NOTIFY changed)
    Q_PROPERTY(QVariantMap detail READ detail NOTIFY changed)
    Q_PROPERTY(int selectedCount READ selectedCount NOTIFY changed)
    Q_PROPERTY(QString location READ location NOTIFY changed)
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
    struct Store;
    struct Result {
        bool preflight = false;
        QVariantMap report;
        QList<BrowserRow> rows;
        QString error;
        QString session;
        int total = 0;
        int offset = 0;
    };
    void start(const QString &path, int offset);
    static QString explain(const QString &code);
    ArchiveTableModel m_model;
    std::shared_ptr<Store> m_store;
    QFutureWatcher<Result> m_watcher;
    bool m_busy = false;
    bool m_ready = false;
    bool m_closing = false;
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
