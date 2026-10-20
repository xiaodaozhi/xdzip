#pragma once
#include <QKeySequence>
#include <QObject>
#include <QPointer>
#include <QSystemTrayIcon>
#include <functional>
#include <memory>

class QWindow;
class QAction;
class QMenu;
class QMenuBar;
class QQuickWindow;
class BrowserController;
class UiPreferences;
class ApplicationSessions;

// Platform presentation only: all archive commands stay in the existing controller.
class DesktopIntegration final : public QObject {
    Q_OBJECT
    Q_PROPERTY(bool nativeMenuAvailable READ nativeMenuAvailable CONSTANT)
    Q_PROPERTY(bool trayAvailable READ trayAvailable NOTIFY availabilityChanged)
  public:
    DesktopIntegration(QQuickWindow *window, BrowserController *controller, UiPreferences *settings,
                       bool interactive = true,
                       std::function<bool()> availability = QSystemTrayIcon::isSystemTrayAvailable);
    ~DesktopIntegration() override;
    void bind(QQuickWindow *window, BrowserController *controller);
    void setSessions(ApplicationSessions *sessions);
    bool nativeMenuAvailable() const;
    bool trayAvailable() const {
        return m_available;
    }
    Q_INVOKABLE void restore();
    void hideToTray() {
        hideWindows();
    }
    Q_INVOKABLE void execute(const QString &command);
    Q_INVOKABLE void refresh();
    Q_INVOKABLE void requestQuit();
    QAction *action(const QString &command) const;
    QMenu *trayMenu() const {
        return m_trayMenu.get();
    }
  signals:
    void availabilityChanged();

  protected:
    bool eventFilter(QObject *watched, QEvent *event) override;

  private:
    bool blocked() const;
    bool textEditor() const;
    bool enabled(const QString &command) const;
    void syncActions();
    void rebuildTasks(QMenu *menu);
    void rebuildRecent();
    void rebuildWindows();
    void hideWindows();
    void updateIcon();
    QAction *add(QMenu *menu, const QString &command, const QKeySequence &shortcut = {});
    QPointer<QQuickWindow> m_window;
    QPointer<BrowserController> m_controller;
    QPointer<UiPreferences> m_settings;
    QSystemTrayIcon m_tray;
    std::unique_ptr<QMenu> m_trayMenu;
    std::unique_ptr<QMenuBar> m_bar;
    QMenu *m_recent = nullptr;
    QMenu *m_windowMenu = nullptr;
    QList<QMenu *> m_taskMenus;
    QList<QPointer<QWindow>> m_hiddenWindows;
    QList<QAction *> m_actions;
    std::function<bool()> m_availability;
    bool m_interactive = true;
    bool m_available = false;
    bool m_hidden = false;
    QString m_iconVariant;
    QString m_session;
    QPointer<ApplicationSessions> m_sessions;
};
