// apps/desktop/ApplicationSessions.cpp

#include "ApplicationSessions.h"
#include "DesktopIntegration.h"
#include "UiPreferences.h"
#include "WindowActivation.h"
#include <QApplication>
#include <QDialog>
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
    for (const auto &connection : m_modalConnections)
        disconnect(connection);
    delete m_limitDialog.data();
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
    m_presentationReady |= window->isVisible();
    connect(window, &QWindow::visibleChanged, this, [this](bool visible) {
        if (!visible || m_presentationReady)
            return;
        m_presentationReady = true;
        for (auto *candidate : windows())
            if (!candidate->property("retiredWindow").toBool())
                WindowActivation::reveal(candidate);
        WindowActivation::reveal(m_active);
    });
    connect(window, &QWindow::activeChanged, this, [this, window] {
        if (window->isActive())
            focusWindow(window);
    });
    auto *page = window->property("activePage").value<QObject *>();
    auto *session = add(window, page);
    activate(session, false);
}
ApplicationSessions::Session *ApplicationSessions::add(QQuickWindow *window, QObject *page) {
    if (!page)
        return nullptr;
    auto value = std::make_unique<Session>();
    auto *session = value.get();
    session->id = QUuid::createUuid().toString(QUuid::WithoutBraces);
    session->window = window;
    for (const auto &existing : m_sessions)
        if (existing->window == window && !existing->closed)
            session->tabOrder = std::max(session->tabOrder, existing->tabOrder + 1);
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
    connect(session->controller, &BrowserController::changed, this, [this, session] {
        if (session->page && !session->controller->browsingBusy())
            session->page->setProperty("externalFileOpening", false);
        refresh();
    });
    connect(session->controller, &BrowserController::tasksChanged, this, [this] {
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
            for (auto *candidate : windows())
                ensureWelcome(candidate);
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
ApplicationSessions::Session *ApplicationSessions::create(bool newWindow, QQuickWindow *target) {
    if (m_quitting)
        return nullptr;
    if (m_sessions.size() >= 32) {
        if (!m_limitDialog) {
            m_limitDialog = new QMessageBox(QMessageBox::Information, QStringLiteral("xdzip"),
                                            qtTrId("sessions.limit"), QMessageBox::Ok);
            m_limitDialog->setAttribute(Qt::WA_DeleteOnClose);
            m_limitDialog->setWindowModality(Qt::ApplicationModal);
            m_limitDialog->winId();
            m_limitDialog->windowHandle()->setTransientParent(m_active);
            m_limitDialog->show();
        }
        m_limitDialog->raise();
        m_limitDialog->activateWindow();
        return nullptr;
    }
    if (newWindow || (!target && !m_active)) {
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
    auto *owner = target ? target : m_active.data();
    QQmlComponent component(m_engine, QUrl("qrc:/xdzip/ui/SessionPage.qml"));
    auto *page = qobject_cast<QQuickItem *>(
        component.createWithInitialProperties({{"windowHost", QVariant::fromValue(owner)}}));
    if (!page)
        return nullptr;
    page->setParent(owner);
    page->setParentItem(owner->contentItem());
    return add(owner, page);
}
void ApplicationSessions::ensureWelcome(QQuickWindow *window) {
    if (m_quitting || !window || !window->property("needsWelcome").toBool() ||
        m_sessions.size() >= 32)
        return;
    auto *oldPage = window->property("activePage").value<QObject *>();
    auto *blank = create(false, window);
    if (!blank)
        return;
    window->setProperty("needsWelcome", false);
    if (oldPage)
        oldPage->setProperty("visible", false);
    blank->page->setProperty("visible", true);
    window->setProperty("activePage", QVariant::fromValue(blank->page.data()));
    if (m_active == window)
        focusWindow(window);
    for (const auto &session : m_sessions)
        if (session->page == oldPage && session->closed)
            reap(session.get());
    refresh();
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
    session->page->setProperty("externalFileOpening", true);
    session->controller->openPath(QFileInfo(path).absoluteFilePath());
    activate(session);
}
void ApplicationSessions::activate(Session *session, bool reveal) {
    if (!session || session->retiring || !session->window || !session->page)
        return;
    cancelPendingActivation();
    const auto generation = m_activationGeneration;
    session->closed = false;
    session->page->setProperty("sessionClosed", false);
    session->page->setProperty("closedPage", false);
    auto *window = session->window.data();
    window->setProperty("needsWelcome", false);
    window->setProperty("retiredWindow", false);
    for (const auto &other : m_sessions)
        if (other->window == window && other->page)
            other->page->setProperty("visible", other.get() == session);
    window->setProperty("activePage", QVariant::fromValue(session->page.data()));
    focusWindow(window);
    refresh();
    if (!reveal || !m_presentationReady)
        return;
    const auto wake = [this, id = session->id, generation] {
        QTimer::singleShot(0, this, [this, id, generation] {
            if (!m_quitting && generation == m_activationGeneration)
                activateId(id);
        });
    };
    if (auto *widget = QApplication::activeModalWidget()) {
        widget->raise();
        widget->activateWindow();
        if (auto *dialog = qobject_cast<QDialog *>(widget))
            m_modalConnections.append(
                connect(dialog, &QDialog::finished, this, [wake](int) { wake(); }));
        m_modalConnections.append(connect(widget, &QObject::destroyed, this, wake));
    } else if (auto *modal = QGuiApplication::modalWindow()) {
        WindowActivation::reveal(modal);
        m_modalConnections.append(
            connect(modal, &QWindow::visibleChanged, this, [wake](bool visible) {
                if (!visible)
                    wake();
            }));
        m_modalConnections.append(connect(modal, &QObject::destroyed, this, wake));
    } else
        WindowActivation::reveal(window);
}
void ApplicationSessions::cancelPendingActivation() {
    ++m_activationGeneration;
    for (const auto &connection : m_modalConnections)
        disconnect(connection);
    m_modalConnections.clear();
}
void ApplicationSessions::focusWindow(QQuickWindow *window) {
    if (!window || m_quitting)
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
void ApplicationSessions::moveTab(const QString &id, int targetIndex) {
    if (m_quitting || targetIndex < 0)
        return;
    const auto found = std::find_if(m_sessions.begin(), m_sessions.end(), [&](const auto &value) {
        return value->id == id && !value->closed;
    });
    if (found == m_sessions.end())
        return;
    auto ordered = visibleTabs((*found)->window);
    const auto from = std::find(ordered.begin(), ordered.end(), found->get()) - ordered.begin();
    if (targetIndex >= int(ordered.size()) || targetIndex == from)
        return;
    if (from < targetIndex)
        std::rotate(ordered.begin() + from, ordered.begin() + from + 1,
                    ordered.begin() + targetIndex + 1);
    else
        std::rotate(ordered.begin() + targetIndex, ordered.begin() + from,
                    ordered.begin() + from + 1);
    // Presentation order must not change controller ownership or admission order.
    for (size_t index = 0; index < ordered.size(); ++index)
        ordered[index]->tabOrder = index;
    refresh();
}
void ApplicationSessions::closeSession(const QString &id) {
    for (const auto &session : m_sessions) {
        if (session->id != id || session->closed)
            continue;
        closeTabSet(session.get(), {session.get()});
        return;
    }
}
void ApplicationSessions::closeOtherSessions(const QString &id) {
    closeAround(id, false);
}
void ApplicationSessions::closeSessionsToRight(const QString &id) {
    closeAround(id, true);
}
void ApplicationSessions::closeAround(const QString &id, bool rightOnly) {
    if (QApplication::activeModalWidget() || QGuiApplication::modalWindow())
        return;
    const auto found = std::find_if(m_sessions.begin(), m_sessions.end(), [&](const auto &value) {
        return value->id == id && !value->closed;
    });
    if (found == m_sessions.end())
        return;
    auto *anchor = found->get();
    const auto ordered = visibleTabs(anchor->window);
    std::vector<Session *> targets;
    bool afterAnchor = false;
    for (auto *candidate : ordered) {
        if (candidate == anchor) {
            afterAnchor = true;
            continue;
        }
        if (!rightOnly || afterAnchor)
            targets.push_back(candidate);
    }
    closeTabSet(anchor, targets);
}
void ApplicationSessions::closeTabSet(Session *anchor, const std::vector<Session *> &targets) {
    if (m_quitting || !anchor || !anchor->window || targets.empty())
        return;
    cancelPendingActivation();
    auto *owner = anchor->window.data();
    const auto ordered = visibleTabs(owner);
    const auto activePage = owner->property("activePage").value<QObject *>();
    const auto closing = [&](Session *candidate) {
        return std::find(targets.begin(), targets.end(), candidate) != targets.end();
    };
    Session *next = nullptr;
    for (auto *candidate : ordered)
        if (!closing(candidate) && candidate->page == activePage)
            next = candidate;
    if (!next && !closing(anchor))
        next = anchor;
    if (!next) {
        const auto position = std::find(ordered.begin(), ordered.end(), anchor);
        for (auto iterator = position; iterator != ordered.end(); ++iterator)
            if (!closing(*iterator)) {
                next = *iterator;
                break;
            }
        if (!next)
            for (auto iterator = position; iterator != ordered.begin();) {
                --iterator;
                if (!closing(*iterator)) {
                    next = *iterator;
                    break;
                }
            }
    }
    for (auto *value : targets) {
        value->closed = true;
        value->page->setProperty("sessionClosed", true);
        value->page->setProperty("visible", false);
        QMetaObject::invokeMethod(value->page, "hideSessionWindows");
    }
    if (!next && m_sessions.size() < 32)
        next = create(false, owner);
    if (next && next->page != activePage)
        activate(next);
    else if (!next) {
        owner->setProperty("needsWelcome", true);
        anchor->page->setProperty("closedPage", true);
        anchor->page->setProperty("visible", true);
        owner->setProperty("activePage", QVariant::fromValue(anchor->page.data()));
    }
    for (auto *value : targets)
        reap(value);
    refresh();
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
    if (m_quitting || !window)
        return;
    if (QApplication::activeModalWidget() || QGuiApplication::modalWindow()) {
        if (m_desktop)
            m_desktop->restore();
        return;
    }
    cancelPendingActivation();
    int visibleWindows = 0;
    for (auto *candidate : windows())
        if (!candidate->property("retiredWindow").toBool())
            ++visibleWindows;
    if (visibleWindows <= 1) {
        if (m_desktop && m_desktop->trayAvailable()) {
            focusWindow(window);
            // Retain only the presentation shell while connections with tasks
            // keep their identities. A fresh blank session replaces it as soon
            // as an asynchronous join frees a slot; never create a 33rd runtime.
            auto *shell = window->property("activePage").value<QObject *>();
            window->setProperty("needsWelcome", true);
            for (const auto &session : m_sessions)
                if (session->window == window) {
                    session->closed = true;
                    session->page->setProperty("sessionClosed", true);
                    session->page->setProperty("closedPage", true);
                    session->page->setProperty("visible", session->page == shell);
                    QMetaObject::invokeMethod(session->page, "hideSessionWindows");
                    if (session->page != shell)
                        reap(session.get());
                    else if (session->controller->browsingBusy())
                        session->controller->cancel();
                }
            ensureWelcome(window);
            refresh();
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
            session->page->setProperty("sessionClosed", true);
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
    else if (QStringList{"open", "new", "settings", "about"}.contains(command))
        QMetaObject::invokeMethod(m_active->property("activePage").value<QObject *>(),
                                  "runDesktopCommand", Q_ARG(QVariant, command));
}
void ApplicationSessions::showAboutFor(QObject *page) {
    if (!page)
        return;
    for (const auto &session : m_sessions) {
        auto *popup =
            session->page ? session->page->findChild<QQuickWindow *>("aboutWindow") : nullptr;
        if (popup && popup->isVisible()) {
            WindowActivation::reveal(popup);
            return;
        }
    }
    for (const auto &session : m_sessions)
        if (session->page == page && !session->retiring) {
            QMetaObject::invokeMethod(page, "presentAbout");
            return;
        }
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
    if (slash <= 0 || slash == identity.size() - 1 || m_quitting)
        return;
    const auto taskId = identity.mid(slash + 1);
    for (const auto &session : m_sessions)
        if (session->id == identity.left(slash) && !session->retiring && session->page &&
            session->controller) {
            const auto tasks = session->controller->tasks();
            if (std::none_of(tasks.begin(), tasks.end(), [&taskId](const QVariant &value) {
                    return value.toMap().value("id").toString() == taskId;
                }))
                return;
            // Retained controllers own task dialogs; accessing a task never
            // reopens its closed browsing page or changes the selected tab.
            QMetaObject::invokeMethod(session->page, "showTask",
                                      Q_ARG(QString, taskId));
            return;
        }
}
bool ApplicationSessions::hasFinishedTasks() const {
    return std::any_of(m_sessions.cbegin(), m_sessions.cend(), [](const auto &session) {
        return session->controller && session->controller->hasFinishedTasks();
    });
}
void ApplicationSessions::clearFinishedTasks() {
    for (const auto &session : m_sessions)
        if (session->controller)
            session->controller->clearFinishedTasks();
    for (const auto &session : m_sessions)
        if (session->closed)
            reap(session.get());
}
std::vector<ApplicationSessions::Session *>
ApplicationSessions::visibleTabs(QQuickWindow *window) const {
    std::vector<Session *> result;
    for (const auto &session : m_sessions)
        if (session->window == window && !session->closed)
            result.push_back(session.get());
    std::stable_sort(result.begin(), result.end(), [](const auto *left, const auto *right) {
        return left->tabOrder < right->tabOrder;
    });
    return result;
}
void ApplicationSessions::refresh() {
    for (auto *window : windows()) {
        QVariantList tabs;
        const auto page = window->property("activePage").value<QObject *>();
        for (const auto *session : visibleTabs(window))
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
    if (m_updatePending && m_quitEmitted) {
        for (auto *window : windows()) {
            window->setProperty("allowManagedClose", true);
            window->close();
        }
        emit readyToQuit();
        return;
    }
    if (m_quitting || !m_active)
        return;
    if (activeTaskCount()) {
        m_wait = false;
        setQuitDecisionPending(true);
        if (!QMetaObject::invokeMethod(m_active, "presentQuit"))
            setQuitDecisionPending(false);
        return;
    }
    confirmQuit(false);
}
void ApplicationSessions::requestRestart() {
    m_restart = true;
    requestQuit();
}
bool ApplicationSessions::prepareForUpdate() {
    if (m_quitting || QApplication::activeModalWidget() || QGuiApplication::modalWindow())
        return false;
    for (const auto &session : m_sessions)
        if (session->controller &&
            (session->controller->activeTaskCount() || session->controller->taskStarting()))
            return false;
    m_updatePending = true;
    m_restart = false;
    m_quitting = true;
    setQuitDecisionPending(true);
    cancelPendingActivation();
    emit closing();
    for (const auto &session : m_sessions)
        if (session->controller)
            session->controller->requestClose();
    checkQuit();
    return true;
}
void ApplicationSessions::confirmQuit(bool wait) {
    if (m_quitting)
        return;
    m_wait = wait;
    if (wait && activeTaskCount()) {
        setQuitDecisionPending(false);
        return;
    }
    m_quitting = true;
    setQuitDecisionPending(true);
    emit closing();
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
    if (m_updatePending) {
        emit readyForUpdate();
        return;
    }
    for (auto *window : windows()) {
        window->setProperty("allowManagedClose", true);
        window->close();
    }
    emit readyToQuit();
}
