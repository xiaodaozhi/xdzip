#pragma once
#include <QAbstractTableModel>
#include <QFutureWatcher>
#include <QObject>
#include <QStringList>
#include <QtQml/qqmlregistration.h>
#include <memory>

// Only one bounded page crosses to Qt. The complete metadata index remains in Rust.
struct BrowserRow {
    QString id;
    QStringList cells;
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

  private:
    QList<BrowserRow> m_rows;
};

class BrowserController : public QObject {
    Q_OBJECT
    QML_ELEMENT
    Q_PROPERTY(QAbstractItemModel *entries READ entries CONSTANT)
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
    Q_INVOKABLE void openPath(const QString &path);
    Q_INVOKABLE void chooseArchive();
    Q_INVOKABLE void fetchPage(int offset);
    Q_INVOKABLE void cancel();
    Q_INVOKABLE bool requestClose();
  signals:
    void changed();
    void readyToClose();

  private:
    struct Store;
    struct Result {
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
    QString m_title;
    QString m_message;
    QString m_session;
    int m_total = 0;
    int m_offset = 0;
};
