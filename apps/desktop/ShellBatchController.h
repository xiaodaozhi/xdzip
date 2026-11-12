#pragma once
#include "BrowserController.h"
#include "ShellActionRequest.h"
#include <QSet>

class ShellBatchController : public QObject {
    Q_OBJECT
    QML_ELEMENT
    Q_PROPERTY(BrowserController *reader READ reader CONSTANT)
    Q_PROPERTY(bool active READ active NOTIFY changed)
    Q_PROPERTY(bool finished READ finished NOTIFY changed)
    Q_PROPERTY(bool choosing READ choosing NOTIFY changed)
    Q_PROPERTY(QVariantList actions READ actions NOTIFY changed)
    Q_PROPERTY(QVariantList results READ results NOTIFY changed)
    Q_PROPERTY(QString summary READ summary NOTIFY changed)
    Q_PROPERTY(int archiveCount READ archiveCount NOTIFY changed)
  public:
    explicit ShellBatchController(QObject *parent = nullptr);
    BrowserController *reader() {
        return &m_reader;
    }
    void begin(const ShellActionRequest &request);
    bool active() const;
    bool finished() const {
        return m_stage == Finished;
    }
    bool choosing() const {
        return m_stage == Choosing;
    }
    int archiveCount() const {
        return m_request.paths.size();
    }
    QVariantList actions() const;
    QVariantList results() const {
        return m_results;
    }
    QString summary() const;
    Q_INVOKABLE void choose(const QString &action);
    Q_INVOKABLE void cancel();
    Q_INVOKABLE void openTarget(int index);
    Q_INVOKABLE void configureExtraction(const QString &target, int layout, int conflict,
                                         bool highRatio, bool openAfter);
    Q_INVOKABLE void configuringCreation();
  signals:
    void changed();
    void configureCreate(const QVariantList &inputs, const QString &target, const QString &name,
                         bool sevenZip);
    void configureExtract();
    void showTask(const QString &id);
    void showSummary();

  private:
    enum Stage { Choosing, Configuring, Opening, Target, Submitting, Running, Finished };
    void schedule();
    void step();
    void next();
    void record(const QString &status, const QString &error = {}, const QString &target = {});
    void accepted(const QString &id);
    BrowserController m_reader;
    ShellActionRequest m_request;
    Stage m_stage = Choosing;
    QList<QStringList> m_jobs;
    QVariantList m_results;
    QSet<QString> m_groups;
    QString m_task, m_target;
    int m_index = 0, m_layout = 2, m_conflict = 0;
    bool m_scheduled = false, m_cancelled = false, m_highRatio = false, m_openAfter = false;
    bool m_chooseTarget = false;
};
