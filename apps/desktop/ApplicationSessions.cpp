// apps/desktop/ApplicationSessions.cpp

#include "ApplicationSessions.h"
#include "DesktopIntegration.h"
#include "UiPreferences.h"
#include "WindowActivation.h"
#include <QApplication>
#include <QDir>
#include <QFileInfo>
#include <QMessageBox>
#include <QQmlComponent>
#include <QQmlEngine>
#include <QQuickItem>
#include <QTimer>
#include <QUuid>
#include <algorithm>
#ifdef Q_OS_MACOS
void integrateMacTitlebar(QQuickWindow *window);
#endif
namespace {
QString key(const QString &path) {
    const QFileInfo file(path);
    auto value = file.canonicalFilePath();
    if (value.isEmpty())
        value = file.absoluteFilePath();
#ifdef Q_OS_WIN
    value = value.toCaseFolded();
#endif
    return QDir::cleanPath(value);
}
} // namespace
ApplicationSessions::ApplicationSessions(QQmlEngine *engine, UiPreferences *settings,
                                         QObject *parent)
    : QObject(parent), m_engine(engine), m_settings(settings) {
    QQmlEngine::setObjectOwnership(this, QQmlEngine::CppOwnership);
}
ApplicationSessions::~ApplicationSessions() {
    m_quitting = true;
    for (const auto &session : m_sessions) {
        if (session->controller) {
            disconnect(session->controller, nullptr, this, nullptr);
            session->controller->openIntent = {};
        }
        if (session->page)
            for (const auto &name : {"settingsWindow", "aboutWindow"})
                if (auto *popup = session->page->findChild<QQuickWindow *>(name))
                    disconnect(popup, nullptr, this, nullptr);
    }
    for (const auto &window : m_windows)
        if (window && window->property("sessionsOwned").toBool())
            delete window.data();
}
void ApplicationSessions::adopt(QQuickWindow *window) {
    window->setProperty("sessionManager", QVariant::fromValue(static_cast<QObject *>(this)));
    window->setProperty("managedWindow", true);
    m_windows.append(window);
    connect(window, &QWindow::activeChanged, this, [this, window] {
        if (window->isActive())
            focusWindow(window);
    });
    auto *page = window->property("activePage").value<QObject *>();
    auto *session = add(window, page);
    activate(session);
}
ApplicationSessions::Session *ApplicationSessions::add(QQuickWindow *window, QObject *page) {
    if (!page)
        return nullptr;
    auto value = std::make_unique<Session>();
    auto *session = value.get();
    session->id = QUuid::createUuid().toString(QUuid::WithoutBraces);
    session->window = window;
    session->page = page;
    session->controller = page->findChild<BrowserController *>();
    if (!session->controller)
        return nullptr;
    page->setProperty("sessionIdentity", session->id);
    session->controller->openIntent = [this](const QString &path) { openPath(path); };
    for (const auto &name : {"settingsWindow", "aboutWindow"}) {
        if (auto *popup = page->findChild<QQuickWindow *>(name))
            connect(popup, &QWindow::visibleChanged, this, [this, session] {
                if (session->closed)
                    reap(session);
            });
    }
    connect(session->controller, &BrowserController::changed, this, [this] { refresh(); });
    connect(session->controller, &BrowserController::tasksChanged, this, [this, session] {
        refresh();
        checkQuit();
    });
    connect(session->controller, &BrowserController::readyToClose, this, [this, session] {
        if (m_quitting) {
            checkQuit();
            return;
        }
        if (!session->closed)
            return;
        QTimer::singleShot(0, this, [this, session] {
            const QPointer<QQuickWindow> owner = session->window;
            if (session->controller) {
                disconnect(session->controller, nullptr, this, nullptr);
                session->controller->openIntent = {};
            }
            if (session->page)
                for (const auto &name : {"settingsWindow", "aboutWindow"})
                    if (auto *popup = session->page->findChild<QQuickWindow *>(name))
                        disconnect(popup, nullptr, this, nullptr);
            if (session->page && !session->page->property("retainShellPage").toBool())
                session->page->deleteLater();
            m_sessions.erase(
                std::remove_if(m_sessions.begin(), m_sessions.end(),
                               [session](const auto &value) { return value.get() == session; }),
                m_sessions.end());
            if (owner && owner->property("retiredWindow").toBool() &&
                std::none_of(m_sessions.cbegin(), m_sessions.cend(),
                             [&](const auto &other) { return other->window == owner; }))
                owner->deleteLater();
            refresh();
        });
    });
    m_sessions.push_back(std::move(value));
    refresh();
    return session;
}
void ApplicationSessions::setDesktop(DesktopIntegration *desktop) {
    m_desktop = desktop;
    desktop->setSessions(this);
    focusWindow(m_active);
}
QList<QQuickWindow *> ApplicationSessions::windows() const {
    QList<QQuickWindow *> result;
    for (const auto &window : m_windows)
        if (window)
            result.append(window);
    return result;
}
ApplicationSessions::Session *ApplicationSessions::current() const {
    if (!m_active)
        return nullptr;
    const auto page = m_active->property("activePage").value<QObject *>();
    for (const auto &session : m_sessions)
        if (session->page == page && !session->closed)
            return session.get();
    return nullptr;
}
ApplicationSessions::Session *ApplicationSessions::create(bool newWindow) {
    if (m_quitting)
        return nullptr;
    if (m_sessions.size() >= 32) {
        QMessageBox::information(nullptr, QStringLiteral("xdzip"), qtTrId("sessions.limit"));
        return nullptr;
    }
    if (newWindow || !m_active) {
        QQmlComponent component(m_engine, QUrl("qrc:/xdzip/ui/Browser.qml"));
        auto *window = qobject_cast<QQuickWindow *>(
            component.createWithInitialProperties({{"visible", false}}));
        if (!window)
            return nullptr;
        QQmlEngine::setObjectOwnership(window, QQmlEngine::CppOwnership);
        window->setProperty("sessionsOwned", true);
#ifdef Q_OS_MACOS
        integrateMacTitlebar(window);
#endif
        adopt(window);
        return current();
    }
    QQmlComponent component(m_engine, QUrl("qrc:/xdzip/ui/SessionPage.qml"));
    auto *page = qobject_cast<QQuickItem *>(component.createWithInitialProperties(
        {{"windowHost", QVariant::fromValue(m_active.data())}}));
    if (!page)
        return nullptr;
    page->setParent(m_active);
    page->setParentItem(m_active->contentItem());
    return add(m_active, page);
}
void ApplicationSessions::newSession() {
    activate(create(m_settings->openMode() == "windows"));
}
void ApplicationSessions::open(const QStringList &paths) {
    if (paths.isEmpty()) {
        if (m_desktop)
            m_desktop->restore();
        return;
    }
    for (const auto &path : paths)
        openPath(path);
}
void ApplicationSessions::openPath(const QString &path) {
    if (m_quitting || path.isEmpty() || path.contains(QChar::Null))
        return;
    const auto identity = key(path);
    for (const auto &session : m_sessions) {
        if (!session->retiring && session->path == identity) {
            // A closed page with a task remains a real session and can be reopened.
            session->closed = false;
            activate(session.get());
            return;
        }
    }
    auto *session = current();
    if (!session || !session->path.isEmpty() || session->controller->busy() ||
        session->controller->activeTaskCount()) {
        session = nullptr;
        for (const auto &candidate : m_sessions)
            if (!candidate->closed && candidate->path.isEmpty() && !candidate->controller->busy() &&
                !candidate->controller->activeTaskCount()) {
                session = candidate.get();
                break;
            }
    }
    if (!session)
        session = create(m_settings->openMode() == "windows");
    if (!session)
        return;
    session->path = identity;
    session->controller->openPath(QFileInfo(path).absoluteFilePath());
    activate(session);
}
void ApplicationSessions::activate(Session *session) {
    if (!session || session->retiring || !session->window || !session->page)
        return;
    session->closed = false;
    auto *window = session->window.data();
    window->setProperty("retiredWindow", false);
    for (const auto &other : m_sessions)
        if (other->window == window && other->page)
            other->page->setProperty("visible", other.get() == session);
    window->setProperty("activePage", QVariant::fromValue(session->page.data()));
    focusWindow(window);
    refresh();
    if (auto *modal = QGuiApplication::modalWindow())
        WindowActivation::reveal(modal);
    else
        WindowActivation::reveal(window);
}
void ApplicationSessions::focusWindow(QQuickWindow *window) {
    if (!window)
        return;
    m_active = window;
    auto *session = current();
    if (m_desktop && session)
        m_desktop->bind(window, session->controller);
}
void ApplicationSessions::activateId(const QString &id) {
    for (const auto &session : m_sessions)
        if (session->id == id) {
            activate(session.get());
            return;
        }
}
void ApplicationSessions::closeSession(const QString &id) {
    for (const auto &session : m_sessions) {
        if (session->id != id || session->closed)
            continue;
        auto *value = session.get();
        value->closed = true;
        value->page->setProperty("visible", false);
        QMetaObject::invokeMethod(value->page, "hideSessionWindows");
        Session *next = nullptr;
        for (const auto &other : m_sessions)
            if (other->window == value->window && !other->closed) {
                next = other.get();
                break;
            }
        if (!next)
            next = create(false);
        activate(next);
        reap(value);
        refresh();
        return;
    }
}
void ApplicationSessions::reap(Session *session) {
    if (session->retiring)
        return;
    if (session->page)
        for (const auto &name : {"settingsWindow", "aboutWindow"})
            if (auto *popup = session->page->findChild<QQuickWindow *>(name);
                popup && popup->isVisible())
                return;
    // Completed results still belong to the global task entry. Keep their
    // controller until application exit so output/password/conflict routing
    // never falls back to another session's numerically equal task identity.
    if (!session->controller || !session->controller->tasks().isEmpty() ||
        session->controller->activeTaskCount() || session->controller->taskStarting())
        return;
    session->retiring = true;
    if (session->controller->requestClose())
        QTimer::singleShot(0, session->controller, &BrowserController::readyToClose);
    // Otherwise the receiver emits readyToClose after its asynchronous join.
}
void ApplicationSessions::closeWindow(QQuickWindow *window) {
    if (m_quitting)
        return;
    int visibleWindows = 0;
    for (auto *candidate : windows())
        if (!candidate->property("retiredWindow").toBool())
            ++visibleWindows;
    if (visibleWindows <= 1) {
        if (m_desktop && m_desktop->trayAvailable()) {
            focusWindow(window);
            m_desktop->hideToTray();
            return;
        }
        requestQuit();
        return;
    }
    window->setProperty("retiredWindow", true);
    window->hide();
    for (const auto &session : m_sessions)
        if (session->window == window) {
            session->closed = true;
            QMetaObject::invokeMethod(session->page, "hideSessionWindows");
            reap(session.get());
        }
    for (auto *candidate : windows())
        if (!candidate->property("retiredWindow").toBool()) {
            focusWindow(candidate);
            break;
        }
    refresh();
}
void ApplicationSessions::command(const QString &command) {
    if (!m_active)
        return;
    if (command == "closeArchive") {
        if (auto *session = current())
            closeSession(session->id);
        return;
    }
    if (command == "settings" || command == "about") {
        for (const auto &session : m_sessions) {
            auto *popup = session->page
                              ? session->page->findChild<QQuickWindow *>(command + "Window")
                              : nullptr;
            if (popup && popup->isVisible()) {
                WindowActivation::reveal(popup);
                return;
            }
        }
    }
    if (auto *session = current())
        QMetaObject::invokeMethod(session->page, "runDesktopCommand", Q_ARG(QVariant, command));
}
QVariantList ApplicationSessions::tasks() const {
    QVariantList result;
    for (const auto &session : m_sessions)
        if (session->controller) {
            for (const auto &value : session->controller->tasks()) {
                auto task = value.toMap();
                task["backendId"] = task.value("id");
                task["id"] = session->id + '/' + task.value("id").toString();
                result.append(task);
            }
        }
    return result;
}
int ApplicationSessions::activeTaskCount() const {
    int count = 0;
    for (const auto &session : m_sessions)
        if (session->controller)
            count += session->controller->activeTaskCount();
    return count;
}
void ApplicationSessions::showTask(const QString &identity) {
    const auto slash = identity.indexOf('/');
    for (const auto &session : m_sessions)
        if (session->id == identity.left(slash)) {
            if (session->closed)
                session->window->setProperty("retiredWindow", false);
            activate(session.get());
            QMetaObject::invokeMethod(session->page, "showTask",
                                      Q_ARG(QString, identity.mid(slash + 1)));
            return;
        }
}
void ApplicationSessions::refresh() {
    for (auto *window : windows()) {
        QVariantList tabs;
        const auto page = window->property("activePage").value<QObject *>();
        for (const auto &session : m_sessions)
            if (session->window == window && !session->closed)
                tabs.append(QVariantMap{{"id", session->id},
                                        {"title", session->controller->title()},
                                        {"selected", session->page == page}});
        if (window->property("sessionTabs").toList() != tabs)
            window->setProperty("sessionTabs", tabs);
    }
    m_tasks.sync(tasks());
    emit tasksChanged();
    emit sessionsChanged();
    if (m_wait && !activeTaskCount())
        confirmQuit(false);
}
void ApplicationSessions::requestQuit() {
    if (m_quitting || !m_active)
        return;
    if (activeTaskCount()) {
        QMetaObject::invokeMethod(m_active, "presentQuit");
        return;
    }
    confirmQuit(false);
}
void ApplicationSessions::requestRestart() {
    m_restart = true;
    requestQuit();
}
void ApplicationSessions::confirmQuit(bool wait) {
    m_wait = wait;
    if (wait && activeTaskCount())
        return;
    m_quitting = true;
    for (const auto &session : m_sessions)
        if (session->controller)
            session->controller->requestClose();
    checkQuit();
}
void ApplicationSessions::checkQuit() {
    if (!m_quitting || m_quitEmitted)
        return;
    for (const auto &session : m_sessions)
        if (session->controller && !session->controller->shutdownComplete())
            return;
    m_quitEmitted = true;
    for (auto *window : windows()) {
        window->setProperty("allowManagedClose", true);
        window->close();
    }
    emit readyToQuit();
}
