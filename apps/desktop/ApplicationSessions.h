// apps/desktop/ApplicationSessions.h

#pragma once
#include "BrowserController.h"
#include <QPointer>
#include <QQuickWindow>
#include <memory>
#include <vector>
class QQmlEngine;
class DesktopIntegration;
class UiPreferences;
class QMessageBox;

class ApplicationSessions final : public QObject {
    Q_OBJECT
    Q_PROPERTY(QVariantList tasks READ tasks NOTIFY tasksChanged)
    Q_PROPERTY(QAbstractItemModel *taskModel READ taskModel CONSTANT)
    Q_PROPERTY(int activeTaskCount READ activeTaskCount NOTIFY tasksChanged)
    Q_PROPERTY(int sessionCount READ sessionCount NOTIFY sessionsChanged)
    Q_PROPERTY(bool quitDecisionPending READ quitDecisionPending NOTIFY quitDecisionPendingChanged)
  public:
    ApplicationSessions(QQmlEngine *engine, UiPreferences *settings, QObject *parent = nullptr);
    ~ApplicationSessions() override;
    void adopt(QQuickWindow *window);
    void setDesktop(DesktopIntegration *desktop);
    void open(const QStringList &paths);
    Q_INVOKABLE void openPath(const QString &path);
    Q_INVOKABLE void newSession();
    Q_INVOKABLE void activateId(const QString &id);
    Q_INVOKABLE void moveTab(const QString &id, int targetIndex);
    Q_INVOKABLE void closeSession(const QString &id);
    Q_INVOKABLE void closeOtherSessions(const QString &id);
    Q_INVOKABLE void closeSessionsToRight(const QString &id);
    Q_INVOKABLE void closeWindow(QQuickWindow *window);
    Q_INVOKABLE void focusWindow(QQuickWindow *window);
    Q_INVOKABLE void command(const QString &command);
    Q_INVOKABLE void showAboutFor(QObject *page);
    Q_INVOKABLE void showTask(const QString &identity);
    Q_INVOKABLE void requestQuit();
    Q_INVOKABLE void requestRestart();
    // Called only after an update provider has prepared its installation handoff.
    // Busy tasks are never cancelled implicitly for an update.
    bool prepareForUpdate();
    bool updatePending() const {
        return m_updatePending;
    }
    bool quitDecisionPending() const {
        return m_quitDecisionPending;
    }
    Q_INVOKABLE void cancelQuit() {
        if (m_quitting)
            return;
        m_wait = false;
        m_restart = false;
        setQuitDecisionPending(false);
    }
    bool restartRequested() const {
        return m_restart;
    }
    bool isClosing() const {
        return m_quitting;
    }
    Q_INVOKABLE void confirmQuit(bool wait);
    QVariantList tasks() const;
    int activeTaskCount() const;
    int sessionCount() const {
        return int(m_sessions.size());
    }
    QAbstractItemModel *taskModel() {
        return &m_tasks;
    }
    QQuickWindow *activeWindow() const {
        return m_active;
    }
    QList<QQuickWindow *> windows() const;
  signals:
    void tasksChanged();
    void sessionsChanged();
    void quitDecisionPendingChanged();
    void closing();
    void readyToQuit();
    void readyForUpdate();

  private:
    struct Session {
        QString id, path;
        quint64 tabOrder = 0;
        QPointer<QObject> page;
        QPointer<BrowserController> controller;
        QPointer<QQuickWindow> window;
        bool closed = false;
        bool retiring = false;
    };
    Session *add(QQuickWindow *window, QObject *page);
    Session *create(bool newWindow, QQuickWindow *target = nullptr);
    void ensureWelcome(QQuickWindow *window);
    void cancelPendingActivation();
    Session *current() const;
    std::vector<Session *> visibleTabs(QQuickWindow *window) const;
    void activate(Session *session, bool reveal = true);
    void refresh();
    void reap(Session *session);
    void closeTabSet(Session *anchor, const std::vector<Session *> &targets);
    void closeAround(const QString &id, bool rightOnly);
    void checkQuit();
    void setQuitDecisionPending(bool pending) {
        if (m_quitDecisionPending == pending)
            return;
        m_quitDecisionPending = pending;
        emit quitDecisionPendingChanged();
    }
    QQmlEngine *m_engine;
    UiPreferences *m_settings;
    QPointer<DesktopIntegration> m_desktop;
    QPointer<QQuickWindow> m_active;
    QList<QPointer<QQuickWindow>> m_windows;
    std::vector<std::unique_ptr<Session>> m_sessions;
    TaskListModel m_tasks;
    bool m_quitting = false, m_wait = false, m_quitEmitted = false, m_restart = false;
    bool m_presentationReady = false;
    bool m_updatePending = false;
    bool m_quitDecisionPending = false;
    quint64 m_activationGeneration = 0;
    QList<QMetaObject::Connection> m_modalConnections;
    QPointer<QMessageBox> m_limitDialog;
};
