#pragma once
#include "RuntimeConnection.h"
#include <QAbstractTableModel>
#include <QObject>
#include <QSet>
#include <QStandardItemModel>
#include <QStringList>
#include <QTimer>
#include <QUrl>
#include <QVariantMap>
#include <QWindow>
#include <QtQml/qqmlregistration.h>
#include <limits>
#include <memory>

// Transport batches are bounded; Qt commits a complete query of at most 100,000 rows.
struct BrowserRow {
    QString id;
    QStringList cells;
    bool directory = false;
    bool navigable = false;
    QString method;
    QString status;
    bool attention = false;
    QString path;
    QString navigationId;
    bool physical = true;
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
    bool isSelected(int row) const;
    void selectOnly(int row);
    void selectAll(bool selected);
    void selectRange(int first, int last);
    QStringList selectedIds() const {
        return m_selected.values();
    }
    quint64 selectionRevision() const {
        return m_selectionRevision;
    }
    int selectedCount() const {
        return m_selected.size();
    }

  private:
    QList<BrowserRow> m_rows;
    QSet<QString> m_selected;
    quint64 m_selectionRevision = 0;
};

class BrowserController : public QObject {
    Q_OBJECT
    QML_ELEMENT
    Q_PROPERTY(QAbstractItemModel *entries READ entries CONSTANT)
    Q_PROPERTY(QAbstractItemModel *directories READ directories CONSTANT)
    Q_PROPERTY(QString formatLabel READ formatLabel NOTIFY changed)
    Q_PROPERTY(QString directoryId READ directoryId NOTIFY changed)
    Q_PROPERTY(QVariantList breadcrumbs READ breadcrumbs NOTIFY changed)
    Q_PROPERTY(bool navigationLimited READ navigationLimited NOTIFY changed)
    Q_PROPERTY(int navigationWidth READ navigationWidth NOTIFY changed)
    Q_PROPERTY(int sortOrder READ sortOrder NOTIFY changed)
    Q_PROPERTY(QVariantMap preflightReport READ preflightReport NOTIFY changed)
    Q_PROPERTY(QVariantMap detail READ detail NOTIFY changed)
    Q_PROPERTY(int selectedCount READ selectedCount NOTIFY changed)
    Q_PROPERTY(bool canPreflight READ canPreflight NOTIFY changed)
    Q_PROPERTY(QString selectedBytes READ selectedBytes NOTIFY changed)
    Q_PROPERTY(QString location READ location NOTIFY changed)
    Q_PROPERTY(bool runtimeAvailable READ runtimeAvailable NOTIFY changed)
    Q_PROPERTY(bool busy READ busy NOTIFY changed)
    Q_PROPERTY(bool taskStarting READ taskStarting NOTIFY changed)
    Q_PROPERTY(bool browsingBusy READ browsingBusy NOTIFY changed)
    Q_PROPERTY(bool browserQueryAvailable READ browserQueryAvailable NOTIFY changed)
    Q_PROPERTY(bool queryAvailable READ queryAvailable NOTIFY changed)
    Q_PROPERTY(bool ready READ ready NOTIFY changed)
    Q_PROPERTY(QString title READ title NOTIFY changed)
    Q_PROPERTY(QString message READ message NOTIFY changed)
    Q_PROPERTY(QString creationError READ creationError NOTIFY changed)
    Q_PROPERTY(QVariantList creationInputs READ creationInputs NOTIFY creationInputsChanged)
    Q_PROPERTY(int total READ total NOTIFY changed)
    Q_PROPERTY(int archiveTotal READ archiveTotal NOTIFY changed)
    Q_PROPERTY(int offset READ offset NOTIFY changed)
    Q_PROPERTY(QVariantMap extractionIntent READ extractionIntent NOTIFY extractionChanged)
    Q_PROPERTY(QVariantMap targetCheck READ targetCheck NOTIFY extractionChanged)
    Q_PROPERTY(QVariantList tasks READ tasks NOTIFY tasksChanged)
    Q_PROPERTY(int activeTaskCount READ activeTaskCount NOTIFY tasksChanged)
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
    QVariantList breadcrumbs() const;
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
    Q_INVOKABLE QStringList directoryAncestors(const QString &id) const;
    // Columns 0..4 toggle ascending/descending; -1 restores archive order.
    Q_INVOKABLE void sortByColumn(int column);
    bool runtimeAvailable() const {
        return !m_closing && !m_faulted && !m_connection->stopped();
    }
    bool queryAvailable() const {
        return m_ready && !m_closing && !m_result.preflight && !m_result.closeSession &&
               !m_result.extraction && !m_result.targetCheck;
    }
    bool taskStarting() const {
        return m_busy && !m_closing && !m_faulted && (m_result.extraction || m_result.creating);
    }
    bool browsingBusy() const {
        return m_busy && !taskStarting();
    }
    bool browserQueryAvailable() const {
        return m_ready && runtimeAvailable() && (queryAvailable() || taskStarting());
    }
    bool busy() const {
        return m_busy;
    }
    bool ready() const {
        return m_ready;
    }
    QString formatLabel() const {
        return m_format;
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
    int archiveTotal() const {
        return m_archiveTotal;
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
    QString selectedBytes() const;
    bool canPreflight() const;
    QString location() const {
        return m_location;
    }
    QVariantMap preflightReport() const {
        return m_preflightReport;
    }
    QVariantMap extractionIntent() const {
        return m_extractionIntent;
    }
    QVariantMap targetCheck() const {
        return m_targetCheck;
    }
    QVariantList tasks() const;
    int activeTaskCount() const;
    Q_INVOKABLE void prepareExtraction(bool all, const QStringList &selectors = {});
    Q_INVOKABLE QString chooseTarget(const QString &initial, QWindow *owner = nullptr);
    Q_INVOKABLE QStringList chooseCreationInputs(bool directories, QWindow *owner = nullptr);
    Q_INVOKABLE void inspectCreationInputs(const QStringList &paths);
    QVariantList creationInputs() const {
        return m_creationInputs;
    }
    Q_INVOKABLE QString defaultTargetDirectory() const;
    QString creationError() const {
        return m_creationError;
    }
    Q_INVOKABLE void clearCreationError() {
        m_creationError.clear();
        emit changed();
    }
    Q_INVOKABLE void startCreation(const QStringList &inputs, const QString &target,
                                   const QString &name, int method, int level, bool openAfter,
                                   qulonglong volumeBytes = 0, const QString &password = QString());
    Q_INVOKABLE void checkTarget(const QString &path);
    Q_INVOKABLE void checkCreationTarget(const QString &path);
    Q_INVOKABLE bool targetMatches(const QString &path) const;
    Q_INVOKABLE void startExtraction(const QString &target, bool all, bool createFolder,
                                     int conflict, bool allowHighRatio, bool openAfter);
    Q_INVOKABLE void cancelTask(const QString &id);
    Q_INVOKABLE bool decideTask(const QString &id, const QString &prompt, int policy, bool all);
    Q_INVOKABLE void openOutput(const QString &id);
    Q_INVOKABLE void inspectSelection();
    Q_INVOKABLE void selectRow(int row);
    Q_INVOKABLE void selectOnlyRow(int row);
    Q_INVOKABLE void selectRangeTo(int row);
    Q_INVOKABLE void selectForContext(int row);
    Q_INVOKABLE void selectAll(bool selected);
    Q_INVOKABLE void copySelection(bool includeArchivePath);
    Q_INVOKABLE void enterDirectory(int row);
    Q_INVOKABLE void showRoot();
    Q_INVOKABLE void search(const QString &text);
    Q_INVOKABLE void openPath(const QString &path);
    Q_INVOKABLE void chooseArchive(QWindow *owner = nullptr);
    Q_INVOKABLE bool canOpenDroppedUrls(const QList<QUrl> &urls) const;
    Q_INVOKABLE bool openDroppedUrls(const QList<QUrl> &urls);
    Q_INVOKABLE void closeArchive();
    Q_INVOKABLE void cancel();
    Q_INVOKABLE bool requestClose();
  signals:
    void changed();
    void readyToClose();
    void archiveOpened(const QString &path);
    void preflightReady();
    void extractionChanged();
    void extractionReady();
    void extractionStarted(const QString &task);
    void creationStarted(const QString &task);
    void creationInputsChanged();
    void tasksChanged();

  private:
    struct DirectoryRow {
        QString id, parent, name, path;
        int count = 0;
    };
    struct QueryState {
        QString directory, location, search;
        int sort = 0;
    };
    struct Result {
        bool preflight = false;
        bool extraction = false;
        bool creating = false;
        bool inputPreview = false;
        QStringList previewPaths;
        QVariantList inputInfo;
        bool targetCheck = false;
        QString taskId;
        quint64 leftovers = 0;
        bool closeSession = false;
        QVariantMap report;
        QList<BrowserRow> rows;
        QList<DirectoryRow> directories;
        bool replaceDirectories = false;
        bool navigationLimited = false;
        QString error;
        QString session;
        QueryState criteria;
        quint64 query = 0, generation = 0, revision = 0;
        int total = 0;
        int archiveTotal = 0;
        int offset = 0;
    };
    struct DeferredQuery {
        xdzip::reader::Command command;
        QueryState criteria;
    };
    std::optional<DeferredQuery> m_deferredQuery;
    quint64 m_generation = 0;
    QueryState m_committedQuery;
    void restoreQuery();
    void invalidateSnapshot();
    void showRowDetail(int row);
    void updateSelectionSummary() const;
    mutable quint64 m_summaryRevision = std::numeric_limits<quint64>::max();
    mutable QString m_summaryBytes;
    mutable bool m_summaryPhysical = true;
    void start(const QString &path, int offset);
    static QString explain(const QString &code);
    void replaceDirectories(const QList<DirectoryRow> &rows);
    ArchiveTableModel m_model;
    QString m_selectionAnchor;
    int m_archiveTotal = 0;
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
    QVariantMap m_extractionIntent, m_targetCheck;
    QStringList m_extractionSelectors;
    QString m_extractionDirectory, m_sourcePath, m_checkedTarget;
    QMap<quint64, QVariantMap> m_tasks;
    QSet<quint64> m_openAfterTasks;
    bool m_pendingOpenAfter = false;
    QString m_creationError;
    QVariantList m_creationInputs;
    QStringList m_creationPreviewPaths;
    bool m_creationPreviewPending = false;
    QString m_pendingCreationTarget;
    QStringList m_createdArchivesToOpen;
    void drainCreationUiWhenIdle();
    QVariantMap m_preflightReport;
    QVariantMap m_detail;
    QString m_directory;
    QString m_location;
    QString m_search;
    QString m_title;
    QString m_format;
    QString m_archiveFolder;
    QString m_message;
    QString m_session;
    int m_total = 0;
    int m_offset = 0;
};
