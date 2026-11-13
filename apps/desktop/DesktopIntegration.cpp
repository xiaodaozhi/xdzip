#include "DesktopIntegration.h"
#include "ApplicationSessions.h"
#include "BrowserController.h"
#include "UiPreferences.h"
#include "WindowActivation.h"
#include <QAction>
#include <QApplication>
#include <QClipboard>
#include <QDesktopServices>
#include <QEvent>
#include <QFileInfo>
#include <QHash>
#include <QMenu>
#include <QMenuBar>
#include <QMimeData>
#include <QPainter>
#include <QQmlEngine>
#include <QQuickItem>
#include <QQuickWindow>
#include <QTimer>
#include <QUrl>
#include <algorithm>

#ifdef Q_OS_MACOS
bool macNativeEditorActive();
bool macNativeEditorEnabled(const QString &command);
void macNativeEditorExecute(const QString &command);
void syncMacApplicationMenuLanguage();
void *createMacQuitMenuBridge(DesktopIntegration *owner);
void syncMacQuitMenuBridge(void *bridge);
void destroyMacQuitMenuBridge(void *bridge);
#endif

namespace {
void hideForTray(QWindow *window) {
    window->hide();
#ifdef Q_OS_WIN
    auto *quick = qobject_cast<QQuickWindow *>(window);
    if (!quick || QGuiApplication::platformName() != QStringLiteral("windows"))
        return;
    // DWM may reuse the last presented image before Quick paints after a show.
    // Keep the HWND/state and reveal it only after the first fresh frame, without
    // readback, nested event processing or a timer. Re-arming deletes old callbacks.
    const auto saved = window->property("xdzipTrayRestoreOpacity");
    const qreal opacity = saved.isValid() ? saved.toReal() : window->opacity();
    delete window->findChild<QObject *>(QStringLiteral("xdzipTrayPresentation"),
                                        Qt::FindDirectChildrenOnly);
    auto *gate = new QObject(window);
    gate->setObjectName(QStringLiteral("xdzipTrayPresentation"));
    window->setProperty("xdzipTrayRestoreOpacity", opacity);
    window->setOpacity(0);
    QObject::connect(quick, &QQuickWindow::afterAnimating, gate, [quick, gate, opacity] {
        if (!quick->isVisible() || quick->windowState() == Qt::WindowMinimized ||
            gate->property("armed").toBool())
            return;
        gate->setProperty("armed", true);
        // afterAnimating runs on the GUI thread before this frame's sync. A new
        // connection cannot consume a queued swap from the previous hidden frame.
        QObject::connect(
            quick, &QQuickWindow::frameSwapped, gate,
            [quick, gate, opacity] {
                if (!quick->isVisible() || quick->windowState() == Qt::WindowMinimized ||
                    gate->property("completed").toBool())
                    return;
                gate->setProperty("completed", true);
                quick->setOpacity(opacity);
                quick->setProperty("xdzipTrayRestoreOpacity", QVariant());
                gate->deleteLater();
            },
            Qt::QueuedConnection);
    });
#endif
}
QString text(const QString &command) {
    using Translation = QString (*)();
    static const QHash<QString, Translation> labels{
        {QStringLiteral("application"), [] { return qtTrId("desktop.application"); }},
        {QStringLiteral("file"), [] { return qtTrId("desktop.file"); }},
        {QStringLiteral("edit"), [] { return qtTrId("desktop.edit"); }},
        {QStringLiteral("view"), [] { return qtTrId("desktop.view"); }},
        {QStringLiteral("operation"), [] { return qtTrId("desktop.operation"); }},
        {QStringLiteral("window"), [] { return qtTrId("desktop.window"); }},
        {QStringLiteral("help"), [] { return qtTrId("desktop.help"); }},
        {QStringLiteral("guide"), [] { return qtTrId("desktop.guide"); }},
        {QStringLiteral("feedback"), [] { return qtTrId("desktop.feedback"); }},
        {QStringLiteral("about"), [] { return qtTrId("desktop.about"); }},
        {QStringLiteral("settings"), [] { return qtTrId("desktop.settings"); }},
        {QStringLiteral("quit"), [] { return qtTrId("desktop.quit"); }},
        {QStringLiteral("show"), [] { return qtTrId("desktop.show"); }},
        {QStringLiteral("open"), [] { return qtTrId("desktop.open"); }},
        {QStringLiteral("new"), [] { return qtTrId("desktop.new"); }},
        {QStringLiteral("recent"), [] { return qtTrId("desktop.recent"); }},
        {QStringLiteral("emptyRecent"), [] { return qtTrId("desktop.emptyRecent"); }},
        {QStringLiteral("closeArchive"), [] { return qtTrId("desktop.closeArchive"); }},
        {QStringLiteral("closeWindow"), [] { return qtTrId("desktop.closeWindow"); }},
        {QStringLiteral("undo"), [] { return qtTrId("desktop.undo"); }},
        {QStringLiteral("redo"), [] { return qtTrId("desktop.redo"); }},
        {QStringLiteral("cut"), [] { return qtTrId("desktop.cut"); }},
        {QStringLiteral("copy"), [] { return qtTrId("desktop.copy"); }},
        {QStringLiteral("paste"), [] { return qtTrId("desktop.paste"); }},
        {QStringLiteral("selectAll"), [] { return qtTrId("desktop.selectAll"); }},
        {QStringLiteral("find"), [] { return qtTrId("desktop.find"); }},
        {QStringLiteral("list"), [] { return qtTrId("desktop.list"); }},
        {QStringLiteral("grid"), [] { return qtTrId("desktop.grid"); }},
        {QStringLiteral("inspector"), [] { return qtTrId("desktop.inspector"); }},
        {QStringLiteral("testArchive"), [] { return qtTrId("desktop.testArchive"); }},
        {QStringLiteral("extractAll"), [] { return qtTrId("desktop.extractAll"); }},
        {QStringLiteral("extractSelected"), [] { return qtTrId("desktop.extractSelected"); }},
        {QStringLiteral("preflight"), [] { return qtTrId("desktop.preflight"); }},
        {QStringLiteral("minimize"), [] { return qtTrId("desktop.minimize"); }},
        {QStringLiteral("zoom"), [] { return qtTrId("desktop.zoom"); }},
        {QStringLiteral("tasks"), [] { return qtTrId("desktop.tasks"); }},
        {QStringLiteral("noTasks"), [] { return qtTrId("desktop.noTasks"); }},
        {QStringLiteral("task"), [] { return qtTrId("desktop.task"); }},
        {QStringLiteral("createTask"), [] { return qtTrId("desktop.createTask"); }},
        {QStringLiteral("moveTask"), [] { return qtTrId("desktop.moveTask"); }},
        {QStringLiteral("extractTask"), [] { return qtTrId("desktop.extractTask"); }},
        {QStringLiteral("checkTask"), [] { return qtTrId("desktop.checkTask"); }},
        {QStringLiteral("trayEnabled"), [] { return qtTrId("desktop.trayEnabled"); }},
        {QStringLiteral("trayTip"), [] { return qtTrId("desktop.trayTip"); }},
    };
    const auto translate = labels.value(command, nullptr);
    return translate ? translate() : QString{};
}
QString taskStateText(const QString &status) {
    static const QHash<QString, const char *> labels{
        {"queued", "ui.d6f766f2adf5"},     {"preparing", "ui.b53be09930f4"},
        {"running", "ui.dc9591e56d50"},    {"verifying", "ui.784f9b54ae2b"},
        {"waiting", "ui.8fb3a3261b43"},    {"waiting_password", "password.waiting"},
        {"committing", "ui.a1a1720d3eb5"}, {"cancelling", "ui.72eea71385de"},
        {"cleaning", "ui.718b6372d7fb"},   {"succeeded", "ui.f28461bb49c8"},
        {"failed", "ui.6707de42c29d"},     {"cancelled", "ui.a37778f17c5f"}};
    return qtTrId(labels.value(status, "ui.b53be09930f4"));
}
bool ownedWindow(QWindow *window, QWindow *main) {
    if (!qobject_cast<QQuickWindow *>(window) || window->type() != Qt::Window)
        return false;
    for (auto *owner = window; owner; owner = owner->transientParent())
        if (owner == main)
            return true;
    return false;
}
QIcon statusIcon(const QString &status) {
    QColor color = status == "succeeded"                                 ? QColor("#21a86a")
                   : status == "failed"                                  ? QColor("#df4b55")
                   : status == "waiting" || status == "waiting_password" ? QColor("#d08a18")
                   : status == "cancelled"                               ? QColor("#8893a6")
                                                                         : QColor("#2364ed");
    QPixmap image(32, 32);
    image.fill(Qt::transparent);
    QPainter painter(&image);
    painter.setRenderHint(QPainter::Antialiasing);
    painter.setPen(Qt::NoPen);
    painter.setBrush(color);
    painter.drawEllipse(QRectF(8, 8, 16, 16));
    return QIcon(image);
}
bool inputMethod(QObject *object, const char *method) {
    return object && object->metaObject()->indexOfMethod(method) >= 0;
}
} // namespace

