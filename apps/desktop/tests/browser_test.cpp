#include "../BuildInfo.h"
#include "AppQuitGuard.h"
#include "ApplicationSessions.h"
#include "BrowserController.h"
#include "DesktopIntegration.h"
#include "FileOpenRouter.h"
#include "FocusDismissPolicy.h"
#include "NativeDialogPolicy.h"
#include "SingleInstance.h"
#include "UiFont.h"
#include "UiPreferences.h"
#include "UiText.h"
#include "WindowActivation.h"
#include "visual_audit.h"
#include <QAbstractItemModelTester>
#include <QAccessible>
#include <QAction>
#include <QClipboard>
#include <QCursor>
#include <QDesktopServices>
#include <QDialog>
#include <QElapsedTimer>
#include <QFile>
#include <QInputMethodEvent>
#include <QJSValue>
#include <QJsonDocument>
#include <QLockFile>
#include <QMenu>
#include <QPersistentModelIndex>
#include <QPointer>
#include <QProcess>
#include <QProcessEnvironment>
#include <QQmlApplicationEngine>
#include <QQmlComponent>
#include <QQuickItem>
#include <QQuickStyle>
#include <QQuickWindow>
#include <QRegularExpression>
#include <QSGRendererInterface>
#include <QScopeGuard>
#include <QScreen>
#include <QSignalSpy>
#include <QTemporaryDir>
#include <QTextStream>
#include <QTimer>
#include <QUuid>
#include <QtTest>
#include <thread>
#ifdef Q_OS_WIN
#include <qt_windows.h>
#endif
#ifdef Q_OS_MACOS
void scheduleNativeFileDialogCancellation();
bool nativeWindowMatchesState(QWindow *window, Qt::WindowState state);
bool watchNativeWindowTransitions(QWindow *window);
#endif

// Read the native WM state, not the value optimistically set by QWindow.
static bool nativeStateSettled(QWindow *window, Qt::WindowState state) {
#ifdef Q_OS_MACOS
    if (QGuiApplication::platformName() == QStringLiteral("cocoa"))
        return nativeWindowMatchesState(window, state);
#endif
#ifdef Q_OS_LINUX
    if (QGuiApplication::platformName() == QStringLiteral("xcb")) {
        QProcess property;
        property.start(QStringLiteral("xprop"),
                       {QStringLiteral("-id"),
                        QStringLiteral("0x") + QString::number(window->winId(), 16),
                        QStringLiteral("_NET_WM_STATE")});
        if (!property.waitForFinished(1000) || property.exitCode())
            return false;
        const auto flags = property.readAllStandardOutput();
        const bool hidden = flags.contains("_NET_WM_STATE_HIDDEN");
        const bool full = flags.contains("_NET_WM_STATE_FULLSCREEN");
        const bool maximum = flags.contains("_NET_WM_STATE_MAXIMIZED_VERT") &&
                             flags.contains("_NET_WM_STATE_MAXIMIZED_HORZ");
        if (state == Qt::WindowMinimized)
            return hidden;
        if (state == Qt::WindowFullScreen)
            return !hidden && full;
        if (state == Qt::WindowMaximized)
            return !hidden && !full && maximum;
        return !hidden && !full && !maximum;
    }
#endif
    return window->windowState() == state;
}

// TableView delegates belong to its visual tree, not necessarily its QObject tree.
static QQuickItem *findVisual(QQuickItem *item, const QString &name) {
    if (item->property("inReusePool").toBool())
        return nullptr;
    if (item->objectName() == name)
        return item;
    for (auto *child : item->childItems()) {
        if (auto *match = findVisual(child, name))
            return match;
    }
    return nullptr;
}

// Missing images can leave layout dimensions intact. Verify the actual resource state.
static bool visibleImagesReady(QQuickItem *item) {
    if (!item->isVisible())
        return true;
    if (QString::fromLatin1(item->metaObject()->className()).startsWith("QQuickImage") &&
        !item->property("source").toUrl().isEmpty() && item->property("status").toInt() != 1)
        return false;
    for (auto *child : item->childItems())
        if (!visibleImagesReady(child))
            return false;
    return true;
}

// Diagnose only owned fixture directories; never emit their absolute paths.
static bool checkedTarget(BrowserController *controller, const QString &fixturePath) {
    const bool valid = controller->targetCheck().value("valid").toBool();
    if (valid)
        return true;
    qWarning().noquote() << "Owned target check:"
                         << controller->targetCheck().value("errorCode").toString();
#ifdef Q_OS_WIN
    QProcess acl;
    const auto quoted = QString(fixturePath).replace("'", "''");
    const auto shell = qEnvironmentVariable("XDZIP_CI_POWERSHELL", "powershell.exe");
    acl.start(shell, {"-NoProfile", "-Command", "(Get-Acl -LiteralPath '" + quoted + "').Sddl"});
    if (acl.waitForFinished(5000) && acl.exitCode() == 0)
        qWarning().noquote() << "Owned fixture ACL:" << acl.readAllStandardOutput();
    else
        qWarning() << "Owned fixture ACL probe:" << acl.error() << acl.exitCode();
#else
    Q_UNUSED(fixturePath)
#endif
    return false;
}

class DesktopUrlRecorder : public QObject {
    Q_OBJECT
  public:
    int count = 0;
  public slots:
    void record(const QUrl &) {
        ++count;
    }
};

class BrowserTest : public QObject {
    QTemporaryDir m_settingsProfile;
    Q_OBJECT
  private slots:
    void globalWaitForExitDoesNotCloseACompletedPageEarly() {
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *preferences =
            engine.singletonInstance<UiPreferences *>("Xdzip.Settings", "AppSettings");
        QVERIFY(
            preferences->saveDetails("system", 1, "zh-CN", "standard", true, false, false, "tabs"));
        ApplicationSessions sessions(&engine, preferences);
        sessions.adopt(window);
        auto *first = window->findChild<BrowserController *>();
        DesktopIntegration desktop(window, first, preferences, false);
        sessions.setDesktop(&desktop);
        sessions.newSession();
        auto *page = window->property("activePage").value<QObject *>();
        auto *second = page->findChild<BrowserController *>();
        QTRY_VERIFY(first->runtimeAvailable() && second->runtimeAvailable());
        // Controlled task metadata isolates the global waiting decision from
        // worker speed; no synthetic task command is sent to either runtime.
        xdzip::reader::Event task{};
        task.kind = xdzip::reader::EventKind::TaskUpdate;
        task.task.id = 71;
        task.task.status = rust::String("running");
        first->receive(task);
        second->receive(task);
        QCOMPARE(sessions.activeTaskCount(), 2);
        auto *confirmation = page->findChild<QQuickWindow *>("quitWindow");
        sessions.requestQuit();
        QTRY_VERIFY(confirmation->isVisible());
        QVERIFY(QMetaObject::invokeMethod(confirmation, "waitRequested"));
        QTRY_VERIFY(!confirmation->isVisible());
        QSignalSpy joined(&sessions, &ApplicationSessions::readyToQuit);
        task.task.status = rust::String("succeeded");
        second->receive(task);
        QCoreApplication::processEvents();
        QCOMPARE(sessions.activeTaskCount(), 1);
        QVERIFY(window->isVisible());
        QVERIFY(!confirmation->isVisible());
        QVERIFY(joined.isEmpty());
        first->receive(task);
        QTRY_VERIFY(!joined.isEmpty());
        QVERIFY(!window->isVisible());
        QVERIFY(
            preferences->saveDetails("system", 1, "zh-CN", "standard", true, false, true, "tabs"));
    }
    void closedSessionRetainsPasswordPromptUntilTaskIsOpened() {
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *preferences =
            engine.singletonInstance<UiPreferences *>("Xdzip.Settings", "AppSettings");
        QVERIFY(
            preferences->saveDetails("system", 1, "zh-CN", "standard", true, false, true, "tabs"));
        ApplicationSessions sessions(&engine, preferences);
        sessions.adopt(window);
        auto *controller = window->findChild<BrowserController *>();
        QTRY_VERIFY(!controller->busy());
        const auto identity =
            window->property("sessionTabs").toList().first().toMap().value("id").toString();
        // Controlled receiver metadata exercises UI ownership without secrets or
        // a synthetic request being sent to the Rust password channel.
        xdzip::reader::Event waiting{};
        waiting.kind = xdzip::reader::EventKind::TaskUpdate;
        waiting.task.id = 71;
        waiting.task.status = rust::String("waiting_password");
        waiting.task.password_prompt_id = 4;
        waiting.task.password_prompt_name = rust::String("owned.txt");
        controller->receive(waiting);
        auto *prompt = window->findChild<QQuickWindow *>("passwordWindow");
        QVERIFY(prompt);
        sessions.closeSession(identity);
        QCoreApplication::processEvents();
        QVERIFY(!prompt->isVisible());
        QCOMPARE(controller->passwordPrompt().value("promptId").toString(), "4");
        sessions.showTask(identity + "/71");
        QTRY_VERIFY(prompt->isVisible());
        QCOMPARE(prompt->property("shownPromptId").toString(), "4");
        sessions.closeSession(identity);
        waiting.task.password_prompt_id = 5;
        controller->receive(waiting);
        QCoreApplication::processEvents();
        QVERIFY(!prompt->isVisible());
        sessions.showTask(identity + "/71");
        QTRY_VERIFY(prompt->isVisible());
        QCOMPARE(prompt->property("shownPromptId").toString(), "5");
        waiting.task.status = rust::String("succeeded");
        waiting.task.password_prompt_id = 0;
        controller->receive(waiting);
        QTRY_VERIFY(!prompt->isVisible());
        QSignalSpy joined(&sessions, &ApplicationSessions::readyToQuit);
        sessions.confirmQuit(false);
        QTRY_VERIFY(!joined.isEmpty());
    }
    void modalOpenDefersNewWindowUntilUnblocked() {
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *preferences =
            engine.singletonInstance<UiPreferences *>("Xdzip.Settings", "AppSettings");
        QVERIFY(preferences->saveDetails("system", 1, "zh-CN", "standard", true, false, false,
                                         "windows"));
        ApplicationSessions sessions(&engine, preferences);
        sessions.adopt(window);
        window->showMaximized();
        QTRY_COMPARE(window->windowState(), Qt::WindowMaximized);
        QQuickWindow blocker;
        blocker.resize(240, 120);
        blocker.setTransientParent(window);
        blocker.setModality(Qt::ApplicationModal);
        blocker.show();
        QTRY_COMPARE(QGuiApplication::modalWindow(), &blocker);
        sessions.open(
            {QStringLiteral(XDZIP_FIXTURES "/normal.zip"), QStringLiteral(XDZIP_M2_FIXTURE)});
        QCOMPARE(sessions.windows().size(), 2);
        auto *last = sessions.activeWindow();
        QVERIFY(last != window);
        QVERIFY(!last->isVisible());
        QCOMPARE(window->windowState(), Qt::WindowMaximized);
        blocker.close();
        QTRY_VERIFY(last->isVisible());
        QTRY_COMPARE(sessions.activeWindow(), last);
        QCOMPARE(window->windowState(), Qt::WindowMaximized);
        QSignalSpy joined(&sessions, &ApplicationSessions::readyToQuit);
        sessions.confirmQuit(false);
        QTRY_VERIFY(!joined.isEmpty());
        QVERIFY(
            preferences->saveDetails("system", 1, "zh-CN", "standard", true, false, true, "tabs"));
    }
    void globalSettingsKeepDraftAndAboutCallerAcrossWindows() {
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *preferences =
            engine.singletonInstance<UiPreferences *>("Xdzip.Settings", "AppSettings");
        QVERIFY(preferences->saveDetails("system", 1, "zh-CN", "standard", true, false, false,
                                         "windows"));
        ApplicationSessions sessions(&engine, preferences);
        sessions.adopt(window);
        DesktopIntegration desktop(window, window->findChild<BrowserController *>(), preferences,
                                   false);
        sessions.setDesktop(&desktop);
        window->showMaximized();
        QTRY_COMPARE(window->windowState(), Qt::WindowMaximized);
        QSignalSpy changes(window, &QWindow::windowStateChanged);
        desktop.execute("settings");
        auto *settings = window->findChild<QQuickWindow *>("settingsWindow");
        QTRY_VERIFY(settings->isVisible());
        settings->setProperty("draftTheme", "dark");
        sessions.newSession();
        QVERIFY(sessions.activeWindow() != window);
        desktop.execute("settings");
        QCOMPARE(settings->property("draftTheme").toString(), "dark");
        QVERIFY(QMetaObject::invokeMethod(settings, "aboutRequested"));
        auto *about = window->findChild<QQuickWindow *>("aboutWindow");
        QTRY_VERIFY(about->isVisible());
        QCOMPARE(about->transientParent(), settings);
        about->close();
        settings->close();
        QCOMPARE(preferences->theme(), preferences->savedTheme());
        QCOMPARE(window->windowState(), Qt::WindowMaximized);
        for (const auto &change : changes)
            QVERIFY(change.first().value<Qt::WindowState>() != Qt::WindowNoState);
        QSignalSpy joined(&sessions, &ApplicationSessions::readyToQuit);
        sessions.confirmQuit(false);
        QTRY_VERIFY(!joined.isEmpty());
        QVERIFY(
            preferences->saveDetails("system", 1, "zh-CN", "standard", true, false, true, "tabs"));
    }
    void sessionLimitKeepsBackgroundTaskEntryReachable() {
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *preferences =
            engine.singletonInstance<UiPreferences *>("Xdzip.Settings", "AppSettings");
        QVERIFY(
            preferences->saveDetails("system", 1, "zh-CN", "standard", true, false, false, "tabs"));
        ApplicationSessions sessions(&engine, preferences);
        sessions.adopt(window);
        for (int i = 1; i < 32; ++i)
            sessions.newSession();
        QCOMPARE(sessions.sessionCount(), 32);
        sessions.newSession();
        QTRY_VERIFY(QApplication::activeModalWidget());
        auto *limitDialog = QApplication::activeModalWidget();
        sessions.newSession();
        QCOMPARE(QApplication::activeModalWidget(), limitDialog);
        QCOMPARE(sessions.sessionCount(), 32);
        limitDialog->close();
        QTRY_VERIFY(!QApplication::activeModalWidget());
        // Controlled snapshots keep all 32 real connections alive. This test
        // checks the UI/admission boundary, not backend extraction semantics.
        for (auto *controller : window->findChildren<BrowserController *>()) {
            controller->m_tasks[1] = {{"id", "1"}, {"status", "running"}, {"terminal", false}};
            emit controller->tasksChanged();
        }
        const auto tabs = window->property("sessionTabs").toList();
        QCOMPARE(tabs.size(), 32);
        for (const auto &value : tabs)
            sessions.closeSession(value.toMap().value("id").toString());
        QCOMPARE(sessions.sessionCount(), 32);
        QCOMPARE(sessions.activeTaskCount(), 32);
        QVERIFY(window->property("welcomeVisible").toBool());
        QVERIFY(window->property("activePage").value<QObject *>()->property("visible").toBool());
        auto *page = qobject_cast<QQuickItem *>(window->property("activePage").value<QObject *>());
        auto *taskEntry = findVisual(page, "browserTasks");
        QVERIFY(taskEntry && taskEntry->isVisible());
        sessions.showTask(tabs.first().toMap().value("id").toString() + "/1");
        QVERIFY(
            !window->property("activePage").value<QObject *>()->property("closedPage").toBool());
        QSignalSpy joined(&sessions, &ApplicationSessions::readyToQuit);
        sessions.confirmQuit(false);
        QTRY_VERIFY(!joined.isEmpty());
        QVERIFY(
            preferences->saveDetails("system", 1, "zh-CN", "standard", true, false, true, "tabs"));
    }
    void coldOpenSessionsWaitForNativeLaunch() {
        QQmlApplicationEngine engine;
        engine.setInitialProperties({{"visible", false}});
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *preferences =
            engine.singletonInstance<UiPreferences *>("Xdzip.Settings", "AppSettings");
        QVERIFY(preferences->saveDetails("system", 1, "zh-CN", "standard", true, false, false,
                                         "windows"));
        ApplicationSessions sessions(&engine, preferences);
        sessions.adopt(window);
        sessions.open(
            {QStringLiteral(XDZIP_FIXTURES "/normal.zip"), QStringLiteral(XDZIP_M2_FIXTURE)});
        QCOMPARE(sessions.windows().size(), 2);
        for (auto *candidate : sessions.windows()) {
            QVERIFY(!candidate->isVisible());
            QVERIFY(candidate->property("activePage")
                        .value<QObject *>()
                        ->property("externalFileOpening")
                        .toBool());
        }
        WindowActivation::reveal(window);
        for (auto *candidate : sessions.windows()) {
            QTRY_VERIFY(candidate->isVisible());
            auto *controller = candidate->findChild<BrowserController *>();
            QTRY_VERIFY(controller->ready() && !controller->busy());
            QVERIFY(!candidate->property("activePage")
                         .value<QObject *>()
                         ->property("externalFileOpening")
                         .toBool());
        }
        QSignalSpy joined(&sessions, &ApplicationSessions::readyToQuit);
        sessions.confirmQuit(false);
        QTRY_VERIFY(!joined.isEmpty());
        QVERIFY(
            preferences->saveDetails("system", 1, "zh-CN", "standard", true, false, true, "tabs"));
    }
    void singleInstanceConcurrentStartupAndCrashRecovery() {
        const auto nameSpace = QUuid::createUuid().toString();
        SingleInstance owner;
        QCOMPARE(owner.start({}, nameSpace), SingleInstance::Primary);
        SingleInstance unresponsive;
        // This thread deliberately does not dispatch the owner's server while
        // the isolated client waits. Timeout is failure, never another primary.
        QCOMPARE(unresponsive.start({}, nameSpace, 200), SingleInstance::Failed);
        std::vector<std::unique_ptr<QProcess>> clients;
        for (int i = 0; i < 4; ++i) {
            auto client = std::make_unique<QProcess>();
            client->start(QCoreApplication::applicationFilePath(),
                          {"--instance-client", nameSpace,
                           QDir::current().absoluteFilePath(QString::number(i) + " file.zip")});
            clients.push_back(std::move(client));
        }
        for (const auto &client : clients) {
            QTRY_COMPARE_WITH_TIMEOUT(client->state(), QProcess::NotRunning, 20000);
            QCOMPARE(client->exitCode(), 0);
        }
        QStringList accepted;
        owner.attach([&](const QStringList &paths) { accepted += paths; });
        QCOMPARE(accepted.size(), 4); // Intents accepted while QML is not yet attached.
        owner.stopAccepting();
        QProcess rejected;
        rejected.start(
            QCoreApplication::applicationFilePath(),
            {"--instance-client", nameSpace, QDir::current().absoluteFilePath("closing.zip")});
        QTRY_COMPARE_WITH_TIMEOUT(rejected.state(), QProcess::NotRunning, 20000);
        QCOMPARE(rejected.exitCode(), 1);
        QCOMPARE(accepted.size(), 4);
        owner.release();
        SingleInstance replacement;
        QCOMPARE(replacement.start({}, nameSpace), SingleInstance::Primary);
        replacement.attach([&](const QStringList &paths) { accepted += paths; });
        owner.release(); // Idempotent teardown cannot remove the replacement endpoint.
        QProcess reopened;
        reopened.start(
            QCoreApplication::applicationFilePath(),
            {"--instance-client", nameSpace, QDir::current().absoluteFilePath("restart.zip")});
        QTRY_COMPARE_WITH_TIMEOUT(reopened.state(), QProcess::NotRunning, 20000);
        QCOMPARE(reopened.exitCode(), 0);
        QCOMPARE(accepted.size(), 5);
        const auto crashSpace = QUuid::createUuid().toString();
        QProcess primary;
        primary.start(QCoreApplication::applicationFilePath(), {"--instance-primary", crashSpace});
        QTRY_VERIFY_WITH_TIMEOUT(primary.bytesAvailable() > 0, 10000);
        QVERIFY(primary.readAllStandardOutput().contains("PRIMARY"));
        primary.kill(); // Only our isolated test child, leaving the election/socket residue.
        QTRY_COMPARE(primary.state(), QProcess::NotRunning);
        SingleInstance recovery;
        QCOMPARE(recovery.start({}, crashSpace), SingleInstance::Primary);
    }
    void backgroundTasksKeepSessionIdentityAndSerialAdmission_data() {
        QTest::addColumn<bool>("separateWindows");
        QTest::newRow("closed-tab") << false;
        QTest::newRow("closed-window") << true;
    }
    void backgroundTasksKeepSessionIdentityAndSerialAdmission() {
        QFETCH(bool, separateWindows);
        QTemporaryDir output;
        QVERIFY(output.isValid());
        const auto input = output.filePath("input.txt");
        QFile file(input);
        QVERIFY(file.open(QIODevice::WriteOnly));
        QCOMPARE(file.write(QByteArray(1024 * 1024, 'a')), 1024 * 1024);
        file.close();
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *preferences =
            engine.singletonInstance<UiPreferences *>("Xdzip.Settings", "AppSettings");
        QVERIFY(preferences->saveDetails("system", 1, "zh-CN", "standard", true, false, false,
                                         separateWindows ? "windows" : "tabs"));
        ApplicationSessions sessions(&engine, preferences);
        sessions.adopt(window);
        auto *first = window->findChild<BrowserController *>();
        sessions.newSession();
        auto *second = sessions.activeWindow()
                           ->property("activePage")
                           .value<QObject *>()
                           ->findChild<BrowserController *>();
        QVERIFY(second != first);
        QSignalSpy firstStarted(first, &BrowserController::creationStarted);
        QSignalSpy secondStarted(second, &BrowserController::creationStarted);
        bool overlapping = false;
        connect(second, &BrowserController::creationStarted, this,
                [&] { overlapping |= first->activeTaskCount() > 0; });
        first->startCreation({input}, output.path(), "first.zip", 0, 6, false);
        second->startCreation({input}, output.path(), "second.zip", 0, 6, false);
        QVERIFY(first->taskStarting());
        QVERIFY(second->taskStarting());
        const auto firstTab =
            window->property("sessionTabs").toList().first().toMap().value("id").toString();
        if (!separateWindows) {
            sessions.moveTab(firstTab, 1);
            QCOMPARE(window->property("sessionTabs").toList().last().toMap().value("id").toString(),
                     firstTab);
            QCOMPARE(sessions.sessionCount(), 2);
            QVERIFY(first->taskStarting() && second->taskStarting());
        }
        if (separateWindows)
            sessions.closeWindow(window);
        else
            sessions.closeSession(firstTab);
        QTRY_VERIFY(!firstStarted.isEmpty());
        QTRY_VERIFY_WITH_TIMEOUT(!secondStarted.isEmpty(), 10000);
        QTRY_COMPARE_WITH_TIMEOUT(sessions.activeTaskCount(), 0, 10000);
        QVERIFY(!overlapping);
        QVERIFY(QFileInfo::exists(output.filePath("first.zip")));
        QVERIFY(QFileInfo::exists(output.filePath("second.zip")));
        QVERIFY(!window->findChild<QQuickWindow *>("taskWindow")->isVisible());
        QSet<QString> identities;
        for (const auto &value : sessions.tasks())
            identities.insert(value.toMap().value("id").toString());
        QCOMPARE(identities.size(), 2);
        sessions.showTask(firstTab + '/' + firstStarted.first().first().toString());
        QCOMPARE(
            window->property("activePage").value<QObject *>()->findChild<BrowserController *>(),
            first);
        QSignalSpy joined(&sessions, &ApplicationSessions::readyToQuit);
        sessions.confirmQuit(false);
        QTRY_VERIFY(!joined.isEmpty());
        QVERIFY(
            preferences->saveDetails("system", 1, "zh-CN", "standard", true, false, true, "tabs"));
    }
    void sessionTabReorderingPreservesControllers_data() {
        QTest::addColumn<double>("scale");
        QTest::newRow("100") << 1.;
        QTest::newRow("125") << 1.25;
        QTest::newRow("150") << 1.5;
    }
    void sessionTabReorderingPreservesControllers() {
        QFETCH(double, scale);
        const bool previousAccessibility = QAccessible::isActive();
        QAccessible::setActive(true);
        const auto restoreAccessibility =
            qScopeGuard([&] { QAccessible::setActive(previousAccessibility); });
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *preferences =
            engine.singletonInstance<UiPreferences *>("Xdzip.Settings", "AppSettings");
        const auto restorePreferences = qScopeGuard([&] {
            preferences->saveDetails("system", 1, "zh-CN", "standard", true, false, true, "tabs");
        });
        QVERIFY(preferences->saveDetails("system", scale, "zh-CN", "standard", true, false, false,
                                         "tabs"));
        ApplicationSessions sessions(&engine, preferences);
        sessions.adopt(window);
        QVERIFY(QTest::qWaitForWindowExposed(window));
        auto *first = window->findChild<BrowserController *>();
        first->openPath(QStringLiteral(XDZIP_FIXTURES "/normal.zip"));
        QTRY_VERIFY(first->ready() && !first->busy());
        first->selectAll(true);
        const auto selection = first->selectedCount();
        QVERIFY(selection > 0);
        sessions.newSession();
        auto *second =
            window->property("activePage").value<QObject *>()->findChild<BrowserController *>();
        sessions.newSession();
        const auto activePage = window->property("activePage").value<QObject *>();
        const auto ids = [](QQuickWindow *owner) {
            QStringList result;
            for (const auto &value : owner->property("sessionTabs").toList())
                result.append(value.toMap().value("id").toString());
            return result;
        };
        const auto original = ids(window);
        QCOMPARE(original.size(), 3);
        auto *header = findVisual(qobject_cast<QQuickItem *>(activePage), "archiveTitlebar");
        QVERIFY(header);
        QTRY_VERIFY(findVisual(header, "archiveTab-0"));
        QTRY_VERIFY(findVisual(header, "activeArchiveTab"));
        auto *source = findVisual(header, "archiveTab-0");
        auto *target = findVisual(header, "activeArchiveTab");
        QVERIFY(source && target);
        const auto start =
            source->mapToScene(QPointF(source->width() / 2, source->height() / 2)).toPoint();
        const auto finish =
            target->mapToScene(QPointF(target->width() / 2, target->height() / 2)).toPoint();
        QSignalSpy nativeDrag(header, SIGNAL(dragRequested()));
        const auto position = window->position();
        QTest::mousePress(window, Qt::LeftButton, Qt::NoModifier, start);
        QTest::mouseMove(window, start + QPoint(24, 0), 20);
        QTest::mouseMove(window, finish, 20);
        QTRY_VERIFY(header->property("reordering").toBool());
        QTest::mouseRelease(window, Qt::LeftButton, Qt::NoModifier, finish);
        QTRY_COMPARE(ids(window), QStringList({original[1], original[2], original[0]}));
        QCOMPARE(window->property("activePage").value<QObject *>(), activePage);
        QCOMPARE(window->position(), position);
        QCOMPARE(nativeDrag.count(), 0);
        QCOMPARE(first->selectedCount(), selection);
        QCOMPARE(sessions.sessionCount(), 3);
        QTest::keyClick(window, Qt::Key_PageUp, Qt::ControlModifier | Qt::ShiftModifier);
        QTRY_COMPARE(ids(window), QStringList({original[2], original[1], original[0]}));
        const auto ordered = ids(window);
        sessions.moveTab("unknown", 0);
        sessions.moveTab(original[0], -1);
        sessions.moveTab(original[0], 32);
        QCOMPARE(ids(window), ordered);
        QVERIFY(preferences->saveDetails("system", scale, "zh-CN", "standard", true, false, false,
                                         "windows"));
        sessions.newSession();
        auto *other = sessions.activeWindow();
        QVERIFY(other != window);
        QVERIFY(preferences->saveDetails("system", scale, "zh-CN", "standard", true, false, false,
                                         "tabs"));
        sessions.newSession();
        const auto otherOrder = ids(other);
        const auto otherPage = other->property("activePage").value<QObject *>();
        sessions.moveTab(original[0], 0);
        QCOMPARE(ids(other), otherOrder);
        QCOMPARE(other->property("activePage").value<QObject *>(), otherPage);
        QCOMPARE(sessions.activeWindow(), other);
        QTRY_VERIFY(findVisual(header, "archiveTab-2"));
        auto *inactive = findVisual(header, "archiveTab-2");
        auto *accessible = QAccessible::queryAccessibleInterface(inactive);
        QVERIFY(accessible && accessible->actionInterface());
        accessible->actionInterface()->doAction(QAccessibleActionInterface::pressAction());
        QTRY_COMPARE(sessions.activeWindow(), window);
        QTRY_COMPARE(
            window->property("activePage").value<QObject *>()->findChild<BrowserController *>(),
            second);
        const auto selectedPage = window->property("activePage").value<QObject *>();
        sessions.closeSession(original[2]);
        QCOMPARE(window->property("activePage").value<QObject *>(), selectedPage);
        QCOMPARE(ids(other), otherOrder);
        for (int index = 0; index < 8; ++index)
            sessions.newSession();
        const auto beforeClose = ids(window);
        sessions.closeSession(beforeClose.last());
        QTRY_VERIFY(
            window->property("sessionTabs").toList().last().toMap().value("selected").toBool());
        QCOMPARE(ids(window).last(), beforeClose[beforeClose.size() - 2]);
        window->resize(832, window->height());
        auto *page = qobject_cast<QQuickItem *>(window->property("activePage").value<QObject *>());
        header = findVisual(page, "archiveTitlebar");
        QVERIFY(header);
        QTRY_VERIFY(header->property("overflowing").toBool());
        auto *strip = findVisual(header, "archiveTabStrip");
        QVERIFY(strip);
        QTRY_VERIFY(findVisual(header, "activeArchiveTab"));
        auto *selected = findVisual(header, "activeArchiveTab");
        QTRY_VERIFY(selected->mapToItem(strip, QPointF()).x() >= -.5 &&
                    selected->mapToItem(strip, QPointF(selected->width(), 0)).x() <=
                        strip->width() + .5);
        auto *list = findVisual(header, "archiveTabList");
        auto *settings = findVisual(header, "archiveSettings");
        QVERIFY(list && settings);
        QVERIFY(list->mapToItem(header, QPointF(list->width(), 0)).x() <= settings->x());
        const auto captures = qEnvironmentVariable("XDZIP_TABS_VISUAL_AUDIT");
        if (!captures.isEmpty()) {
            QVERIFY(QDir().mkpath(captures));
            QVERIFY(visual_audit::save(window, captures,
                                       preferences->activeLanguage() + "-tabs-" +
                                           QString::number(scale)));
        }
        QCOMPARE(first->selectedCount(), selection);
        QSignalSpy joined(&sessions, &ApplicationSessions::readyToQuit);
        sessions.confirmQuit(false);
        QTRY_VERIFY(!joined.isEmpty());
        QVERIFY(
            preferences->saveDetails("system", 1, "zh-CN", "standard", true, false, true, "tabs"));
    }
    void independentSessionsAndOpenModes() {
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *preferences =
            engine.singletonInstance<UiPreferences *>("Xdzip.Settings", "AppSettings");
        QVERIFY(
            preferences->saveDetails("system", 1, "zh-CN", "standard", true, false, false, "tabs"));
        ApplicationSessions sessions(&engine, preferences);
        sessions.adopt(window);
        DesktopIntegration desktop(window, window->findChild<BrowserController *>(), preferences,
                                   false);
        sessions.setDesktop(&desktop);
        sessions.open(
            {QStringLiteral(XDZIP_FIXTURES "/normal.zip"), QStringLiteral(XDZIP_M2_FIXTURE)});
        QCOMPARE(sessions.sessionCount(), 2);
        QCOMPARE(sessions.windows().size(), 1);
        const auto controllers = window->findChildren<BrowserController *>();
        QCOMPARE(controllers.size(), 2);
        for (auto *controller : controllers)
            QTRY_VERIFY(controller->ready() && !controller->busy());
        controllers.first()->selectAll(true);
        QVERIFY(controllers.first()->selectedCount() > 0);
        QCOMPARE(controllers.last()->selectedCount(), 0);
        sessions.openPath(QStringLiteral(XDZIP_FIXTURES "/normal.zip"));
        QCOMPARE(sessions.sessionCount(), 2);
        QVERIFY(preferences->saveDetails("system", 1, "zh-CN", "standard", true, false, false,
                                         "windows"));
        sessions.newSession();
        QCOMPARE(sessions.windows().size(), 2);
        auto *second = sessions.activeWindow();
        QVERIFY(second != window);
        auto *secondController = second->findChild<BrowserController *>();
        QTRY_VERIFY(!secondController->busy());
        sessions.closeWindow(second);
        QVERIFY(!second->isVisible());
        QCOMPARE(sessions.activeWindow(), window);
        QSignalSpy joined(&sessions, &ApplicationSessions::readyToQuit);
        sessions.confirmQuit(false);
        QTRY_VERIFY(!joined.isEmpty());
        QVERIFY(
            preferences->saveDetails("system", 1, "zh-CN", "standard", true, false, true, "tabs"));
    }
    void singleInstanceStructuredForwarding() {
        const auto nameSpace = QUuid::createUuid().toString();
        SingleInstance owner;
        QCOMPARE(owner.start({}, nameSpace), SingleInstance::Primary);
        QStringList received;
        owner.attach([&](const QStringList &paths) { received += paths; });
        QProcess client;
        client.start(QCoreApplication::applicationFilePath(),
                     {"--instance-client", nameSpace,
                      QDir::current().absoluteFilePath(QString::fromUtf8("中文 空格;&.zip"))});
        QTRY_COMPARE_WITH_TIMEOUT(client.state(), QProcess::NotRunning, 20000);
        QCOMPARE(client.exitCode(), 0);
        QCOMPARE(received.size(), 1);
        QVERIFY(received.first().endsWith(QString::fromUtf8("中文 空格;&.zip")));
    }
    void desktopActivationPreservesWindowState() {
        WindowActivation::install(qApp);
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *controller = window->findChild<BrowserController *>();
        auto *preferences =
            engine.singletonInstance<UiPreferences *>("Xdzip.Settings", "AppSettings");
        DesktopIntegration desktop(window, controller, preferences, false);
        controller->openPath(QStringLiteral(XDZIP_FIXTURES "/normal.zip"));
        QTRY_VERIFY(controller->ready() && !controller->busy());
#ifdef Q_OS_MACOS
        if (QGuiApplication::platformName() == QStringLiteral("cocoa"))
            QTRY_VERIFY(watchNativeWindowTransitions(window));
#endif
        for (const auto state : {Qt::WindowNoState, Qt::WindowMaximized, Qt::WindowFullScreen}) {
            window->setWindowState(state);
            QTRY_COMPARE(window->windowState(), state);
            if (QGuiApplication::platformName() != QStringLiteral("offscreen")) {
                QVERIFY(QTest::qWaitForWindowExposed(window));
#ifdef Q_OS_MACOS
                if (QGuiApplication::platformName() == QStringLiteral("cocoa"))
                    QTRY_VERIFY(nativeWindowMatchesState(window, state));
#endif
            }
            // Native and offscreen backends can deliver old state events after
            // the setter returns. Drain the preceding minimize/restore before
            // starting the strict popup-state measurement.
            QTest::qWait(150);
            QTRY_COMPARE(window->windowState(), state);
            QTRY_VERIFY(nativeStateSettled(window, state));
            QSignalSpy changes(window, &QWindow::windowStateChanged);
#ifdef Q_OS_WIN
            if (state == Qt::WindowFullScreen &&
                QGuiApplication::platformName() == QStringLiteral("windows")) {
                auto *frame = window->findChild<QObject *>("windowsNativeFrame");
                QVariantMap diagnostics;
                QVERIFY(QMetaObject::invokeMethod(frame, "diagnostics",
                                                  Q_RETURN_ARG(QVariantMap, diagnostics)));
                QCOMPARE(diagnostics.value("topInsetPixels").toInt(), 0);
                QVERIFY(!diagnostics.value("resizeStyle").toBool());
            }
#endif
            for (const auto &command : {"settings", "about", "new"}) {
                qInfo() << "Activating" << command << "from" << state;
                desktop.execute(command);
                QCoreApplication::processEvents();
                QCOMPARE(window->windowState(), state);
                for (auto *popup : window->findChildren<QQuickWindow *>())
                    popup->close();
                QTRY_VERIFY(!controller->busy());
            }
            desktop.execute("extractAll");
            auto *extract = window->findChild<QQuickWindow *>("extractWindow");
            QTRY_VERIFY(extract->isVisible());
            QCOMPARE(window->windowState(), state);
            extract->close();
            QTRY_VERIFY(!controller->busy());
            for (const auto &name : {"taskWindow", "passwordWindow", "conflictWindow", "quitWindow",
                                     "preflightWindow"}) {
                auto *popup = window->findChild<QQuickWindow *>(name);
                QVERIFY(popup);
                QVERIFY(QMetaObject::invokeMethod(popup, "present"));
                QCoreApplication::processEvents();
                QCOMPARE(window->windowState(), state);
                popup->close();
            }
            auto *inspector = window->findChild<QQuickWindow *>("inspectorWindow");
            WindowActivation::reveal(inspector);
            QCOMPARE(window->windowState(), state);
            inspector->close();
            qInfo() << "Opening native file chooser from" << state;
#ifdef Q_OS_MACOS
            if (QGuiApplication::platformName() == QStringLiteral("cocoa"))
                scheduleNativeFileDialogCancellation();
#endif
            QTimer::singleShot(100, qApp, [] {
                for (auto *widget : QApplication::topLevelWidgets())
                    if (auto *dialog = qobject_cast<QFileDialog *>(widget))
                        dialog->reject();
            });
            controller->chooseArchive(window);
            QCOMPARE(window->windowState(), state);
            for (const auto &change : changes)
                QCOMPARE(change.first().value<Qt::WindowState>(), state);
            window->hide();
            if (QGuiApplication::platformName() != QStringLiteral("offscreen"))
                QTRY_VERIFY(!window->isExposed());
            qInfo() << "Restoring hidden window" << window->windowStates() << "saved"
                    << window->property("xdzipRestoreState");
            desktop.restore();
            QTRY_COMPARE(window->windowState(), state);
            if (QGuiApplication::platformName() != QStringLiteral("offscreen"))
                QVERIFY(QTest::qWaitForWindowExposed(window));
            QTRY_VERIFY(nativeStateSettled(window, state));
            QTRY_COMPARE(window->windowState(), state);
#ifdef Q_OS_MACOS
            if (QGuiApplication::platformName() == QStringLiteral("cocoa"))
                QTRY_VERIFY(nativeWindowMatchesState(window, state));
#endif
            window->showMinimized();
            QTRY_VERIFY(window->windowStates().testFlag(Qt::WindowMinimized));
            if (QGuiApplication::platformName() != QStringLiteral("offscreen"))
                QTRY_VERIFY(!window->isExposed());
            QTRY_VERIFY(nativeStateSettled(window, Qt::WindowMinimized));
            qInfo() << "Restoring native minimized state" << window->windowStates() << "saved"
                    << window->property("xdzipRestoreState");
            desktop.restore();
            QTRY_COMPARE(window->windowState(), state);
            if (QGuiApplication::platformName() != QStringLiteral("offscreen")) {
                QVERIFY(QTest::qWaitForWindowExposed(window));
                QVERIFY(QTest::qWaitForWindowActive(window));
                QTRY_COMPARE(window->windowState(), state);
                QTRY_VERIFY(nativeStateSettled(window, state));
#ifdef Q_OS_MACOS
                if (QGuiApplication::platformName() == QStringLiteral("cocoa"))
                    QTRY_VERIFY(nativeWindowMatchesState(window, state));
#endif
            }
            // A second activation may arrive while native minimization is still
            // in flight. It must be retained rather than cancelling that transition.
            window->showMinimized();
            desktop.restore();
            QTRY_COMPARE(window->windowState(), state);
            if (QGuiApplication::platformName() != QStringLiteral("offscreen")) {
                QVERIFY(QTest::qWaitForWindowExposed(window));
                QVERIFY(QTest::qWaitForWindowActive(window));
                QTRY_COMPARE(window->windowState(), state);
                QTRY_VERIFY(nativeStateSettled(window, state));
#ifdef Q_OS_MACOS
                if (QGuiApplication::platformName() == QStringLiteral("cocoa"))
                    QTRY_VERIFY(nativeWindowMatchesState(window, state));
#endif
            }
        }
        controller->requestClose();
        QTRY_VERIFY(controller->requestClose());
    }
    void nativeEditRoleTranslationsFollowMenuLanguage() {
        for (const auto &language : {QStringLiteral("zh-CN"), QStringLiteral("en-US")}) {
            UiTranslator translator(nullptr);
            QVERIFY(translator.read(language));
            for (const auto *command : {"Cut", "Copy", "Paste", "Select All"})
                QVERIFY(!translator.translate("QCocoaMenuItem", command, nullptr, -1).isEmpty());
            QCOMPARE(translator.translate("QCocoaMenuItem", "Copy", nullptr, -1),
                     translator.translate(nullptr, "desktop.copy", nullptr, -1));
            QCOMPARE(translator.translate("QCocoaMenuBar", "Edit", nullptr, -1),
                     translator.translate(nullptr, "desktop.edit", nullptr, -1));
            QVERIFY(translator.translate("QFileDialog", "Copy", nullptr, -1).isEmpty());
        }
    }
    void desktopTraySettingsCompatibility() {
        QTemporaryDir profile;
        const auto previous = qgetenv("XDZIP_SETTINGS_PATH");
        qputenv("XDZIP_SETTINGS_PATH", profile.filePath("settings.ini").toUtf8());
        const auto restore = qScopeGuard([&] { qputenv("XDZIP_SETTINGS_PATH", previous); });
        UiPreferences preferences;
        QVERIFY(preferences.trayEnabled());
        QVERIFY(preferences.saveDetails("system", 1, "zh-CN", "standard", true, false, false));
        UiPreferences restored;
        QVERIFY(!restored.trayEnabled());
        // The historical six-argument API must retain the new setting.
        QVERIFY(restored.saveDetails("dark", 1.25, "zh-CN", "compact", true, false));
        UiPreferences again;
        QVERIFY(!again.trayEnabled());
        QLockFile lock(profile.filePath("settings.ini.xdzip-lock"));
        QVERIFY(lock.tryLock());
        QVERIFY(!again.saveDetails("light", 1, "zh-CN", "standard", true, false, true));
        QVERIFY(!again.trayEnabled());
        QCOMPARE(again.recentArchives().size(), 0);
    }
    void desktopTrayLifecycleAndCommands() {
        const bool previousQuit = QApplication::quitOnLastWindowClosed();
        const auto restoreQuit =
            qScopeGuard([&] { QApplication::setQuitOnLastWindowClosed(previousQuit); });
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *controller = window->findChild<BrowserController *>();
        auto *preferences =
            engine.singletonInstance<UiPreferences *>("Xdzip.Settings", "AppSettings");
        QVERIFY(preferences);
        bool available = true;
        DesktopIntegration desktop(window, controller, preferences, true,
                                   [&] { return available; });
        QVERIFY(desktop.trayAvailable());
        QVERIFY(!QApplication::quitOnLastWindowClosed());
        QVERIFY(desktop.action("show"));
        QVERIFY(!desktop.trayMenu()->actions().isEmpty());
        desktop.execute("settings");
        auto *settings = window->findChild<QQuickWindow *>("settingsWindow");
        QTRY_VERIFY(settings->isVisible());
        settings->close();
        auto *create = window->findChild<QQuickWindow *>("createWindow");
        desktop.execute("new");
        QTRY_VERIFY(create->isVisible());
        QDialog modal;
        modal.setWindowModality(Qt::ApplicationModal);
        modal.show();
        QTRY_VERIFY(QApplication::activeModalWidget());
        window->close();
        QVERIFY(window->isVisible()); // A modal selector prevents hiding its owner.
        modal.close();
        window->close();
        QVERIFY(!window->isVisible());
        QVERIFY(!create->isVisible());
        QVERIFY(controller->runtimeAvailable());
        desktop.restore();
        QVERIFY(window->isVisible());
        QVERIFY(create->isVisible()); // A nonmodal form retains its draft across hiding.
        create->close();
        window->close();
        available = false;
        desktop.refresh();
        QVERIFY(!desktop.trayAvailable());
        QVERIFY(window->isVisible()); // Tray-host loss cannot strand a hidden app.
        QVERIFY(QApplication::quitOnLastWindowClosed());
        available = true;
        desktop.refresh();
        QTRY_VERIFY2(!controller->busy(), qPrintable(controller->message()));
        controller->openPath(QStringLiteral(XDZIP_FIXTURES "/normal.zip"));
        QTRY_VERIFY2(controller->ready() && !controller->busy(), qPrintable(controller->message()));
        desktop.execute("grid");
        QCOMPARE(window->property("viewMode").toString(), "grid");
        desktop.execute("list");
        QCOMPARE(window->property("viewMode").toString(), "list");
        controller->selectAll(false);
        const bool fileFocus = QGuiApplication::focusWindow() == window;
        desktop.execute("selectAll");
        if (fileFocus)
            QTRY_VERIFY(controller->selectedCount() > 0);
        else
            QCOMPARE(controller->selectedCount(), 0); // Background native apps have no file focus.
        desktop.execute("closeArchive");
        QTRY_VERIFY(!controller->ready() && !controller->busy());
        window->close();
        QVERIFY(!window->isVisible());
        desktop.requestQuit();
        QTRY_VERIFY(!controller->runtimeAvailable());
        QTRY_VERIFY(controller->requestClose());
        QVERIFY(window->property("desktopExitRequested").toBool());
    }
    void desktopNativeEditFocusRouting() {
        const bool previousQuit = QApplication::quitOnLastWindowClosed();
        const auto restoreQuit =
            qScopeGuard([&] { QApplication::setQuitOnLastWindowClosed(previousQuit); });
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *controller = window->findChild<BrowserController *>();
        auto *preferences =
            engine.singletonInstance<UiPreferences *>("Xdzip.Settings", "AppSettings");
        DesktopIntegration desktop(window, controller, preferences, false);
        QQmlComponent component(&engine);
        component.setData(
            "import QtQuick; Window { width: 300; height: 100; visible: true; TextInput { "
            "objectName: \"editor\"; text: \"focus route\"; focus: true } }",
            QUrl());
        QScopedPointer<QObject> object(component.create());
        QVERIFY2(object, qPrintable(component.errorString()));
        auto *editorWindow = qobject_cast<QQuickWindow *>(object.data());
        auto *editor = editorWindow->findChild<QQuickItem *>("editor");
        QVERIFY(editor);
        editorWindow->requestActivate();
        editor->forceActiveFocus();
        QTRY_COMPARE(QGuiApplication::focusWindow(), editorWindow);
        desktop.execute("selectAll");
        QCOMPARE(editor->property("selectedText").toString(), "focus route");
        desktop.execute("copy");
        QCOMPARE(QGuiApplication::clipboard()->text(), "focus route");
        desktop.execute("cut");
        QCOMPARE(editor->property("text").toString(), "");
        desktop.execute("paste");
        QCOMPARE(editor->property("text").toString(), "focus route");
        desktop.execute("undo");
        QCOMPARE(editor->property("text").toString(), "");
        desktop.execute("redo");
        QCOMPARE(editor->property("text").toString(), "focus route");
        QDialog modal;
        modal.setWindowModality(Qt::ApplicationModal);
        modal.show();
        desktop.execute("new");
        QVERIFY(!window->findChild<QQuickWindow *>("createWindow")->isVisible());
        modal.close();
    }
    void settingsRejectFutureSchemaAndPreserveAtomicBackup() {
        QTemporaryDir profile;
        QVERIFY(profile.isValid());
        const auto path = profile.filePath("settings.ini");
        const auto oldOverride = qgetenv("XDZIP_SETTINGS_PATH");
        qputenv("XDZIP_SETTINGS_PATH", path.toUtf8());
        const auto reset = qScopeGuard([&] { qputenv("XDZIP_SETTINGS_PATH", oldOverride); });
        auto bytes = [](const QString &name) {
            QFile input(name);
            return input.open(QIODevice::ReadOnly) ? input.readAll() : QByteArray{};
        };
        QFile input(path);
        QVERIFY(input.open(QIODevice::WriteOnly));
        const QByteArray original = "[General]\nschemaVersion=1\n[future]\nkeep=owned-canary\n"
                                    "[appearance]\ntextScale=2\ntheme=dark\n";
        QCOMPARE(input.write(original), original.size());
        input.close();
        UiPreferences preferences;
        QCOMPARE(preferences.savedTextScale(), 1.5);
        QCOMPARE(bytes(path), original); // Loading legacy 200% never mutates the file.
        QVERIFY(preferences.save("light", 1.25, "en-US"));
        QCOMPARE(bytes(path + ".bak"), original);
        const auto saved = bytes(path);
        QVERIFY(saved.contains("keep=owned-canary"));
        QLockFile otherWriter(path + ".xdzip-lock");
        QVERIFY(otherWriter.tryLock());
        QVERIFY(!preferences.save("dark", 1, "zh-CN"));
        QCOMPARE(bytes(path), saved);
        QCOMPARE(bytes(path + ".bak"), original);
        otherWriter.unlock();
        // Backup failure must not remove either the saved profile or old backup.
        QVERIFY(QFile::remove(path + ".bak"));
        QVERIFY(QDir().mkdir(path + ".bak"));
        QVERIFY(!preferences.save("dark", 1, "zh-CN"));
        QCOMPARE(bytes(path), saved);
        QVERIFY(QDir().rmdir(path + ".bak"));
        QVERIFY(input.open(QIODevice::WriteOnly | QIODevice::Truncate));
        const QByteArray future = "[General]\nschemaVersion=2\n[future]\nkeep=do-not-downgrade\n";
        QCOMPARE(input.write(future), future.size());
        input.close();
        UiPreferences older;
        QVERIFY(!older.error().isEmpty());
        QVERIFY(!older.saveDetails("light", 1, "en-US", "compact", false, true));
        QCOMPARE(bytes(path), future);
        QVERIFY(!QFile::exists(path + ".bak"));
    }

