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

class ApplicationSessions final : public QObject {
    Q_OBJECT
    Q_PROPERTY(QVariantList tasks READ tasks NOTIFY tasksChanged)
    Q_PROPERTY(QAbstractItemModel *taskModel READ taskModel CONSTANT)
    Q_PROPERTY(int activeTaskCount READ activeTaskCount NOTIFY tasksChanged)
    Q_PROPERTY(int sessionCount READ sessionCount NOTIFY sessionsChanged)
  public:
    ApplicationSessions(QQmlEngine *engine, UiPreferences *settings, QObject *parent = nullptr);
    ~ApplicationSessions() override;
    void adopt(QQuickWindow *window);
    void setDesktop(DesktopIntegration *desktop);
    void open(const QStringList &paths);
    Q_INVOKABLE void openPath(const QString &path);
    Q_INVOKABLE void newSession();
    Q_INVOKABLE void activateId(const QString &id);
    Q_INVOKABLE void closeSession(const QString &id);
    Q_INVOKABLE void closeWindow(QQuickWindow *window);
    Q_INVOKABLE void focusWindow(QQuickWindow *window);
    Q_INVOKABLE void command(const QString &command);
    Q_INVOKABLE void showTask(const QString &identity);
    Q_INVOKABLE void requestQuit();
    Q_INVOKABLE void requestRestart();
    Q_INVOKABLE void cancelQuit() {
        m_wait = false;
        m_restart = false;
    }
    bool restartRequested() const {
        return m_restart;
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
    void readyToQuit();

  private:
    struct Session {
        QString id, path;
        QPointer<QObject> page;
        QPointer<BrowserController> controller;
        QPointer<QQuickWindow> window;
        bool closed = false;
        bool retiring = false;
    };
    Session *add(QQuickWindow *window, QObject *page);
    Session *create(bool newWindow);
    Session *current() const;
    void activate(Session *session, bool reveal = true);
    void refresh();
    void reap(Session *session);
    void checkQuit();
    QQmlEngine *m_engine;
    UiPreferences *m_settings;
    DesktopIntegration *m_desktop = nullptr;
    QPointer<QQuickWindow> m_active;
    QList<QPointer<QQuickWindow>> m_windows;
    std::vector<std::unique_ptr<Session>> m_sessions;
    TaskListModel m_tasks;
    bool m_quitting = false, m_wait = false, m_quitEmitted = false, m_restart = false;
    bool m_presentationReady = false;
};
