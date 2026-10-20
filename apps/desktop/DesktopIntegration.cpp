#include "DesktopIntegration.h"
#include "BrowserController.h"
#include "UiPreferences.h"
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
#include <QSettings>
#include <QStyleHints>
#include <QTimer>
#include <QUrl>

#ifdef Q_OS_MACOS
bool macNativeEditorActive();
bool macNativeEditorEnabled(const QString &command);
void macNativeEditorExecute(const QString &command);
#endif

namespace {
QString text(const QString &command) {
    static const QHash<QString, QString> labels{
        {QStringLiteral("application"), qtTrId("desktop.application")},
        {QStringLiteral("file"), qtTrId("desktop.file")},
        {QStringLiteral("edit"), qtTrId("desktop.edit")},
        {QStringLiteral("view"), qtTrId("desktop.view")},
        {QStringLiteral("operation"), qtTrId("desktop.operation")},
        {QStringLiteral("window"), qtTrId("desktop.window")},
        {QStringLiteral("help"), qtTrId("desktop.help")},
        {QStringLiteral("guide"), qtTrId("desktop.guide")},
        {QStringLiteral("feedback"), qtTrId("desktop.feedback")},
        {QStringLiteral("about"), qtTrId("desktop.about")},
        {QStringLiteral("settings"), qtTrId("desktop.settings")},
        {QStringLiteral("quit"), qtTrId("desktop.quit")},
        {QStringLiteral("show"), qtTrId("desktop.show")},
        {QStringLiteral("open"), qtTrId("desktop.open")},
        {QStringLiteral("new"), qtTrId("desktop.new")},
        {QStringLiteral("recent"), qtTrId("desktop.recent")},
        {QStringLiteral("emptyRecent"), qtTrId("desktop.emptyRecent")},
        {QStringLiteral("closeArchive"), qtTrId("desktop.closeArchive")},
        {QStringLiteral("closeWindow"), qtTrId("desktop.closeWindow")},
        {QStringLiteral("undo"), qtTrId("desktop.undo")},
        {QStringLiteral("redo"), qtTrId("desktop.redo")},
        {QStringLiteral("cut"), qtTrId("desktop.cut")},
        {QStringLiteral("copy"), qtTrId("desktop.copy")},
        {QStringLiteral("paste"), qtTrId("desktop.paste")},
        {QStringLiteral("selectAll"), qtTrId("desktop.selectAll")},
        {QStringLiteral("find"), qtTrId("desktop.find")},
        {QStringLiteral("list"), qtTrId("desktop.list")},
        {QStringLiteral("grid"), qtTrId("desktop.grid")},
        {QStringLiteral("inspector"), qtTrId("desktop.inspector")},
        {QStringLiteral("extractAll"), qtTrId("desktop.extractAll")},
        {QStringLiteral("extractSelected"), qtTrId("desktop.extractSelected")},
        {QStringLiteral("preflight"), qtTrId("desktop.preflight")},
        {QStringLiteral("minimize"), qtTrId("desktop.minimize")},
        {QStringLiteral("zoom"), qtTrId("desktop.zoom")},
        {QStringLiteral("tasks"), qtTrId("desktop.tasks")},
        {QStringLiteral("noTasks"), qtTrId("desktop.noTasks")},
        {QStringLiteral("task"), qtTrId("desktop.task")},
        {QStringLiteral("createTask"), qtTrId("desktop.createTask")},
        {QStringLiteral("extractTask"), qtTrId("desktop.extractTask")},
        {QStringLiteral("trayEnabled"), qtTrId("desktop.trayEnabled")},
        {QStringLiteral("trayTip"), qtTrId("desktop.trayTip")},
    };
    return labels.value(command);
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
    window->setProperty("desktopShell", QVariant::fromValue(static_cast<QObject *>(this)));
    m_tray.setContextMenu(m_trayMenu.get());
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
        auto *application = m_bar->addMenu(QStringLiteral("xdzip"));
        add(application, "about")->setMenuRole(QAction::AboutRole);
        add(application, "settings", QKeySequence::Preferences)
            ->setMenuRole(QAction::PreferencesRole);
        add(application, "quit", QKeySequence::Quit)->setMenuRole(QAction::QuitRole);
        auto *file = m_bar->addMenu(text("file"));
        add(file, "open", QKeySequence::Open);
        add(file, "new", QKeySequence::New);
        m_recent = file->addMenu(text("recent"));
        connect(m_recent, &QMenu::aboutToShow, this, &DesktopIntegration::rebuildRecent);
        file->addSeparator();
        add(file, "closeArchive", QKeySequence(Qt::CTRL | Qt::SHIFT | Qt::Key_W));
        add(file, "closeWindow", QKeySequence::Close);
        auto *edit = m_bar->addMenu(text("edit"));
        add(edit, "undo", QKeySequence::Undo);
        add(edit, "redo", QKeySequence::Redo);
        edit->addSeparator();
        add(edit, "cut", QKeySequence::Cut);
        add(edit, "copy", QKeySequence::Copy);
        add(edit, "paste", QKeySequence::Paste);
        add(edit, "selectAll", QKeySequence::SelectAll);
        add(edit, "find", QKeySequence::Find);
        auto *view = m_bar->addMenu(text("view"));
        add(view, "list")->setCheckable(true);
        add(view, "grid")->setCheckable(true);
        view->addSeparator();
        add(view, "inspector")->setCheckable(true);
        // AppKit appends Full Screen with an icon. Keep its icon column out of our group.
        view->setSeparatorsCollapsible(false);
        view->addSeparator();
        auto *operation = m_bar->addMenu(text("operation"));
        add(operation, "extractAll");
        add(operation, "extractSelected");
        add(operation, "preflight");
        m_windowMenu = m_bar->addMenu(text("window"));
        add(m_windowMenu, "minimize", QKeySequence(Qt::CTRL | Qt::Key_M));
        add(m_windowMenu, "zoom");
        m_windowMenu->addSeparator();
        add(m_windowMenu, "show");
        m_taskMenus.append(m_windowMenu->addMenu(text("tasks").arg(0)));
        m_windowMenu->addSeparator();
        connect(m_windowMenu, &QMenu::aboutToShow, this, &DesktopIntegration::rebuildWindows);
        auto *help = m_bar->addMenu(text("help"));
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
    connect(controller, &BrowserController::changed, this, &DesktopIntegration::syncActions);
    connect(controller, &BrowserController::tasksChanged, this, &DesktopIntegration::syncActions);
    connect(settings, &UiPreferences::savedChanged, this, &DesktopIntegration::refresh);
    connect(settings, &UiPreferences::recentChanged, this, [this] {
        if (m_recent && m_recent->isVisible())
            rebuildRecent();
    });
    connect(QGuiApplication::styleHints(), &QStyleHints::colorSchemeChanged, this,
            &DesktopIntegration::updateIcon);
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
    connect(window, &QWindow::activeChanged, this, &DesktopIntegration::syncActions);
    connect(window, &QQuickWindow::activeFocusItemChanged, this, &DesktopIntegration::syncActions);
    connect(QGuiApplication::clipboard(), &QClipboard::dataChanged, this,
            &DesktopIntegration::syncActions);
    connect(qGuiApp, &QGuiApplication::applicationStateChanged, this,
            [this](Qt::ApplicationState state) {
                Q_UNUSED(this);
#ifdef Q_OS_MACOS
                if (m_hidden && state == Qt::ApplicationActive)
                    QTimer::singleShot(0, this, [this] {
                        if (m_hidden && !m_trayMenu->isVisible())
                            restore();
                    });
#else
                Q_UNUSED(state);
#endif
            });
    qApp->installEventFilter(this);
    auto *timer = new QTimer(this);
    timer->setInterval(1500);
    connect(timer, &QTimer::timeout, this, &DesktopIntegration::refresh);
    if (interactive)
        timer->start();
    updateIcon();
    refresh();
}
DesktopIntegration::~DesktopIntegration() {
    qApp->removeEventFilter(this);
    if (m_window)
        m_window->setProperty("desktopShell", QVariant{});
    m_tray.hide();
    // QSystemTrayIcon does not own its QMenu. Detach before deleting the menu.
    m_tray.setContextMenu(nullptr);
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
    if (!m_window || !m_controller || !m_settings)
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
    const bool ready = m_controller->ready() && !m_controller->browsingBusy();
    if (QStringList{"copy", "selectAll", "find"}.contains(command) &&
        QGuiApplication::focusWindow() != m_window)
        return false;
    if (command == "open" || command == "new")
        return m_controller->runtimeAvailable() && !m_controller->browsingBusy();
    if (command == "closeArchive" || command == "extractAll" || command == "selectAll")
        return ready;
    if (command == "extractSelected" || command == "copy")
        return ready && m_controller->selectedCount() > 0;
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
    restore();
    if (blocked())
        return;
    QMetaObject::invokeMethod(m_window, "runDesktopCommand", Q_ARG(QVariant, command));
    syncActions();
}
void DesktopIntegration::requestQuit() {
    if (!m_window || !m_controller)
        return;
    if (blocked()) {
        restore();
        return;
    }
    restore();
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
            window->hide();
        }
    }
    m_hidden = true;
    m_window->hide();
}
void DesktopIntegration::restore() {
    if (!m_window)
        return;
    if (auto *modal = QApplication::activeModalWidget()) {
        modal->show();
        modal->raise();
        modal->activateWindow();
        return;
    }
    if (auto *modal = QGuiApplication::modalWindow()) {
        modal->show();
        modal->raise();
        modal->requestActivate();
        return;
    }
    if (m_window->windowState() == Qt::WindowMinimized)
        m_window->showNormal();
    else
        m_window->show();
    m_hidden = false;
    for (auto &window : m_hiddenWindows)
        if (window)
            window->show();
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
        QApplication::setQuitOnLastWindowClosed(!available);
        emit availabilityChanged();
    }
    updateIcon();
    syncActions();
}
void DesktopIntegration::syncActions() {
    if (!m_controller || !m_window)
        return;
    if (m_session != m_controller->sessionId() || m_controller->browsingBusy()) {
        m_session = m_controller->sessionId();
        m_trayMenu->close();
        if (m_bar)
            for (auto *menu : m_bar->findChildren<QMenu *>())
                menu->close();
    }
    for (auto *item : m_actions) {
        const auto command = item->data().toString();
        item->setEnabled(enabled(command));
        if (command == "list" || command == "grid")
            item->setChecked(m_window->property("viewMode") == command);
        if (command == "inspector")
            item->setChecked(m_window->property("desktopInspectorVisible").toBool());
    }
    const auto count = m_controller->activeTaskCount();
    for (auto *menu : m_taskMenus) {
        menu->setTitle(text("tasks").arg(count));
        if (menu->isVisible()) {
            const auto tasks = m_controller->tasks();
            for (auto *item : menu->actions()) {
                for (const auto &value : tasks) {
                    const auto task = value.toMap();
                    if (item->data() != task.value("id"))
                        continue;
                    QString state;
                    QMetaObject::invokeMethod(m_window, "taskStateLabel",
                                              Q_RETURN_ARG(QString, state),
                                              Q_ARG(QString, task.value("status").toString()));
                    item->setText(text("task").arg(
                        text(task.value("creating").toBool() ? "createTask" : "extractTask"),
                        task.value("id").toString(), state));
                    item->setIcon(statusIcon(task.value("status").toString()));
                    item->setEnabled(!blocked());
                }
            }
        }
    }
    m_tray.setToolTip(text("trayTip").arg(count));
}
void DesktopIntegration::rebuildTasks(QMenu *menu) {
    menu->clear();
    const auto tasks = m_controller->tasks();
    if (tasks.isEmpty()) {
        menu->addAction(text("noTasks"))->setEnabled(false);
        return;
    }
    for (const auto &value : tasks) {
        const auto task = value.toMap();
        QString state;
        QMetaObject::invokeMethod(m_window, "taskStateLabel", Q_RETURN_ARG(QString, state),
                                  Q_ARG(QString, task.value("status").toString()));
        auto *item = menu->addAction(
            text("task").arg(text(task.value("creating").toBool() ? "createTask" : "extractTask"),
                             task.value("id").toString(), state));
        item->setData(task.value("id"));
        item->setIcon(statusIcon(task.value("status").toString()));
        item->setEnabled(!blocked());
        connect(item, &QAction::triggered, this, [this, id = task.value("id")] {
            restore();
            if (!blocked())
                QMetaObject::invokeMethod(m_window, "showTask", Q_ARG(QString, id.toString()));
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
            m_controller->openPath(path);
        });
    }
}
void DesktopIntegration::rebuildWindows() {
    for (auto *item : m_windowMenu->actions())
        if (item->property("desktopWindow").toBool())
            delete item;
    for (auto *window : QGuiApplication::topLevelWindows()) {
        if (!ownedWindow(window, m_window) || (!window->isVisible() && window != m_window))
            continue;
        auto *item = m_windowMenu->addAction(window->title().replace('&', "&&"));
        item->setProperty("desktopWindow", true);
        item->setCheckable(true);
        item->setChecked(window->isActive());
        QPointer<QWindow> target(window);
        connect(item, &QAction::triggered, this, [this, target] {
            restore();
            if (target && !blocked()) {
                target->show();
                target->raise();
                target->requestActivate();
            }
        });
    }
}
void DesktopIntegration::updateIcon() {
    bool dark = QGuiApplication::styleHints()->colorScheme() == Qt::ColorScheme::Dark;
#ifdef Q_OS_WIN
    // Windows taskbar appearance is independent of the application's theme.
    QSettings desktop(
        QStringLiteral(
            "HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize"),
        QSettings::NativeFormat);
    dark = desktop.value(QStringLiteral("SystemUsesLightTheme"), 1).toInt() == 0;
#endif
    QIcon icon;
#ifdef Q_OS_MACOS
    const auto variant = QStringLiteral("black");
#elif defined(Q_OS_WIN)
    const auto variant = dark ? QStringLiteral("white") : QStringLiteral("black");
#else
    // Linux panels need not share the app palette. A dual-tone glyph works on both.
    const auto variant = QStringLiteral("contrast");
#endif
    Q_UNUSED(dark);
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
    if (watched == m_window && event->type() == QEvent::Close && m_available &&
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