DesktopIntegration::DesktopIntegration(QQuickWindow *window, BrowserController *controller,
                                       UiPreferences *settings, bool interactive,
                                       std::function<bool()> availability)
    : m_window(window), m_controller(controller), m_settings(settings),
      m_trayMenu(std::make_unique<QMenu>()), m_availability(std::move(availability)),
      m_interactive(interactive) {
    QQmlEngine::setObjectOwnership(this, QQmlEngine::CppOwnership);
    if (window)
        window->setProperty("desktopShell", QVariant::fromValue(static_cast<QObject *>(this)));
    m_tray.setContextMenu(m_trayMenu.get());
    connect(m_trayMenu.get(), &QMenu::aboutToShow, this, [this] { m_trayInteraction = true; });
    connect(m_trayMenu.get(), &QMenu::aboutToHide, this,
            [this] { QTimer::singleShot(0, this, [this] { m_trayInteraction = false; }); });
    add(m_trayMenu.get(), "show");
    add(m_trayMenu.get(), "open");
    add(m_trayMenu.get(), "new");
    m_trayMenu->addSeparator();
    auto *tasks = m_trayMenu->addMenu(text("tasks").arg(0));
    m_taskMenus.append(tasks);
    m_trayMenu->addSeparator();
    add(m_trayMenu.get(), "settings");
    add(m_trayMenu.get(), "about");
    m_trayMenu->addSeparator();
    add(m_trayMenu.get(), "quit");
    if (nativeMenuAvailable()) {
        m_bar = std::make_unique<QMenuBar>();
        m_bar->setNativeMenuBar(true);
        const auto addMenu = [this](const QString &command) {
            auto *menu = m_bar->addMenu(text(command));
            m_menuTitles.insert(command, menu);
            return menu;
        };
        auto *application = m_bar->addMenu(QStringLiteral("xdzip"));
        add(application, "about")->setMenuRole(QAction::AboutRole);
        add(application, "settings", QKeySequence::Preferences)
            ->setMenuRole(QAction::PreferencesRole);
        add(application, "quit", QKeySequence::Quit)->setMenuRole(QAction::QuitRole);
        auto *file = addMenu("file");
        add(file, "open", QKeySequence::Open);
        add(file, "new", QKeySequence::New);
        m_recent = file->addMenu(text("recent"));
        connect(m_recent, &QMenu::aboutToShow, this, &DesktopIntegration::rebuildRecent);
        file->addSeparator();
        add(file, "closeArchive", QKeySequence(Qt::CTRL | Qt::SHIFT | Qt::Key_W));
        add(file, "closeWindow", QKeySequence::Close);
        auto *edit = addMenu("edit");
        add(edit, "undo", QKeySequence::Undo);
        add(edit, "redo", QKeySequence::Redo);
        edit->addSeparator();
        add(edit, "cut", QKeySequence::Cut);
        add(edit, "copy", QKeySequence::Copy);
        add(edit, "paste", QKeySequence::Paste);
        add(edit, "selectAll", QKeySequence::SelectAll);
        add(edit, "find", QKeySequence::Find);
        auto *view = addMenu("view");
        add(view, "list")->setCheckable(true);
        add(view, "grid")->setCheckable(true);
        view->addSeparator();
        add(view, "inspector")->setCheckable(true);
        // AppKit appends Full Screen with an icon. Keep its icon column out of our group.
        view->setSeparatorsCollapsible(false);
        view->addSeparator();
        auto *operation = addMenu("operation");
        add(operation, "testArchive");
        add(operation, "extractAll");
        add(operation, "extractSelected");
        add(operation, "preflight");
        m_windowMenu = addMenu("window");
        add(m_windowMenu, "minimize", QKeySequence(Qt::CTRL | Qt::Key_M));
        add(m_windowMenu, "zoom");
        m_windowMenu->addSeparator();
        add(m_windowMenu, "show");
        m_taskMenus.append(m_windowMenu->addMenu(text("tasks").arg(0)));
        m_windowMenu->addSeparator();
        connect(m_windowMenu, &QMenu::aboutToShow, this, &DesktopIntegration::rebuildWindows);
        auto *help = addMenu("help");
        add(help, "guide");
        add(help, "feedback");
        help->addSeparator();
        add(help, "about");
        for (auto *menu : m_bar->findChildren<QMenu *>())
            connect(menu, &QMenu::aboutToShow, this, &DesktopIntegration::syncActions);
    }
    for (auto *menu : m_taskMenus)
        connect(menu, &QMenu::aboutToShow, this, [this, menu] { rebuildTasks(menu); });
    connect(m_trayMenu.get(), &QMenu::aboutToShow, this, &DesktopIntegration::syncActions);
    if (controller)
        connect(controller, &BrowserController::changed, this, &DesktopIntegration::syncActions);
    if (controller)
        connect(controller, &BrowserController::tasksChanged, this,
                &DesktopIntegration::syncActions);
    m_menuLanguage = settings->activeLanguage();
    connect(settings, &UiPreferences::savedChanged, this, [this] {
        retranslateMenus();
        refresh();
    });
    connect(settings, &UiPreferences::recentChanged, this, [this] {
        if (m_recent && m_recent->isVisible())
            rebuildRecent();
    });
    connect(&m_tray, &QSystemTrayIcon::activated, this,
            [this](QSystemTrayIcon::ActivationReason reason) {
                Q_UNUSED(this);
#ifndef Q_OS_MACOS
                if (reason == QSystemTrayIcon::Trigger || reason == QSystemTrayIcon::DoubleClick)
                    restore();
#else
        Q_UNUSED(reason);
#endif
            });
    if (window)
        connect(window, &QWindow::activeChanged, this, &DesktopIntegration::syncActions);
    if (window)
        connect(window, &QQuickWindow::activeFocusItemChanged, this,
                &DesktopIntegration::syncActions);
    connect(QGuiApplication::clipboard(), &QClipboard::dataChanged, this,
            &DesktopIntegration::syncActions);
    connect(qGuiApp, &QGuiApplication::applicationStateChanged, this,
            [this](Qt::ApplicationState state) {
                Q_UNUSED(this);
#ifdef Q_OS_MACOS
                if (m_hidden && state == Qt::ApplicationActive)
                    QTimer::singleShot(0, this, [this] {
                        const auto windows = QGuiApplication::topLevelWindows();
                        const bool settingsVisible = std::any_of(
                            windows.cbegin(), windows.cend(), [](const QWindow *window) {
                                return window && window->isVisible() &&
                                       window->objectName() == "settingsWindow";
                            });
                        if (m_hidden && !m_trayInteraction && !m_trayMenu->isVisible() &&
                            !settingsVisible && (!m_sessions || !m_sessions->isClosing()))
                            restore();
                    });
#else
                Q_UNUSED(state);
#endif
            });
    qApp->installEventFilter(this);
#ifdef Q_OS_MACOS
    if (nativeMenuAvailable()) {
        m_macQuitBridge = createMacQuitMenuBridge(this);
        connect(qGuiApp, &QGuiApplication::focusWindowChanged, this, [this](QWindow *) {
            // Qt updates its native menu as the modal gains focus. Reapply the
            // native Quit policy after that update, not while it is running.
            QTimer::singleShot(0, this, &DesktopIntegration::syncActions);
        });
        QTimer::singleShot(0, this, &DesktopIntegration::syncActions);
    }
#endif
    auto *timer = new QTimer(this);
    timer->setInterval(1500);
    connect(timer, &QTimer::timeout, this, &DesktopIntegration::refresh);
    if (interactive)
        timer->start();
    updateIcon();
    refresh();
}
DesktopIntegration::~DesktopIntegration() {
#ifdef Q_OS_MACOS
    destroyMacQuitMenuBridge(m_macQuitBridge);
#endif
    qApp->removeEventFilter(this);
    if (m_window)
        m_window->setProperty("desktopShell", QVariant{});
    m_tray.hide();
    // QSystemTrayIcon does not own its QMenu. Detach before deleting the menu.
    m_tray.setContextMenu(nullptr);
}
void DesktopIntegration::setSessions(ApplicationSessions *sessions) {
    m_sessions = sessions;
    connect(sessions, &ApplicationSessions::tasksChanged, this, &DesktopIntegration::syncActions);
    connect(this, &DesktopIntegration::availabilityChanged, sessions,
            &ApplicationSessions::shellAvailabilityChanged);
}
void DesktopIntegration::bind(QQuickWindow *window, BrowserController *controller) {
    m_window = window;
    m_controller = controller;
    window->setProperty("desktopShell", QVariant::fromValue(static_cast<QObject *>(this)));
    if (controller)
        connect(controller, &BrowserController::changed, this, &DesktopIntegration::syncActions,
                Qt::UniqueConnection);
    if (controller)
        connect(controller, &BrowserController::tasksChanged, this,
                &DesktopIntegration::syncActions, Qt::UniqueConnection);
    syncActions();
}
bool DesktopIntegration::nativeMenuAvailable() const {
#ifdef Q_OS_MACOS
    return QGuiApplication::platformName() == QStringLiteral("cocoa");
#else
    return false;
#endif
}
QAction *DesktopIntegration::add(QMenu *menu, const QString &command,
                                 const QKeySequence &shortcut) {
    auto *item = new QAction(text(command), menu);
    item->setObjectName(QStringLiteral("desktop-") + command);
    item->setData(command);
    item->setMenuRole(QStringList{"cut", "copy", "paste", "selectAll"}.contains(command)
                          ? QAction::TextHeuristicRole
                          : QAction::NoRole);
    // Cocoa resolves menu roles on insertion; set NoRole before it can merge About.
    menu->addAction(item);
    if (!shortcut.isEmpty())
        item->setShortcut(shortcut);
    connect(item, &QAction::triggered, this, [this, command] { execute(command); });
    m_actions.append(item);
    return item;
}
QAction *DesktopIntegration::action(const QString &command) const {
    for (auto *item : m_actions)
        if (item->data().toString() == command)
            return item;
    return nullptr;
}
bool DesktopIntegration::blocked() const {
    return QApplication::activeModalWidget() || QGuiApplication::modalWindow();
}
bool DesktopIntegration::textEditor() const {
#ifdef Q_OS_MACOS
    // A native panel can own the responder while Qt still remembers its caller.
    if (macNativeEditorActive())
        return false;
#endif
    auto *window = qobject_cast<QQuickWindow *>(QGuiApplication::focusWindow());
    auto *item = window ? window->activeFocusItem() : nullptr;
    return inputMethod(item, "selectAll()") && item->property("text").isValid();
}
bool DesktopIntegration::enabled(const QString &command) const {
    if (!m_settings)
        return false;
    if (m_sessions && !m_window)
        return (command == "quit" && taskPromptMayQuit()) ||
               (QStringList{"show", "quit", "open", "new", "settings", "about"}.contains(command) &&
                !blocked());
    if (!m_window || !m_controller)
        return false;
    if (command == "quit" || command == "show" || command == "closeWindow")
        return true;
    const bool editing = textEditor();
    if (QStringList{"undo", "redo", "cut", "copy", "paste", "selectAll"}.contains(command) &&
        editing) {
        auto *window = qobject_cast<QQuickWindow *>(QGuiApplication::focusWindow());
        auto *item = window->activeFocusItem();
        if (command == "selectAll")
            return !item->property("text").toString().isEmpty();
        if (command == "copy")
            return !item->property("selectedText").toString().isEmpty();
        if (item->property("readOnly").toBool())
            return false;
        if (command == "cut")
            return !item->property("selectedText").toString().isEmpty();
        if (command == "paste")
            return QGuiApplication::clipboard()->mimeData()->hasText();
        return item->property(command == "undo" ? "canUndo" : "canRedo").toBool();
    }
#ifdef Q_OS_MACOS
    if (blocked() &&
        QStringList{"undo", "redo", "cut", "copy", "paste", "selectAll"}.contains(command))
        return macNativeEditorEnabled(command);
#endif
    if (blocked())
        return false;
    const bool ready = m_controller->ready() && !m_controller->browsingBusy() &&
                       !m_window->property("welcomeVisible").toBool();
    if (QStringList{"copy", "selectAll", "find"}.contains(command) &&
        QGuiApplication::focusWindow() != m_window)
        return false;
    if (command == "open" || command == "new")
        return m_controller->runtimeAvailable() && !m_controller->browsingBusy();
    if (command == "closeArchive" || command == "extractAll" || command == "selectAll")
        return ready;
    if (command == "extractSelected" || command == "copy")
        return ready && m_controller->selectedCount() > 0;
    if (command == "testArchive")
        return ready && !m_controller->busy() && m_controller->runtimeAvailable();
    if (command == "preflight")
        return ready && m_controller->canPreflight();
    if (command == "find")
        return m_controller->browserQueryAvailable();
    if (QStringList{"undo", "redo", "cut", "paste"}.contains(command))
        return false;
    if (command == "list" || command == "grid" || command == "inspector")
        return ready;
    return true;
}
void DesktopIntegration::execute(const QString &command) {
    if (!enabled(command))
        return;
    if (command == "quit") {
        requestQuit();
        return;
    }
    if (command == "show") {
        restore();
        return;
    }
    if (command == "closeWindow") {
        if (auto *modal = QApplication::activeModalWidget()) {
            modal->close();
            return;
        }
        auto *window = QGuiApplication::modalWindow();
        if (!window)
            window = QGuiApplication::focusWindow();
        if (window || m_window)
            (window ? window : m_window.data())->close();
        return;
    }
#ifdef Q_OS_MACOS
    if (blocked() && macNativeEditorEnabled(command)) {
        macNativeEditorExecute(command);
        return;
    }
#endif
    if (textEditor() &&
        QStringList{"undo", "redo", "cut", "copy", "paste", "selectAll"}.contains(command)) {
        auto *window = qobject_cast<QQuickWindow *>(QGuiApplication::focusWindow());
        QMetaObject::invokeMethod(window->activeFocusItem(), qPrintable(command));
        return;
    }
    if (command == "guide" || command == "feedback") {
        QDesktopServices::openUrl(QUrl(command == "guide"
                                           ? "https://github.com/xiaodaozhi/xdzip#readme"
                                           : "https://github.com/xiaodaozhi/xdzip/issues"));
        return;
    }
    // Settings and About are independent windows. A tray command must not
    // revive a hidden browser just to find their presenter.
    if (!(m_sessions && (command == "settings" || command == "about")))
        restore();
    if (blocked())
        return;
    if (m_sessions)
        m_sessions->command(command);
    else
        QMetaObject::invokeMethod(m_window, "runDesktopCommand", Q_ARG(QVariant, command));
    syncActions();
}
bool DesktopIntegration::taskPromptMayQuit() const {
    auto *modal = qobject_cast<QQuickWindow *>(QGuiApplication::modalWindow());
    return m_sessions && !QApplication::activeModalWidget() && modal &&
           (modal->objectName() == "passwordWindow" || modal->objectName() == "conflictWindow") &&
           m_sessions->ownsTaskPrompt(modal);
}
bool DesktopIntegration::nativeQuitMenuAllowed() const {
    return (m_sessions || (m_window && m_controller)) && (!blocked() || taskPromptMayQuit());
}
void DesktopIntegration::requestQuit() {
    if (!m_sessions && (!m_window || !m_controller))
        return;
    // A task prompt may be superseded by the global quit decision. Other
    // modals retain their normal protection, including native file selectors.
    if (blocked() && !taskPromptMayQuit()) {
        restore();
        return;
    }
    if (m_sessions)
        m_sessions->requestQuit();
    else
        QMetaObject::invokeMethod(m_window, "requestDesktopExit");
}
void DesktopIntegration::hideWindows() {
    if (blocked()) {
        restore();
        return;
    }
    m_hiddenWindows.clear();
    // Only product Quick windows are hidden; never manipulate native file dialogs or menus.
    for (auto *window : QGuiApplication::topLevelWindows()) {
        if (ownedWindow(window, m_window) && window->isVisible() && window != m_window) {
            m_hiddenWindows.append(window);
            hideForTray(window);
        }
    }
    m_hidden = true;
    hideForTray(m_window);
}
void DesktopIntegration::restore() {
    if (!m_window && m_sessions) {
        m_sessions->showMainWindow();
        return;
    }
    if (!m_window)
        return;
    if (auto *modal = QApplication::activeModalWidget()) {
        modal->show();
        modal->raise();
        modal->activateWindow();
        return;
    }
    if (auto *modal = QGuiApplication::modalWindow()) {
        WindowActivation::reveal(modal);
        return;
    }
    WindowActivation::reveal(m_window);
    m_hidden = false;
    for (auto &window : m_hiddenWindows)
        if (window)
            WindowActivation::reveal(window);
    m_hiddenWindows.clear();
    m_window->raise();
    m_window->requestActivate();
}
void DesktopIntegration::refresh() {
    const bool available =
        m_interactive && m_settings && m_settings->trayEnabled() && m_availability();
    if (available != m_available) {
        // Restore before removing the only visible entry point, including tray-host loss.
        if (!available && m_hidden)
            restore();
        m_available = available;
        m_tray.setVisible(available);
        QApplication::setQuitOnLastWindowClosed(!m_sessions && !available);
        emit availabilityChanged();
    }
    updateIcon();
    syncActions();
}
void DesktopIntegration::retranslateMenus() {
    const auto language = m_settings->activeLanguage();
    if (language == m_menuLanguage)
        return;
    m_menuLanguage = language;
    for (auto it = m_menuTitles.cbegin(); it != m_menuTitles.cend(); ++it)
        it.value()->setTitle(text(it.key()));
    if (m_recent) {
        m_recent->setTitle(text("recent"));
        rebuildRecent();
    }
    for (auto *item : m_actions)
        item->setText(text(item->data().toString()));
    if (m_controller && m_window)
        for (auto *menu : m_taskMenus)
            rebuildTasks(menu);
}
void DesktopIntegration::syncActions() {
    if (m_controller && m_window &&
        (m_session != m_controller->sessionId() || m_controller->browsingBusy())) {
        m_session = m_controller->sessionId();
        m_trayMenu->close();
        if (m_bar)
            for (auto *menu : m_bar->findChildren<QMenu *>())
                menu->close();
    }
    for (auto *item : m_actions) {
        const auto command = item->data().toString();
        item->setEnabled(enabled(command));
        if (m_window && (command == "list" || command == "grid"))
            item->setChecked(m_window->property("viewMode") == command);
        if (m_window && command == "inspector")
            item->setChecked(m_window->property("desktopInspectorVisible").toBool());
    }
    const auto count = m_sessions     ? m_sessions->activeTaskCount()
                       : m_controller ? m_controller->activeTaskCount()
                                      : 0;
    for (auto *menu : m_taskMenus) {
        menu->setTitle(text("tasks").arg(count));
        if (menu->isVisible()) {
            const auto tasks = m_sessions     ? m_sessions->tasks()
                               : m_controller ? m_controller->tasks()
                                              : QVariantList{};
            for (auto *item : menu->actions()) {
                for (const auto &value : tasks) {
                    const auto task = value.toMap();
                    if (item->data() != task.value("id"))
                        continue;
                    const auto state = taskStateText(task.value("status").toString());
                    item->setText(
                        text("task").arg(text(task.value("kind") == "move"      ? "moveTask"
                                              : task.value("kind") == "check"   ? "checkTask"
                                              : task.value("creating").toBool() ? "createTask"
                                                                                : "extractTask"),
                                         task.value("backendId", task.value("id")).toString(),
                                         state) +
                        (m_sessions ? qtTrId("common.separator") + task.value("archive").toString()
                                    : QString{}));
                    item->setIcon(statusIcon(task.value("status").toString()));
                    item->setEnabled(!blocked());
                }
            }
        }
    }
    m_tray.setToolTip(text("trayTip").arg(count));
#ifdef Q_OS_MACOS
    if (nativeMenuAvailable()) {
        syncMacApplicationMenuLanguage();
        syncMacQuitMenuBridge(m_macQuitBridge);
    }
#endif
}
void DesktopIntegration::rebuildTasks(QMenu *menu) {
    menu->clear();
    const auto tasks = m_sessions     ? m_sessions->tasks()
                       : m_controller ? m_controller->tasks()
                                      : QVariantList{};
    if (tasks.isEmpty()) {
        menu->addAction(text("noTasks"))->setEnabled(false);
        return;
    }
    for (const auto &value : tasks) {
        const auto task = value.toMap();
        const auto state = taskStateText(task.value("status").toString());
        auto *item = menu->addAction(
            text("task").arg(text(task.value("kind") == "move"      ? "moveTask"
                                  : task.value("kind") == "check"   ? "checkTask"
                                  : task.value("creating").toBool() ? "createTask"
                                                                    : "extractTask"),
                             task.value("backendId", task.value("id")).toString(), state) +
            (m_sessions ? qtTrId("common.separator") + task.value("archive").toString()
                        : QString{}));
        item->setData(task.value("id"));
        item->setIcon(statusIcon(task.value("status").toString()));
        item->setEnabled(!blocked());
        connect(item, &QAction::triggered, this, [this, id = task.value("id")] {
            if (!m_sessions)
                restore();
            if (!blocked()) {
                if (m_sessions)
                    m_sessions->showTask(id.toString());
                else
                    QMetaObject::invokeMethod(m_window, "showTask", Q_ARG(QString, id.toString()));
            }
        });
    }
}
void DesktopIntegration::rebuildRecent() {
    m_recent->clear();
    const auto paths = m_settings->recentArchives();
    if (paths.isEmpty()) {
        m_recent->addAction(text("emptyRecent"))->setEnabled(false);
        return;
    }
    for (const auto &path : paths) {
        auto *item = m_recent->addAction(QFileInfo(path).fileName().replace('&', "&&"));
        item->setToolTip(path);
        item->setEnabled(enabled("open"));
        connect(item, &QAction::triggered, this, [this, path] {
            if (!enabled("open"))
                return;
            restore();
            if (m_sessions)
                m_sessions->openPath(path);
            else if (m_controller)
                m_controller->requestOpen(path);
        });
    }
}
void DesktopIntegration::rebuildWindows() {
    for (auto *item : m_windowMenu->actions())
        if (item->property("desktopWindow").toBool())
            delete item;
    for (auto *window : QGuiApplication::topLevelWindows()) {
        if ((!m_sessions && !ownedWindow(window, m_window)) ||
            (m_sessions && !window->property("managedWindow").toBool() &&
             !ownedWindow(window, m_window)) ||
            window->property("retiredWindow").toBool() ||
            (!window->isVisible() && window != m_window))
            continue;
        auto *item = m_windowMenu->addAction(window->title().replace('&', "&&"));
        item->setProperty("desktopWindow", true);
        item->setCheckable(true);
        item->setChecked(window->isActive());
        QPointer<QWindow> target(window);
        connect(item, &QAction::triggered, this, [this, target] {
            restore();
            if (target && !blocked()) {
                WindowActivation::reveal(target);
            }
        });
    }
}
void DesktopIntegration::updateIcon() {
    QIcon icon;
#ifdef Q_OS_MACOS
    const auto variant = QStringLiteral("black");
#else
    // The taskbar/panel background can differ from both app and system themes.
    // White ink with a dark outline stays visible without a theme-read fallback.
    const auto variant = QStringLiteral("contrast");
#endif
    if (m_iconVariant == variant)
        return;
    m_iconVariant = variant;
    for (const auto size : {16, 20, 22, 24, 32, 44, 48, 64})
        icon.addFile(QStringLiteral(":/xdzip/assets/tray-%1-%2.png").arg(variant).arg(size),
                     QSize(size, size));
#ifdef Q_OS_MACOS
    icon.setIsMask(true);
#endif
    m_tray.setIcon(icon);
}
bool DesktopIntegration::eventFilter(QObject *watched, QEvent *event) {
    if (!m_sessions && watched == m_window && event->type() == QEvent::Close && m_available &&
        !m_window->property("desktopExitRequested").toBool() &&
        !m_window->property("restartAfterClose").toBool()) {
        event->ignore();
        hideWindows();
        return true;
    }
    if (event->type() == QEvent::FileOpen && m_hidden)
        restore();
    if (event->type() == QEvent::KeyRelease || event->type() == QEvent::MouseButtonRelease)
        QTimer::singleShot(0, this, &DesktopIntegration::syncActions);
    return false;
}