    void nativeFileOpenQueuesBeforeReadyAndDuringRead() {
        QTemporaryDir temporary;
        QVERIFY(temporary.isValid());
        const auto path = temporary.filePath(QStringLiteral("中文 空格 ; --smoke-test.zip"));
        QVERIFY(QFile::copy(QStringLiteral(XDZIP_FIXTURES "/normal.zip"), path));
        FileOpenRouter router;
        QFileOpenEvent early(QUrl::fromLocalFile(path));
        QVERIFY(router.eventFilter(nullptr, &early));
        QQmlApplicationEngine engine;
        engine.setInitialProperties({{"visible", false}});
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *controller = window->findChild<BrowserController *>();
        QVERIFY(controller);
        router.attach(controller, window);
        QVERIFY(!window->isVisible());
        QVERIFY(controller->browsingBusy());
        auto *welcomeContent = findVisual(window->contentItem(), "welcomeContent");
        QVERIFY(welcomeContent);
        QVERIFY(!welcomeContent->isVisible());
        bool firstFrameSeen = false;
        bool firstFrameShowedWelcome = false;
        QObject firstFrameObserver;
        connect(window, &QQuickWindow::frameSwapped, &firstFrameObserver, [&] {
            if (!firstFrameSeen) {
                firstFrameSeen = true;
                firstFrameShowedWelcome = welcomeContent->isVisible();
            }
        });
        window->show();
        QTRY_VERIFY(firstFrameSeen);
        QVERIFY(!firstFrameShowedWelcome);
        QTRY_VERIFY(controller->ready() && !controller->busy());
        QCOMPARE(controller->title(), QFileInfo(path).fileName());
        QFileOpenEvent remote(QUrl(QStringLiteral("https://example.invalid/archive.zip")));
        QVERIFY(router.eventFilter(nullptr, &remote));
        QTest::qWait(20);
        QCOMPARE(controller->title(), QFileInfo(path).fileName());
        controller->openPath(QStringLiteral(XDZIP_M2_FIXTURE));
        QVERIFY(controller->busy());
        QFileOpenEvent whileBusy(QUrl::fromLocalFile(path));
        QVERIFY(router.eventFilter(nullptr, &whileBusy));
        QTRY_VERIFY(controller->ready() && !controller->busy() &&
                    controller->title() == QFileInfo(path).fileName());
        window->close();
    }
    void externalFileOpenFailureRestoresWelcome_data() {
        QTest::addColumn<bool>("cancel");
        QTest::newRow("failure") << false;
        QTest::newRow("cancel") << true;
    }
    void externalFileOpenFailureRestoresWelcome() {
        QFETCH(bool, cancel);
        FileOpenRouter router;
        router.enqueue(cancel ? QStringLiteral(XDZIP_M2_FIXTURE)
                              : QStringLiteral(XDZIP_FIXTURES "/invalid.zip"));
        QQmlApplicationEngine engine;
        engine.setInitialProperties({{"visible", false}});
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *controller = window->findChild<BrowserController *>();
        auto *content = findVisual(window->contentItem(), "welcomeContent");
        QVERIFY(controller && content);
        router.attach(controller, window);
        QVERIFY(controller->browsingBusy());
        QVERIFY(!content->isVisible());
        if (cancel)
            controller->cancel();
        window->show();
        QTRY_VERIFY(!controller->browsingBusy());
        QVERIFY(!controller->ready());
        QTRY_VERIFY(content->isVisible());
        QVERIFY(!controller->message().isEmpty());
        window->close();
    }
    void settingsRestartConfirmation() {
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *owner = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        QVERIFY(owner);
        auto *preferences =
            engine.singletonInstance<UiPreferences *>("Xdzip.Settings", "AppSettings");
        QVERIFY(preferences);
        const auto theme = preferences->savedTheme();
        const auto scale = preferences->savedTextScale();
        const auto language = preferences->language();
        const auto density = preferences->savedDensity();
        const auto recent = preferences->recentEnabled();
        const auto restore = qScopeGuard(
            [&] { preferences->saveDetails(theme, scale, language, density, recent, false); });
        QVERIFY(preferences->saveDetails(theme, scale, preferences->activeLanguage(), density,
                                         recent, false));
        auto *settings = owner->findChild<QQuickWindow *>("settingsWindow");
        auto *confirmation = owner->findChild<QQuickWindow *>("restartWindow");
        QVERIFY(settings && confirmation);
        auto *save = settings->findChild<QQuickItem *>("settingsSave");
        auto *later = confirmation->findChild<QQuickItem *>("restartLater");
        auto *now = confirmation->findChild<QQuickItem *>("restartNow");
        QVERIFY(save && later && now);
        const auto changedLanguage = preferences->activeLanguage() == "en-US" ? "zh-CN" : "en-US";
        QVERIFY(QMetaObject::invokeMethod(settings, "configure"));
        settings->setProperty("draftLanguage", changedLanguage);
        settings->close(); // Cancel does not save or ask to restart.
        QTest::qWait(20);
        QVERIFY(!confirmation->isVisible());
        QCOMPARE(preferences->language(), preferences->activeLanguage());
        QVERIFY(QMetaObject::invokeMethod(settings, "configure"));
        settings->setProperty("draftLanguage", changedLanguage);
        QVERIFY(QMetaObject::invokeMethod(save, "clicked"));
        QTRY_VERIFY(confirmation->isVisible());
        QVERIFY(!settings->isVisible());
        QCOMPARE(confirmation->modality(), Qt::ApplicationModal);
        QCOMPARE(confirmation->transientParent(), owner);
        QCOMPARE(confirmation->width(), 491);
        QCOMPARE(later->mapToScene(QPointF()).y(), now->mapToScene(QPointF()).y());
        QVERIFY(now->mapToScene(QPointF(now->width(), now->height())).y() <=
                confirmation->height());
        const auto captures = qEnvironmentVariable("XDZIP_SETTINGS_VISUAL_AUDIT");
        if (!captures.isEmpty()) {
            QSignalSpy frames(confirmation, &QQuickWindow::frameSwapped);
            confirmation->update();
            QTRY_VERIFY(frames.count() > 0);
            QVERIFY(QDir().mkpath(captures));
            QVERIFY(visual_audit::save(confirmation, captures, "restart-confirmation"));
        }
        QCOMPARE(preferences->language(), QString::fromLatin1(changedLanguage));
        QVERIFY(QMetaObject::invokeMethod(later, "clicked"));
        QTRY_VERIFY(!confirmation->isVisible());
        QVERIFY(owner->isVisible());
        QVERIFY(!owner->property("restartAfterClose").toBool());
        QVERIFY(QMetaObject::invokeMethod(settings, "configure"));
        QVERIFY(QMetaObject::invokeMethod(save, "clicked")); // Unchanged saved language.
        QTest::qWait(20);
        QVERIFY(!confirmation->isVisible());
        QVERIFY(QMetaObject::invokeMethod(settings, "configure"));
        settings->setProperty("draftLanguage", preferences->activeLanguage());
        QVERIFY(QMetaObject::invokeMethod(save, "clicked")); // Back to running language.
        QTest::qWait(20);
        QVERIFY(!confirmation->isVisible());
        QVERIFY(QMetaObject::invokeMethod(settings, "configure"));
        settings->setProperty("draftLanguage", changedLanguage);
        QVERIFY(QMetaObject::invokeMethod(save, "clicked"));
        QTRY_VERIFY(confirmation->isVisible());
        QVERIFY(QMetaObject::invokeMethod(now, "clicked"));
        QVERIFY(owner->property("restartAfterClose").toBool());
        QTRY_VERIFY(!owner->isVisible()); // Existing asynchronous shutdown barrier completes.
    }
    void versionAboutWindow() {
        const auto restoreLanguage = qScopeGuard([] { initializeUiText(QStringLiteral("zh-CN")); });
        QVERIFY(!BuildInfo::version().isEmpty());
        QVERIFY(BuildInfo::buildNumber() > 0);
        for (const auto &language : {QStringLiteral("zh-CN"), QStringLiteral("en-US")}) {
            QVERIFY(initializeUiText(language));
            QQmlApplicationEngine engine;
            engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
            QVERIFY(!engine.rootObjects().isEmpty());
            auto *owner = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
            QVERIFY(owner);
            QSignalSpy initialFrame(owner, &QQuickWindow::frameSwapped);
            owner->update();
            QTRY_VERIFY(initialFrame.count() > 0);
            auto *settings = owner->findChild<QQuickWindow *>("settingsWindow");
            auto *about = owner->findChild<QQuickWindow *>("aboutWindow");
            QVERIFY(settings && about);
            QVERIFY(QMetaObject::invokeMethod(settings, "configure"));
            auto *entry = settings->findChild<QQuickItem *>("settingsAbout");
            QVERIFY(entry);
            QVERIFY(QMetaObject::invokeMethod(entry, "clicked"));
            QTRY_VERIFY(about->isVisible());
            QCOMPARE(about->transientParent(), settings);
            QQmlComponent changes(&engine);
            changes.setData("import QtQuick\nimport \"qrc:/xdzip/ui\" as Ui\n"
                            "QtObject { property bool dark: false; property real factor: 1; "
                            "onDarkChanged: Ui.Theme.dark = dark; onFactorChanged: "
                            "Ui.Theme.fontScale = factor }",
                            QUrl());
            std::unique_ptr<QObject> appearance(changes.create());
            QVERIFY(appearance);
            for (const bool dark : {false, true}) {
                appearance->setProperty("dark", dark);
                for (const double scale : {1., 1.25, 1.5}) {
                    appearance->setProperty("factor", scale);
                    QTest::qWait(30);
                    QCOMPARE(about->width(), 491);
                    QVERIFY(about->height() <= 640);
                    auto *artwork = findVisual(about->contentItem(), "aboutArtwork");
                    QVERIFY(artwork && visibleImagesReady(about->contentItem()));
                    QVERIFY(artwork->property("sourceSize").toSize().width() >=
                            qCeil(artwork->width() * about->devicePixelRatio()));
                    auto *version = findVisual(about->contentItem(), "aboutValue-1");
                    QVERIFY(version);
                    QCOMPARE(version->property("text").toString(), BuildInfo::version());
                    auto *copy = about->findChild<QQuickItem *>("aboutCopy");
                    auto *done = about->findChild<QQuickItem *>("aboutDone");
                    QVERIFY(copy && done);
                    QCOMPARE(copy->mapToScene(QPointF()).y(), done->mapToScene(QPointF()).y());
                    QVERIFY(done->mapToScene(QPointF(done->width(), done->height())).y() <=
                            about->height());
                    const auto captures = qEnvironmentVariable("XDZIP_ABOUT_AUDIT");
                    if (!captures.isEmpty()) {
                        QVERIFY(QDir().mkpath(captures));
                        QVERIFY(visual_audit::save(about, captures,
                                                   language + (dark ? "-dark-" : "-light-") +
                                                       QString::number(scale)));
                    }
                }
            }
            about->setProperty("requestedHeight", 400);
            QTRY_COMPARE(about->height(), 400);
            auto *homepage = findVisual(about->contentItem(), "aboutHomepage");
            auto *footer = findVisual(about->contentItem(), "aboutFooter");
            QVERIFY(homepage && footer);
            about->requestActivate();
            QVERIFY(QTest::qWaitForWindowActive(about));
            homepage->forceActiveFocus(Qt::TabFocusReason);
            QTRY_VERIFY(homepage->hasActiveFocus());
            auto *scroll = findVisual(about->contentItem(), "aboutScroll");
            QVERIFY(scroll);
            QTRY_VERIFY2(
                homepage->mapToScene(QPointF(0, homepage->height())).y() <= footer->y() + 1,
                qPrintable(
                    QStringLiteral("active=%1 focus=%2 bottom=%3 footer=%4 view=%5 content=%6")
                        .arg(about->isActive())
                        .arg(homepage->hasActiveFocus())
                        .arg(homepage->mapToScene(QPointF(0, homepage->height())).y())
                        .arg(footer->y())
                        .arg(scroll->height())
                        .arg(scroll->property("contentHeight").toReal())));
            auto *copy = about->findChild<QObject *>("aboutCopy");
            QVERIFY(QMetaObject::invokeMethod(copy, "clicked"));
            const auto diagnostic =
                QJsonDocument::fromJson(QGuiApplication::clipboard()->text().toUtf8()).object();
            QCOMPARE(diagnostic.value("version").toString(), BuildInfo::version());
            QCOMPARE(diagnostic.value("build").toInt(), BuildInfo::buildNumber());
            QVERIFY(!diagnostic.contains("path"));
            auto *done = about->findChild<QObject *>("aboutDone");
            QVERIFY(QMetaObject::invokeMethod(done, "clicked"));
            QTRY_VERIFY(!about->isVisible());
            appearance->setProperty("factor", 1.);
            appearance->setProperty("dark", false);
            settings->close();
            owner->close();
        }
        QVERIFY(initializeUiText(QStringLiteral("zh-CN")));
    }
    void windowsCaptionControlsAndWindowState() {
#ifdef Q_OS_WIN
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        QVERIFY(window);
        QVERIFY(!window->flags().testFlag(Qt::FramelessWindowHint));
        QTRY_VERIFY(window->isVisible());
        QCOMPARE(window->size(), QSize(1024, 622));
        QVariantMap nativeFrame;
        if (QGuiApplication::platformName() == QStringLiteral("windows")) {
            auto *frame = window->findChild<QObject *>("windowsNativeFrame");
            QVERIFY(frame);
            QVERIFY(QMetaObject::invokeMethod(frame, "diagnostics",
                                              Q_RETURN_ARG(QVariantMap, nativeFrame)));
            QVERIFY(nativeFrame.value("attached").toBool());
            QVERIFY(nativeFrame.value("captionStyle").toBool());
            QVERIFY(nativeFrame.value("resizeStyle").toBool());
            if (nativeFrame.value("composition").toBool())
                QVERIFY(nativeFrame.value("nonClientRendering").toBool());
            QCOMPARE(nativeFrame.value("topInsetPixels").toInt(), 1);
            if (nativeFrame.value("cornerPreferenceSupported").toBool())
                QCOMPARE(nativeFrame.value("cornerPreference").toInt(), 2);
        }
        auto *bar = findVisual(window->contentItem(), "archiveTitlebar");
        auto *settings = findVisual(window->contentItem(), "archiveSettings");
        auto *minimize = findVisual(window->contentItem(), "windowMinimize");
        auto *maximize = findVisual(window->contentItem(), "windowMaximize");
        auto *close = findVisual(window->contentItem(), "windowClose");
        QVERIFY(bar && settings && minimize && maximize && close);
        QCOMPARE(bar->height(), 48.0);
        QCOMPARE(minimize->width(), 46.0);
        QVERIFY(settings->mapToScene(QPointF(settings->width(), 0)).x() <
                minimize->mapToScene(QPointF()).x());
        QCOMPARE(close->mapToScene(QPointF(close->width(), 0)).x(), qreal(window->width()));
        const auto audit = qEnvironmentVariable("XDZIP_WINDOWS_AUDIT");
        if (!audit.isEmpty()) {
            QVERIFY(QDir().mkpath(audit));
            QVERIFY(visual_audit::save(window, audit, "windows-caption"));
            QFile frameRecord(audit + "/windows-native-frame.json");
            QVERIFY(frameRecord.open(QIODevice::WriteOnly));
            QVERIFY(frameRecord.write(
                        QJsonDocument(QJsonObject::fromVariantMap(nativeFrame)).toJson()) > 0);
            if (QGuiApplication::platformName() == QStringLiteral("windows")) {
                // A solid, owned backdrop makes the external DWM border/shadow
                // visible without recording unrelated desktop content.
                window->resize(900, 560);
                const auto available = window->screen()->availableGeometry();
                window->setPosition(available.center() - QPoint(450, 280));
                QQuickWindow backdrop;
                backdrop.setFlags(Qt::Tool | Qt::FramelessWindowHint | Qt::WindowStaysOnTopHint);
                backdrop.setColor(QColor("#CFD8E5"));
                backdrop.setGeometry(window->frameGeometry().adjusted(-32, -32, 32, 32));
                backdrop.show();
                window->setFlag(Qt::WindowStaysOnTopHint, true);
                window->show();
                window->raise();
                window->requestActivate();
                QTest::qWait(300);
                QVariantMap frameState;
                QVERIFY(QMetaObject::invokeMethod(
                    window->findChild<QObject *>("windowsNativeFrame"), "diagnostics",
                    Q_RETURN_ARG(QVariantMap, frameState)));
                QVERIFY(frameState.value("visibleAtCenter").toBool());
                const auto bounds = frameState.value("frameBoundsNative").toList();
                const auto origin = frameState.value("monitorOriginNative").toList();
                QCOMPARE(bounds.size(), 4);
                QCOMPARE(origin.size(), 2);
                const QRect crop(bounds[0].toInt() - origin[0].toInt() - 24,
                                 bounds[1].toInt() - origin[1].toInt() - 24, bounds[2].toInt() + 48,
                                 bounds[3].toInt() + 48);
                const auto desktop = window->screen()->grabWindow(0).toImage();
                QVERIFY(desktop.rect().contains(crop));
                const auto image = desktop.copy(crop);
                for (const auto point :
                     {crop.topLeft(), crop.topRight(), crop.bottomLeft(), crop.bottomRight()}) {
                    const POINT nativePoint{point.x() + origin[0].toInt(),
                                            point.y() + origin[1].toInt()};
                    const auto owner = WindowFromPoint(nativePoint);
                    QVERIFY(owner == reinterpret_cast<HWND>(backdrop.winId()));
                    const auto color = image.pixelColor(point - crop.topLeft());
                    // DWM's shadow darkens the owned blue backdrop. Verify its
                    // hue and ownership instead of requiring an unshaded pixel.
                    const auto base = backdrop.color();
                    QVERIFY(color.blue() <= base.blue() + 5 && color.blue() >= base.blue() / 3);
                    QVERIFY(qAbs(color.red() * base.blue() - color.blue() * base.red()) <=
                            8 * base.blue());
                    QVERIFY(qAbs(color.green() * base.blue() - color.blue() * base.green()) <=
                            8 * base.blue());
                }
                QVERIFY(image.save(audit + "/windows-dwm-frame.png"));
                window->setFlag(Qt::WindowStaysOnTopHint, false);
                window->show();
            }
        }
        const auto click = [window](QQuickItem *item) {
            QTest::mouseClick(
                window, Qt::LeftButton, Qt::NoModifier,
                item->mapToScene(QPointF(item->width() / 2, item->height() / 2)).toPoint());
        };
        click(maximize);
        QTRY_COMPARE(window->visibility(), QWindow::Maximized);
        if (QGuiApplication::platformName() == QStringLiteral("windows"))
            QTRY_VERIFY(window->screen()
                            ->availableGeometry()
                            .adjusted(-1, -1, 1, 1)
                            .contains(window->geometry()));
        QCOMPARE(maximize->property("captionAction").toString(), QStringLiteral("restore"));
        click(maximize);
        QTRY_COMPARE(window->visibility(), QWindow::Windowed);
        QTRY_COMPARE(close->mapToScene(QPointF(close->width(), 0)).x(), qreal(window->width()));
        QTest::mouseDClick(window, Qt::LeftButton, Qt::NoModifier, QPoint(window->width() / 2, 24));
        QTRY_COMPARE(window->visibility(), QWindow::Maximized);
        QVERIFY(QMetaObject::invokeMethod(window, "toggleMaximized"));
        QTRY_COMPARE(window->visibility(), QWindow::Windowed);
        QTRY_COMPARE(close->mapToScene(QPointF(close->width(), 0)).x(), qreal(window->width()));
        click(minimize);
        QTRY_COMPARE(window->visibility(), QWindow::Minimized);
        window->showNormal();
        QTRY_COMPARE(window->visibility(), QWindow::Windowed);
        QTRY_COMPARE(close->mapToScene(QPointF(close->width(), 0)).x(), qreal(window->width()));
        // Closing the new button still passes through the runtime shutdown barrier.
        click(close);
        QTRY_VERIFY(!window->isVisible());
#else
        QSKIP("Windows custom caption regression");
#endif
    }
    void initTestCase() {
        FocusDismissPolicy::install(QCoreApplication::instance());
        QQuickStyle::setStyle("Basic");
        Q_INIT_RESOURCE(ui);
        QVERIFY(m_settingsProfile.isValid());
        if (qEnvironmentVariableIsEmpty("XDZIP_SETTINGS_PATH"))
            qputenv("XDZIP_SETTINGS_PATH", m_settingsProfile.filePath("settings.ini").toUtf8());
        QVERIFY(initializeUiText(qEnvironmentVariable("XDZIP_UI_TEST_LANGUAGE", "zh-CN")));
        QVERIFY(installUiFont());
        qmlRegisterType<BrowserController>("Xdzip.Native", 1, 0, "BrowserController");
    }
    void init() {
        // Platform offscreen notices are expected; broken bindings or image
        // resources must never pass a visual or interaction regression silently.
        QTest::failOnWarning(QRegularExpression(
            "Binding loop detected|Cannot open:|ReferenceError|TypeError|Unable to assign"));
    }
    void cleanup() {
        QTRY_COMPARE_WITH_TIMEOUT(RuntimeReceiver::activeReceivers(), 0, 5000);
    }
    void privateTestDesktopInputAndAccessibility() {
        const bool accessibilityWasActive = QAccessible::isActive();
        QAccessible::setActive(true);
        const auto restoreAccessibility =
            qScopeGuard([&] { QAccessible::setActive(accessibilityWasActive); });
        QElapsedTimer firstFrame;
        firstFrame.start();
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        QVERIFY(window);
        auto *create = window->findChild<QQuickWindow *>("createWindow");
        QVERIFY(create);
        QSignalSpy frames(create, &QQuickWindow::frameSwapped);
        QVERIFY(QMetaObject::invokeMethod(create, "configure"));
        QTRY_VERIFY(create->isVisible());
        create->update();
        QTRY_VERIFY_WITH_TIMEOUT(!frames.isEmpty(), 5000);
        const auto firstPresentedMs = firstFrame.elapsed();
        QVERIFY(!create->grabWindow().isNull());
        const auto api = create->rendererInterface()->graphicsApi();
        const bool hardwareApi =
            api == QSGRendererInterface::Metal || api == QSGRendererInterface::OpenGL ||
            api == QSGRendererInterface::Direct3D11 || api == QSGRendererInterface::Direct3D12 ||
            api == QSGRendererInterface::Vulkan;
        if (qEnvironmentVariable("XDZIP_REQUIRE_GPU") == "1")
            QVERIFY2(hardwareApi, "GPU validation must not silently fall back to software or Null");
        auto *field = findVisual(create->contentItem(), "createName");
        auto *action = findVisual(create->contentItem(), "startCreate");
        QVERIFY(field && action);
        field->setProperty("text", QString{});
        field->forceActiveFocus();
        QTRY_VERIFY(field->hasActiveFocus());
        QInputMethodEvent composing(QStringLiteral("中文"), {});
        QCoreApplication::sendEvent(create, &composing);
        QTRY_VERIFY(field->property("inputMethodComposing").toBool());
        QCOMPARE(field->property("text").toString(), QString{});
        QInputMethodEvent commit;
        const auto name = QString::fromUtf8("中文文件📦.zip");
        commit.setCommitString(name);
        QCoreApplication::sendEvent(create, &commit);
        QTRY_COMPARE(field->property("text").toString(), name);
        QTRY_VERIFY(!field->property("inputMethodComposing").toBool());
        auto *editable = QAccessible::queryAccessibleInterface(field);
        auto *button = QAccessible::queryAccessibleInterface(action);
        QVERIFY(editable && button);
        QCOMPARE(editable->role(), QAccessible::EditableText);
        QCOMPARE(editable->text(QAccessible::Name), qtTrId("ui.0d568cd7b07d"));
        QCOMPARE(editable->text(QAccessible::Value), name);
        QVERIFY(editable->textInterface());
        QCOMPARE(button->role(), QAccessible::Button);
        QCOMPARE(button->text(QAccessible::Name), qtTrId("ui.e185d04bcc5f"));
        QVERIFY(button->state().disabled);
        QTest::keyClick(create, Qt::Key_Escape);
        QTRY_VERIFY(!field->hasActiveFocus());
        QCOMPARE(field->property("text").toString(), name);
        const auto directory = qEnvironmentVariable("XDZIP_PRIVATE_TEST_AUDIT");
        if (!directory.isEmpty()) {
            QVERIFY(QDir().mkpath(directory));
            const QJsonObject evidence{
                {"schema", 1},
                {"qt", qVersion()},
                {"platform", QGuiApplication::platformName()},
                {"architecture", QSysInfo::currentCpuArchitecture()},
                {"graphicsApi", static_cast<int>(api)},
                {"hardwareApi", hardwareApi},
                {"framesPresented", frames.count()},
                {"firstPresentedMs", firstPresentedMs},
                {"measurementScope",
                 "Warm in-process test startup, not release cold-start benchmark"},
                {"syntheticIme", true},
                {"accessibleInterfaces", true},
                {"nativeImeManual", "pending"},
                {"screenReaderManual", "pending"}};
            QFile output(QDir(directory).filePath("desktop-input-accessibility.json"));
            QVERIFY(output.open(QIODevice::WriteOnly));
            const auto bytes = QJsonDocument(evidence).toJson();
            QCOMPARE(output.write(bytes), bytes.size());
        }
        create->close();
    }
    void accessibleRadioActionsCommitSelection() {
        QTemporaryDir profile;
        QVERIFY(profile.isValid());
        const auto previous = qgetenv("XDZIP_SETTINGS_PATH");
        qputenv("XDZIP_SETTINGS_PATH", profile.filePath("settings.ini").toUtf8());
        const auto restoreProfile = qScopeGuard([&] {
            if (previous.isNull())
                qunsetenv("XDZIP_SETTINGS_PATH");
            else
                qputenv("XDZIP_SETTINGS_PATH", previous);
        });
        const bool wasActive = QAccessible::isActive();
        QAccessible::setActive(false);
        const auto restoreAccessibility = qScopeGuard([&] { QAccessible::setActive(wasActive); });
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *preferences =
            engine.singletonInstance<UiPreferences *>("Xdzip.Settings", "AppSettings");
        QVERIFY(window && preferences);
        const auto restorePreview = qScopeGuard([&] { preferences->cancelPreview(); });
        auto *settings = window->findChild<QQuickWindow *>("settingsWindow");
        QVERIFY(settings);
        QVERIFY(QMetaObject::invokeMethod(settings, "configure"));
        QVERIFY(QTest::qWaitForWindowExposed(settings));
        QVERIFY(findVisual(settings->contentItem(), "settingsTheme-2"));
        // Native clients can attach after controls were created. Qt resynchronizes
        // the attached state from the underlying control at this boundary.
        QAccessible::setActive(true);
        const QStringList themes{"system", "light", "dark"};
        const QStringList actions{QAccessibleActionInterface::pressAction(),
                                  QAccessibleActionInterface::toggleAction()};
        for (const auto &action : actions) {
            // Repeat the selected option: a radio action must never deselect it.
            for (const int index : {2, 1, 2, 2, 0}) {
                auto *card =
                    findVisual(settings->contentItem(), "settingsTheme-" + QString::number(index));
                QVERIFY(card);
                auto *accessible = QAccessible::queryAccessibleInterface(card);
                QVERIFY(accessible && accessible->actionInterface());
                QCOMPARE(accessible->role(), QAccessible::RadioButton);
                QVERIFY(accessible->state().checkable);
                QCOMPARE(bool(accessible->state().checkable), card->property("checkable").toBool());
                QVERIFY(accessible->actionInterface()->actionNames().contains(action));
                accessible->actionInterface()->doAction(action);
                QTRY_COMPARE(settings->property("draftTheme").toString(), themes[index]);
                QTRY_COMPARE(preferences->theme(), themes[index]);
                if (index != 0)
                    QTRY_COMPARE(window->property("darkMode").toBool(), index == 2);
                for (int choice = 0; choice < themes.size(); ++choice) {
                    auto *item = findVisual(settings->contentItem(),
                                            "settingsTheme-" + QString::number(choice));
                    QVERIFY(item);
                    QTRY_COMPARE(item->property("checked").toBool(), choice == index);
                    QTRY_COMPARE(QAccessible::queryAccessibleInterface(item)->state().checked,
                                 choice == index);
                }
                QCOMPARE(preferences->savedTheme(), QStringLiteral("system"));
            }
        }
        auto *selectedTheme = findVisual(settings->contentItem(), "settingsTheme-0");
        QVERIFY(selectedTheme);
        QTest::mouseClick(
            settings, Qt::LeftButton, Qt::NoModifier,
            selectedTheme
                ->mapToScene(QPointF(selectedTheme->width() / 2, selectedTheme->height() / 2))
                .toPoint());
        QCOMPARE(settings->property("draftTheme").toString(), QStringLiteral("system"));
        QVERIFY(selectedTheme->property("checked").toBool());
        settings->close();
        QCOMPARE(preferences->theme(), QStringLiteral("system"));
        auto *create = window->findChild<QQuickWindow *>("createWindow");
        QVERIFY(create);
        QVERIFY(QMetaObject::invokeMethod(create, "configure"));
        QVERIFY(QTest::qWaitForWindowExposed(create));
        auto *formats = findVisual(create->contentItem(), "createFormat");
        QVERIFY(formats);
        const QStringList extensions{".zip", ".tar", ".tar.gz"};
        for (const auto &action : actions) {
            for (const int index : {1, 2, 0, 0}) {
                auto *item =
                    findVisual(create->contentItem(), "createFormat-" + QString::number(index));
                QVERIFY(item);
                auto *accessible = QAccessible::queryAccessibleInterface(item);
                QVERIFY(accessible && accessible->actionInterface());
                accessible->actionInterface()->doAction(action);
                QTRY_COMPARE(formats->property("currentIndex").toInt(), index);
                QTRY_COMPARE(create->property("formatExtension").toString(), extensions[index]);
                for (int choice = 0; choice < extensions.size(); ++choice)
                    QTRY_COMPARE(
                        findVisual(create->contentItem(), "createFormat-" + QString::number(choice))
                            ->property("checked")
                            .toBool(),
                        choice == index);
            }
        }
        auto *selectedFormat = findVisual(create->contentItem(), "createFormat-0");
        QVERIFY(selectedFormat);
        QTest::mouseClick(
            create, Qt::LeftButton, Qt::NoModifier,
            selectedFormat
                ->mapToScene(QPointF(selectedFormat->width() / 2, selectedFormat->height() / 2))
                .toPoint());
        QCOMPARE(formats->property("currentIndex").toInt(), 0);
        QVERIFY(selectedFormat->property("checked").toBool());
        formats->setProperty("options",
                             QVariantList{QVariantMap{{"text", "ZIP"}},
                                          QVariantMap{{"text", "TAR"}, {"enabled", false}},
                                          QVariantMap{{"text", "TAR.GZ"}}});
        QTRY_VERIFY(findVisual(create->contentItem(), "createFormat-1"));
        auto *disabled = findVisual(create->contentItem(), "createFormat-1");
        QVERIFY(!disabled->isEnabled());
        for (const auto &action : actions) {
            QAccessible::queryAccessibleInterface(disabled)->actionInterface()->doAction(action);
            QCOMPARE(formats->property("currentIndex").toInt(), 0);
        }
        create->close();
    }
    void accessibleCheckboxActionsCommitIntent() {
        QTemporaryDir profile;
        QVERIFY(profile.isValid());
        const auto previous = qgetenv("XDZIP_SETTINGS_PATH");
        qputenv("XDZIP_SETTINGS_PATH", profile.filePath("settings.ini").toUtf8());
        const auto restore = qScopeGuard([&] {
            if (previous.isNull())
                qunsetenv("XDZIP_SETTINGS_PATH");
            else
                qputenv("XDZIP_SETTINGS_PATH", previous);
        });
        const bool wasActive = QAccessible::isActive();
        QAccessible::setActive(true);
        const auto restoreAccessibility = qScopeGuard([&] { QAccessible::setActive(wasActive); });
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        QVERIFY(window && QTest::qWaitForWindowExposed(window));
        auto *settings = window->findChild<QQuickWindow *>("settingsWindow");
        QVERIFY(settings && QMetaObject::invokeMethod(settings, "configure"));
        QVERIFY(QTest::qWaitForWindowExposed(settings));
        const QStringList actions{QAccessibleActionInterface::pressAction(),
                                  QAccessibleActionInterface::toggleAction()};
        for (const auto &action : actions) {
            for (const auto &name : {QStringLiteral("Tray"), QStringLiteral("Recent")}) {
                auto *item = findVisual(settings->contentItem(), "settings" + name);
                QVERIFY(item);
                auto *accessible = QAccessible::queryAccessibleInterface(item);
                QVERIFY(accessible && accessible->actionInterface());
                const auto draft = ("draft" + name).toUtf8();
                const bool before = settings->property(draft).toBool();
                accessible->actionInterface()->doAction(action);
                QTRY_COMPARE(settings->property(draft).toBool(), !before);
                QCOMPARE(item->property("checked").toBool(), !before);
                accessible->actionInterface()->doAction(action);
                QTRY_COMPARE(settings->property(draft).toBool(), before);
            }
        }
        settings->close();
        auto *controller = window->findChild<BrowserController *>();
        QVERIFY(controller);
        controller->openPath(QStringLiteral(XDZIP_FIXTURES "/../extract/tree.zip"));
        QTRY_VERIFY(controller->ready() && !controller->busy());
        auto *header = findVisual(window->contentItem(), "selectPage");
        QVERIFY(header);
        for (const auto &action : actions) {
            auto *accessible = QAccessible::queryAccessibleInterface(header);
            QVERIFY(accessible && accessible->actionInterface());
            accessible->actionInterface()->doAction(action);
            QTRY_COMPARE(controller->selectedCount(), controller->entries()->rowCount());
            accessible->actionInterface()->doAction(action);
            QTRY_COMPARE(controller->selectedCount(), 0);
        }
        QQuickItem *entry = nullptr;
        QTRY_VERIFY((entry = findVisual(window->contentItem(), "entrySelection")));
        auto *accessible = QAccessible::queryAccessibleInterface(entry);
        QVERIFY(accessible && accessible->actionInterface());
        accessible->actionInterface()->doAction(QAccessibleActionInterface::pressAction());
        QTRY_COMPARE(controller->selectedCount(), 1);
        QCOMPARE(header->property("checkState").toInt(), int(Qt::PartiallyChecked));
        accessible->actionInterface()->doAction(QAccessibleActionInterface::toggleAction());
        QTRY_COMPARE(controller->selectedCount(), 0);
        controller->selectAll(true);
        controller->prepareExtraction(false);
        auto *extract = window->findChild<QQuickWindow *>("extractWindow");
        QTRY_VERIFY(extract && extract->isVisible());
        for (const auto &action : actions) {
            for (const bool all : {true, true, false, false}) {
                auto *scope = findVisual(extract->contentItem(),
                                         all ? "extractAllScope" : "extractSelectedScope");
                QVERIFY(scope);
                auto *scopeAccessible = QAccessible::queryAccessibleInterface(scope);
                QVERIFY(scopeAccessible && scopeAccessible->actionInterface());
                scopeAccessible->actionInterface()->doAction(action);
                QTRY_COMPARE(extract->property("allFiles").toBool(), all);
                QCOMPARE(scope->property("checked").toBool(), true);
            }
        }
        extract->close();
        QTRY_VERIFY(!controller->busy());
        controller->selectAll(false);
        entry->setEnabled(false);
        accessible->actionInterface()->doAction(QAccessibleActionInterface::pressAction());
        QCOMPARE(controller->selectedCount(), 0);
    }
    void settingsOpenModeSelectionPersists_data() {
        QTest::addColumn<bool>("pointer");
        QTest::newRow("pointer") << true;
        QTest::newRow("popup-keyboard") << false;
    }
    void settingsOpenModeSelectionPersists() {
        QFETCH(bool, pointer);
        if (qEnvironmentVariable("XDZIP_OPEN_MODE_CHILD") == "1") {
            UiPreferences restored;
            QCOMPARE(restored.openMode(), QStringLiteral("windows"));
            return;
        }
        QTemporaryDir profile;
        QVERIFY(profile.isValid());
        const auto previous = qgetenv("XDZIP_SETTINGS_PATH");
        const auto restore = qScopeGuard([previous] { qputenv("XDZIP_SETTINGS_PATH", previous); });
        const auto path = profile.filePath("settings.ini");
        qputenv("XDZIP_SETTINGS_PATH", path.toUtf8());
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *preferences =
            engine.singletonInstance<UiPreferences *>("Xdzip.Settings", "AppSettings");
        ApplicationSessions sessions(&engine, preferences);
        sessions.adopt(window);
        auto *settings = window->findChild<QQuickWindow *>("settingsWindow");
        QVERIFY(settings);
        auto *choice = findVisual(settings->contentItem(), "settingsOpenMode");
        auto *save = findVisual(settings->contentItem(), "settingsSave");
        auto *cancel = findVisual(settings->contentItem(), "settingsCancel");
        auto *defaults = findVisual(settings->contentItem(), "settingsDefaults");
        auto *footer = findVisual(settings->contentItem(), "settingsFooter");
        QVERIFY(choice && save && cancel && defaults && footer);
        const auto click = [](QQuickWindow *owner, QQuickItem *item) {
            QTest::mouseClick(
                owner, Qt::LeftButton, Qt::NoModifier,
                item->mapToScene(QPointF(item->width() / 2, item->height() / 2)).toPoint());
        };
        auto *popup = choice->property("popup").value<QObject *>();
        QVERIFY(popup);
        QSignalSpy activated(choice, SIGNAL(activated(int)));
        const auto selectWindows = [&] {
            choice->forceActiveFocus(Qt::TabFocusReason);
            QTRY_VERIFY(choice->mapToScene(QPointF(0, choice->height())).y() <= footer->y() + 1);
            click(settings, choice);
            QTRY_VERIFY(popup->property("visible").toBool());
            if (pointer) {
                auto *content = popup->property("contentItem").value<QQuickItem *>();
                QVERIFY(content);
                QTRY_VERIFY(findVisual(content, "settingsOpenModeOption-1"));
                click(settings, findVisual(content, "settingsOpenModeOption-1"));
            } else {
                QTest::keyClick(settings, Qt::Key_End);
                QTest::keyClick(settings, Qt::Key_Return);
            }
            QTRY_VERIFY(!popup->property("visible").toBool());
        };
        QVERIFY(QMetaObject::invokeMethod(settings, "configure"));
        QVERIFY(QTest::qWaitForWindowExposed(settings));
        choice->forceActiveFocus(Qt::TabFocusReason);
        QVERIFY(QMetaObject::invokeMethod(popup, "open"));
        QTRY_VERIFY(popup->property("visible").toBool());
        QTest::keyClick(settings, Qt::Key_End);
        QTest::keyClick(settings, Qt::Key_Escape);
        QTRY_VERIFY(!popup->property("visible").toBool());
        QCOMPARE(choice->property("currentIndex").toInt(), 0);
        QCOMPARE(activated.count(), 0);
        selectWindows();
        QTRY_COMPARE(choice->property("currentIndex").toInt(), 1);
        QCOMPARE(settings->property("draftOpenMode").toString(), QStringLiteral("windows"));
        QCOMPARE(activated.count(), 1);
        QCOMPARE(activated.first().first().toInt(), 1);
        QCOMPARE(preferences->openMode(), QStringLiteral("tabs"));
        click(settings, cancel);
        QTRY_VERIFY(!settings->isVisible());
        QVERIFY(!QFile::exists(path));
        QVERIFY(QMetaObject::invokeMethod(settings, "configure"));
        QTRY_COMPARE(choice->property("currentIndex").toInt(), 0);
        selectWindows();
        settings->setProperty("draftTheme", "dark"); // Other controls must not undo the choice.
        QCOMPARE(choice->property("currentIndex").toInt(), 1);
        QLockFile lock(path + ".xdzip-lock");
        QVERIFY(lock.tryLock());
        click(settings, save);
        QVERIFY(settings->isVisible());
        QVERIFY(!preferences->error().isEmpty());
        QCOMPARE(settings->property("draftOpenMode").toString(), QStringLiteral("windows"));
        QCOMPARE(preferences->openMode(), QStringLiteral("tabs"));
        lock.unlock();
        click(settings, save);
        QTRY_VERIFY(!settings->isVisible());
        QCOMPARE(preferences->openMode(), QStringLiteral("windows"));
        QSettings persisted(path, QSettings::IniFormat);
        QCOMPARE(persisted.value("desktop/openMode").toString(), QStringLiteral("windows"));
        QProcess child;
        child.setProcessChannelMode(QProcess::MergedChannels);
        auto environment = QProcessEnvironment::systemEnvironment();
        environment.insert("XDZIP_OPEN_MODE_CHILD", "1");
        child.setProcessEnvironment(environment);
        child.start(QCoreApplication::applicationFilePath(),
                    {QStringLiteral("settingsOpenModeSelectionPersists:") +
                     QString::fromLatin1(QTest::currentDataTag())});
        QVERIFY(child.waitForFinished(20000));
        QVERIFY2(child.exitStatus() == QProcess::NormalExit && child.exitCode() == 0,
                 child.readAllStandardOutput().constData());
        sessions.newSession();
        QCOMPARE(sessions.windows().size(), 2);
        QVERIFY(QMetaObject::invokeMethod(settings, "configure"));
        QCOMPARE(settings->property("draftOpenMode").toString(), QStringLiteral("windows"));
        QCOMPARE(choice->property("currentIndex").toInt(), 1);
        activated.clear();
        click(settings, defaults);
        QCOMPARE(choice->property("currentIndex").toInt(), 0);
        QCOMPARE(activated.count(),
                 0); // Programmatic draft synchronization is not user activation.
        QCOMPARE(preferences->openMode(), QStringLiteral("windows"));
        click(settings, save);
        QTRY_VERIFY(!settings->isVisible());
        QCOMPARE(preferences->openMode(), QStringLiteral("tabs"));
        sessions.newSession();
        QCOMPARE(sessions.windows().size(), 2);
        QSignalSpy joined(&sessions, &ApplicationSessions::readyToQuit);
        sessions.confirmQuit(false);
        QTRY_VERIFY(!joined.isEmpty());
    }
    void settingsWindowPersistsAndRollsBack() {
        QTemporaryDir profile;
        QVERIFY(profile.isValid());
        const auto previous = qgetenv("XDZIP_SETTINGS_PATH");
        const auto restore = qScopeGuard([previous] {
            if (previous.isNull())
                qunsetenv("XDZIP_SETTINGS_PATH");
            else
                qputenv("XDZIP_SETTINGS_PATH", previous);
        });
        const auto path = profile.filePath("settings.ini");
        qputenv("XDZIP_SETTINGS_PATH", path.toUtf8());
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        QVERIFY(QTest::qWaitForWindowExposed(window));
        auto *settings = window->findChild<QQuickWindow *>("settingsWindow");
        auto *entry = findVisual(window->contentItem(), "archiveSettings");
        QVERIFY(settings && entry);
        const auto click = [](QQuickWindow *owner, QQuickItem *item) {
            QTest::mouseClick(
                owner, Qt::LeftButton, Qt::NoModifier,
                item->mapToScene(QPointF(item->width() / 2, item->height() / 2)).toPoint());
        };
        const bool originalDark = window->property("darkMode").toBool();
        click(window, entry);
        QTRY_VERIFY(settings->isVisible());
        QCOMPARE(settings->width(), 612);
        QVERIFY(visibleImagesReady(settings->contentItem()));
        const auto captureSettings = [&](const QString &name) {
            const auto directory = qEnvironmentVariable("XDZIP_SETTINGS_VISUAL_AUDIT");
            return directory.isEmpty() ||
                   (QDir().mkpath(directory) && visual_audit::save(settings, directory, name));
        };
        QVERIFY(captureSettings("settings-light"));
        auto *themeChoice = findVisual(settings->contentItem(),
                                       originalDark ? "settingsTheme-1" : "settingsTheme-2");
        QVERIFY(themeChoice);
        click(settings, themeChoice);
        QTRY_COMPARE(window->property("darkMode").toBool(), !originalDark);
        auto *scaleChoice = findVisual(settings->contentItem(), "settingsTextScale");
        QVERIFY(scaleChoice);
        QCOMPARE(scaleChoice->property("font").value<QFont>().pixelSize(), 13);
        scaleChoice->forceActiveFocus();
        QTest::keyClick(settings, Qt::Key_Down);
        QTest::keyClick(settings, Qt::Key_End);
        QTest::keyClick(settings, Qt::Key_Return);
        QTRY_COMPARE(settings->property("draftScale").toDouble(), 1.5);
        QCOMPARE(scaleChoice->property("font").value<QFont>().pixelSize(), 20);
        QCOMPARE(scaleChoice->property("count").toInt(), 3);
        auto *cancel = findVisual(settings->contentItem(), "settingsCancel");
        auto *save = findVisual(settings->contentItem(), "settingsSave");
        QVERIFY(cancel && save);
        auto *defaults = findVisual(settings->contentItem(), "settingsDefaults");
        QVERIFY(defaults);
        QTRY_VERIFY(qAbs(cancel->mapToScene(QPointF()).y() - save->mapToScene(QPointF()).y()) <= 1);
        QTRY_VERIFY(qAbs(defaults->mapToScene(QPointF()).y() - save->mapToScene(QPointF()).y()) <=
                    1);
        QTRY_VERIFY(save->mapToScene(QPointF(0, save->height())).y() <= settings->height() + 1);
        QVERIFY(captureSettings("settings-large"));
        auto *languageChoice = findVisual(settings->contentItem(), "settingsLanguage");
        auto *settingsFooter = findVisual(settings->contentItem(), "settingsFooter");
        QVERIFY(languageChoice && settingsFooter);
        languageChoice->forceActiveFocus();
        auto *languagePopup = languageChoice->property("popup").value<QObject *>();
        QVERIFY(languagePopup);
        QVERIFY(QMetaObject::invokeMethod(languagePopup, "open"));
        QTRY_VERIFY(languagePopup->property("visible").toBool());
        QVERIFY(captureSettings("settings-language-menu-large"));
        QTest::keyClick(settings, Qt::Key_Escape);
        languageChoice->forceActiveFocus();
        QTRY_VERIFY(languageChoice->mapToScene(QPointF(0, languageChoice->height())).y() <=
                    settingsFooter->y() + 1);
        auto *densityChoice = findVisual(settings->contentItem(), "settingsDensity");
        QVERIFY(densityChoice);
        settings->setProperty("draftDensity", "compact");
        settings->setProperty("clearRecent", true);
        click(settings, cancel);
        QTRY_VERIFY(!settings->isVisible());
        QTRY_COMPARE(window->property("darkMode").toBool(), originalDark);
        QVERIFY(!QFile::exists(path));
        click(window, entry);
        QTRY_VERIFY(settings->isVisible());
        QCOMPARE(settings->property("draftScale").toDouble(), 1.0);
        settings->setProperty("draftTheme", "dark");
        settings->setProperty("draftLanguage", "en-US");
        QCOMPARE(settings->property("draftDensity").toString(), "standard");
        QCOMPARE(settings->property("clearRecent").toBool(), false);
        settings->setProperty("draftDensity", "compact");
        click(settings, save);
        QTRY_VERIFY(!settings->isVisible());
        QSettings persisted(path, QSettings::IniFormat);
        QCOMPARE(persisted.value("locale/language").toString(), "en-US");
        QCOMPARE(persisted.value("appearance/theme").toString(), "dark");
        QCOMPARE(persisted.value("schemaVersion").toInt(), 1);
        QCOMPARE(qtTrId("settings.title"), QStringLiteral("设置"));
        UiPreferences restored;
        QCOMPARE(restored.language(), "en-US");
        QCOMPARE(restored.savedTheme(), "dark");
        QCOMPARE(restored.savedDensity(), "compact");
        const auto recentPath = profile.filePath("recent.json");
        restored.rememberArchive(QStringLiteral(XDZIP_FIXTURES "/normal.zip"));
        QVERIFY(QFile::exists(recentPath));
        QFile recent(recentPath);
        QVERIFY(recent.open(QIODevice::ReadOnly));
        const auto before = recent.readAll();
        recent.close();
        QVERIFY(restored.saveDetails("dark", 1, "en-US", "compact", false, false));
        restored.rememberArchive(QStringLiteral(XDZIP_FIXTURES "/invalid.zip"));
        QVERIFY(recent.open(QIODevice::ReadOnly));
        QCOMPARE(recent.readAll(), before);
        recent.close();
        QVERIFY(restored.saveDetails("dark", 1, "en-US", "compact", false, true));
        QVERIFY(!QFile::exists(recentPath));
        QVERIFY(QFile::exists(QStringLiteral(XDZIP_FIXTURES "/normal.zip")));
        persisted.setValue("appearance/textScale", 2);
        persisted.sync();
        UiPreferences migrated;
        QCOMPARE(migrated.savedTextScale(), 1.5);
        QVERIFY(!migrated.save("dark", 2, "en-US"));
        QVERIFY(!restored.save("invalid", 1, "en-US"));
        QCOMPARE(restored.savedTheme(), "dark");
        // A separate process proves language restoration across an actual restart.
        QProcess child;
        child.setProcessChannelMode(QProcess::MergedChannels);
        auto environment = QProcessEnvironment::systemEnvironment();
        environment.insert("XDZIP_UI_TEST_LANGUAGE", "");
        environment.remove("XDZIP_LANGUAGE");
        environment.insert("XDZIP_SETTINGS_CHILD", "1");
        child.setProcessEnvironment(environment);
        child.start(QCoreApplication::applicationFilePath(), {"settingsEnglishStartup"});
        QVERIFY(child.waitForFinished(20000));
        QVERIFY2(child.exitStatus() == QProcess::NormalExit && child.exitCode() == 0,
                 child.readAllStandardOutput().constData());
        window->close();
    }
    void qtRuntimeTranslations() {
        QTranslator translator;
        QVERIFY(loadQtSystemTranslation(translator, QLocale(QStringLiteral("zh_CN")),
                                        QLibraryInfo::path(QLibraryInfo::TranslationsPath)));
        QVERIFY(translator.translate("QFileDialog", "Open").contains(QStringLiteral("打开")));
    }
    void settingsEnglishStartup() {
        if (qEnvironmentVariable("XDZIP_SETTINGS_CHILD") != "1")
            QSKIP("Covered by the isolated settings restart child process");
        QCOMPARE(QCoreApplication::instance()->property("xdzipUiLanguage").toString(), "en-US");
        QCOMPARE(qtTrId("settings.title"), QStringLiteral("Settings"));
        const auto systemLanguages = QLocale::system().uiLanguages();
        const bool systemChinese =
            !systemLanguages.isEmpty() && systemLanguages.first().startsWith("zh");
        QCOMPARE(systemDialogText("ui.1b59353f1874"),
                 systemChinese ? QStringLiteral("打开压缩包") : QStringLiteral("Open archive"));
        if (systemChinese)
            QVERIFY(QCoreApplication::translate("QFileDialog", "Open")
                        .contains(QStringLiteral("打开")));
        QCOMPARE(qtTrId("ui.2e84a56c5f19", 1), QStringLiteral("1 file"));
        QCOMPARE(qtTrId("ui.2e84a56c5f19", 2), QStringLiteral("2 files"));
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        QVERIFY(QTest::qWaitForWindowExposed(window));
        auto *open = findVisual(window->contentItem(), "welcomeOpen");
        QVERIFY(open);
        QCOMPARE(open->property("title").toString(), "Open archive");
        auto *settings = window->findChild<QQuickWindow *>("settingsWindow");
        QVERIFY(settings);
        QVERIFY(QMetaObject::invokeMethod(settings, "configure"));
        QTRY_VERIFY(settings->isVisible());
        QCOMPARE(settings->title(), "Settings");
        const auto directory = qEnvironmentVariable("XDZIP_SETTINGS_VISUAL_AUDIT");
        if (!directory.isEmpty())
            QVERIFY(visual_audit::save(settings, directory, "settings-en-US"));
        for (auto *owned : window->findChildren<QQuickWindow *>())
            QVERIFY(!owned->title().contains(QRegularExpression("[\\x{3400}-\\x{9fff}]")));
        settings->close();
        auto *controller = window->findChild<BrowserController *>();
        QVERIFY(controller);
        controller->openPath(QStringLiteral(XDZIP_FIXTURES "/normal.zip"));
        QTRY_VERIFY(controller->ready() && !controller->busy());
        QVERIFY(!controller->message().contains(QRegularExpression("[\\x{3400}-\\x{9fff}]")));
        window->close();
    }
    void controlsDismissFocusWithoutLosingActions() {
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        QVERIFY(QTest::qWaitForWindowExposed(window));
        QTest::qWait(50);
        auto *open = findVisual(window->contentItem(), "welcomeOpen");
        QVERIFY(open);
        open->forceActiveFocus();
        QVERIFY(open->hasActiveFocus());
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_VERIFY(!open->hasActiveFocus());
        QVERIFY(window->isVisible());
        open->forceActiveFocus();
        QTest::mouseClick(window, Qt::LeftButton, Qt::NoModifier,
                          QPoint(window->width() - 8, window->height() - 8));
        QTRY_VERIFY(!open->hasActiveFocus());
        auto *create = findVisual(window->contentItem(), "welcomeCreate");
        auto *dialog = window->findChild<QQuickWindow *>("createWindow");
        QVERIFY(create && dialog);
        create->forceActiveFocus();
        QTest::keyClick(window, Qt::Key_Return);
        QTRY_VERIFY(dialog->isVisible());
        QTRY_VERIFY(!create->hasActiveFocus());
        for (auto *ownedWindow : window->findChildren<QQuickWindow *>())
            QVERIFY(!ownedWindow->title().contains(QStringLiteral(" — xdzip")));
        auto *name = findVisual(dialog->contentItem(), "createName");
        QVERIFY(name);
        name->forceActiveFocus();
        QTest::keyClick(dialog, Qt::Key_Escape);
        QTRY_VERIFY(!name->hasActiveFocus());
        QVERIFY(dialog->isVisible());
        name->forceActiveFocus();
        QTest::keyClick(dialog, Qt::Key_Enter);
        QTRY_VERIFY(!name->hasActiveFocus());
        QVERIFY(dialog->isVisible());
        dialog->close();
        window->close();
    }
    void welcomeOpensRealArchivesAndReturns() {
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *controller = window->findChild<BrowserController *>();
        auto *welcome = findVisual(window->contentItem(), "welcomePage");
        auto *workspace = findVisual(window->contentItem(), "archiveWorkspace");
        auto *open = findVisual(window->contentItem(), "welcomeOpen");
        auto *createAction = findVisual(window->contentItem(), "welcomeCreate");
        auto *create = window->findChild<QQuickWindow *>("createWindow");
        QVERIFY(controller && welcome && workspace && open && createAction && create);
        QTRY_VERIFY(controller->runtimeAvailable() && welcome->isVisible());
        QVERIFY(!workspace->isVisible());
        window->resize(1024, 622);
        QTRY_COMPARE(open->width(), 296.0);
        QTRY_COMPARE(open->height(), 124.0);
        QTRY_VERIFY(visibleImagesReady(window->contentItem()));
        auto *titlebar = findVisual(window->contentItem(), "archiveTitlebar");
        auto *tab = findVisual(window->contentItem(), "activeArchiveTab");
        auto *settings = findVisual(window->contentItem(), "archiveSettings");
        QVERIFY(titlebar && tab && settings);
        const QRectF titlebarBefore(titlebar->position(), titlebar->size());
        const QRectF tabBefore(tab->position(), tab->size());
        const QRectF settingsBefore(settings->position(), settings->size());
        const auto captures = qEnvironmentVariable("XDZIP_WELCOME_VISUAL_AUDIT");
        if (!captures.isEmpty()) {
            QVERIFY(QDir().mkpath(captures));
            QVERIFY(visual_audit::save(window, captures, "welcome-light"));
            window->setProperty("darkMode", true);
            QVERIFY(visual_audit::save(window, captures, "welcome-dark"));
            window->resize(832, 544);
            QVERIFY(visual_audit::save(window, captures, "welcome-dark-minimum"));
            window->setProperty("darkMode", false);
            QVERIFY(visual_audit::save(window, captures, "welcome-minimum"));
            window->resize(1024, 622);
        }
        const auto click = [&](QQuickItem *item) {
            const auto point = item->mapToScene(QPointF(item->width() / 2, item->height() / 2));
            QTest::mouseClick(window, Qt::LeftButton, Qt::NoModifier, point.toPoint());
        };
        click(findVisual(window->contentItem(), "browserTasks"));
        auto *taskMenu = window->findChild<QObject *>("browserTaskMenu");
        QVERIFY(taskMenu);
        QTRY_VERIFY(taskMenu->property("visible").toBool());
        QCOMPARE(taskMenu->property("width").toReal(), 240.0);
        QCOMPARE(taskMenu->property("font").value<QFont>().pixelSize(), 13);
        QVERIFY(QMetaObject::invokeMethod(taskMenu, "close"));
        click(createAction);
        QTRY_VERIFY(create->isVisible());
        create->close();
        QTRY_VERIFY(!create->isVisible() && welcome->isVisible() && !controller->busy());
        // The welcome entry must not fabricate a task when a draft is dismissed.
        QCOMPARE(controller->tasks().size(), 0);
        const auto archive = QUrl::fromLocalFile(QStringLiteral(XDZIP_FIXTURES "/normal.zip"));
        QVERIFY(!controller->openDroppedUrls({}));
        QVERIFY(!controller->openDroppedUrls({archive, archive}));
        QVERIFY(!controller->openDroppedUrls({QUrl("https://example.invalid/a.zip")}));
        QVERIFY(
            !controller->openDroppedUrls({QUrl::fromLocalFile(QStringLiteral(XDZIP_FIXTURES))}));
        QTemporaryDir inputs;
        QVERIFY(inputs.isValid());
        QFile text(inputs.filePath("notes.txt"));
        QVERIFY(text.open(QIODevice::WriteOnly));
        text.write("not an archive");
        text.close();
        QVERIFY(!controller->openDroppedUrls({QUrl::fromLocalFile(text.fileName())}));
        QVERIFY(welcome->isVisible());
        QVERIFY(controller->canOpenDroppedUrls({archive}));
        QQuickWindow dragSource;
        dragSource.show();
        QVERIFY(QTest::qWaitForWindowExposed(&dragSource));
        dragSource.requestActivate();
        if (QGuiApplication::platformName() == "cocoa")
            QTRY_COMPARE(QGuiApplication::focusWindow(), &dragSource);
        QVERIFY(QMetaObject::invokeMethod(
            welcome, "archiveDropped", Q_ARG(QVariant, QVariant::fromValue(QList<QUrl>{archive}))));
        QTRY_VERIFY(!controller->busy() && controller->ready());
        QVERIFY(!welcome->isVisible() && workspace->isVisible());
        if (QGuiApplication::platformName() == "cocoa")
            QTRY_COMPARE(QGuiApplication::focusWindow(), window);
        QCOMPARE(QRectF(titlebar->position(), titlebar->size()), titlebarBefore);
        QCOMPARE(QRectF(tab->position(), tab->size()), tabBefore);
        QCOMPARE(QRectF(settings->position(), settings->size()), settingsBefore);
        controller->closeArchive();
        QTRY_VERIFY(!controller->busy() && !controller->ready());
        QTRY_VERIFY(welcome->isVisible() && !workspace->isVisible());
        QTRY_VERIFY(open->hasActiveFocus());
        // An empty, valid archive is a browser state, never the welcome page.
        QVERIFY(controller->openDroppedUrls(
            {QUrl::fromLocalFile(QStringLiteral(XDZIP_FIXTURES "/empty.zip"))}));
        QTRY_VERIFY(!controller->busy() && controller->ready());
        QCOMPARE(controller->archiveTotal(), 0);
        QVERIFY(!welcome->isVisible());
        controller->closeArchive();
        QTRY_VERIFY(!controller->busy() && welcome->isVisible());
        QFile invalid(inputs.filePath("broken.zip"));
        QVERIFY(invalid.open(QIODevice::WriteOnly));
        invalid.write("not an archive");
        invalid.close();
        QVERIFY(controller->openDroppedUrls({QUrl::fromLocalFile(invalid.fileName())}));
        QTRY_VERIFY(!controller->busy());
        QVERIFY(!controller->ready() && welcome->isVisible());
        QVERIFY(!controller->message().isEmpty());
        QCOMPARE(controller->tasks().size(), 0);
        controller->requestClose();
    }
    void welcomeOpeningCanCancelRealRead() {
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *controller = window->findChild<BrowserController *>();
        auto *welcome = findVisual(window->contentItem(), "welcomePage");
        auto *overlay = findVisual(window->contentItem(), "welcomeOpeningOverlay");
        auto *cancel = findVisual(window->contentItem(), "welcomeOpeningCancel");
        QVERIFY(controller && welcome && overlay && cancel);
        controller->openPath(QStringLiteral(XDZIP_M2_FIXTURE));
        QVERIFY(controller->browsingBusy() && !controller->ready());
        QVERIFY(overlay->isVisible());
        QVERIFY(QMetaObject::invokeMethod(cancel, "clicked"));
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy(), 20000);
        QVERIFY(!overlay->isVisible());
        // Completion may beat cancellation; either terminal state must remove the mask.
        if (!controller->ready())
            QVERIFY(welcome->isVisible());
        controller->openPath(QStringLiteral(XDZIP_FIXTURES "/normal.zip"));
        QTRY_VERIFY_WITH_TIMEOUT(controller->ready() && !controller->busy(), 10000);
        QVERIFY(!welcome->isVisible() && !overlay->isVisible());
    }
    void creationControlsMatchReviewedSource() {
        QTemporaryDir input, output;
        QVERIFY(input.isValid() && output.isValid());
        const QStringList names{QStringLiteral("冰川湖.jpg"), QStringLiteral("山间日出.jpg"),
                                QStringLiteral("行程手册.pdf")};
        const QList<qint64> sizes{8400000, 6200000, 12800000};
        QStringList paths;
        for (int i = 0; i < names.size(); ++i) {
            paths << input.filePath(names[i]);
            QFile file(paths.last());
            QVERIFY(file.open(QIODevice::WriteOnly));
            QVERIFY(file.resize(sizes[i]));
        }
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *controller = window->findChild<BrowserController *>();
        auto *create = window->findChild<QQuickWindow *>("createWindow");
        QVERIFY(controller && create);
        QVERIFY(QMetaObject::invokeMethod(create, "configure"));
        QTRY_VERIFY(create->isVisible() && !controller->busy());
        QVERIFY(QMetaObject::invokeMethod(create, "addInputPaths", Q_ARG(QVariant, QVariant(paths)),
                                          Q_ARG(bool, false)));
        // Metadata is advisory and asynchronous; wait for it rather than putting
        // the illustrative 27.4 MB string directly into the product.
        QTRY_COMPARE(controller->creationInputs().size(), 3);
        auto *summary = findVisual(create->contentItem(), "createInputSummary");
        QVERIFY(summary);
        QTRY_COMPARE(summary->property("text").toString(), QStringLiteral("3 个文件，27.4 MB"));
        auto *name = findVisual(create->contentItem(), "createName");
        auto *target = findVisual(create->contentItem(), "createTarget");
        auto *method = findVisual(create->contentItem(), "createLevel");
        auto *remove = findVisual(create->contentItem(), "createRemoveInput-0");
        auto *group = findVisual(create->contentItem(), "createFormat");
        QVERIFY(name && target && method && remove && group);
        name->setProperty("text", QStringLiteral("北境旅行.zip"));
        target->setProperty("text", output.path());
        controller->checkCreationTarget(output.path());
        QTRY_VERIFY(!controller->busy() && checkedTarget(controller, output.path()));
        auto *volume = findVisual(create->contentItem(), "createVolume");
        auto *unit = findVisual(create->contentItem(), "createVolumeUnit");
        auto *usePassword = findVisual(create->contentItem(), "createUsePassword");
        auto *password = findVisual(create->contentItem(), "createPassword");
        auto *start = findVisual(create->contentItem(), "startCreate");
        QVERIFY(volume && unit && usePassword && password && start);
        QCOMPARE(method->property("currentIndex").toInt(), 2);
        volume->setProperty("currentIndex", 13);
        QTRY_COMPARE(volume->property("editText").toString(), QString{});
        QTRY_VERIFY(!start->isEnabled());
        volume->setProperty("editText", QStringLiteral("25"));
        unit->setProperty("currentIndex", 2);
        QTRY_COMPARE(create->property("volumeBytes").toDouble(), 25000000.0);
        QTRY_VERIFY(start->isEnabled());
        volume->setProperty("editText", QStringLiteral("0"));
        QTRY_VERIFY(!start->isEnabled());
        volume->setProperty("currentIndex", 0);
        usePassword->setProperty("checked", true);
        QTRY_VERIFY(password->isEnabled() && !start->isEnabled());
        password->setProperty("text", QStringLiteral("public fixture"));
        QTRY_VERIFY(start->isEnabled());
        password->setProperty("text", QStringLiteral("invalid\n"));
        QTRY_VERIFY(!start->isEnabled());
        usePassword->setProperty("checked", false);
        QTRY_COMPARE(password->property("text").toString(), QString{});
        QTRY_VERIFY(start->isEnabled());
        auto *tar = findVisual(create->contentItem(), "createFormat-1");
        auto *zip = findVisual(create->contentItem(), "createFormat-0");
        QVERIFY(zip && tar && zip->isEnabled() && tar->isEnabled());
        zip->forceActiveFocus(Qt::TabFocusReason);
        QTest::keyClick(create, Qt::Key_Right);
        QCOMPARE(group->property("currentIndex").toInt(), 1);
        QTRY_VERIFY(name->property("text").toString().endsWith(QStringLiteral(".tar")));
        QVERIFY(!method->isEnabled() && !volume->isEnabled() && !usePassword->isEnabled());
        group->setProperty("currentIndex", 2);
        QTRY_VERIFY(name->property("text").toString().endsWith(QStringLiteral(".tar.gz")));
        QVERIFY(method->isEnabled() && !volume->isEnabled());
        group->setProperty("currentIndex", 0);
        QTRY_VERIFY(method->isEnabled() && volume->isEnabled());
        name->forceActiveFocus(Qt::TabFocusReason);
        QTest::keyClick(create, Qt::Key_Tab);
        const auto captures = qEnvironmentVariable("XDZIP_CREATE_CONTROLS_AUDIT");
        const auto capture = [&](const QString &scene) {
            return captures.isEmpty() ||
                   (QDir().mkpath(captures) && visual_audit::save(create, captures, scene));
        };
        // Clear keyboard focus for normal-state border sampling.
        target->setProperty("focus", false);
        create->contentItem()->forceActiveFocus();
        QTRY_VERIFY(visibleImagesReady(create->contentItem()));
        QVERIFY(capture("create-light"));
        window->setProperty("darkMode", true);
        QTRY_VERIFY(visibleImagesReady(create->contentItem()));
        QVERIFY(capture("create-dark"));
        window->setProperty("darkMode", false);
        remove->forceActiveFocus(Qt::TabFocusReason);
        QTest::qWait(20); // reveal controls in the body on a short high-DPR screen
        create->raise();
        create->requestActivate();
        QVERIFY(QTest::qWaitForWindowActive(create));
        create->contentItem()->forceActiveFocus();
        QTest::mouseMove(
            create,
            remove->mapToScene(QPointF(remove->width() / 2, remove->height() / 2)).toPoint());
        QTRY_VERIFY(remove->property("hovered").toBool());
        QVERIFY(capture("create-remove-hover"));
        name->forceActiveFocus(Qt::TabFocusReason);
        QVERIFY(capture("create-field-focus"));
        method->forceActiveFocus(Qt::TabFocusReason);
        QTest::qWait(20);
        QTest::mouseClick(
            create, Qt::LeftButton, Qt::NoModifier,
            method->mapToScene(QPointF(method->width() / 2, method->height() / 2)).toPoint());
        auto *popup = method->property("popup").value<QObject *>();
        QVERIFY(popup);
        QTRY_VERIFY(popup->property("visible").toBool());
        QVERIFY(capture("create-dropdown"));
        QTest::keyClick(create, Qt::Key_End);
        QTest::keyClick(create, Qt::Key_Return);
        QTRY_COMPARE(method->property("currentIndex").toInt(), 3);
        QTRY_VERIFY(!popup->property("visible").toBool());
        auto *level = findVisual(create->contentItem(), "createLevel");
        QVERIFY(level && level->isEnabled());
        remove->forceActiveFocus(Qt::TabFocusReason);
        QTest::qWait(20);
        QTest::mouseClick(
            create, Qt::LeftButton, Qt::NoModifier,
            remove->mapToScene(QPointF(remove->width() / 2, remove->height() / 2)).toPoint());
        QTRY_COMPARE(controller->creationInputs().size(), 2);
        QTRY_COMPARE(summary->property("text").toString(), QStringLiteral("2 个文件，19.0 MB"));
        // Removing an input is selection editing, never a filesystem deletion.
        for (const auto &path : paths)
            QVERIFY(QFileInfo::exists(path));
        QVERIFY(capture("create-removed"));
        create->close();
        controller->requestClose();
        QTRY_VERIFY(controller->requestClose());
    }
    void tarCreationBrowseAndSafeExtraction() {
        for (int formatIndex : {1, 2}) {
            QTemporaryDir input, output, extracted;
            QVERIFY(input.isValid() && output.isValid() && extracted.isValid());
            const auto source = input.path() + QStringLiteral("/资料");
            QVERIFY(QDir().mkpath(source + QStringLiteral("/空目录")));
            const auto data = QStringLiteral("TAR 与 GZIP 真实读写\n").repeated(200).toUtf8();
            QFile file(source + QStringLiteral("/内容.txt"));
            QVERIFY(file.open(QIODevice::WriteOnly));
            QCOMPARE(file.write(data), data.size());
            file.close();
            QQmlApplicationEngine engine;
            engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
            QVERIFY(!engine.rootObjects().isEmpty());
            auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
            auto *controller = window->findChild<BrowserController *>();
            auto *create = window->findChild<QQuickWindow *>("createWindow");
            QVERIFY(controller && create);
            QVERIFY(QMetaObject::invokeMethod(create, "configure"));
            QTRY_VERIFY(create->isVisible() && !controller->busy());
            QCOMPARE(create->width(), 491);
            const auto captures = qEnvironmentVariable("XDZIP_TAR_VISUAL_AUDIT");
            if (!captures.isEmpty())
                QVERIFY(QDir().mkpath(captures));
            QVERIFY(QMetaObject::invokeMethod(create, "addInputPaths",
                                              Q_ARG(QVariant, QVariant(QStringList{source})),
                                              Q_ARG(bool, true)));
            auto *group = findVisual(create->contentItem(), "createFormat");
            auto *name = findVisual(create->contentItem(), "createName");
            auto *target = findVisual(create->contentItem(), "createTarget");
            QVERIFY(group && name && target);
            group->setProperty("currentIndex", formatIndex);
            const QString suffix =
                formatIndex == 1 ? QStringLiteral(".tar") : QStringLiteral(".tar.gz");
            QTRY_VERIFY(name->property("text").toString().endsWith(suffix));
            name->setProperty("text", QStringLiteral("归档") + suffix);
            target->setProperty("text", output.path());
            QTRY_VERIFY(!controller->busy());
            controller->checkCreationTarget(output.path());
            QTRY_VERIFY(!controller->busy());
            QVERIFY(checkedTarget(controller, output.path()));
            auto *start = findVisual(create->contentItem(), "startCreate");
            QVERIFY(start);
            QTRY_VERIFY(start->isEnabled());
            if (!captures.isEmpty())
                QVERIFY(visual_audit::save(create, captures,
                                           formatIndex == 1 ? "create-tar" : "create-tar-gzip"));
            QSignalSpy started(controller, &BrowserController::creationStarted);
            QTest::mouseClick(
                create, Qt::LeftButton, Qt::NoModifier,
                start->mapToScene(QPointF(start->width() / 2, start->height() / 2)).toPoint());
            QTRY_COMPARE(started.count(), 1);
            QTRY_COMPARE_WITH_TIMEOUT(controller->activeTaskCount(), 0, 10000);
            QCOMPARE(controller->tasks().first().toMap().value("status").toString(),
                     QString("succeeded"));
            QTRY_VERIFY_WITH_TIMEOUT(controller->ready() && !controller->busy() &&
                                         controller->title() == QStringLiteral("归档") + suffix,
                                     10000);
            QCOMPARE(controller->formatLabel(),
                     formatIndex == 1 ? QString("TAR") : QString("TAR.GZ"));
            QCOMPARE(controller->archiveTotal(), 3);
            auto *badge = findVisual(window->contentItem(), "archiveFormatBadge");
            auto *label = findVisual(window->contentItem(), "archiveFormatLabel");
            QVERIFY(badge && label);
            QTRY_VERIFY(badge->width() >= label->width() + 12);
            controller->enterDirectory(0);
            QTRY_VERIFY(!controller->busy());
            controller->selectAll(true);
            QVERIFY(!controller->detail().value("modified").toString().contains(QChar(0x2014)));
            QCOMPARE(controller->detail().value("packed").toString().isEmpty(), formatIndex == 2);
            if (!captures.isEmpty())
                QVERIFY(visual_audit::save(window, captures,
                                           formatIndex == 1 ? "opened-tar" : "opened-tar-gzip"));
            controller->prepareExtraction(true);
            auto *extract = window->findChild<QQuickWindow *>("extractWindow");
            QVERIFY(extract);
            QTRY_VERIFY(extract->isVisible());
            auto *extractTarget = findVisual(extract->contentItem(), "extractTarget");
            QVERIFY(extractTarget);
            extractTarget->setProperty("text", extracted.path());
            QTRY_VERIFY(!controller->busy());
            controller->checkTarget(extracted.path());
            QTRY_VERIFY(!controller->busy());
            QVERIFY(checkedTarget(controller, extracted.path()));
            auto *folder = findVisual(extract->contentItem(), "extractSameFolder");
            QVERIFY(folder);
            QCOMPARE(controller->extractionIntent().value("folder").toString(),
                     QStringLiteral("归档"));
            QTRY_COMPARE(extract->property("archiveFolder").toString(), QStringLiteral("归档"));
            if (formatIndex == 1)
                folder->setProperty("checked", false);
            const auto resultRoot =
                extracted.path() + (formatIndex == 2 ? QStringLiteral("/归档") : QString{});
            auto *extractStart = findVisual(extract->contentItem(), "startExtract");
            QVERIFY(extractStart);
            QTRY_VERIFY(extractStart->isEnabled());
            QTest::mouseClick(
                extract, Qt::LeftButton, Qt::NoModifier,
                extractStart
                    ->mapToScene(QPointF(extractStart->width() / 2, extractStart->height() / 2))
                    .toPoint());
            QTRY_COMPARE_WITH_TIMEOUT(controller->tasks().size(), 2, 10000);
            QTRY_COMPARE_WITH_TIMEOUT(controller->activeTaskCount(), 0, 10000);
            for (const auto &task : controller->tasks())
                QCOMPARE(task.toMap().value("status").toString(), QString("succeeded"));
            QFile actual(resultRoot + QStringLiteral("/资料/内容.txt"));
            QVERIFY(actual.open(QIODevice::ReadOnly));
            QCOMPARE(actual.readAll(), data);
            QVERIFY(QFileInfo(resultRoot + QStringLiteral("/资料/空目录")).isDir());
            if (!captures.isEmpty())
                QVERIFY(visual_audit::save(window->findChild<QQuickWindow *>("taskWindow"),
                                           captures,
                                           formatIndex == 1 ? "result-tar" : "result-tar-gzip"));
            controller->requestClose();
            QTRY_VERIFY(controller->requestClose());
        }
    }
    void m4CreationWindowAndRoundTrip() {
        QTemporaryDir input, output;
        QVERIFY(input.isValid() && output.isValid());
        const auto source = input.path() + QStringLiteral("/输入");
        QVERIFY(QDir().mkpath(source + QStringLiteral("/空目录")));
        QFile file(source + QStringLiteral("/文件.txt"));
        QVERIFY(file.open(QIODevice::WriteOnly));
        const auto data = QStringLiteral("真实 ZIP 创建与读回\n").repeated(1024).toUtf8();
        QCOMPARE(file.write(data), data.size());
        file.close();
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *controller = window->findChild<BrowserController *>();
        auto *create = window->findChild<QQuickWindow *>("createWindow");
        QVERIFY(controller && create);
        auto *newButton = findVisual(window->contentItem(), "welcomeCreate");
        QVERIFY(newButton && newButton->isEnabled());
        QTest::mouseClick(
            window, Qt::LeftButton, Qt::NoModifier,
            newButton->mapToScene(QPointF(newButton->width() / 2, newButton->height() / 2))
                .toPoint());
        QTRY_VERIFY(create->isVisible());
        QTRY_VERIFY(!controller->busy());
        QCOMPARE(create->width(), 491);
        QVERIFY(QMetaObject::invokeMethod(create, "addInputPaths",
                                          Q_ARG(QVariant, QVariant(QStringList{source})),
                                          Q_ARG(bool, true)));
        QVERIFY(QMetaObject::invokeMethod(
            create, "addInputPaths",
            Q_ARG(QVariant, QVariant(QStringList{source + QStringLiteral("/文件.txt")})),
            Q_ARG(bool, false)));
        QVERIFY(QMetaObject::invokeMethod(
            create, "addInputPaths",
            Q_ARG(QVariant, QVariant(QStringList{source + QStringLiteral("/空目录")})),
            Q_ARG(bool, true)));
        auto *name = findVisual(create->contentItem(), "createName");
        auto *target = findVisual(create->contentItem(), "createTarget");
        auto *method = findVisual(create->contentItem(), "createLevel");
        QVERIFY(name && target && method);
        name->setProperty("text", QStringLiteral("创建.zip"));
        target->setProperty("text", output.path());
        method->setProperty("currentIndex", 0);
        auto *level = findVisual(create->contentItem(), "createLevel");
        QVERIFY(level);
        QTRY_VERIFY(level->isEnabled());
        controller->checkCreationTarget(output.path());
        QTRY_VERIFY(!controller->busy());
        QVERIFY(checkedTarget(controller, output.path()));
        QTRY_VERIFY(visibleImagesReady(create->contentItem()));
        const auto captures = qEnvironmentVariable("XDZIP_M4_VISUAL_AUDIT");
        if (!captures.isEmpty()) {
            QVERIFY(QDir().mkpath(captures));
            QVERIFY(visual_audit::save(create, captures, "create-selected"));
            window->setProperty("darkMode", true);
            QTest::qWait(20);
            QVERIFY(visual_audit::save(create, captures, "create-dark"));
            window->setProperty("darkMode", false);
        }
        auto *start = findVisual(create->contentItem(), "startCreate");
        QVERIFY(start);
        QTRY_VERIFY(start->isEnabled());
        QSignalSpy started(controller, &BrowserController::creationStarted);
        QTest::mouseClick(
            create, Qt::LeftButton, Qt::NoModifier,
            start->mapToScene(QPointF(start->width() / 2, start->height() / 2)).toPoint());
        QTRY_COMPARE(started.count(), 1);
        QTRY_VERIFY(!create->isVisible());
        QTRY_COMPARE_WITH_TIMEOUT(controller->activeTaskCount(), 0, 10000);
        const auto tasks = controller->tasks();
        QVERIFY(!tasks.isEmpty());
        const auto task = tasks.first().toMap();
        QCOMPARE(task.value("status").toString(), QString("succeeded"));
        QVERIFY(task.value("creating").toBool() && task.value("totalKnown").toBool());
        QCOMPARE(task.value("committedFiles").toString(), QString("1"));
        const auto archive = output.path() + QStringLiteral("/创建.zip");
        QVERIFY(QFileInfo::exists(archive));
        // Successful creation opens its committed output without a second user action.
        QTRY_VERIFY_WITH_TIMEOUT(controller->ready() && !controller->busy() &&
                                     controller->title() == QStringLiteral("创建.zip"),
                                 10000);
        QCOMPARE(controller->archiveTotal(), 3);
        QCOMPARE(controller->entries()->rowCount(), 1);
        controller->enterDirectory(0);
        QTRY_VERIFY(!controller->busy());
        QCOMPARE(controller->entries()->rowCount(), 2);
        if (!captures.isEmpty()) {
            auto *progress = window->findChild<QQuickWindow *>("taskWindow");
            QVERIFY(progress);
            QVERIFY(visual_audit::save(progress, captures, "create-completed"));
            QVERIFY(visual_audit::save(window, captures, "created-archive-open"));
        }
        QFile archiveFile(archive);
        QVERIFY(archiveFile.open(QIODevice::ReadOnly));
        const auto originalArchive = archiveFile.readAll();
        archiveFile.close();
        // Drive the actual conflict action, then prove whole-archive cancellation
        // preserves the existing file and removes only our staging directory.
        controller->startCreation({source}, output.path(), QStringLiteral("创建.zip"), 8, 6, false);
        auto *conflict = window->findChild<QQuickWindow *>("conflictWindow");
        QVERIFY(conflict);
        QTRY_VERIFY(conflict->isVisible());
        auto *applyAll = findVisual(conflict->contentItem(), "conflictApplyAll");
        QVERIFY(applyAll && !applyAll->isVisible());
        auto *skip = findVisual(conflict->contentItem(), "conflictSkip");
        QVERIFY(skip && skip->isEnabled());
        QTest::mouseClick(
            conflict, Qt::LeftButton, Qt::NoModifier,
            skip->mapToScene(QPointF(skip->width() / 2, skip->height() / 2)).toPoint());
        QTRY_COMPARE(controller->activeTaskCount(), 0);
        QVERIFY(archiveFile.open(QIODevice::ReadOnly));
        QCOMPARE(archiveFile.readAll(), originalArchive);
        archiveFile.close();
        QCOMPARE(QDir(output.path()).entryList(QDir::NoDotAndDotDot | QDir::AllEntries).size(), 1);
        controller->requestClose();
        QTRY_VERIFY(controller->requestClose());
    }
    void creationPreservesSelectedDestination_data() {
        QTest::addColumn<int>("formatIndex");
        QTest::addColumn<QString>("decision");
        QTest::addColumn<bool>("onlyOutput");
        for (int index = 0; index < 3; ++index) {
            const QByteArray format = QList<QByteArray>{"zip", "tar", "tar-gzip"}.at(index);
            QTest::newRow((format + "-rename").constData())
                << index << QString("conflictRename") << false;
            QTest::newRow((format + "-cancel").constData())
                << index << QString("conflictSkip") << false;
            QTest::newRow((format + "-only-output").constData())
                << index << QString("conflictRename") << true;
        }
    }
    void creationPreservesSelectedDestination() {
        QFETCH(int, formatIndex);
        QFETCH(QString, decision);
        QFETCH(bool, onlyOutput);
        QTemporaryDir input, output, extracted;
        QVERIFY(input.isValid() && output.isValid() && extracted.isValid());
        const QString extension = QStringList{".zip", ".tar", ".tar.gz"}.at(formatIndex);
        const auto name = QStringLiteral("owned") + extension;
        const auto archive = output.filePath(name);
        const QByteArray original("owned destination is also a selected input\n");
        const QByteArray payload("owned additional payload\n");
        const auto source = input.filePath("payload.txt");
        const auto write = [](const QString &path, const QByteArray &data) {
            QFile file(path);
            return file.open(QIODevice::WriteOnly) && file.write(data) == data.size();
        };
        QVERIFY(write(archive, original));
        QVERIFY(write(source, payload));

        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        QVERIFY(window);
        auto *controller = window->findChild<BrowserController *>();
        auto *create = window->findChild<QQuickWindow *>("createWindow");
        auto *conflict = window->findChild<QQuickWindow *>("conflictWindow");
        auto *taskWindow = window->findChild<QQuickWindow *>("taskWindow");
        QVERIFY(controller && create && conflict && taskWindow);
        QVERIFY(QMetaObject::invokeMethod(create, "configure"));
        QTRY_VERIFY(create->isVisible() && !controller->busy());
        const QStringList paths = onlyOutput ? QStringList{archive} : QStringList{source, archive};
        QVERIFY(QMetaObject::invokeMethod(create, "addInputPaths", Q_ARG(QVariant, QVariant(paths)),
                                          Q_ARG(bool, false)));
        auto *format = findVisual(create->contentItem(), "createFormat");
        auto *nameField = findVisual(create->contentItem(), "createName");
        auto *target = findVisual(create->contentItem(), "createTarget");
        auto *start = findVisual(create->contentItem(), "startCreate");
        QVERIFY(format && nameField && target && start);
        format->setProperty("currentIndex", formatIndex);
        nameField->setProperty("text", name);
        target->setProperty("text", output.path());
        QTRY_VERIFY(!controller->busy());
        controller->checkCreationTarget(output.path());
        QTRY_VERIFY(!controller->busy() && checkedTarget(controller, output.path()));
        QTRY_VERIFY(start->isEnabled());
        QSignalSpy started(controller, &BrowserController::creationStarted);
        QTest::mouseClick(
            create, Qt::LeftButton, Qt::NoModifier,
            start->mapToScene(QPointF(start->width() / 2, start->height() / 2)).toPoint());
        QTRY_COMPARE(started.count(), 1);
        QTRY_VERIFY(!create->isVisible() && taskWindow->isVisible());
        QTRY_COMPARE(controller->tasks().size(), 1);
        const auto taskId = started.first().first().toString();
        // All selected files are encoded, including the existing destination. The
        // final publication guard asks for rename/cancel and forbids replacing input.
        QTRY_VERIFY_WITH_TIMEOUT(conflict->isVisible(), 10000);
        const auto waiting = controller->tasks().first().toMap();
        QCOMPARE(waiting.value("id").toString(), taskId);
        QCOMPARE(waiting.value("status").toString(), "waiting");
        QVERIFY(waiting.value("errorCode").toString().isEmpty());
        QVERIFY(waiting.value("totalKnown").toBool());
        QCOMPARE(waiting.value("files").toInt(), onlyOutput ? 1 : 2);
        const auto expectedBytes = qulonglong(original.size() + (onlyOutput ? 0 : payload.size()));
        QCOMPARE(waiting.value("declaredBytes").toULongLong(), expectedBytes);
        QCOMPARE(waiting.value("decodedBytes").toULongLong(), expectedBytes);
        QVERIFY(!waiting.value("overwriteAllowed").toBool());
        auto *applyAll = findVisual(conflict->contentItem(), "conflictApplyAll");
        auto *overwrite = findVisual(conflict->contentItem(), "conflictOverwrite");
        auto *action = findVisual(conflict->contentItem(), decision);
        auto *notice = findVisual(conflict->contentItem(), "conflictNotice");
        QVERIFY(applyAll && !applyAll->isVisible() && overwrite && action && notice);
        QVERIFY(!overwrite->isEnabled() && action->isEnabled());
        QCOMPARE(notice->property("text").toString(), qtTrId("ui.baaef55f9a78"));
        QFile existing(archive);
        QVERIFY(existing.open(QIODevice::ReadOnly));
        QCOMPARE(existing.readAll(), original);
        existing.close();

        // The source-protection explanation remains reachable in the scrolling
        // body while the real conflict actions stay fixed at the window bottom.
        auto *scroll = notice->parentItem();
        while (scroll && !scroll->inherits("QQuickScrollView"))
            scroll = scroll->parentItem();
        QVERIFY(scroll);
        auto *flickable = scroll->property("contentItem").value<QQuickItem *>();
        auto *footer = action->parentItem();
        while (footer && !footer->property("minimumFooterHeight").isValid())
            footer = footer->parentItem();
        QVERIFY(flickable && footer);
        QTRY_VERIFY(notice->height() > 0 && flickable->height() > 0);
        QTRY_VERIFY(notice->mapToScene(QPointF()).x() >= 0);
        QTRY_VERIFY(notice->mapToScene(QPointF(notice->width(), 0)).x() <= conflict->width() + 1);
        const auto textFits = [&](auto &&self, QQuickItem *item) -> bool {
            if (item->isVisible() && item->inherits("QQuickText")) {
                if (item->mapToScene(QPointF()).x() < -1 ||
                    item->mapToScene(QPointF(item->width(), 0)).x() > conflict->width() + 1 ||
                    item->property("contentWidth").toReal() > item->width() + 1)
                    return false;
            }
            for (auto *child : item->childItems())
                if (!self(self, child))
                    return false;
            return true;
        };
        QTRY_VERIFY(textFits(textFits, conflict->contentItem()));
        const auto maximum =
            std::max(qreal(0), flickable->property("contentHeight").toReal() - flickable->height());
        const auto noticeTop = notice->mapToItem(flickable, QPointF()).y();
        QVERIFY(flickable->setProperty("contentY", std::clamp(noticeTop, qreal(0), maximum)));
        QTRY_VERIFY(notice->mapToItem(flickable, QPointF()).y() >= -1);
        QTRY_VERIFY(notice->mapToItem(flickable, QPointF(0, notice->height())).y() <=
                    flickable->height() + 1);
        QCOMPARE(footer->mapToScene(QPointF(0, footer->height())).y(), qreal(conflict->height()));
        QVERIFY(action->mapToScene(QPointF()).y() >= footer->mapToScene(QPointF()).y());
        QVERIFY(action->mapToScene(QPointF(action->width(), action->height())).x() <=
                conflict->width() + 1);
        QVERIFY(action->mapToScene(QPointF(action->width(), action->height())).y() <=
                conflict->height() + 1);
        const auto audit = qEnvironmentVariable("XDZIP_CREATE_OVERLAP_AUDIT");
        if (!audit.isEmpty()) {
            QVERIFY(QDir().mkpath(audit));
            QVERIFY(visual_audit::save(conflict, audit,
                                       QString::fromLatin1(QTest::currentDataTag()) + "-conflict"));
        }
        QTest::mouseClick(
            conflict, Qt::LeftButton, Qt::NoModifier,
            action->mapToScene(QPointF(action->width() / 2, action->height() / 2)).toPoint());
        QTRY_COMPARE_WITH_TIMEOUT(controller->activeTaskCount(), 0, 10000);
        QTRY_VERIFY(!conflict->isVisible());
        const auto task = controller->tasks().first().toMap();
        QCOMPARE(task.value("id").toString(), taskId);
        QVERIFY(!task.value("cleanupWarning").toBool());
        QVERIFY(!task.value("outcomeUncertain").toBool());
        const bool cancelled = decision == "conflictSkip";
        if (cancelled) {
            QCOMPARE(task.value("status").toString(), "cancelled");
            QCOMPARE(task.value("errorCode").toString(), "cancelled");
            QCOMPARE(task.value("committedFiles").toString(), "0");
            QCOMPARE(task.value("outputParts").toString(), "0");
            QVERIFY(!controller->ready());
        } else {
            QCOMPARE(task.value("status").toString(), "succeeded");
            QVERIFY(task.value("errorCode").toString().isEmpty());
            QCOMPARE(task.value("committedFiles").toInt(), onlyOutput ? 1 : 2);
            const auto resultName = QStringLiteral("owned (2)") + extension;
            QTRY_VERIFY_WITH_TIMEOUT(controller->ready() && !controller->busy() &&
                                         controller->title() == resultName,
                                     10000);
            QCOMPARE(controller->archiveTotal(), onlyOutput ? 1 : 2);
            QCOMPARE(controller->entries()->rowCount(), onlyOutput ? 1 : 2);
        }
        QVERIFY(existing.open(QIODevice::ReadOnly));
        QCOMPARE(existing.readAll(), original);
        existing.close();
        // Count hidden names too, so successful rename and cancellation cannot hide
        // a leaked staging workspace in an otherwise correct output directory.
        QCOMPARE(
            QDir(output.path())
                .entryList(QDir::NoDotAndDotDot | QDir::AllEntries | QDir::Hidden | QDir::System)
                .size(),
            cancelled ? 1 : 2);
        if (!audit.isEmpty())
            QVERIFY(visual_audit::save(taskWindow, audit, QTest::currentDataTag()));
        if (!cancelled) {
            controller->prepareExtraction(true);
            QTRY_VERIFY(!controller->busy());
            controller->checkTarget(extracted.path());
            QTRY_VERIFY(!controller->busy() && checkedTarget(controller, extracted.path()));
            controller->startExtraction(extracted.path(), true, false, 0, false, false);
            QTRY_COMPARE_WITH_TIMEOUT(controller->tasks().size(), 2, 10000);
            QTRY_COMPARE_WITH_TIMEOUT(controller->activeTaskCount(), 0, 10000);
            QCOMPARE(controller->tasks().first().toMap().value("status").toString(), "succeeded");
            QFile oldArchive(extracted.filePath(name));
            QVERIFY(oldArchive.open(QIODevice::ReadOnly));
            QCOMPARE(oldArchive.readAll(), original);
            QStringList expectedNames{name};
            if (!onlyOutput) {
                QFile actual(extracted.filePath("payload.txt"));
                QVERIFY(actual.open(QIODevice::ReadOnly));
                QCOMPARE(actual.readAll(), payload);
                expectedNames.append("payload.txt");
            }
            expectedNames.sort();
            QCOMPARE(QDir(extracted.path())
                         .entryList(QDir::NoDotAndDotDot | QDir::AllEntries | QDir::Hidden |
                                    QDir::System),
                     expectedNames);
        }
        controller->requestClose();
        QTRY_VERIFY(controller->requestClose());
    }
    void explicitOpenConsumesQueuedCreationOutput() {
        DesktopUrlRecorder recorder;
        QDesktopServices::setUrlHandler("file", &recorder, "record");
        const auto restoreHandler = qScopeGuard([] { QDesktopServices::unsetUrlHandler("file"); });
        QTemporaryDir input, output;
        QVERIFY(input.isValid() && output.isValid());
        const auto source = input.path() + "/data.txt";
        QFile file(source);
        QVERIFY(file.open(QIODevice::WriteOnly));
        QCOMPARE(file.write("owned payload"), qint64(13));
        file.close();
        BrowserController controller;
        // Explicit spelling differs from the runtime path, including native separators on Windows.
        const auto archive = output.path() + "/./created.zip";
        bool openedExplicitly = false;
        connect(&controller, &BrowserController::tasksChanged, &controller, [&] {
            if (openedExplicitly || controller.busy() || controller.tasks().isEmpty())
                return;
            if (controller.tasks().first().toMap().value("status").toString() == "succeeded") {
                openedExplicitly = true;
                controller.openPath(archive);
            }
        });
        QSignalSpy opened(&controller, &BrowserController::archiveOpened);
        controller.startCreation({source}, output.path(), "created.zip", 8, 6, true);
        QTRY_VERIFY_WITH_TIMEOUT(openedExplicitly && controller.ready() && !controller.busy(),
                                 10000);
        QTest::qWait(100);
        QCOMPARE(opened.count(), 1);
        QCOMPARE(controller.archiveTotal(), 1);
        QCOMPARE(recorder.count, 0);
        controller.requestClose();
        QTRY_VERIFY(controller.requestClose());
    }
    void operationPreparationKeepsBrowserPresentationStable_data() {
        QTest::addColumn<int>("operation");
        QTest::newRow("extract-all") << 0;
        QTest::newRow("extract-selection") << 1;
        QTest::newRow("target-recheck") << 2;
        QTest::newRow("creation-target") << 3;
        QTest::newRow("creation-input-preview") << 4;
    }
    void operationPreparationKeepsBrowserPresentationStable() {
        QFETCH(int, operation);
        QTemporaryDir target;
        QVERIFY(target.isValid());
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        QVERIFY(QTest::qWaitForWindowExposed(window));
        auto *controller = window->findChild<BrowserController *>();
        QVERIFY(controller);
        controller->openPath(QStringLiteral(XDZIP_FIXTURES "/../extract/tree.zip"));
        QTRY_VERIFY(controller->ready() && !controller->busy());
        controller->selectAll(operation != 0);
        window->requestActivate();
        QVERIFY(QTest::qWaitForWindowActive(window));
        QVERIFY(QTest::qWaitFor([&] {
            auto *button = findVisual(window->contentItem(), "archiveExtract");
            return button && button->isVisible() && button->isEnabled() && button->width() > 0 &&
                   button->height() > 0;
        }));
        QSignalSpy toolbarFrame(window, &QQuickWindow::frameSwapped);
        window->update();
        QTRY_VERIFY(!toolbarFrame.isEmpty());
        const auto rows = controller->entries()->rowCount();
        const auto selected = controller->selectedCount();
        auto *overlay = findVisual(window->contentItem(), "archiveLoadingOverlay");
        QVERIFY(overlay && !overlay->isVisible());
        bool controlsStable = true, overlayStable = true, sawPending = false;
        bool presentationStable = true;
        QObject monitor;
        for (const auto &name :
             {"archiveOpen", "archiveNew", "archiveMore", "archiveSearch", "archiveExtract"}) {
            auto *control = findVisual(window->contentItem(), QString::fromLatin1(name));
            QVERIFY(control && control->isEnabled());
            connect(control, &QQuickItem::enabledChanged, &monitor,
                    [&, control] { controlsStable &= control->isEnabled(); });
        }
        connect(overlay, &QQuickItem::visibleChanged, &monitor,
                [&] { overlayStable &= !overlay->isVisible(); });
        connect(controller, &BrowserController::changed, &monitor, [&] {
            if (controller->busy()) {
                sawPending = true;
                presentationStable &=
                    !controller->browsingBusy() && controller->browserQueryAvailable();
            }
        });
        QSignalSpy resets(controller->entries(), &QAbstractItemModel::modelReset);
        if (operation == 0) {
            auto *button = findVisual(window->contentItem(), "archiveExtract");
            QTest::mouseClick(
                window, Qt::LeftButton, Qt::NoModifier,
                button->mapToScene(QPointF(button->width() / 4, button->height() / 2)).toPoint());
        } else if (operation == 1) {
            controller->prepareExtraction(false);
        } else if (operation == 2) {
            controller->prepareExtraction(true);
            QTRY_VERIFY(!controller->busy());
            controller->checkTarget(target.path());
        } else if (operation == 3) {
            controller->checkCreationTarget(target.path());
        } else {
            QFile input(target.filePath("input.txt"));
            QVERIFY(input.open(QIODevice::WriteOnly));
            QCOMPARE(input.write("preview"), 7);
            input.close();
            auto *create = window->findChild<QQuickWindow *>("createWindow");
            QVERIFY(create);
            QVERIFY(QMetaObject::invokeMethod(
                create, "addInputPaths", Q_ARG(QVariant, QVariant(QStringList{input.fileName()})),
                Q_ARG(bool, false)));
            QTRY_COMPARE(controller->creationInputs().size(), 1);
        }
        QTRY_VERIFY(!controller->busy());
        QVERIFY(sawPending);
        QVERIFY(presentationStable);
        QVERIFY(controlsStable);
        QVERIFY(overlayStable);
        QCOMPARE(resets.count(), 0);
        QCOMPARE(controller->entries()->rowCount(), rows);
        QCOMPARE(controller->selectedCount(), selected);
        window->close();
    }
    void taskModelUpdatesRowsWithoutResettingRetainedTasks() {
        BrowserController controller;
        auto *model = controller.taskModel();
        QVERIFY(model);
        QAbstractItemModelTester modelCheck(model,
                                            QAbstractItemModelTester::FailureReportingMode::QtTest);
        const auto role = model->roleNames().key(QByteArrayLiteral("taskData"), -1);
        QVERIFY(role >= Qt::UserRole);
        QSignalSpy resets(model, &QAbstractItemModel::modelReset);
        QSignalSpy inserts(model, &QAbstractItemModel::rowsInserted);
        QSignalSpy removals(model, &QAbstractItemModel::rowsRemoved);
        QSignalSpy changes(model, &QAbstractItemModel::dataChanged);
        xdzip::reader::Event progress{};
        progress.kind = xdzip::reader::EventKind::TaskUpdate;
        progress.task.id = 1;
        progress.task.status = rust::String("running");
        progress.task.total_known = true;
        progress.task.declared_bytes = 100;
        controller.receive(progress);
        QCOMPARE(model->rowCount(), 1);
        const QPersistentModelIndex active(model->index(0, 0));
        QCOMPARE(model->data(active, role).toMap().value("id").toString(), "1");
        for (quint64 id = 2; id <= 65; ++id) {
            xdzip::reader::Event finished{};
            finished.kind = xdzip::reader::EventKind::TaskUpdate;
            finished.task.id = id;
            finished.task.status = rust::String("succeeded");
            controller.receive(finished);
            QVERIFY(active.isValid());
            QCOMPARE(model->data(active, role).toMap().value("id").toString(), "1");
            QCOMPARE(model->data(model->index(0, 0), role).toMap().value("id").toString(),
                     QString::number(id));
        }
        // Retaining at most 64 rows evicts terminal history, never an active task.
        QCOMPARE(model->rowCount(), 64);
        QCOMPARE(inserts.count(), 65);
        QCOMPARE(removals.count(), 1);
        QCOMPARE(active.row(), 63);
        changes.clear();
        for (quint64 bytes = 1; bytes <= 24; ++bytes) {
            progress.task.decoded_bytes = bytes;
            controller.receive(progress);
            QCOMPARE(model->data(active, role).toMap().value("decodedBytes").toString(),
                     QString::number(bytes));
        }
        QCOMPARE(changes.count(), 24);
        controller.receive(progress); // An identical snapshot must not invalidate delegates.
        QCOMPARE(changes.count(), 24);
        progress.task.status = rust::String("cancelled");
        controller.receive(progress);
        QCOMPARE(model->data(active, role).toMap().value("status").toString(), "cancelled");
        QVERIFY(model->data(active, role).toMap().value("terminal").toBool());
        QCOMPARE(model->rowCount(), 64);
        QCOMPARE(resets.count(), 0);
    }
    void taskMenuProgressPreservesHoverAndFocus_data() {
        QTest::addColumn<bool>("creating");
        QTest::newRow("extraction") << false;
        QTest::newRow("creation") << true;
    }
    void taskMenuProgressPreservesHoverAndFocus() {
        QFETCH(bool, creating);
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        QVERIFY(window && QTest::qWaitForWindowExposed(window));
        auto *controller = window->findChild<BrowserController *>();
        QVERIFY(controller);
        controller->openPath(QStringLiteral(XDZIP_FIXTURES "/normal.zip"));
        QTRY_VERIFY(controller->ready() && !controller->busy());

        // Exercise the production receiver with controlled bridge snapshots. Real
        // I/O is covered separately; every progress tick here remains observable.
        xdzip::reader::Event progress{};
        progress.kind = xdzip::reader::EventKind::TaskUpdate;
        progress.task.id = creating ? 102 : 101;
        progress.task.creating = creating;
        progress.task.total_known = true;
        progress.task.status = rust::String("running");
        progress.task.declared_bytes = 64 * 1024;
        progress.task.files = 1;
        // A multi-part creation snapshot does not enqueue auto-opening a real file.
        progress.task.output_parts = creating ? 2 : 0;
        controller->receive(progress);
        const auto taskId = QString::number(progress.task.id);
        const auto objectName = QStringLiteral("taskMenuItem-") + taskId;
        auto *menu = window->findChild<QObject *>("browserTaskMenu");
        auto *button = findVisual(window->contentItem(), "browserTasks");
        QVERIFY(menu && button);
        window->raise();
        window->requestActivate();
        QTRY_VERIFY(window->isActive());
        QTest::mouseClick(
            window, Qt::LeftButton, Qt::NoModifier,
            button->mapToScene(QPointF(button->width() / 2, button->height() / 2)).toPoint());
        QTRY_VERIFY(menu->property("opened").toBool());
        QPointer<QQuickItem> entry =
            qobject_cast<QQuickItem *>(window->findChild<QObject *>(objectName));
        QVERIFY(entry);
        QTRY_VERIFY(entry->isVisible() && entry->width() > 0 && entry->height() > 0);
        QSignalSpy frames(window, &QQuickWindow::afterAnimating);
        window->requestUpdate();
        QTRY_VERIFY(frames.count() > 0);
        const auto pointer =
            entry->mapToScene(QPointF(entry->width() / 2, entry->height() / 2)).toPoint();
        QVERIFY2(QRect(QPoint(), window->size()).contains(pointer),
                 "The task row must finish popup positioning before pointer input");
        // The offscreen platform disables desktop hover effects in its style
        // hints. Enable only this test item for synthetic pointer coverage.
        if (QGuiApplication::platformName() == QStringLiteral("offscreen"))
            QVERIFY(entry->setProperty("hoverEnabled", true));
        QVERIFY(entry->property("hoverEnabled").toBool());
        const auto platform = QGuiApplication::platformName();
        const bool nativePointer = platform == QStringLiteral("cocoa") ||
                                   platform == QStringLiteral("windows") ||
                                   platform == QStringLiteral("xcb");
        const auto originalCursor = QCursor::pos();
        const auto restoreCursor = qScopeGuard([&] {
            if (nativePointer)
                QCursor::setPos(originalCursor);
        });
        const auto expectedCursor = window->mapToGlobal(pointer);
        // QTest injects Qt mouse events without moving the native cursor. Align
        // them once so subsequent native enter/leave events use the same point.
        if (nativePointer)
            QCursor::setPos(expectedCursor);
        QTest::mouseMove(window, QPoint(1, 1));
        QTest::mouseMove(window, pointer);
        QTRY_VERIFY(entry && entry->property("hovered").toBool());
        QTRY_VERIFY(entry->property("highlighted").toBool());
        entry->forceActiveFocus(Qt::TabFocusReason);
        QTRY_VERIFY(entry->hasActiveFocus());
        const auto selectedIndex = menu->property("currentIndex").toInt();
        QVERIFY(selectedIndex >= 0);
        const auto runningLabel = entry->property("statusLabel").toString();
        QVERIFY(!runningLabel.isEmpty());
        QSignalSpy destroyed(entry, &QObject::destroyed);
        QSignalSpy hoverChanges(entry, SIGNAL(hoveredChanged()));
        QSignalSpy highlightChanges(entry, SIGNAL(highlightedChanged()));
        QSignalSpy focusChanges(entry, &QQuickItem::activeFocusChanged);
        QVERIFY(hoverChanges.isValid() && highlightChanges.isValid());
        const auto auditDirectory = qEnvironmentVariable("XDZIP_TASK_MENU_AUDIT");
        const auto scene = creating ? QStringLiteral("creation") : QStringLiteral("extraction");
        if (!auditDirectory.isEmpty()) {
            QVERIFY(QDir().mkpath(auditDirectory));
            QVERIFY(visual_audit::save(window, auditDirectory, scene + "-hover-before"));
        }
        for (quint64 tick = 1; tick <= 48; ++tick) {
            progress.task.decoded_bytes = tick * 1024;
            progress.task.written_bytes = tick * 1024;
            progress.task.elapsed_ms = tick * 100;
            controller->receive(progress);
            const auto frameCount = frames.count();
            window->requestUpdate();
            QTRY_VERIFY(frames.count() > frameCount);
            const auto pointText = [](QPointF point) {
                return QStringLiteral("(%1,%2)").arg(point.x()).arg(point.y());
            };
            const auto center =
                entry ? entry->mapToScene(QPointF(entry->width() / 2, entry->height() / 2))
                      : QPointF(-1, -1);
            const auto diagnostic =
                QStringLiteral("tick=%1 physicalCursor=%2 expectedCursor=%3 rowCenter=%4 "
                               "expectedCenter=%5 rowSize=%6 windowPosition=%7 windowSize=%8")
                    .arg(tick)
                    .arg(pointText(QCursor::pos()))
                    .arg(pointText(expectedCursor))
                    .arg(pointText(center))
                    .arg(pointText(pointer))
                    .arg(pointText(entry ? QPointF(entry->width(), entry->height()) : QPointF()))
                    .arg(pointText(window->position()))
                    .arg(pointText(QPointF(window->width(), window->height())));
            // These checks detect delegate replacement even if the new row happens
            // to acquire the same objectName or highlight on the following frame.
            QVERIFY2(entry,
                     qPrintable("A progress snapshot destroyed the hovered task menu item, " +
                                diagnostic));
            QCOMPARE(window->findChild<QObject *>(objectName), entry.data());
            QVERIFY2(entry->property("hovered").toBool(), qPrintable(diagnostic));
            QVERIFY2(entry->property("highlighted").toBool(), qPrintable(diagnostic));
            QVERIFY2(entry->hasActiveFocus(), qPrintable(diagnostic));
            QCOMPARE(menu->property("currentIndex").toInt(), selectedIndex);
            QCOMPARE(controller->tasks().first().toMap().value("decodedBytes").toString(),
                     QString::number(tick * 1024));
        }
        progress.task.status = rust::String("succeeded");
        controller->receive(progress);
        QTRY_VERIFY(entry && entry->property("statusLabel").toString() != runningLabel);
        QVERIFY(entry->property("hovered").toBool());
        QVERIFY(entry->property("highlighted").toBool());
        QVERIFY(entry->hasActiveFocus());
        QCOMPARE(menu->property("currentIndex").toInt(), selectedIndex);
        QCOMPARE(destroyed.count(), 0);
        QCOMPARE(hoverChanges.count(), 0);
        QCOMPARE(highlightChanges.count(), 0);
        QCOMPARE(focusChanges.count(), 0);

        if (!auditDirectory.isEmpty()) {
            QVERIFY(visual_audit::save(window, auditDirectory, scene + "-hover-after"));
            auto *cell = findVisual(window->contentItem(), "entry-0-0");
            auto *label = findVisual(entry, objectName + "Label");
            QVERIFY(cell && label);
            int rowFont = 0;
            for (auto *child : cell->childItems())
                if (child->property("text") == cell->property("display"))
                    rowFont = child->property("font").value<QFont>().pixelSize();
            const auto menuFont = label->property("font").value<QFont>().pixelSize();
            QVERIFY(rowFont > 0);
            const auto ratio = qreal(menuFont) / rowFont;
            QVERIFY(ratio >= 0.95 && ratio <= 1.10);
            const QJsonObject evidence{
                {"taskKind", scene},
                {"ticks", 48},
                {"platform", QGuiApplication::platformName()},
                {"graphicsApi", int(window->rendererInterface()->graphicsApi())},
                {"dpr", window->devicePixelRatio()},
                {"windowWidth", window->width()},
                {"windowHeight", window->height()},
                {"menuWidth", menu->property("width").toDouble()},
                {"menuHeight", menu->property("height").toDouble()},
                {"rowHeight", cell->height()},
                {"menuRowHeight", entry->height()},
                {"rowFontPixels", rowFont},
                {"menuFontPixels", menuFont},
                {"menuToRowFontRatio", ratio},
                {"delegateReplacements", destroyed.count()},
                {"hoverChanges", hoverChanges.count()},
                {"highlightChanges", highlightChanges.count()},
                {"focusChanges", focusChanges.count()}};
            QFile file(auditDirectory + "/" + scene + "-stability.json");
            QVERIFY(file.open(QIODevice::WriteOnly));
            QVERIFY(file.write(QJsonDocument(evidence).toJson()) > 0);
        }
        xdzip::reader::Event another{};
        another.kind = xdzip::reader::EventKind::TaskUpdate;
        another.task.id = progress.task.id + 1;
        another.task.status = rust::String("succeeded");
        controller->receive(another);
        const auto frameCount = frames.count();
        window->requestUpdate();
        QTRY_VERIFY(frames.count() > frameCount);
        QVERIFY(entry);
        QCOMPARE(window->findChild<QObject *>(objectName), entry.data());
        QTRY_VERIFY(entry->property("hovered").toBool());
        QVERIFY(entry->property("highlighted").toBool());
        QVERIFY(entry->hasActiveFocus());
        QCOMPARE(menu->property("currentIndex").toInt(), selectedIndex + 1);
        QTest::mouseClick(window, Qt::LeftButton, Qt::NoModifier, pointer);
        QTRY_VERIFY(!menu->property("visible").toBool());
        QCOMPARE(window->property("activeTaskId").toString(), taskId);
        auto *task = window->findChild<QQuickWindow *>("taskWindow");
        QTRY_VERIFY(task && task->isVisible());
        auto snapshot = window->property("taskSnapshot");
        if (snapshot.metaType() == QMetaType::fromType<QJSValue>())
            snapshot = snapshot.value<QJSValue>().toVariant();
        QCOMPARE(snapshot.toMap().value("id").toString(), taskId);
        QCOMPARE(snapshot.toMap().value("status").toString(), "succeeded");
        task->close();
        window->requestActivate();
        QTRY_VERIFY(window->isActive());
        QTest::mouseClick(
            window, Qt::LeftButton, Qt::NoModifier,
            button->mapToScene(QPointF(button->width() / 2, button->height() / 2)).toPoint());
        QTRY_VERIFY(menu->property("opened").toBool());
        QTest::keyClick(window, Qt::Key_Home);
        QTRY_COMPARE(menu->property("currentIndex").toInt(), selectedIndex);
        QTest::keyClick(window, Qt::Key_Down);
        QTRY_VERIFY(entry->hasActiveFocus());
        QCOMPARE(menu->property("currentIndex").toInt(), selectedIndex + 1);
        QTest::keyClick(window, Qt::Key_Return);
        QTRY_VERIFY(!menu->property("visible").toBool() && task->isVisible());
        QCOMPARE(window->property("activeTaskId").toString(), taskId);
        task->close();
        window->close();
    }
    void extractionSubmissionKeepsBrowserPresentationStable() {
        QTemporaryDir target;
        QVERIFY(target.isValid());
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        QVERIFY(QTest::qWaitForWindowExposed(window));
        auto *controller = window->findChild<BrowserController *>();
        QVERIFY(controller);
        controller->openPath(QStringLiteral(XDZIP_FIXTURES "/../extract/tree.zip"));
        QVERIFY(controller->busy());
        QVERIFY(controller->browsingBusy());
        QVERIFY(!controller->taskStarting());
        QTRY_VERIFY(controller->ready() && !controller->busy());
        controller->selectAll(true);
        const auto selected = controller->selectedCount();
        const auto rows = controller->entries()->rowCount();
        controller->prepareExtraction(false);
        auto *extract = window->findChild<QQuickWindow *>("extractWindow");
        QVERIFY(extract);
        QTRY_VERIFY(extract->isVisible());
        auto *field = findVisual(extract->contentItem(), "extractTarget");
        auto *openAfter = findVisual(extract->contentItem(), "extractOpenAfter");
        auto *start = findVisual(extract->contentItem(), "startExtract");
        auto *change = findVisual(extract->contentItem(), "extractChangeTarget");
        auto *overlay = findVisual(window->contentItem(), "archiveLoadingOverlay");
        QVERIFY(field && openAfter && start && change && overlay);
        for (const auto &name :
             {"archiveOpen", "archiveNew", "archiveTest", "archiveMore", "archiveExtract"}) {
            auto *action = findVisual(window->contentItem(), QString::fromLatin1(name));
            QVERIFY(action);
            QVERIFY(!action->property("tooltipEnabled").toBool());
        }
        field->setProperty("text", target.path());
        QTRY_VERIFY(!controller->busy());
        controller->checkTarget(target.path());
        QTRY_VERIFY(!controller->busy());
        QVERIFY(checkedTarget(controller, target.path()));
        openAfter->setProperty("checked", false);
        QTRY_VERIFY(start->isEnabled());
        QVERIFY(!overlay->isVisible());
        bool controlsStable = true;
        bool overlayStable = true;
        bool sawSubmitting = false;
        bool sawStartDisabled = false;
        bool retried = false;
        QObject monitor;
        for (const auto &name :
             {"archiveOpen", "archiveNew", "archiveMore", "archiveSearch", "archiveExtract"}) {
            auto *control = findVisual(window->contentItem(), QString::fromLatin1(name));
            QVERIFY(control && control->isEnabled());
            connect(control, &QQuickItem::enabledChanged, &monitor,
                    [&, control] { controlsStable &= control->isEnabled(); });
        }
        QVERIFY(change->isEnabled());
        connect(change, &QQuickItem::enabledChanged, &monitor,
                [&] { controlsStable &= change->isEnabled(); });
        connect(overlay, &QQuickItem::visibleChanged, &monitor,
                [&] { overlayStable &= !overlay->isVisible(); });
        connect(controller, &BrowserController::changed, &monitor, [&] {
            if (controller->taskStarting()) {
                sawSubmitting = true;
                if (!retried) {
                    retried = true;
                    controller->startExtraction(target.path(), false, true, 0, false, false);
                }
                controlsStable &=
                    !controller->browsingBusy() && controller->browserQueryAvailable();
            }
        });
        connect(start, &QQuickItem::enabledChanged, &monitor,
                [&] { sawStartDisabled |= controller->taskStarting() && !start->isEnabled(); });
        QSignalSpy resets(controller->entries(), &QAbstractItemModel::modelReset);
        QSignalSpy started(controller, &BrowserController::extractionStarted);
        // Use the real pointer path; the changed handler also attempts a duplicate request.
        QTest::mouseClick(
            extract, Qt::LeftButton, Qt::NoModifier,
            start->mapToScene(QPointF(start->width() / 2, start->height() / 2)).toPoint());
        QTRY_COMPARE(started.count(), 1);
        QTRY_VERIFY(!extract->isVisible());
        QTRY_VERIFY_WITH_TIMEOUT(!controller->tasks().isEmpty() &&
                                     controller->tasks().first().toMap().value("terminal").toBool(),
                                 10000);
        QVERIFY(sawSubmitting);
        QVERIFY(!sawStartDisabled);
        QVERIFY(controlsStable);
        QVERIFY(overlayStable);
        QCOMPARE(resets.count(), 0);
        QCOMPARE(controller->entries()->rowCount(), rows);
        QCOMPARE(controller->selectedCount(), selected);
        QCOMPARE(controller->tasks().size(), 1);
        QCOMPARE(controller->tasks().first().toMap().value("status").toString(), "succeeded");
        window->close();
    }
    void rejectedExtractionKeepsStartButtonEnabledWithoutDuplicateSubmission() {
        QTemporaryDir target;
        QVERIFY(target.isValid());
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        QVERIFY(QTest::qWaitForWindowExposed(window));
        auto *controller = window->findChild<BrowserController *>();
        QVERIFY(controller);
        controller->openPath(QStringLiteral(XDZIP_FIXTURES "/../extract/unsafe.zip"));
        QTRY_VERIFY(controller->ready() && !controller->busy());
        controller->prepareExtraction(true);
        auto *extract = window->findChild<QQuickWindow *>("extractWindow");
        QTRY_VERIFY(extract && extract->isVisible());
        auto *field = findVisual(extract->contentItem(), "extractTarget");
        auto *start = findVisual(extract->contentItem(), "startExtract");
        QVERIFY(field && start);
        QTRY_VERIFY(!controller->busy());
        field->setProperty("text", target.path());
        controller->checkTarget(target.path());
        QTRY_VERIFY(!controller->busy() && checkedTarget(controller, target.path()));
        QTRY_VERIFY(start->isEnabled());
        bool stayedEnabled = true;
        bool retried = false;
        QObject monitor;
        connect(start, &QQuickItem::enabledChanged, &monitor,
                [&] { stayedEnabled &= start->isEnabled(); });
        connect(controller, &BrowserController::changed, &monitor, [&] {
            if (controller->taskStarting() && !retried) {
                retried = true;
                controller->startExtraction(target.path(), true, false, 0, false, false);
            }
        });
        QSignalSpy accepted(controller, &BrowserController::extractionStarted);
        QTest::mouseClick(
            extract, Qt::LeftButton, Qt::NoModifier,
            start->mapToScene(QPointF(start->width() / 2, start->height() / 2)).toPoint());
        QTRY_VERIFY(retried && !controller->busy());
        QVERIFY(stayedEnabled);
        QVERIFY(start->isEnabled());
        QVERIFY(extract->isVisible());
        QVERIFY(!controller->extractionIntent().value("error").toString().isEmpty());
        QCOMPARE(accepted.count(), 0);
        QCOMPARE(controller->tasks().size(), 0);
        QCOMPARE(QDir(target.path()).entryList(QDir::NoDotAndDotDot | QDir::AllEntries).size(), 0);
        window->close();
    }
    void receiverFaultClearsPasswordDraftImmediately() {
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *controller = window->findChild<BrowserController *>();
        QVERIFY(controller);
        // Inject receiver metadata, not an archive task or a secret-bearing event.
        xdzip::reader::Event waiting{};
        waiting.kind = xdzip::reader::EventKind::TaskUpdate;
        waiting.task.id = 71;
        waiting.task.status = rust::String("waiting_password");
        waiting.task.password_prompt_id = 4;
        waiting.task.password_prompt_name = rust::String("owned.txt");
        controller->receive(waiting);
        auto *promptWindow = window->findChild<QQuickWindow *>("passwordWindow");
        QVERIFY(promptWindow);
        QTRY_VERIFY(promptWindow->isVisible());
        auto *field = findVisual(promptWindow->contentItem(), "extractionPassword");
        QVERIFY(field);
        field->setProperty("text", QStringLiteral("discarded local draft"));
        xdzip::reader::Event fault{};
        fault.kind = xdzip::reader::EventKind::RuntimeFault;
        controller->receive(fault);
        QTRY_VERIFY(!promptWindow->isVisible());
        QVERIFY(field->property("text").toString().isEmpty());
        QVERIFY(controller->passwordPrompt().isEmpty());
        controller->requestClose();
        QTRY_VERIFY(controller->requestClose());
    }
    void passwordExtractionRetriesCancelsAndStopsWithApplication() {
        QTemporaryDir input, archives, generated, target, cancelled, exiting;
        QVERIFY(input.isValid() && archives.isValid() && target.isValid() && cancelled.isValid() &&
                exiting.isValid() && generated.isValid());
        const auto source = input.filePath(QStringLiteral("owned.txt"));
        QFile file(source);
        QVERIFY(file.open(QIODevice::WriteOnly));
        QCOMPARE(file.write("owned AES payload\n"), qint64(18));
        file.close();
        // An owned fixture secret, never compared with value-printing test macros.
        const auto password = QStringLiteral(" owned AES 密码 ");
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *controller = window->findChild<BrowserController *>();
        QVERIFY(controller);
        controller->startCreation({source}, archives.path(), QStringLiteral("encrypted.zip"), 8, 6,
                                  false, 0, password);
        QTRY_VERIFY_WITH_TIMEOUT(controller->ready() && !controller->busy() &&
                                     controller->title() == QStringLiteral("encrypted.zip"),
                                 10000);
        QCOMPARE(controller->archiveTotal(), 1);
        QVERIFY(!controller->tasks().first().toMap().value("externalReader").toBool());
        const auto start = [&](const QString &destination) {
            controller->prepareExtraction(true);
            auto *extract = window->findChild<QQuickWindow *>("extractWindow");
            auto *targetField =
                extract ? findVisual(extract->contentItem(), "extractTarget") : nullptr;
            return targetField && targetField->setProperty("text", destination);
        };
        QVERIFY(start(generated.path()));
        QTRY_VERIFY(!controller->busy());
        controller->checkTarget(generated.path());
        QTRY_VERIFY(!controller->busy() && controller->targetMatches(generated.path()));
        controller->startExtraction(generated.path(), true, false, 0, false, false);
        QTRY_VERIFY(!controller->passwordPrompt().isEmpty());
        const auto generatedPrompt = controller->passwordPrompt();
        QVERIFY(controller->submitPassword(generatedPrompt.value("taskId").toString(),
                                           generatedPrompt.value("promptId").toString(), password));
        QTRY_VERIFY(controller->activeTaskCount() == 0);
        QCOMPARE(controller->tasks().first().toMap().value("status").toString(), "succeeded");
        QFile generatedResult(generated.filePath("owned.txt"));
        QVERIFY(generatedResult.open(QIODevice::ReadOnly));
        QCOMPARE(generatedResult.readAll(), QByteArray("owned AES payload\n"));
        // This fixed salt's verifier deterministically rejects the wrong fixture
        // secret. Random writer salts would make the retry assertion flaky.
        controller->openPath(QStringLiteral(XDZIP_FIXTURES "/../extract/password-retry.zip"));
        QTRY_VERIFY(controller->ready() && !controller->busy() &&
                    controller->title() == QStringLiteral("password-retry.zip"));
        QVERIFY(start(target.path()));
        QTRY_VERIFY(!controller->busy());
        controller->checkTarget(target.path());
        QTRY_VERIFY(!controller->busy() && controller->targetMatches(target.path()));
        QSignalSpy extractionStarted(controller, &BrowserController::extractionStarted);
        controller->startExtraction(target.path(), true, false, 0, false, false);
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy(), 10000);
        QVERIFY2(!extractionStarted.isEmpty(),
                 qPrintable(controller->extractionIntent().value("error").toString()));
        QTRY_VERIFY_WITH_TIMEOUT(!controller->passwordPrompt().isEmpty() ||
                                     controller->tasks().first().toMap().value("terminal").toBool(),
                                 10000);
        QVERIFY2(!controller->passwordPrompt().isEmpty(),
                 qPrintable(controller->tasks().first().toMap().value("errorCode").toString()));
        auto *promptWindow = window->findChild<QQuickWindow *>("passwordWindow");
        QVERIFY(promptWindow);
        QTRY_VERIFY(promptWindow->isVisible());
        QCOMPARE(promptWindow->modality(), Qt::ApplicationModal);
        QCOMPARE(promptWindow->width(), 491);
        auto *field = findVisual(promptWindow->contentItem(), "extractionPassword");
        auto *submit = findVisual(promptWindow->contentItem(), "passwordSubmit");
        QVERIFY(field && submit);
        QCOMPARE(field->property("echoMode").toInt(), 2);
        auto *editable = QAccessible::queryAccessibleInterface(field);
        QVERIFY(editable && editable->role() == QAccessible::EditableText);
        QVERIFY(editable->state().passwordEdit);
        QVERIFY(!editable->text(QAccessible::Name).isEmpty());
        auto *visibility = findVisual(promptWindow->contentItem(), "passwordVisibility");
        QVERIFY(visibility);
        auto *visibilityInterface = QAccessible::queryAccessibleInterface(visibility);
        QVERIFY(visibilityInterface && visibilityInterface->actionInterface());
        visibilityInterface->actionInterface()->doAction(QAccessibleActionInterface::pressAction());
        QTRY_COMPARE(field->property("echoMode").toInt(), 0);
        visibilityInterface->actionInterface()->doAction(QAccessibleActionInterface::pressAction());
        QTRY_COMPARE(field->property("echoMode").toInt(), 2);
        const auto first = controller->passwordPrompt();
        const auto task = first.value("taskId").toString();
        const auto token = first.value("promptId").toString();
        QVERIFY(!controller->submitPassword(task, "0", QStringLiteral("ignored")));
        QVERIFY(!controller->submitPassword(task, token, QString(400, QChar(0x4e2d))));
        QVERIFY(!QFile::exists(target.filePath("owned.txt")));
        field->setProperty("text", QStringLiteral("wrong owned fixture"));
        auto *submitInterface = QAccessible::queryAccessibleInterface(submit);
        QVERIFY(submitInterface && submitInterface->actionInterface());
        submitInterface->actionInterface()->doAction(QAccessibleActionInterface::pressAction());
        QTRY_VERIFY_WITH_TIMEOUT(controller->passwordPrompt().value("retry").toBool() &&
                                     controller->passwordPrompt().value("promptId") != token,
                                 10000);
        QTRY_VERIFY(field->property("text").toString().isEmpty());
        QVERIFY(!controller->submitPassword(task, token, password));
        QVERIFY(!QFile::exists(target.filePath("owned.txt")));
        QTRY_VERIFY(submit->isEnabled());
        field->setProperty("text", password);
        field->forceActiveFocus();
        QTest::keyClick(promptWindow, Qt::Key_Return);
        QTRY_VERIFY_WITH_TIMEOUT(controller->activeTaskCount() == 0, 10000);
        QCOMPARE(controller->tasks().first().toMap().value("status").toString(), "succeeded");
        QTRY_VERIFY(!promptWindow->isVisible());
        QVERIFY(field->property("text").toString().isEmpty());
        QFile result(target.filePath("owned.txt"));
        QVERIFY(result.open(QIODevice::ReadOnly));
        QCOMPARE(result.readAll(), QByteArray("owned AES payload\n"));
        const auto snapshots = QJsonDocument::fromVariant(controller->tasks()).toJson();
        QVERIFY(!snapshots.contains(password.toUtf8()));

        QVERIFY(start(cancelled.path()));
        QTRY_VERIFY(!controller->busy());
        controller->checkTarget(cancelled.path());
        QTRY_VERIFY(!controller->busy() && controller->targetMatches(cancelled.path()));
        controller->startExtraction(cancelled.path(), true, false, 0, false, false);
        QTRY_VERIFY(!controller->passwordPrompt().isEmpty() && promptWindow->isVisible());
        field->setProperty("text", QStringLiteral("discarded fixture"));
        promptWindow->close();
        QTRY_VERIFY(controller->activeTaskCount() == 0);
        QCOMPARE(controller->tasks().first().toMap().value("status").toString(), "cancelled");
        QVERIFY(controller->passwordPrompt().isEmpty());
        QVERIFY(field->property("text").toString().isEmpty());
        QVERIFY(!QFile::exists(cancelled.filePath("owned.txt")));

        QVERIFY(start(exiting.path()));
        QTRY_VERIFY(!controller->busy());
        controller->checkTarget(exiting.path());
        QTRY_VERIFY(!controller->busy() && controller->targetMatches(exiting.path()));
        controller->startExtraction(exiting.path(), true, false, 0, false, false);
        QTRY_VERIFY(!controller->passwordPrompt().isEmpty());
        QVERIFY(!controller->requestClose());
        QTRY_VERIFY_WITH_TIMEOUT(controller->requestClose(), 10000);
        QVERIFY(controller->passwordPrompt().isEmpty());
        QVERIFY(field->property("text").toString().isEmpty());
        QVERIFY(!QFile::exists(exiting.filePath("owned.txt")));
    }
    void unixBundleExtractionThroughNativeController() {
#ifndef Q_OS_UNIX
        QSKIP("ZIP symbolic link restoration is supported on POSIX only");
#else
        QTemporaryDir target;
        QVERIFY(target.isValid());
        BrowserController controller;
        controller.openPath(QStringLiteral(XDZIP_FIXTURES "/../extract/unix_bundle.zip"));
        QTRY_VERIFY(controller.ready() && !controller.busy());
        controller.prepareExtraction(true);
        controller.checkTarget(target.path());
        QTRY_VERIFY(!controller.busy() && checkedTarget(&controller, target.path()));
        QSignalSpy started(&controller, &BrowserController::extractionStarted);
        controller.startExtraction(target.path(), true, false, 0, false, false);
        QTRY_COMPARE(started.count(), 1);
        QTRY_VERIFY_WITH_TIMEOUT(!controller.tasks().isEmpty() &&
                                     controller.tasks().first().toMap().value("terminal").toBool(),
                                 10000);
        const auto task = controller.tasks().first().toMap();
        QCOMPARE(task.value("status").toString(), QStringLiteral("succeeded"));
        QCOMPARE(task.value("committedFiles").toString(), QStringLiteral("7"));
        const auto base = target.path() + QStringLiteral("/Demo.app/Contents/");
        QVERIFY(QFileInfo(base + "MacOS/demo").permissions().testFlag(QFile::ExeOwner));
        const auto link = base + "Frameworks/F.framework/F";
        QVERIFY(QFileInfo(link).isSymLink());
        QFile file(link);
        QVERIFY(file.open(QIODevice::ReadOnly));
        QCOMPARE(file.readAll(), QByteArray("owned library\n"));
        file.close();
        controller.requestClose();
        QTRY_VERIFY(controller.requestClose());
#endif
    }
    void temporaryFileOpeningUsesAssociationAndHidesOnlyAfterAcceptance_data() {
        QTest::addColumn<QString>("language");
        QTest::addColumn<double>("factor");
        for (const auto &language : {QStringLiteral("zh-CN"), QStringLiteral("en-US")})
            for (const double factor : {1., 1.25, 1.5})
                QTest::newRow(qPrintable(language + QString::number(factor))) << language << factor;
    }
    void temporaryFileOpeningUsesAssociationAndHidesOnlyAfterAcceptance() {
        QFETCH(QString, language);
        QFETCH(double, factor);
        const auto restoreLanguage = qScopeGuard([] { initializeUiText(QStringLiteral("zh-CN")); });
        QVERIFY(initializeUiText(language));
        QTemporaryDir cache;
        QVERIFY(cache.isValid());
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        QQmlComponent changes(&engine);
        changes.setData(
            "import QtQuick\nimport \"qrc:/xdzip/ui\" as Ui\n"
            "QtObject { property real factor: 1; onFactorChanged: Ui.Theme.fontScale = factor }",
            QUrl());
        std::unique_ptr<QObject> appearance(changes.create());
        QVERIFY(appearance);
        appearance->setProperty("factor", factor);
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *controller = window->findChild<BrowserController *>();
        QVERIFY(controller);
        controller->m_temporaryCacheOverride = cache.filePath(QStringLiteral("缓存 % & space"));
        bool accepted = false;
        QList<QUrl> opened;
        controller->m_fileOpener = [&](const QUrl &url) {
            opened.append(url);
            return accepted;
        };
        controller->openPath(QStringLiteral(XDZIP_FIXTURES "/../extract/stored.zip"));
        QTRY_VERIFY(controller->ready() && !controller->busy());
        QSignalSpy started(controller, &BrowserController::extractionStarted);
        controller->enterDirectory(0);
        QTRY_VERIFY(started.count() > 0 || !controller->busy());
        QVERIFY2(started.count() == 1, qPrintable(controller->message()));
        QTRY_COMPARE(opened.size(), 1);
        auto task = controller->tasks().first().toMap();
        QCOMPARE(task.value("status").toString(), QStringLiteral("succeeded"));
        QCOMPARE(task.value("committedFiles").toString(), QStringLiteral("1"));
        QVERIFY(!task.value("openAccepted").toBool());
        QVERIFY(!task.value("error").toString().isEmpty());
        QVERIFY(opened.first().isLocalFile());
        QVERIFY(QFileInfo(opened.first().toLocalFile()).isFile());
        QVERIFY(opened.first().toLocalFile().startsWith(cache.path()));
        QVERIFY(opened.first().toLocalFile().contains(QStringLiteral("缓存 % & space")));
        auto *progress = window->findChild<QQuickWindow *>("taskWindow");
        QVERIFY(progress);
        QTRY_VERIFY(progress->isVisible());
        auto *openButton = findVisual(progress->contentItem(), "openOutputButton");
        QVERIFY(openButton && openButton->isVisible());
        const auto point = openButton->mapToItem(progress->contentItem(), QPointF(0, 0));
        QVERIFY(point.x() >= 0 && point.x() + openButton->width() <= progress->width());
        QVERIFY(point.y() >= 0 && point.y() + openButton->height() <= progress->height());
        auto *scroll = progress->findChild<QObject *>("taskScroll");
        auto *viewport =
            scroll ? qobject_cast<QQuickItem *>(scroll->property("contentItem").value<QObject *>())
                   : nullptr;
        auto *content =
            viewport
                ? qobject_cast<QQuickItem *>(viewport->property("contentItem").value<QObject *>())
                : nullptr;
        auto *errorMessage = findVisual(progress->contentItem(), "resultError");
        QVERIFY(viewport && content && errorMessage);
        QTest::qWait(80); // Let translated multiline text finish its layout pass.
        const auto errorY = errorMessage->mapToItem(content, QPointF()).y();
        viewport->setProperty(
            "contentY", std::clamp(errorY - 16, 0.,
                                   std::max(0., viewport->property("contentHeight").toDouble() -
                                                    viewport->height())));
        QTest::qWait(30);
        const auto errorPoint = errorMessage->mapToItem(viewport, QPointF());
        QVERIFY2(errorPoint.y() >= -1 &&
                     errorPoint.y() + errorMessage->height() <= viewport->height() + 1,
                 qPrintable(QString("error y=%1 height=%2 viewport=%3 scroll=%4")
                                .arg(errorPoint.y())
                                .arg(errorMessage->height())
                                .arg(viewport->height())
                                .arg(viewport->property("contentY").toDouble())));
        if (!qEnvironmentVariableIsEmpty("XDZIP_OPENING_VISUAL_AUDIT")) {
            QVERIFY(QDir().mkpath(qEnvironmentVariable("XDZIP_OPENING_VISUAL_AUDIT")));
            QVERIFY(visual_audit::save(progress, qEnvironmentVariable("XDZIP_OPENING_VISUAL_AUDIT"),
                                       language + "-" + QString::number(factor) + "-" +
                                           QString::number(progress->devicePixelRatio())));
        }
        accepted = true;
        controller->openOutput(task.value("id").toString());
        QCOMPARE(opened.size(), 2);
        QTRY_VERIFY(!progress->isVisible());
        QVERIFY(controller->tasks().first().toMap().value("openAccepted").toBool());
        QVERIFY(QFileInfo(opened.first().toLocalFile()).isFile());
        QTRY_VERIFY(!controller->busy());
        auto *page = window->property("activePage").value<QObject *>();
        QVERIFY(page);
        QVERIFY(QMetaObject::invokeMethod(page, "showTask",
                                          Q_ARG(QString, task.value("id").toString())));
        QTRY_VERIFY(progress->isVisible());
        emit controller->tasksChanged();
        QVERIFY(progress->isVisible()); // Manually reopened history stays visible.
        progress->close();
        controller->enterDirectory(0);
        QTRY_COMPARE(started.count(), 2);
        QTRY_COMPARE(opened.size(), 3);
        QTRY_VERIFY(!progress->isVisible());
        QVERIFY(opened.first() != opened.last());
        QVERIFY(QFile::remove(opened.last().toLocalFile()));
        controller->openOutput(controller->tasks().first().toMap().value("id").toString());
        QCOMPARE(opened.size(), 3);
        QVERIFY(!controller->tasks().first().toMap().value("openAccepted").toBool());
        QVERIFY(!controller->tasks().first().toMap().value("error").toString().isEmpty());
        controller->requestClose();
        QTRY_VERIFY(controller->requestClose());
        // External applications may open asynchronously after the request.
        QVERIFY(QFileInfo(opened.first().toLocalFile()).isFile());
    }
    void terminalTemporaryOpenBeforeAcceptanceDoesNotReopenProgress() {
        QTemporaryDir cache;
        QVERIFY(cache.isValid());
        const auto path = cache.filePath("owned.txt");
        QFile file(path);
        QVERIFY(file.open(QIODevice::WriteOnly));
        QCOMPARE(file.write("owned fixture"), qint64(13));
        file.close();
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *controller = window->findChild<BrowserController *>();
        auto *progress = window->findChild<QQuickWindow *>("taskWindow");
        QVERIFY(controller && progress);
        int opens = 0;
        controller->m_fileOpener = [&](const QUrl &url) {
            ++opens;
            return url.toLocalFile() == path;
        };
        controller->m_pending = 10001;
        controller->m_busy = true;
        controller->m_result = {};
        controller->m_result.extraction = true;
        QSignalSpy visibility(progress, &QWindow::visibleChanged);
        xdzip::reader::Event event{};
        event.kind = xdzip::reader::EventKind::TaskUpdate;
        event.task.id = 9;
        event.task.status = rust::String("succeeded");
        event.task.committed_files = 1;
        event.task.open_path = rust::String(path.toStdString());
        controller->receive(event);
        QCOMPARE(opens, 1);
        event.kind = xdzip::reader::EventKind::TaskAccepted;
        event.request_id = 10001;
        controller->receive(event);
        event.kind = xdzip::reader::EventKind::Finished;
        controller->receive(event);
        QTest::qWait(20);
        QCOMPARE(visibility.count(), 0);
        QVERIFY(!progress->isVisible());
        QCOMPARE(opens, 1);
        controller->requestClose();
        QTRY_VERIFY(controller->requestClose());
    }
    void m3ExtractionWindowsAndRealOutput() {
        QTemporaryDir target;
        QVERIFY(target.isValid());
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *controller = window->findChild<BrowserController *>();
        QVERIFY(controller);
        controller->openPath(QStringLiteral(XDZIP_FIXTURES "/../extract/tree.zip"));
        QTRY_VERIFY(controller->ready() && !controller->busy());
        QCOMPARE(controller->entries()->rowCount(), 3);
        controller->selectAll(true);
        controller->prepareExtraction(false);
        auto *extract = window->findChild<QQuickWindow *>("extractWindow");
        QVERIFY(extract);
        QTRY_VERIFY(extract->isVisible());
        QTRY_VERIFY(visibleImagesReady(extract->contentItem()));
        QCOMPARE(extract->width(), 491);
        auto *targetField = findVisual(extract->contentItem(), "extractTarget");
        QVERIFY(targetField);
        for (const auto key : {Qt::Key_Escape, Qt::Key_Return, Qt::Key_Enter}) {
            targetField->forceActiveFocus(Qt::TabFocusReason);
            QTRY_VERIFY(targetField->hasActiveFocus());
            QTest::keyClick(extract, key);
            QTRY_VERIFY(!targetField->hasActiveFocus());
            QVERIFY(extract->isVisible());
        }
        targetField->forceActiveFocus(Qt::TabFocusReason);
        QTest::mouseClick(extract, Qt::LeftButton, Qt::NoModifier, QPoint(8, 8));
        QTRY_VERIFY(!targetField->hasActiveFocus());
        targetField->setProperty("text", target.path());
        QTRY_VERIFY(!controller->busy());
        controller->checkTarget(target.path());
        QTRY_VERIFY(!controller->busy());
        QVERIFY(checkedTarget(controller, target.path()));
        auto *start = findVisual(extract->contentItem(), "startExtract");
        QVERIFY(start);
        QTRY_VERIFY(start->isEnabled());
        if (!qEnvironmentVariableIsEmpty("XDZIP_M3_VISUAL_AUDIT")) {
            const auto directory = qEnvironmentVariable("XDZIP_M3_VISUAL_AUDIT");
            QVERIFY(QDir().mkpath(directory));
            QVERIFY(visual_audit::save(extract, directory, "extract-selected"));
            window->setProperty("darkMode", true);
            QTRY_VERIFY(visibleImagesReady(extract->contentItem()));
            QVERIFY(visual_audit::save(extract, directory, "extract-dark"));
            window->setProperty("darkMode", false);
            QTRY_VERIFY(visibleImagesReady(extract->contentItem()));
            auto *policy = findVisual(extract->contentItem(), "extractConflictPolicy");
            QVERIFY(policy);
            policy->forceActiveFocus(Qt::TabFocusReason);
            QTest::mouseClick(
                extract, Qt::LeftButton, Qt::NoModifier,
                policy->mapToScene(QPointF(policy->width() / 2, policy->height() / 2)).toPoint());
            auto *popup = policy->property("popup").value<QObject *>();
            QVERIFY(popup);
            QTRY_VERIFY(popup->property("visible").toBool());
            QTest::qWait(50);
            auto *popupContent = popup->property("contentItem").value<QQuickItem *>();
            QVERIFY(popupContent);
            const auto popupTop =
                popupContent->mapToScene(QPointF()).y() - popup->property("topPadding").toDouble();
            const auto fieldTop = policy->mapToScene(QPointF()).y();
            const auto menuGap = popupTop >= fieldTop
                                     ? popupTop - fieldTop - policy->height()
                                     : fieldTop - popupTop - popup->property("height").toDouble();
            QVERIFY(qAbs(menuGap - 4) < 1);
            QVERIFY(visual_audit::save(extract, directory, "extract-dropdown"));
            QTest::keyClick(extract, Qt::Key_Escape);
            QTRY_VERIFY(!popup->property("visible").toBool());
            QVERIFY(extract->isVisible());
        }
        // Test the explicit action without opening a desktop file manager on CI.
        controller->startExtraction(target.path(), false, false, 0, false, false);
        QTRY_VERIFY_WITH_TIMEOUT(!controller->tasks().isEmpty() &&
                                     controller->tasks().first().toMap().value("terminal").toBool(),
                                 10000);
        auto result = controller->tasks().first().toMap();
        QCOMPARE(result.value("status").toString(), "succeeded");
        QCOMPARE(result.value("committedFiles").toString(), "3");
        QCOMPARE(result.value("skippedFiles").toString(), "0");
        QVERIFY(QFile::exists(target.filePath(QStringLiteral("照片/说明.txt"))));
        QFile binary(target.filePath("data.bin"));
        QVERIFY(binary.open(QIODevice::ReadOnly));
        const auto data = binary.readAll();
        QCOMPARE(data.size(), 262144);
        for (qsizetype i = 0; i < data.size(); ++i)
            QCOMPARE(static_cast<unsigned char>(data[i]), static_cast<unsigned char>(i % 256));
        binary.close();
        QCOMPARE(controller->entries()->rowCount(), 3);
        QCOMPARE(controller->selectedCount(), 3);
        auto *task = window->findChild<QQuickWindow *>("taskWindow");
        QVERIFY(task);
        QTRY_VERIFY(task->isVisible());
        QTRY_VERIFY(visibleImagesReady(task->contentItem()));
        QCOMPARE(task->width(), 491);
        QVERIFY(!(task->flags() & Qt::FramelessWindowHint));
        QCOMPARE(task->transientParent(), window);
        const auto button = findVisual(task->contentItem(), "cancelButton");
        QVERIFY(button);
        QVERIFY(!button->isVisible());
        const auto capture = [&](QQuickWindow *w, const QString &name) {
            if (qEnvironmentVariableIsEmpty("XDZIP_M3_VISUAL_AUDIT"))
                return true;
            const auto directory = qEnvironmentVariable("XDZIP_M3_VISUAL_AUDIT");
            return QDir().mkpath(directory) && visual_audit::save(w, directory, name);
        };
        QVERIFY(capture(task, "task-success"));
        auto *tasksButton = findVisual(window->contentItem(), "browserTasks");
        QVERIFY(tasksButton);
        QCOMPARE(tasksButton->property("text").toString(),
                 QStringLiteral("任务 %1").arg(controller->tasks().size()));
        QCOMPARE(controller->activeTaskCount(), 0);
        window->requestActivate();
        QTRY_VERIFY(window->isActive());
        QTest::mouseClick(
            window, Qt::LeftButton, Qt::NoModifier,
            tasksButton->mapToScene(QPointF(tasksButton->width() / 2, tasksButton->height() / 2))
                .toPoint());
        auto *tasksMenu = window->findChild<QObject *>("browserTaskMenu");
        QVERIFY(tasksMenu);
        QTRY_VERIFY(tasksMenu->property("visible").toBool());
        QCOMPARE(tasksMenu->property("width").toDouble(), 240.0);
        QTRY_VERIFY(tasksMenu->property("height").toDouble() <= 50.0);
        auto *taskEntry = window->findChild<QObject *>(QStringLiteral("taskMenuItem-") +
                                                       result.value("id").toString());
        QVERIFY(taskEntry);
        QCOMPARE(taskEntry->property("statusLabel").toString(), QStringLiteral("已完成"));
        QCOMPARE(taskEntry->property("labelSize").toDouble(), 13.0);
        QVERIFY(capture(window, "tasks-menu"));
        auto *taskItem = qobject_cast<QQuickItem *>(taskEntry);
        QVERIFY(taskItem);
        taskItem->forceActiveFocus(Qt::TabFocusReason);
        QTest::keyClick(window, Qt::Key_Return);
        QTRY_VERIFY(!tasksMenu->property("visible").toBool());
        QCOMPARE(window->property("activeTaskId").toString(), result.value("id").toString());
        task->requestActivate();
        QTRY_VERIFY(task->isActive());
        QCOMPARE(result.value("details").toList().size(), 3);
        auto *detailsToggle = findVisual(task->contentItem(), "resultDetailsToggle");
        QVERIFY(detailsToggle);
        QTRY_COMPARE(task->height(), task->property("requestedHeight").toInt());
        const auto collapsedHeight = task->height();
        QVERIFY(collapsedHeight < 600);
        detailsToggle->forceActiveFocus(Qt::TabFocusReason);
        QTRY_VERIFY(detailsToggle->hasActiveFocus());
        QTest::keyClick(task, Qt::Key_Space);
        QTRY_VERIFY(detailsToggle->property("checked").toBool());
        const auto availableHeight =
            std::min(600, task->screen()->availableGeometry().height() - 80);
        QTRY_VERIFY(task->height() > collapsedHeight || collapsedHeight >= availableHeight);
        QVERIFY(task->height() <= 600);
        QTRY_VERIFY(visibleImagesReady(task->contentItem()));
        QVERIFY(capture(task, "task-result-details"));
        QTest::keyClick(task, Qt::Key_Space);
        QTRY_VERIFY(!detailsToggle->property("checked").toBool());
        QTRY_COMPARE(task->height(), collapsedHeight);
        controller->prepareExtraction(true);
        QTRY_VERIFY(extract->isVisible());
        QTRY_VERIFY(!controller->busy());
        targetField->setProperty("text", target.path());
        controller->checkTarget(target.path());
        QTRY_VERIFY(!controller->busy());
        QVERIFY(checkedTarget(controller, target.path()));
        QVERIFY(capture(extract, "extract-options"));
        controller->startExtraction(target.path(), true, false, 0, false, false);
        QTRY_VERIFY_WITH_TIMEOUT(controller->activeTaskCount() == 1 &&
                                     controller->tasks().first().toMap().value("status") ==
                                         "waiting",
                                 10000);
        result = controller->tasks().first().toMap();
        const auto taskId = result.value("id").toString();
        const auto prompt = result.value("promptId").toString();
        QVERIFY(
            !controller->decideTask(taskId, QString::number(prompt.toULongLong() + 1), 3, false));
        auto *conflict = window->findChild<QQuickWindow *>("conflictWindow");
        QVERIFY(conflict);
        QTRY_VERIFY(conflict->isVisible());
        QTRY_VERIFY(visibleImagesReady(conflict->contentItem()));
        QCOMPARE(conflict->width(), 491);
        QVERIFY(capture(conflict, "conflict"));
        QVERIFY(capture(task, "task-waiting"));
        conflict->close();
        task->close();
        QCOMPARE(controller->activeTaskCount(), 1);
        const bool previousQuit = QApplication::quitOnLastWindowClosed();
        const auto restoreQuit =
            qScopeGuard([&] { QApplication::setQuitOnLastWindowClosed(previousQuit); });
        auto *preferences =
            engine.singletonInstance<UiPreferences *>("Xdzip.Settings", "AppSettings");
        DesktopIntegration desktop(window, controller, preferences, true, [] { return true; });
        window->close();
        QVERIFY(!window->isVisible());
        QCOMPARE(controller->activeTaskCount(), 1);
        QMenu *taskMenu = nullptr;
        for (auto *item : desktop.trayMenu()->actions())
            if (item->menu())
                taskMenu = item->menu();
        QVERIFY(taskMenu);
        QVERIFY(QMetaObject::invokeMethod(taskMenu, "aboutToShow"));
        QVERIFY(!taskMenu->actions().isEmpty());
        QCOMPARE(taskMenu->actions().first()->data().toString(), taskId);
        QString taskStatus;
        QVERIFY(QMetaObject::invokeMethod(window, "taskStateLabel",
                                          Q_RETURN_ARG(QString, taskStatus),
                                          Q_ARG(QString, QStringLiteral("waiting"))));
        QVERIFY(!taskStatus.isEmpty());
        QVERIFY(taskMenu->actions().first()->text().contains(taskStatus));
        QVERIFY(taskMenu->actions().first()->text().contains(
            window->property("activeTaskId").toString()));
        taskMenu->actions().first()->trigger();
        QVERIFY(window->isVisible());
        QTRY_VERIFY(task->isVisible());
        conflict->close();
        task->close();
        AppQuitGuard guard;
        guard.controller = controller;
        guard.window = window;
        QEvent quitEvent(QEvent::Quit);
        QVERIFY(guard.eventFilter(nullptr, &quitEvent));
        QVERIFY(window->isVisible());
        QCOMPARE(controller->activeTaskCount(), 1);
        auto *quit = window->findChild<QQuickWindow *>("quitWindow");
        QVERIFY(quit);
        QTRY_VERIFY(quit->isVisible());
        QVERIFY(capture(quit, "quit"));
        quit->close();
        controller->closeArchive();
        QTRY_VERIFY(!controller->ready() && !controller->busy());
        // A task retains its input authority after its browser session is closed.
        QVERIFY(controller->decideTask(taskId, prompt, 1, true));
        QVERIFY(!controller->decideTask(taskId, prompt, 3, false));
        QTRY_COMPARE_WITH_TIMEOUT(controller->activeTaskCount(), 0, 10000);
        QCOMPARE(controller->tasks().first().toMap().value("skippedFiles").toString(), "3");
        QCOMPARE(controller->tasks().first().toMap().value("committedFiles").toString(), "0");
        controller->requestClose();
    }
    void partialResultUsesActualOutcomes() {
        QTemporaryDir target;
        QVERIFY(target.isValid());
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *controller = window->findChild<BrowserController *>();
        QVERIFY(controller);
        controller->openPath(QStringLiteral(XDZIP_FIXTURES "/../extract/partial_crc.zip"));
        QTRY_VERIFY(controller->ready() && !controller->busy());
        controller->prepareExtraction(true);
        auto *extract = window->findChild<QQuickWindow *>("extractWindow");
        QVERIFY(extract);
        auto *field = findVisual(extract->contentItem(), "extractTarget");
        QVERIFY(field);
        field->setProperty("text", target.path());
        QTRY_VERIFY(!controller->busy());
        controller->checkTarget(target.path());
        QTRY_VERIFY(!controller->busy() && checkedTarget(controller, target.path()));
        controller->startExtraction(target.path(), true, false, 0, false, false);
        QTRY_VERIFY(!controller->tasks().isEmpty() &&
                    controller->tasks().first().toMap().value("terminal").toBool());
        const auto taskData = controller->tasks().first().toMap();
        QCOMPARE(taskData.value("status").toString(), "failed");
        QCOMPARE(taskData.value("committedFiles").toString(), "2");
        QCOMPARE(taskData.value("errorCode").toString(), "integrity_error");
        QCOMPARE(taskData.value("details").toList().size(), 3);
        QVERIFY(QFileInfo::exists(target.filePath(QStringLiteral("冰川湖.jpg"))));
        QVERIFY(QFileInfo::exists(target.filePath(QStringLiteral("山间日出.jpg"))));
        QVERIFY(!QFileInfo::exists(target.filePath(QStringLiteral("行程手册.pdf"))));
        auto *task = window->findChild<QQuickWindow *>("taskWindow");
        QVERIFY(task);
        QTRY_VERIFY(task->isVisible() && visibleImagesReady(task->contentItem()));
        const auto output = qEnvironmentVariable("XDZIP_RESULT_AUDIT");
        const auto capture = [&](const QString &name) {
            return output.isEmpty() ||
                   (QDir().mkpath(output) && visual_audit::save(task, output, name));
        };
        QVERIFY(capture("result-partial"));
        window->setProperty("darkMode", true);
        QTRY_VERIFY(visibleImagesReady(task->contentItem()));
        QVERIFY(capture("result-dark"));
        window->setProperty("darkMode", false);
        auto *toggle = findVisual(task->contentItem(), "resultDetailsToggle");
        QVERIFY(toggle);
        toggle->forceActiveFocus(Qt::TabFocusReason);
        QTest::qWait(20);
        QTest::keyClick(task, Qt::Key_Space);
        QTRY_VERIFY(toggle->property("checked").toBool());
        QTRY_VERIFY(visibleImagesReady(task->contentItem()));
        QVERIFY(capture("result-details"));
        controller->requestClose();
        QTRY_VERIFY(controller->requestClose());
    }
    void m3IntegrityFailurePreservesDestinationAndCancelEndsWaiting() {
        QTemporaryDir target;
        QVERIFY(target.isValid());
        const auto leaf = target.filePath(QStringLiteral("说明.txt"));
        QFile old(leaf);
        QVERIFY(old.open(QIODevice::WriteOnly));
        old.write("old important file");
        old.close();
        BrowserController controller;
        const auto start = [&](const QString &fixture, int policy) {
            controller.openPath(QStringLiteral(XDZIP_FIXTURES "/../extract/") + fixture);
            QTRY_VERIFY(controller.ready() && !controller.busy());
            controller.prepareExtraction(true);
            controller.checkTarget(target.path());
            QTRY_VERIFY(!controller.busy());
            QVERIFY(checkedTarget(&controller, target.path()));
            controller.startExtraction(target.path(), true, false, policy, false, false);
        };
        start("crc_bad.zip", 3);
        QTRY_VERIFY(!controller.tasks().isEmpty() &&
                    controller.tasks().first().toMap().value("terminal").toBool());
        QCOMPARE(controller.tasks().first().toMap().value("errorCode").toString(),
                 "integrity_error");
        QVERIFY(old.open(QIODevice::ReadOnly));
        QCOMPARE(old.readAll(), QByteArray("old important file"));
        old.close();
        start("deflate.zip", 0);
        QTRY_VERIFY(controller.activeTaskCount() == 1 &&
                    controller.tasks().first().toMap().value("status") == "waiting");
        controller.cancelTask(controller.tasks().first().toMap().value("id").toString());
        QTRY_COMPARE(controller.activeTaskCount(), 0);
        QCOMPARE(controller.tasks().first().toMap().value("status").toString(), "cancelled");
        QVERIFY(old.open(QIODevice::ReadOnly));
        QCOMPARE(old.readAll(), QByteArray("old important file"));
        old.close();
        QCOMPARE(QDir(target.path())
                     .entryList(QDir::Files | QDir::Dirs | QDir::NoDotAndDotDot | QDir::Hidden)
                     .size(),
                 1);
        controller.requestClose();
    }
    void mailboxBackpressureShutdownAndIdle() {
        RuntimeConnection connection;
        QSignalSpy disconnected(&connection, &RuntimeConnection::disconnected);
        QSet<quint64> accepted;
        QSet<quint64> finished;
        bool duplicate = false;
        bool probeScheduled = false;
        bool guiYieldedBeforeDrainCompleted = false;
        connection.setConsumer([&](const xdzip::reader::Event &event) {
            if (!probeScheduled) {
                probeScheduled = true;
                // Posted during the first batch, before drain posts its continuation. Unlike a
                // millisecond timer, this tests yielding independently of machine speed.
                QMetaObject::invokeMethod(
                    &connection,
                    [&] { guiYieldedBeforeDrainCompleted = finished.size() < accepted.size(); },
                    Qt::QueuedConnection);
            }
            if (event.kind == xdzip::reader::EventKind::Finished &&
                accepted.contains(event.request_id)) {
                duplicate |= finished.contains(event.request_id);
                finished.insert(event.request_id);
            }
        });
        QElapsedTimer limit;
        limit.start();
        // Deliberately do not drain GUI notifications while filling both bounded transports.
        while (connection.statistics().peakQueued < 256 && limit.elapsed() < 5000) {
            xdzip::reader::Command command{};
            command.kind = xdzip::reader::CommandKind::Snapshot;
            const auto result = connection.submit(std::move(command));
            if (result.status == xdzip::reader::Admission::Accepted)
                accepted.insert(result.request_id);
            else
                QVERIFY(result.status == xdzip::reader::Admission::Busy);
            std::this_thread::yield();
        }
        QCOMPARE(connection.statistics().peakQueued, std::size_t(256));
        QVERIFY(connection.statistics().peakBytes <= 8 * 1024 * 1024);
        connection.shutdown();
        connection.shutdown();
        QTRY_COMPARE_WITH_TIMEOUT(disconnected.count(), 1, 5000);
        QVERIFY(disconnected.first().first().toBool());
        QCOMPARE(finished, accepted);
        QVERIFY(!duplicate);
        QVERIFY(guiYieldedBeforeDrainCompleted);
        QVERIFY(connection.stopped());
        const auto notifications = connection.statistics().notifications;
        QTest::qWait(30);
        QCOMPARE(connection.statistics().notifications, notifications);
    }
    void idleAndDestructionDuringWork() {
        {
            RuntimeConnection connection;
            QTest::qWait(30);
            QCOMPARE(connection.statistics().notifications, std::size_t(0));
            QSignalSpy stopped(&connection, &RuntimeConnection::disconnected);
            connection.shutdown();
            QTRY_COMPARE_WITH_TIMEOUT(stopped.count(), 1, 5000);
            QVERIFY(stopped.first().first().toBool());
        }
        for (int i = 0; i < 20; ++i) {
            auto controller = std::make_unique<BrowserController>();
            controller->openPath(QStringLiteral(XDZIP_FIXTURES "/pages.zip"));
            // Destructor requests shutdown but never waits on a worker or queued GUI notification.
        }
        QTRY_COMPARE_WITH_TIMEOUT(RuntimeReceiver::activeReceivers(), 0, 5000);
    }
    void consumerCanDestroyConnectionAndInvalidInputIsExplicit() {
        auto connection = std::make_unique<RuntimeConnection>();
        connection->setConsumer([&](const xdzip::reader::Event &) { connection.reset(); });
        bool submitted = false;
        QTRY_VERIFY_WITH_TIMEOUT(([&] {
                                     if (submitted)
                                         return true;
                                     xdzip::reader::Command command{};
                                     command.kind = xdzip::reader::CommandKind::Snapshot;
                                     submitted = connection->submit(std::move(command)).status ==
                                                 xdzip::reader::Admission::Accepted;
                                     return submitted;
                                 })(),
                                 5000);
        QTRY_VERIFY_WITH_TIMEOUT(!connection, 5000);
        BrowserController controller;
        controller.openPath(QString(4097, 'x'));
        QVERIFY(!controller.busy());
        QVERIFY(!controller.ready());
        QVERIFY(controller.message().contains(QString::fromUtf8("支持范围")));
        controller.requestClose();
        QTRY_VERIFY_WITH_TIMEOUT(!controller.runtimeAvailable(), 5000);
    }
    void realPagesAndFailure() {
        BrowserController controller;
        controller.openPath(QStringLiteral(XDZIP_FIXTURES "/pages.zip"));
        QVERIFY(controller.busy());
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        QVERIFY(controller.ready());
        QCOMPARE(controller.total(), 1);
        QCOMPARE(controller.archiveTotal(), 270);
        QCOMPARE(controller.entries()->data(controller.entries()->index(0, 0)).toString(),
                 "folder");
        controller.selectRow(0);
        QVERIFY(!controller.canPreflight());
        controller.enterDirectory(0);
        QTRY_VERIFY(!controller.busy());
        QCOMPARE(controller.total(), 270);
        QCOMPARE(controller.entries()->rowCount(), 270);
        const auto session = controller.sessionId();
        controller.selectRow(0);
        QCOMPARE(controller.selectedCount(), 1);
        QCOMPARE(controller.entries()->rowCount(), 270);
        QCOMPARE(controller.entries()
                     ->data(controller.entries()->index(256, 0), Qt::UserRole)
                     .toString(),
                 "257");
        QCOMPARE(controller.sessionId(), session);
        controller.openPath(QStringLiteral(XDZIP_FIXTURES "/truncated.zip"));
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        QVERIFY(!controller.ready());
        QCOMPARE(controller.entries()->rowCount(), 0);
        QVERIFY(!controller.message().isEmpty());
    }
    void selectionSearchAndDirectoryNavigation() {
        BrowserController controller;
        controller.openPath(QStringLiteral(XDZIP_FIXTURES "/normal.zip"));
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        controller.selectRow(2);
        controller.selectRow(3);
        QCOMPARE(controller.selectedCount(), 2);
        QCOMPARE(controller.detail().value("id").toString(), "5");
        controller.selectRow(2);
        QCOMPARE(controller.selectedCount(), 1);
        controller.enterDirectory(0);
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        QCOMPARE(controller.total(), 1);
        QCOMPARE(controller.selectedCount(), 0);
        QVERIFY(controller.detail().isEmpty());
        QCOMPARE(controller.location(), QString::fromUtf8("旅行/"));
        controller.showRoot();
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        controller.search("NOTES");
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        QCOMPARE(controller.total(), 2);
        QCOMPARE(
            controller.entries()->data(controller.entries()->index(0, 0), Qt::UserRole).toString(),
            "3");
        controller.search("missing");
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        QVERIFY(controller.ready());
        QCOMPARE(controller.total(), 0);
        controller.search("");
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        QCOMPARE(controller.total(), 4);
    }
    void typeHeaderSortGroupsDirectoriesBeforeExtensionlessFiles() {
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *controller = window->findChild<BrowserController *>();
        controller->openPath(QStringLiteral(XDZIP_FIXTURES "/type-order.zip"));
        QTRY_VERIFY(controller->ready() && !controller->busy());
        QVERIFY(QTest::qWaitForWindowExposed(window));
        auto *header = findVisual(window->contentItem(), "sort-column-3");
        QVERIFY(header);
        const auto click = [window, header] {
            QTest::mouseClick(
                window, Qt::LeftButton, Qt::NoModifier,
                header->mapToScene(QPointF(header->width() / 2, header->height() / 2)).toPoint());
        };
        const auto names = [controller] {
            QStringList result;
            for (int row = 0; row < controller->entries()->rowCount(); ++row)
                result.append(
                    controller->entries()->data(controller->entries()->index(row, 0)).toString());
            return result;
        };
        click();
        QTRY_VERIFY(controller->sortOrder() == 9 && !controller->busy());
        QCOMPARE(names(),
                 QStringList({"item-alpha", "item-folder.ZIP", "item-implicit", "item-README",
                              "item-LICENSE", "item-duplicate", "item-duplicate", "item-image.JPG",
                              "item-image.jpeg", "item-last.ZIP"}));
        QCOMPARE(controller->entries()->data(controller->entries()->index(2, 3)).toString(),
                 QString::fromUtf8("文件夹"));
        QCOMPARE(controller->entries()->data(controller->entries()->index(3, 3)).toString(),
                 QString::fromUtf8("文件"));
        click();
        QTRY_VERIFY(controller->sortOrder() == 10 && !controller->busy());
        QCOMPARE(names(),
                 QStringList({"item-alpha", "item-folder.ZIP", "item-implicit", "item-last.ZIP",
                              "item-image.JPG", "item-image.jpeg", "item-README", "item-LICENSE",
                              "item-duplicate", "item-duplicate"}));
        controller->search("item");
        QTRY_VERIFY(!controller->busy());
        QCOMPARE(names(),
                 QStringList({"item-alpha/", "item-folder.ZIP/", "item-last.ZIP", "item-image.JPG",
                              "item-image.jpeg", "item-README", "item-LICENSE", "item-duplicate",
                              "item-duplicate", "item-implicit/item-child"}));
        click();
        QTRY_VERIFY(controller->sortOrder() == 9 && !controller->busy());
        QCOMPARE(names(),
                 QStringList({"item-alpha/", "item-folder.ZIP/", "item-README", "item-LICENSE",
                              "item-duplicate", "item-duplicate", "item-implicit/item-child",
                              "item-image.JPG", "item-image.jpeg", "item-last.ZIP"}));
        controller->requestClose();
        QTRY_VERIFY(!controller->runtimeAvailable());
    }
    void sourceChangeDuringPreflightInvalidatesSnapshot() {
        QTemporaryDir owned;
        const auto path = owned.filePath("preflight.zip");
        QVERIFY(QFile::copy(QStringLiteral(XDZIP_FIXTURES "/normal.zip"), path));
        BrowserController controller;
        QSignalSpy report(&controller, &BrowserController::preflightReady);
        controller.openPath(path);
        QTRY_VERIFY(controller.ready() && !controller.busy());
        controller.selectOnlyRow(1);
        QVERIFY(controller.canPreflight());
        QFile changed(path);
        QVERIFY(changed.open(QIODevice::Append));
        QCOMPARE(changed.write("changed"), 7);
        changed.close();
        controller.inspectSelection();
        QTRY_COMPARE(report.count(), 1);
        QVERIFY(!controller.busy());
        QVERIFY(!controller.ready());
        QCOMPARE(controller.total(), 0);
        QCOMPARE(controller.archiveTotal(), 0);
        QCOMPARE(controller.entries()->rowCount(), 0);
        QCOMPARE(controller.directories()->rowCount(), 0);
        QCOMPARE(controller.selectedCount(), 0);
        QVERIFY(controller.detail().isEmpty());
        QVERIFY(controller.preflightReport().contains("error"));
        controller.requestClose();
        QTRY_VERIFY(!controller.runtimeAvailable());
    }
    void m2LargeArchiveLatestQueryAndSourceChange() {
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *controller = window->findChild<BrowserController *>();
        QVERIFY(controller);
        QElapsedTimer elapsed;
        elapsed.start();
        controller->openPath(QStringLiteral(XDZIP_M2_FIXTURE));
        QTRY_VERIFY_WITH_TIMEOUT(controller->ready() && !controller->busy(), 20000);
        const auto openMilliseconds = elapsed.elapsed();
        const auto capturePath = qEnvironmentVariable("XDZIP_VISUAL_AUDIT");
        if (!capturePath.isEmpty()) {
            QVERIFY(QDir().mkpath(capturePath));
            QVERIFY(visual_audit::save(window, capturePath, "m2-large-flat"));
        }
        QCOMPARE(controller->total(), 100000);
        QCOMPARE(controller->archiveTotal(), 100000);
        QCOMPARE(controller->entries()->rowCount(), 100000);
        QCOMPARE(controller->entries()->data(controller->entries()->index(99999, 0)).toString(),
                 "item-099999.txt");
        QVERIFY(QTest::qWaitForWindowExposed(window));
        // Model publication and TableView incubation are distinct events. Wait
        // for a real visible cell, not a fixed 50ms delay on a shared runner.
        QTRY_VERIFY_WITH_TIMEOUT(findVisual(window->contentItem(), "entry-0-0"), 5000);
        int delegates = 0;
        const auto countDelegates = [&delegates](auto &&self, QQuickItem *item) -> void {
            if (item->objectName().startsWith("entry-") && item->objectName().count('-') == 2)
                ++delegates;
            for (auto *child : item->childItems())
                self(self, child);
        };
        countDelegates(countDelegates, window->contentItem());
        QVERIFY(delegates > 0 && delegates < 512);
        controller->selectAll(true);
        QCOMPARE(controller->selectedCount(), 100000);
        QVERIFY(!controller->canPreflight());
        QCOMPARE(controller->selectedBytes(), "0");
        elapsed.restart();
        controller->sortByColumn(0);
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy(), 20000);
        const auto sortMilliseconds = elapsed.elapsed();
        QCOMPARE(controller->total(), 100000);
        QCOMPARE(controller->selectedCount(), 0);
        controller->selectAll(true);
        controller->search("099998");
        controller->search("099997");
        controller->search("099999");
        controller->sortByColumn(0);
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy(), 10000);
        QCOMPARE(controller->total(), 1);
        QCOMPARE(controller->selectedCount(), 0);
        QCOMPARE(controller->sortOrder(), 2);
        QCOMPARE(controller->entries()->data(controller->entries()->index(0, 0)).toString(),
                 "item-099999.txt");
        if (!capturePath.isEmpty())
            QVERIFY(visual_audit::save(window, capturePath, "m2-latest-query"));
        controller->openPath(QStringLiteral(XDZIP_M2_HIERARCHICAL));
        QTRY_VERIFY_WITH_TIMEOUT(controller->ready() && !controller->busy(), 20000);
        QCOMPARE(controller->total(), 100);
        QCOMPARE(controller->archiveTotal(), 100000);
        controller->navigateDirectory("d1");
        controller->navigateDirectory("d2");
        controller->showRoot();
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy(), 10000);
        QCOMPARE(controller->total(), 100);
        QCOMPARE(controller->location(), "");
        controller->navigateDirectory("d100");
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy(), 10000);
        QCOMPARE(controller->total(), 1000);
        if (!capturePath.isEmpty())
            QVERIFY(visual_audit::save(window, capturePath, "m2-directory"));
        QCOMPARE(controller->entries()->data(controller->entries()->index(0, 0)).toString(),
                 "item-099000.txt");
        controller->search("099999");
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy(), 10000);
        QCOMPARE(controller->entries()->data(controller->entries()->index(0, 0)).toString(),
                 "folder-099/item-099999.txt");
        qInfo().noquote()
            << QString("M2_QT_METRICS open_ms=%1 rows=100000 delegates=%2 full_sort_ms=%3")
                   .arg(openMilliseconds)
                   .arg(delegates)
                   .arg(sortMilliseconds);
        QTemporaryDir owned;
        QVERIFY(owned.isValid());
        const auto path = owned.filePath("source.zip");
        QVERIFY(QFile::copy(QStringLiteral(XDZIP_FIXTURES "/normal.zip"), path));
        controller->openPath(path);
        QTRY_VERIFY(controller->ready() && !controller->busy());
        QFile changed(path);
        QVERIFY(changed.open(QIODevice::Append));
        QCOMPARE(changed.write("x"), 1);
        changed.close();
        controller->search("notes");
        QTRY_VERIFY(!controller->busy());
        QVERIFY(!controller->ready());
        QCOMPARE(controller->entries()->rowCount(), 0);
        QVERIFY(controller->message().contains(QString::fromUtf8("源文件")));
        controller->requestClose();
        QTRY_VERIFY(!controller->runtimeAvailable());
    }
    void completeNavigationAndGlobalSort() {
        BrowserController controller;
        QAbstractItemModelTester modelCheck(controller.directories(),
                                            QAbstractItemModelTester::FailureReportingMode::QtTest);
        controller.openPath(QStringLiteral(XDZIP_FIXTURES "/pages.zip"));
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        auto *tree = controller.directories();
        QCOMPARE(tree->rowCount(), 1);
        const auto node = tree->index(0, 0);
        QCOMPARE(tree->data(node).toString(), "folder");
        QCOMPARE(tree->data(node, Qt::UserRole + 1).toInt(), 270);
        controller.navigateDirectory(tree->data(node, Qt::UserRole).toString());
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        QCOMPARE(controller.total(), 270);
        QCOMPARE(controller.location(), "folder/");
        controller.sortByColumn(0);
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        controller.selectRow(0);
        controller.sortByColumn(0);
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        QCOMPARE(controller.sortOrder(), 2);
        QCOMPARE(controller.selectedCount(), 0);
        QCOMPARE(
            controller.entries()->data(controller.entries()->index(0, 0), Qt::UserRole).toString(),
            "270");
        QCOMPARE(controller.entries()->rowCount(), 270);
        QCOMPARE(controller.entries()
                     ->data(controller.entries()->index(128, 0), Qt::UserRole)
                     .toString(),
                 "142");
        QCOMPARE(tree->rowCount(), 1);
        QCOMPARE(tree->data(tree->index(0, 0), Qt::UserRole + 1).toInt(), 270);
        controller.sortByColumn(-1);
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        QCOMPARE(controller.offset(), 0);
        QCOMPARE(controller.sortOrder(), 0);
        controller.openPath(QStringLiteral(XDZIP_FIXTURES "/navigation-depth.zip"));
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        QVERIFY(controller.ready());
        QVERIFY(controller.navigationLimited());
        QCOMPARE(tree->rowCount(), 0);
        QCOMPARE(controller.total(), 0);
        controller.search("file");
        QTRY_VERIFY(!controller.busy());
        QCOMPARE(controller.total(), 1);
        controller.openPath(QStringLiteral(XDZIP_FIXTURES "/empty.zip"));
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        QVERIFY(controller.ready());
        QVERIFY(!controller.navigationLimited());
        QCOMPARE(tree->rowCount(), 0);
    }
    void contextualSelectionRangeAndClipboard() {
        BrowserController controller;
        controller.openPath(QStringLiteral(XDZIP_FIXTURES "/normal.zip"));
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        QCOMPARE(controller.archiveTotal(), 5);
        controller.selectOnlyRow(2);
        controller.selectRow(3);
        controller.selectForContext(2);
        QCOMPARE(controller.selectedCount(), 2);
        QCOMPARE(controller.detail().value("id").toString(), "4");
        controller.selectForContext(0);
        QCOMPARE(controller.selectedCount(), 1);
        controller.copySelection(false);
        QCOMPARE(QGuiApplication::clipboard()->text(), QString::fromUtf8("旅行"));
        controller.copySelection(true);
        QCOMPARE(QGuiApplication::clipboard()->text(), QString::fromUtf8("旅行/"));
        controller.selectOnlyRow(1);
        controller.selectRangeTo(3);
        QCOMPARE(controller.selectedCount(), 3);
        controller.copySelection(false);
        QCOMPARE(QGuiApplication::clipboard()->text().split('\n').size(), 3);
        controller.selectAll(true);
        QCOMPARE(controller.selectedCount(), 4);
        QVERIFY(!controller.detail().isEmpty());
        QCOMPARE(controller.detail().value("id").toString(), "5");
        QVERIFY(!controller.detail().value("size").toString().isEmpty());
        QVERIFY(!controller.detail().value("method").toString().isEmpty());
        controller.selectAll(false);
        QCOMPARE(controller.selectedCount(), 0);
        QVERIFY(controller.detail().isEmpty());
        controller.search("NOTES");
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        QCOMPARE(controller.archiveTotal(), 5);
        controller.selectRangeTo(1);
        QCOMPARE(controller.selectedCount(), 1);
        controller.selectRangeTo(0);
        QCOMPARE(controller.selectedCount(), 2);
        controller.openPath(QStringLiteral(XDZIP_FIXTURES "/pages.zip"));
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        controller.enterDirectory(0);
        QTRY_VERIFY(!controller.busy());
        controller.selectAll(true);
        QCOMPARE(controller.selectedCount(), 270);
        controller.selectAll(false);
        QCOMPARE(controller.selectedCount(), 0);
        controller.selectRangeTo(269);
        QVERIFY(controller.selectedCount() > 0);
    }
    void treeExpansionAndColumnInteraction() {
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *controller = window->findChild<BrowserController *>();
        QVERIFY(controller);
        controller->openPath(QStringLiteral(XDZIP_FIXTURES "/navigation.zip"));
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy(), 5000);
        QVERIFY(QTest::qWaitForWindowExposed(window));
        auto *treeView = findVisual(window->contentItem(), "directoryTree");
        QVERIFY(treeView);
        QVERIFY(treeView->property("selectionModel").value<QObject *>());
        QTRY_VERIFY(treeView->property("contentWidth").toDouble() <= treeView->width() + 1);
        auto *model = controller->directories();
        QCOMPARE(model->rowCount(), 3);
        const auto parent = model->index(0, 0);
        QCOMPARE(model->rowCount(parent), 1);
        QCOMPARE(model->data(parent, Qt::UserRole + 1).toInt(), 3);
        QQuickItem *node = nullptr;
        QTRY_VERIFY((node = findVisual(window->contentItem(), "directory-d1")));
        auto *expand = findVisual(window->contentItem(), "directoryDisclosure-d1");
        QVERIFY(expand);
        QTest::mouseClick(
            window, Qt::LeftButton, Qt::NoModifier,
            expand->mapToScene(QPointF(expand->width() / 2, expand->height() / 2)).toPoint());
        QQuickItem *child = nullptr;
        QTRY_VERIFY((child = findVisual(window->contentItem(), "directory-d2")));
        QTest::mouseClick(window, Qt::LeftButton, Qt::NoModifier,
                          child->mapToScene(QPointF(80, child->height() / 2)).toPoint());
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy(), 5000);
        QCOMPARE(controller->directoryId(), "d2");
        QCOMPARE(controller->total(), 2);
        QCOMPARE(controller->location(), "photos/2026/");
        // Arrow keys move the tree current item; Enter activates that item's scope.
        treeView->forceActiveFocus();
        QTest::keyClick(window, Qt::Key_Down);
        QCOMPARE(treeView->property("currentRow").toInt(), 2);
        QTest::keyClick(window, Qt::Key_Return);
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy(), 5000);
        QCOMPARE(controller->location(), "documents/");
        QCOMPARE(controller->total(), 1);
        QTRY_VERIFY(!treeView->hasActiveFocus());
        treeView->forceActiveFocus();
        QTest::keyClick(window, Qt::Key_Up);
        QTest::keyClick(window, Qt::Key_Return);
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy(), 5000);
        QCOMPARE(controller->location(), "photos/2026/");
        auto *header = findVisual(window->contentItem(), "sort-column-1");
        QVERIFY(header);
        QTest::mouseClick(
            window, Qt::LeftButton, Qt::NoModifier,
            header->mapToScene(QPointF(header->width() / 2, header->height() / 2)).toPoint());
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy(), 5000);
        QCOMPARE(controller->sortOrder(), 3);
        header->forceActiveFocus();
        QTest::keyClick(window, Qt::Key_Return);
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy(), 5000);
        QCOMPARE(controller->sortOrder(), 4);
        QCOMPARE(controller->entries()
                     ->data(controller->entries()->index(0, 0), Qt::UserRole)
                     .toString(),
                 "2");
        // Any relayout can reuse even the parent delegate for another model identity.
        auto clickParent = [&] {
            auto *button = findVisual(window->contentItem(), "directoryDisclosure-d1");
            if (!button)
                return false;
            QTest::mouseClick(
                window, Qt::LeftButton, Qt::NoModifier,
                button->mapToScene(QPointF(button->width() / 2, button->height() / 2)).toPoint());
            return true;
        };
        QVERIFY(clickParent());
        QTRY_COMPARE(treeView->property("rows").toInt(), 3);
        QCOMPARE(controller->directoryId(), "d2");
        QCOMPARE(controller->location(), "photos/2026/");
        auto branchMarkerVisible = [&] {
            auto *marker = findVisual(window->contentItem(), "directoryBranchMarker-d1");
            return marker && marker->isVisible();
        };
        QTRY_VERIFY(branchMarkerVisible());
        QVERIFY(clickParent());
        QTRY_COMPARE(treeView->property("rows").toInt(), 4);
        QTRY_VERIFY(!branchMarkerVisible());
        // Collapse/reopen reuses delegates. Assert the rendered hierarchy, not merely
        // the expansion flag, before capturing or allowing a later frame to hide drift.
        for (int cycle = 0; cycle < 8; ++cycle) {
            QTRY_VERIFY([&] {
                auto *parentIcon = findVisual(window->contentItem(), "directoryFolder-d1");
                auto *childIcon = findVisual(window->contentItem(), "directoryFolder-d2");
                return parentIcon && childIcon && childIcon->isVisible() &&
                       qAbs(childIcon->mapToScene(QPointF()).x() -
                            parentIcon->mapToScene(QPointF()).x() - 12.8) < 0.5;
            }());
            if (cycle < 7) {
                QVERIFY(clickParent());
                QTRY_COMPARE(treeView->property("rows").toInt(), 3);
                QTRY_VERIFY([&] {
                    auto *collapsed = findVisual(window->contentItem(), "directory-d2");
                    return !collapsed || !collapsed->isVisible();
                }());
                QVERIFY(clickParent());
                QTRY_COMPARE(treeView->property("rows").toInt(), 4);
            }
        }
        if (!qEnvironmentVariableIsEmpty("XDZIP_VISUAL_AUDIT")) {
            const auto directory = qEnvironmentVariable("XDZIP_VISUAL_AUDIT");
            QVERIFY(QDir().mkpath(directory));
            QVERIFY(visual_audit::save(window, directory, "navigation-tree"));
        }
    }
    void treeWidthTracksOnlyExpandedNodes() {
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *controller = window->findChild<BrowserController *>();
        controller->openPath(QStringLiteral(XDZIP_FIXTURES "/tree-layout.zip"));
        QTRY_VERIFY(controller->ready() && !controller->busy());
        auto *tree = findVisual(window->contentItem(), "directoryTree");
        QVERIFY(tree);
        QTRY_COMPARE(tree->property("rows").toInt(), 2);
        QTRY_VERIFY(tree->property("contentWidth").toDouble() <= tree->width() + 1);
        QVERIFY(
            QMetaObject::invokeMethod(tree, "expandRecursively", Q_ARG(int, 0), Q_ARG(int, -1)));
        QTRY_VERIFY(tree->property("rows").toInt() > 12);
        QTRY_VERIFY(tree->property("contentWidth").toDouble() > tree->width());
        QVERIFY(QMetaObject::invokeMethod(tree, "collapseRecursively", Q_ARG(int, 0)));
        QTRY_COMPARE(tree->property("rows").toInt(), 2);
        QTRY_VERIFY(tree->property("contentWidth").toDouble() <= tree->width() + 1);
    }
    void preflightPreservesSelectionAndReportsDuplicates() {
        BrowserController controller;
        controller.openPath(QStringLiteral(XDZIP_FIXTURES "/normal.zip"));
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        controller.selectRow(1);
        controller.selectRow(2);
        QSignalSpy completed(&controller, &BrowserController::preflightReady);
        controller.inspectSelection();
        QTRY_COMPARE_WITH_TIMEOUT(completed.count(), 1, 5000);
        QCOMPARE(controller.selectedCount(), 2);
        QCOMPARE(controller.entries()->rowCount(), 4);
        QCOMPARE(controller.preflightReport().value("entries").toString(), "2");
        QCOMPARE(controller.preflightReport().value("findingCount").toString(), "1");
        controller.selectRow(1);
        QVERIFY(controller.preflightReport().isEmpty());
        controller.selectRow(2);
        controller.selectRow(0);
        controller.inspectSelection();
        QTRY_COMPARE_WITH_TIMEOUT(completed.count(), 2, 5000);
        QCOMPARE(controller.preflightReport().value("entries").toString(), "2");
        QCOMPARE(controller.preflightReport().value("bytes").toString(), "12");
        QVERIFY(controller.preflightReport().value("unicodeReview").toBool());
    }
    void realContextMenusAndGrid() {
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *controller = window->findChild<BrowserController *>();
        QVERIFY(controller);
        controller->openPath(QStringLiteral(XDZIP_FIXTURES "/normal.zip"));
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy(), 5000);
        QVERIFY(QTest::qWaitForWindowExposed(window));
        controller->selectOnlyRow(2);
        controller->selectRow(3);
        QQuickItem *cell = nullptr;
        QTRY_VERIFY((cell = findVisual(window->contentItem(), "entry-2-0")));
        auto *menu = window->findChild<QObject *>("browserEntryMenu");
        QVERIFY(menu);
        QTest::mouseClick(window, Qt::RightButton, Qt::NoModifier,
                          cell->mapToScene(QPointF(80, cell->height() / 2)).toPoint());
        QTRY_VERIFY(menu->property("visible").toBool());
        QCOMPARE(controller->selectedCount(), 2);
        auto *enterDirectory = window->findChild<QQuickItem *>("contextEnterDirectory");
        auto *properties = window->findChild<QQuickItem *>("contextProperties");
        QVERIFY(enterDirectory);
        QVERIFY(properties);
        QVERIFY(!enterDirectory->isVisible());
        QTRY_COMPARE(enterDirectory->height(), 0);
        QTRY_COMPARE(properties->y(), 0);
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_VERIFY(!menu->property("visible").toBool());
        cell->forceActiveFocus();
        QTest::keyClick(window, Qt::Key_F10, Qt::ShiftModifier);
        QTRY_VERIFY(menu->property("visible").toBool());
        controller->search("notes");
        QTRY_VERIFY(!menu->property("visible").toBool());
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy(), 5000);
        QCOMPARE(controller->selectedCount(), 0);
        auto *viewButton = findVisual(window->contentItem(), "browserViewMenuButton");
        QVERIFY(viewButton);
        QTest::mouseClick(window, Qt::RightButton, Qt::NoModifier,
                          viewButton->mapToScene(QPointF(12, 12)).toPoint());
        auto *viewMenu = window->findChild<QObject *>("browserViewMenu");
        QVERIFY(viewMenu);
        QTRY_VERIFY(viewMenu->property("visible").toBool());
        QTest::keyClick(window, Qt::Key_Down);
        QTest::keyClick(window, Qt::Key_Down);
        QTest::keyClick(window, Qt::Key_Return);
        QTRY_COMPARE(window->property("viewMode").toString(), "grid");
        QQuickItem *card = nullptr;
        QTRY_VERIFY((card = findVisual(window->contentItem(), "grid-entry-0")));
        QTest::mouseClick(window, Qt::RightButton, Qt::NoModifier,
                          card->mapToScene(QPointF(30, 30)).toPoint());
        QTRY_VERIFY(menu->property("visible").toBool());
        QCOMPARE(controller->selectedCount(), 1);
        if (!qEnvironmentVariableIsEmpty("XDZIP_VISUAL_AUDIT")) {
            QVERIFY(QDir().mkpath(qEnvironmentVariable("XDZIP_VISUAL_AUDIT")));
            QVERIFY(visual_audit::save(window, qEnvironmentVariable("XDZIP_VISUAL_AUDIT"),
                                       "command-context-grid"));
        }
        QTest::keyClick(window, Qt::Key_Escape);
    }
    void unicodeSourcePath() {
        QTemporaryDir directory;
        QVERIFY(directory.isValid());
        const auto path = directory.filePath(QString::fromUtf8("旅行 archive.zip"));
        QVERIFY(QFile::copy(QStringLiteral(XDZIP_FIXTURES "/normal.zip"), path));
        BrowserController controller;
        controller.openPath(path);
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        QVERIFY(controller.ready());
        QCOMPARE(controller.total(), 4);
    }
    void closeDuringReadRemainsAsynchronous() {
        BrowserController controller;
        QSignalSpy closed(&controller, &BrowserController::readyToClose);
        controller.openPath(QStringLiteral(XDZIP_FIXTURES "/pages.zip"));
        QVERIFY(!controller.requestClose());
        QTRY_COMPARE_WITH_TIMEOUT(closed.count(), 1, 5000);
        QVERIFY(!controller.busy());
        QVERIFY(controller.requestClose());
    }
    void nativeChooserOwnershipAndCancelRestoreCaller() {
        QQuickWindow mainWindow;
        QQuickWindow caller;
        caller.setTransientParent(&mainWindow);
        mainWindow.show();
        caller.show();
        QVERIFY(QTest::qWaitForWindowExposed(&caller));
        for (const auto mode :
             {QFileDialog::ExistingFile, QFileDialog::ExistingFiles, QFileDialog::Directory}) {
            QFileDialog dialog;
            // Inspect the same ownership policy without depending on OS automation permissions.
            dialog.setOption(QFileDialog::DontUseNativeDialog);
            dialog.setFileMode(mode);
            bool owned = false;
            QTimer::singleShot(0, &dialog, [&] {
                owned = dialog.windowHandle()->transientParent() == &caller;
                dialog.reject();
            });
            QVERIFY(!execOwnedFileDialog(dialog, &caller));
            QVERIFY(owned);
            if (QGuiApplication::platformName() == "cocoa")
                QTRY_COMPARE(QGuiApplication::focusWindow(), &caller);
        }
    }
    void rootListingAndReviewedInteractions() {
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *controller = window->findChild<BrowserController *>();
        QVERIFY(controller);
        controller->openPath(QStringLiteral(XDZIP_FIXTURES "/tree-layout.zip"));
        QTRY_VERIFY(!controller->busy());
        QCOMPARE(controller->total(), 2);
        QCOMPARE(controller->archiveTotal(), 2);
        QCOMPARE(controller->entries()->data(controller->entries()->index(0, 0)).toString(),
                 "GitHub Desktop.app");
        controller->selectOnlyRow(0);
        QVERIFY(!controller->canPreflight());
        controller->copySelection(true);
        QCOMPARE(QGuiApplication::clipboard()->text(), "GitHub Desktop.app/");
        controller->selectAll(false);
        QVERIFY(QTest::qWaitForWindowExposed(window));
        const auto item = [window](const char *name) {
            return findVisual(window->contentItem(), QString::fromLatin1(name));
        };
        const auto click = [window](QQuickItem *target) {
            QTest::mouseClick(
                window, Qt::LeftButton, Qt::NoModifier,
                target->mapToScene(QPointF(target->width() / 2, target->height() / 2)).toPoint());
        };
        auto *caption = item("archiveCaption");
        auto *title = item("filePaneTitle");
        QCOMPARE(caption->property("text").toString(), QString::fromUtf8("压缩包"));
        QTRY_VERIFY(qAbs(caption->mapToScene(QPointF(0, caption->height() / 2)).y() -
                         title->mapToScene(QPointF(0, title->height() / 2)).y()) < .5);
        auto *cell = item("entry-0-0");
        QTest::mouseMove(window, cell->mapToScene(QPointF(100, cell->height() / 2)).toPoint());
        for (int column = 0; column < 5; ++column) {
            auto *band = findVisual(window->contentItem(), QString("entryBand-0-%1").arg(column));
            QVERIFY(band);
            auto *separator =
                findVisual(window->contentItem(), QString("entrySeparator-0-%1").arg(column));
            QVERIFY(separator);
            // The upper band's gap to its line equals the next band's gap after that line.
            QCOMPARE(separator->y() - (band->y() + band->height()), band->y());
            QTRY_COMPARE(band->property("color").value<QColor>(), QColor("#EDF2FF"));
        }
        const auto captures = qEnvironmentVariable("XDZIP_VISUAL_AUDIT");
        if (!captures.isEmpty())
            QVERIFY(QDir().mkpath(captures));
        if (!captures.isEmpty())
            QVERIFY(visual_audit::save(window, captures, "root-row-hover"));
        auto *tree = item("directoryTree");
        QVERIFY(tree);
        QVERIFY(QMetaObject::invokeMethod(tree, "expandRecursively", Q_ARG(int, 0), Q_ARG(int, 2)));
        QQuickItem *guide = nullptr, *disclosure = nullptr;
        QTRY_VERIFY((guide = item("directoryParentGuide-d2")));
        QTRY_VERIFY((disclosure = item("directoryDisclosure-d1")));
        QVERIFY(qAbs(guide->mapToScene(QPointF(guide->width() / 2, 0)).x() -
                     disclosure->mapToScene(QPointF(disclosure->width() / 2, 0)).x()) < .01);
        QQmlComponent scaleSettings(&engine);
        scaleSettings.setData(
            "import QtQuick\nimport \"qrc:/xdzip/ui\" as Ui\n"
            "QtObject { property real scale: 1; onScaleChanged: Ui.Theme.fontScale = scale }",
            QUrl());
        std::unique_ptr<QObject> scaling(scaleSettings.create());
        QVERIFY(scaling);
        scaling->setProperty("scale", 2);
        QTRY_VERIFY(qAbs(caption->mapToScene(QPointF(0, caption->height() / 2)).y() -
                         title->mapToScene(QPointF(0, title->height() / 2)).y()) < .5);
        QTRY_VERIFY(qAbs(guide->mapToScene(QPointF(guide->width() / 2, 0)).x() -
                         disclosure->mapToScene(QPointF(disclosure->width() / 2, 0)).x()) < .01);
        scaling->setProperty("scale", 1);
        QTest::qWait(30);
        const auto closePosition = item("inspectorClose")->mapToScene(QPointF());
        click(item("inspectorClose"));
        QQuickItem *toggle = nullptr;
        QTRY_VERIFY((toggle = item("inspectorToggle")) && toggle->isVisible());
        // Visibility updates before the RowLayout's next polish pass.
        // Keep the exact size contract, but inspect the settled layout.
        QTRY_COMPARE(toggle->width(), 28.0);
        QTRY_COMPARE(toggle->height(), 28.0);
        QTRY_VERIFY2(qAbs(toggle->mapToScene(QPointF()).x() - closePosition.x()) < .01,
                     qPrintable(QString("show=%1 close=%2 window=%3")
                                    .arg(toggle->mapToScene(QPointF()).x())
                                    .arg(closePosition.x())
                                    .arg(window->width())));
        QCOMPARE(toggle->mapToScene(QPointF()).y(), closePosition.y());
        click(item("browserGridButton"));
        QQuickItem *card = nullptr, *check = nullptr;
        QTRY_VERIFY((card = item("grid-entry-0")));
        QTRY_VERIFY((check = item("gridSelection-0")));
        auto *grid = item("archiveGrid");
        QVERIFY(qAbs(grid->x() - 16) < .01);
        QTest::mouseMove(window, item("archiveOpen")->mapToScene(QPointF(10, 10)).toPoint());
        QTRY_VERIFY(!check->isVisible());
        QTest::mouseMove(window, card->mapToScene(QPointF(40, 40)).toPoint());
        QTRY_VERIFY(check->isVisible());
        click(check);
        QTRY_COMPARE(controller->selectedCount(), 1);
        QTest::mouseMove(window, item("archiveOpen")->mapToScene(QPointF(10, 10)).toPoint());
        QTRY_VERIFY(check->isVisible());
        if (!captures.isEmpty())
            QVERIFY(visual_audit::save(window, captures, "root-grid-selected"));
        window->setProperty("darkMode", true);
        QTRY_COMPARE(item("actionSplitDivider")->property("color").value<QColor>(),
                     QColor("#363D4B"));
        window->setProperty("darkMode", false);
        click(item("browserViewMenuButton"));
        auto *more = item("archiveMore");
        QTest::mouseMove(window, more->mapToScene(QPointF(10, 10)).toPoint());
        const QImage before = window->grabWindow();
        click(more);
        auto *menu = window->findChild<QObject *>("archiveMoreMenu");
        QVERIFY(menu);
        QTRY_VERIFY(menu->property("visible").toBool());
        auto *content = menu->property("contentItem").value<QQuickItem *>();
        auto *background = menu->property("background").value<QQuickItem *>();
        QVERIFY(background);
        auto *shadow = findVisual(background, "menuShadowLayer-0");
        QVERIFY(shadow && shadow->x() < 0 && shadow->width() > background->width());
        const auto top =
            content->mapToScene(QPointF()).y() - menu->property("topPadding").toDouble();
        QVERIFY(qAbs(top - more->mapToScene(QPointF(0, more->height())).y() - 4) < .1);
        const auto center = background->mapToScene(QPointF(-4, background->height() / 2));
        // Canvas paints its blurred silhouette on the next frame.
        QTest::qWait(50);
        const QImage after = window->grabWindow();
        const auto pixel = QPoint(qRound(center.x() * after.width() / window->width()),
                                  qRound(center.y() * after.height() / window->height()));
        QVERIFY(after.valid(pixel));
        QVERIFY(after.pixelColor(pixel) != before.pixelColor(pixel));
        if (!captures.isEmpty())
            QVERIFY(visual_audit::save(window, captures, "root-menu-shadow"));
        QTest::keyClick(window, Qt::Key_Escape);
        controller->enterDirectory(0);
        QTRY_VERIFY(!controller->busy());
        QCOMPARE(controller->entries()->data(controller->entries()->index(0, 0)).toString(),
                 "Contents");
        controller->showRoot();
        QTRY_VERIFY(!controller->busy());
        controller->search("file.txt");
        QTRY_VERIFY(!controller->busy());
        QCOMPARE(controller->total(), 1);
        QVERIFY(controller->entries()
                    ->data(controller->entries()->index(0, 0))
                    .toString()
                    .contains("GitHub Desktop.app/Contents/Resources/"));
        controller->requestClose();
        QTRY_VERIFY(!controller->runtimeAvailable());
    }
    void reviewedMainControlsAndMenus() {
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *controller = window->findChild<BrowserController *>();
        controller->openPath(QStringLiteral(XDZIP_FIXTURES "/visual-main.zip"));
        QTRY_VERIFY(controller->ready() && !controller->busy());
        QVERIFY(QTest::qWaitForWindowExposed(window));
        QSignalSpy initialFrame(window, &QQuickWindow::afterAnimating);
        window->requestUpdate();
        QTRY_VERIFY(initialFrame.count() > 0);
        const auto item = [window](const char *name) {
            return findVisual(window->contentItem(), QString::fromLatin1(name));
        };
        const auto click = [window](QQuickItem *target) {
            const auto point =
                target->mapToScene(QPointF(target->width() / 2, target->height() / 2)).toPoint();
            // Move before pressing, as a real pointer does. This lets hovered
            // controls dismiss their tooltips before the next hit test.
            QTest::mouseMove(window, point);
            QCoreApplication::processEvents();
            QTest::mouseClick(window, Qt::LeftButton, Qt::NoModifier, point);
        };
        for (const auto *name : {"archiveNewTab", "archiveSettings", "browserViewMenuButton",
                                 "browserGridButton", "browserSortMenuButton", "inspectorClose"}) {
            auto *button = item(name);
            QVERIFY(button);
            auto *icon = findVisual(button, "actionIcon");
            QVERIFY(icon);
            QTRY_VERIFY(
                qAbs(icon->mapToItem(button, QPointF(icon->width() / 2, icon->height() / 2)).x() -
                     button->width() / 2) < .5);
            QVERIFY(
                qAbs(icon->mapToItem(button, QPointF(icon->width() / 2, icon->height() / 2)).y() -
                     button->height() / 2) < .5);
        }
        for (const auto *name : {"archiveOpen", "archiveNew", "archiveTest"}) {
            auto *button = item(name);
            auto *icon = findVisual(button, "actionIcon");
            auto *label = findVisual(button, "actionLabel");
            QVERIFY(qAbs(label->mapToItem(button, QPointF()).x() -
                         icon->mapToItem(button, QPointF(icon->width(), 0)).x() - 6) < .01);
        }
        auto *close = item("inspectorClose");
        // Native pointer delivery may coalesce a repeated position left by
        // the previous window. Enter from outside this control after layout.
        QTest::mouseMove(window, QPoint(0, 0));
        QTest::mouseMove(
            window, close->mapToScene(QPointF(close->width() / 2, close->height() / 2)).toPoint());
        QTRY_VERIFY(close->property("hovered").toBool());
        QVERIFY(findVisual(close, "actionBackground")->property("radius").toDouble() > 0);
        auto *extract = item("archiveExtract");
        QCOMPARE(findVisual(extract, "actionBackground")->property("color").value<QColor>(),
                 QColor("#1956ff"));
        for (const auto *name :
             {"browserViewMenuButton", "browserGridButton", "browserSortMenuButton"}) {
            QCOMPARE(item(name)->width(), 28.0);
            QCOMPARE(item(name)->height(), 28.0);
        }
        auto *grid = item("browserGridButton");
        const auto fixedGrid = grid->mapToScene(QPointF());
        controller->enterDirectory(0);
        QTRY_VERIFY(!controller->busy());
        QTRY_COMPARE(grid->mapToScene(QPointF()), fixedGrid);
        controller->showRoot();
        QTRY_VERIFY(!controller->busy());
        click(grid);
        QTRY_VERIFY(!item("fileTableHeader")->isVisible());
        const auto captures = qEnvironmentVariable("XDZIP_VISUAL_AUDIT");
        if (!captures.isEmpty())
            QVERIFY(QDir().mkpath(captures));
        if (!captures.isEmpty())
            QVERIFY(visual_audit::save(window, captures, "review-grid"));
        QSignalSpy frames(window, &QQuickWindow::afterAnimating);
        click(item("browserViewMenuButton"));
        // Visibility changes synchronously; the layout and hit-test tree update on the next frame.
        QTRY_VERIFY(item("fileTableHeader")->isVisible());
        frames.clear();
        window->requestUpdate();
        QTRY_VERIFY(frames.count() > 0);
        for (int column : {3, 4}) {
            auto *header = item(column == 3 ? "sort-column-3" : "sort-column-4");
            QVERIFY(header && header->isVisible() && header->width() > 0);
            const auto point =
                header->mapToScene(QPointF(header->width() / 2, header->height() / 2));
            auto *bar = item("fileTableHeader");
            QVERIFY(bar->contains(bar->mapFromScene(point)));
            click(header);
            QTRY_COMPARE(controller->sortOrder(), column == 3 ? 9 : 11);
            QTRY_VERIFY(!controller->busy());
            click(header);
            QTRY_COMPARE(controller->sortOrder(), column == 3 ? 10 : 12);
            QTRY_VERIFY(!controller->busy());
        }
        for (const auto &[buttonName, menuName] :
             {std::pair{"browserSortMenuButton", "browserSortMenu"},
              std::pair{"browserTasks", "browserTaskMenu"}}) {
            auto *button = item(buttonName);
            click(button);
            auto *menu = window->findChild<QObject *>(QString::fromLatin1(menuName));
            QVERIFY(menu);
            QTRY_VERIFY(menu->property("visible").toBool());
            // Popup coordinates are relative to its QML parent, which differs for the two menus.
            auto *popup = menu->property("contentItem").value<QQuickItem *>();
            QVERIFY(popup);
            const auto top =
                popup->mapToScene(QPointF()).y() - menu->property("topPadding").toDouble();
            const auto bottom = top + menu->property("height").toDouble();
            const auto buttonTop = button->mapToScene(QPointF()).y();
            if (QString::fromLatin1(buttonName) == "browserTasks")
                QVERIFY(bottom <= buttonTop - 3);
            else
                QVERIFY(top >= buttonTop + button->height() + 3);
            if (!captures.isEmpty())
                QVERIFY(visual_audit::save(window, captures, QString::fromLatin1(buttonName)));
            QTest::keyClick(window, Qt::Key_Escape);
        }
        auto *rootNode = item("directoryRoot");
        QTest::mouseClick(window, Qt::RightButton, Qt::NoModifier,
                          rootNode->mapToScene(QPointF(40, rootNode->height() / 2)).toPoint());
        auto *directoryMenu = window->findChild<QObject *>("browserDirectoryMenu");
        QTRY_VERIFY(directoryMenu->property("visible").toBool());
        auto *menuContent = directoryMenu->property("contentItem").value<QQuickItem *>();
        auto *separator = findVisual(menuContent, "commandSeparator");
        QVERIFY(separator);
        QVERIFY(separator->width() > 200);
        if (!captures.isEmpty())
            QVERIFY(visual_audit::save(window, captures, "review-directory-menu"));
        QTest::keyClick(window, Qt::Key_Escape);
    }
    void mainWindowFidelityAndSessionClose() {
        QTemporaryDir directory;
        QVERIFY(directory.isValid());
        const auto archivePath = directory.filePath(QString::fromUtf8("北境旅行.zip"));
        QVERIFY(QFile::copy(QStringLiteral(XDZIP_FIXTURES "/visual-main.zip"), archivePath));
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        window->raise();
        window->requestActivate();
        QTRY_VERIFY(window->isActive());
        auto *controller = window->findChild<BrowserController *>();
        QVERIFY(controller);
        controller->openPath(archivePath);
        QTRY_VERIFY_WITH_TIMEOUT(controller->ready() && !controller->busy(), 5000);
        QCOMPARE(controller->total(), 9);
        controller->selectOnlyRow(3);
        QCOMPARE(controller->detail().value("type").toString(), "PNG");
        QCOMPARE(controller->detail().value("modified").toString(), QStringLiteral("2026/10/01"));
        QVERIFY(controller->selectedBytes().toULongLong() > 0);
        QVERIFY(QTest::qWaitForWindowExposed(window));
        const auto captures = qEnvironmentVariable("XDZIP_VISUAL_AUDIT");
        auto *more = findVisual(window->contentItem(), "archiveMore");
        QVERIFY(more);
        QTest::mouseClick(
            window, Qt::LeftButton, Qt::NoModifier,
            more->mapToScene(QPointF(more->width() / 2, more->height() / 2)).toPoint());
        auto *dropdown = window->findChild<QObject *>("archiveMoreMenu");
        QVERIFY(dropdown);
        QTRY_VERIFY(dropdown->property("visible").toBool());
        QCOMPARE(dropdown->property("width").toDouble(), 240.0);
        QCOMPARE(dropdown->property("font").value<QFont>().pixelSize(), 13);
        if (!captures.isEmpty()) {
            QVERIFY(QDir().mkpath(captures));
            QVERIFY(visual_audit::save(window, captures, "main-dropdown"));
        }
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_VERIFY(!dropdown->property("visible").toBool());
        QQuickItem *entry = nullptr;
        QTRY_VERIFY((entry = findVisual(window->contentItem(), "entry-3-0")));
        QTest::mouseClick(window, Qt::RightButton, Qt::NoModifier,
                          entry->mapToScene(QPointF(100, entry->height() / 2)).toPoint());
        auto *context = window->findChild<QObject *>("browserEntryMenu");
        QVERIFY(context);
        QTRY_VERIFY(context->property("visible").toBool());
        QCOMPARE(context->property("width").toDouble(), 240.0);
        auto *properties = window->findChild<QQuickItem *>("contextProperties");
        QVERIFY(properties);
        QCOMPARE(properties->property("font").value<QFont>().pixelSize(), 13);
        if (!captures.isEmpty())
            QVERIFY(visual_audit::save(window, captures, "main-context"));
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_VERIFY(!context->property("visible").toBool());
        if (!captures.isEmpty()) {
            QVERIFY(QDir().mkpath(captures));
            QVERIFY(visual_audit::save(window, captures, "main-light"));
            window->setProperty("darkMode", true);
            QVERIFY(visual_audit::save(window, captures, "main-dark"));
            window->setProperty("darkMode", false);
            // Layout evidence only: native Cocoa window buttons are not drawn by Qt.
            window->setProperty("integratedTitlebar", true);
            QVERIFY(visual_audit::save(window, captures, "main-integrated-layout"));
        }
        auto *all = findVisual(window->contentItem(), "selectPage");
        QVERIFY(all);
        const auto selectPoint =
            all->mapToScene(QPointF(all->width() / 2, all->height() / 2)).toPoint();
        QTest::mouseClick(window, Qt::LeftButton, Qt::NoModifier, selectPoint);
        QTRY_COMPARE(controller->selectedCount(), 9);
        QTest::mouseClick(window, Qt::LeftButton, Qt::NoModifier, selectPoint);
        QTRY_COMPARE(controller->selectedCount(), 0);
        const auto session = controller->sessionId();
        auto *close = findVisual(window->contentItem(), "archiveTabClose");
        QVERIFY(close);
        QTest::mouseClick(
            window, Qt::LeftButton, Qt::NoModifier,
            close->mapToScene(QPointF(close->width() / 2, close->height() / 2)).toPoint());
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy() && !controller->ready(), 5000);
        QVERIFY(controller->sessionId().isEmpty());
        QVERIFY(controller->runtimeAvailable());
        QCOMPARE(controller->total(), 0);
        QCOMPARE(controller->selectedBytes(), "0");
        QCOMPARE(controller->entries()->rowCount(), 0);
        QVERIFY(window->isVisible());
        controller->openPath(archivePath);
        QTRY_VERIFY_WITH_TIMEOUT(controller->ready() && !controller->busy(), 5000);
        QVERIFY(controller->sessionId() != session);
        QCOMPARE(controller->total(), 9);
        // Restoring the design's columns must not hide the existing review state.
        controller->openPath(QStringLiteral(XDZIP_FIXTURES "/unsafe-names.zip"));
        QTRY_VERIFY_WITH_TIMEOUT(controller->ready() && !controller->busy(), 5000);
        controller->selectOnlyRow(0);
        QVERIFY(controller->detail().value("attention").toBool());
        QVERIFY(
            controller->detail().value("status").toString().contains(QString::fromUtf8("需检查")));
        QVERIFY(controller->entries()
                    ->data(controller->entries()->index(0, 0), Qt::UserRole + 5)
                    .toBool());
    }
    void renderRealArchive() {
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *controller = window->findChild<BrowserController *>();
        QVERIFY(controller);
        QCOMPARE(window->property("accentTextColor").value<QColor>(), QColor(Qt::white));
        controller->openPath(QStringLiteral(XDZIP_FIXTURES "/normal.zip"));
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy(), 5000);
        QCOMPARE(controller->total(), 4);
        QCOMPARE(controller->entries()
                     ->data(controller->entries()->index(1, 0), Qt::DisplayRole)
                     .toString(),
                 QString::fromUtf8("notes.txt"));
        QVERIFY(QTest::qWaitForWindowExposed(window));
        QQuickItem *cell = nullptr;
        QTRY_VERIFY((cell = findVisual(window->contentItem(), "entry-1-0")));
        const auto point = cell->mapToScene(QPointF(120, 24)).toPoint();
        QTest::mouseClick(window, Qt::LeftButton, Qt::NoModifier, point);
        QTRY_COMPARE(controller->selectedCount(), 1);
        QCOMPARE(controller->detail().value("id").toString(), "3");
        auto *selection = findVisual(cell, "entrySelection");
        QVERIFY(selection);
        const auto checkPoint =
            selection->mapToScene(QPointF(selection->width() / 2, selection->height() / 2))
                .toPoint();
        QTest::mouseClick(window, Qt::LeftButton, Qt::NoModifier, checkPoint);
        QTRY_COMPARE(controller->selectedCount(), 0);
        QTest::mouseClick(window, Qt::LeftButton, Qt::NoModifier, checkPoint);
        QTRY_COMPARE(controller->selectedCount(), 1);
        QTest::qWait(30);
        // Direct CI invocations also run from the source root; never save there.
        const QDir captures(QCoreApplication::applicationDirPath());
        QVERIFY(window->grabWindow().save(captures.filePath("zip-browser.png")));
        window->setProperty("darkMode", true);
        QTest::qWait(30);
        QVERIFY(window->grabWindow().save(captures.filePath("zip-browser-dark.png")));
        window->setProperty("darkMode", false);
        controller->inspectSelection();
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy(), 5000);
        auto *reportWindow = window->findChild<QQuickWindow *>("preflightWindow");
        QVERIFY(reportWindow);
        QTRY_VERIFY(reportWindow->isVisible());
        QCOMPARE(reportWindow->transientParent(), window);
        QCOMPARE(reportWindow->modality(), Qt::NonModal);
        QVERIFY(reportWindow->flags().testFlag(Qt::WindowTitleHint));
        QVERIFY(QTest::qWaitForWindowExposed(reportWindow));
        QVERIFY(reportWindow->grabWindow().save(captures.filePath("zip-preflight.png")));
        if (!qEnvironmentVariableIsEmpty("XDZIP_VISUAL_AUDIT")) {
            const auto directory = qEnvironmentVariable("XDZIP_VISUAL_AUDIT");
            QVERIFY(QDir().mkpath(directory));
            QVERIFY(visual_audit::save(reportWindow, directory, "preflight-light"));
        }
        reportWindow->close();
        QVERIFY(window->isVisible());
        QCOMPARE(controller->selectedCount(), 1);
        QVERIFY(visual_audit::capture(engine, window));
        // Hiding/reopening details must preserve the actual archive selection.
        window->resize(1024, 622);
        auto *inspector = findVisual(window->contentItem(), "archiveInspector");
        auto *toggle = findVisual(window->contentItem(), "inspectorToggle");
        QVERIFY(inspector && toggle);
        QTRY_VERIFY(inspector->isVisible());
        auto *closeDetails = findVisual(inspector, "inspectorClose");
        QVERIFY(closeDetails);
        QTest::mouseClick(
            window, Qt::LeftButton, Qt::NoModifier,
            closeDetails->mapToScene(QPointF(closeDetails->width() / 2, closeDetails->height() / 2))
                .toPoint());
        QTRY_VERIFY(!inspector->isVisible());
        QVERIFY(QMetaObject::invokeMethod(toggle, "clicked"));
        QTRY_VERIFY(inspector->isVisible());
        window->resize(832, 544);
        QTRY_VERIFY(!inspector->isVisible());
        QVERIFY(QMetaObject::invokeMethod(toggle, "clicked"));
        auto *detailsWindow = window->findChild<QQuickWindow *>("inspectorWindow");
        QVERIFY(detailsWindow);
        QTRY_VERIFY(detailsWindow->isVisible());
        QCOMPARE(detailsWindow->transientParent(), window);
        QCOMPARE(detailsWindow->modality(), Qt::NonModal);
        QVERIFY(!detailsWindow->flags().testFlag(Qt::FramelessWindowHint));
        QCOMPARE(window->width(), 832);
        QCOMPARE(controller->selectedCount(), 1);
        if (!qEnvironmentVariableIsEmpty("XDZIP_VISUAL_AUDIT"))
            QVERIFY(visual_audit::save(detailsWindow, qEnvironmentVariable("XDZIP_VISUAL_AUDIT"),
                                       "inspector-independent"));
        detailsWindow->close();
    }
};
int main(int argc, char **argv) {
    QApplication application(argc, argv);
    if (application.arguments().value(1) == "--instance-client") {
        SingleInstance instance;
        return instance.start({application.arguments().value(3)},
                              application.arguments().value(2)) == SingleInstance::Forwarded
                   ? 0
                   : 1;
    }
    if (application.arguments().value(1) == "--instance-primary") {
        SingleInstance instance;
        if (instance.start({}, application.arguments().value(2)) != SingleInstance::Primary)
            return 2;
        QTextStream(stdout) << "PRIMARY\n" << Qt::flush;
        return application.exec();
    }
    BrowserTest test;
    return QTest::qExec(&test, argc, argv);
}
#include "browser_test.moc"
