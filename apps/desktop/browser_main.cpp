#include "AppQuitGuard.h"
#include "ApplicationSessions.h"
#include "AssociationProbe.h"
#include "BrowserController.h"
#include "BuildInfo.h"
#include "DesktopIntegration.h"
#include "DesktopProbe.h"
#include "FileOpenRouter.h"
#include "FocusDismissPolicy.h"
#include "InstalledRuntimeGuard.h"
#include "SingleInstance.h"
#include "UiFont.h"
#include "UiPreferences.h"
#include "UiText.h"
#include "UpdateController.h"
#include "WindowActivation.h"
#include "WindowHoverPolicy.h"
#include <QApplication>
#include <QEvent>
#include <QIcon>
#include <QMessageBox>
#include <QPointer>
#include <QProcess>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include <QQuickStyle>
#include <QQuickWindow>
#include <QResource>
#include <QSaveFile>
#include <QSslSocket>
#include <QTimer>
#include <memory>
#ifdef XDZIP_NATIVE_UPDATE_FIXTURE
#include "tests/native_update_fixture.h"
#endif
#ifdef XDZIP_NATIVE_MAC_UPDATE_FIXTURE
#include "tests/mac_product_update_fixture.h"
#endif
#ifdef Q_OS_WIN
#include <qt_windows.h>
#endif
#ifdef Q_OS_MACOS
#include "MacUpdateProvider.h"
#endif

#ifdef Q_OS_MACOS
void integrateMacTitlebar(QQuickWindow *window);
bool verifyMacTitlebar(QQuickWindow *window);
bool verifyMacDesktopMenus();
void showMacWindowAfterLaunch(QQuickWindow *window);
#endif

int main(int argc, char *argv[]) {
    if (BuildInfo::printVersionIfRequested(argc, argv))
        return 0;
    QApplication application(argc, argv);
    application.setApplicationName(QStringLiteral("xdzip"));
    Q_INIT_RESOURCE(ui);
    WindowActivation::install(&application);
    InstalledRuntimeGuard installedRuntime;
    if (!installedRuntime.acquire())
        return 11;
    FileOpenRouter fileOpens;
    application.installEventFilter(&fileOpens);
    const auto args = application.arguments();
    const bool literalPaths = args.contains(QStringLiteral("--"));
    const auto probeArgument = literalPaths ? -1 : args.indexOf(QStringLiteral("--desktop-probe"));
    const auto updateProbeArgument =
        literalPaths ? -1 : args.indexOf(QStringLiteral("--update-probe"));
    const bool smokeTest = !literalPaths && args.contains(QStringLiteral("--smoke-test"));
    QStringList startupPaths;
    if (!smokeTest && probeArgument < 0 && updateProbeArgument < 0) {
        const auto start = literalPaths ? args.indexOf(QStringLiteral("--")) + 1 : 1;
        for (int i = start; i < args.size(); ++i) {
            if (literalPaths || !args.at(i).startsWith(QLatin1Char('-')))
                startupPaths.append(QFileInfo(args.at(i)).absoluteFilePath());
        }
    }
    SingleInstance instance;
    const bool interactive = !smokeTest && probeArgument < 0 && updateProbeArgument < 0 &&
                             qEnvironmentVariableIsEmpty("XDZIP_ASSOCIATION_PROBE");
    if (interactive) {
        const auto election = instance.start(startupPaths);
        if (election == SingleInstance::Forwarded)
            return 0;
        if (election == SingleInstance::Failed) {
            if (!initializeUiText())
                return 5;
            QMessageBox::critical(nullptr, QStringLiteral("xdzip"), qtTrId("sessions.unavailable"));
            return 12;
        }
    }
    for (const auto &path : startupPaths)
        fileOpens.enqueue(path);
    FocusDismissPolicy::install(&application);
    WindowHoverPolicy::install(&application);
    application.setApplicationName(QStringLiteral("xdzip"));
    application.setApplicationVersion(BuildInfo::version());
    application.setDesktopFileName(QStringLiteral("xdzip"));
    QQuickStyle::setStyle(QStringLiteral("Basic"));
    if (!initializeUiText())
        return 5;
    if (!installUiFont())
        return 5;
    if (updateProbeArgument >= 0) {
        if (updateProbeArgument + 1 >= args.size() ||
            qEnvironmentVariableIsEmpty("XDZIP_SETTINGS_PATH"))
            return 8;
        UpdateController updates;
        const auto report = args.at(updateProbeArgument + 1);
        bool done = false;
        const auto finish = [&](bool timeout) {
            if (done || (!timeout && (updates.state() == "idle" || updates.busy())))
                return;
            done = true;
            const QJsonObject result{
                {"schema_version", 1},
                {"state", updates.state()},
                {"error", timeout ? QStringLiteral("probe_timeout") : updates.errorCode()},
                {"candidate", updates.candidateVersion()},
                {"trusted", updates.trusted()},
                {"ssl_supported", QSslSocket::supportsSsl()},
                {"ssl_backend", QSslSocket::activeBackend()},
                {"version", BuildInfo::version()},
                {"build", BuildInfo::buildNumber()}};
            QSaveFile file(report);
            const auto bytes = QJsonDocument(result).toJson();
            const bool written = file.open(QIODevice::WriteOnly) &&
                                 file.write(bytes) == bytes.size() && file.commit();
            application.exit(written && !timeout && updates.state() != "error" ? 0 : 13);
        };
        QObject::connect(&updates, &UpdateController::changed, &application,
                         [&] { finish(false); });
        QTimer::singleShot(60000, &application, [&] { finish(true); });
        QTimer::singleShot(0, &updates, &UpdateController::check);
        return application.exec();
    }
#ifdef Q_OS_MACOS
    application.setWindowIcon(QIcon(QStringLiteral(":/xdzip/assets/app-icon-macos.png")));
#else
    application.setWindowIcon(QIcon(QStringLiteral(":/xdzip/assets/app-icon.png")));
#endif
    QQmlApplicationEngine engine;
#ifdef XDZIP_NATIVE_MAC_UPDATE_FIXTURE
    UpdateController updates(nullptr, macProductUpdateFixtureTransport(&application));
#elif defined(XDZIP_NATIVE_UPDATE_FIXTURE)
#ifdef Q_OS_WIN
    UpdateController updates(nullptr, nativeUpdateFixtureTransport(&application), {}, {},
                             [] { return true; });
#else
    UpdateController updates(nullptr, nativeUpdateFixtureTransport(&application));
#endif
#else
    UpdateController updates;
#endif
    engine.rootContext()->setContextProperty(QStringLiteral("UpdateService"), &updates);
    engine.setInitialProperties({{QStringLiteral("visible"), false}});
    QObject::connect(
        &engine, &QQmlApplicationEngine::objectCreationFailed, &application,
        [] { QCoreApplication::exit(1); }, Qt::QueuedConnection);
    engine.load(QUrl(QStringLiteral("qrc:/xdzip/ui/Browser.qml")));
    if (engine.rootObjects().isEmpty())
        return 1;
#ifdef Q_OS_MACOS
    integrateMacTitlebar(qobject_cast<QQuickWindow *>(engine.rootObjects().first()));
#endif
    auto *controller = engine.rootObjects().first()->findChild<BrowserController *>();
    if (!controller)
        return 1;
    auto *mainWindow = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
    if (!interactive)
        fileOpens.attach(controller, mainWindow);
    installAssociationProbe(controller, mainWindow);
    AppQuitGuard guard;
    guard.window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
    guard.controller = controller;
    application.installEventFilter(&guard);
    QObject::connect(&updates, &UpdateController::exitBarrierReached, &application, [&] {
        guard.quitHandler = {};
        guard.controller = nullptr;
        for (auto *window : QGuiApplication::allWindows())
            if (window->property("sessionManager").isValid())
                window->setProperty("allowManagedClose", true);
        QApplication::setQuitOnLastWindowClosed(true);
    });
    QObject::connect(&updates, &UpdateController::quitForUpdate, &application, [&] {
        guard.quitHandler = {};
        guard.controller = nullptr;
        application.quit();
    });
    if (!interactive)
        QObject::connect(controller, &BrowserController::readyToClose, &application,
                         &QApplication::quit, Qt::QueuedConnection);
    auto *settings = engine.singletonInstance<UiPreferences *>("Xdzip.Settings", "AppSettings");
    DesktopIntegration desktop(mainWindow, controller, settings, interactive);
    std::unique_ptr<ApplicationSessions> sessions;
    if (interactive) {
        QApplication::setQuitOnLastWindowClosed(false);
        sessions = std::make_unique<ApplicationSessions>(&engine, settings);
        sessions->adopt(mainWindow);
        sessions->setDesktop(&desktop);
        updates.setSessions(sessions.get());
        QObject::connect(sessions.get(), &ApplicationSessions::closing, &instance,
                         &SingleInstance::stopAccepting);
        guard.quitHandler = [&] {
            if (!sessions->isClosing())
                desktop.requestQuit();
        };
        QObject::connect(
            sessions.get(), &ApplicationSessions::readyToQuit, &application,
            [&] {
                const auto quit = [&] {
                    guard.quitHandler = {};
                    guard.controller = nullptr;
                    application.quit();
                };
#ifdef Q_OS_MACOS
                if (deferMacUpdateQuit(quit))
                    return;
#endif
                quit();
            },
            Qt::QueuedConnection);
        fileOpens.attach([&](const QString &path) { sessions->openPath(path); });
        instance.attach([&](const QStringList &paths) { sessions->open(paths); });
    }
    std::unique_ptr<DesktopProbe> probe;
#ifdef XDZIP_NATIVE_MAC_UPDATE_FIXTURE
    installMacProductUpdateFixture(&updates, sessions.get(), settings, mainWindow);
#elif defined(XDZIP_NATIVE_UPDATE_FIXTURE)
    installNativeUpdateFixture(&updates, sessions.get(), settings, mainWindow);
#endif
    if (probeArgument >= 0) {
        if (probeArgument + 1 >= application.arguments().size() ||
            qEnvironmentVariableIsEmpty("XDZIP_SETTINGS_PATH"))
            return 8;
        const auto fixture = application.arguments().at(probeArgument + 1);
        if (!QFileInfo(fixture).isFile())
            return 8;
        probe = std::make_unique<DesktopProbe>(
            controller, qobject_cast<QQuickWindow *>(engine.rootObjects().first()), fixture,
            &application);
    }
    if (smokeTest) {
        if (application.windowIcon().isNull() || application.windowIcon().pixmap(64, 64).isNull())
            return 4;
#ifdef Q_OS_MACOS
        if (QGuiApplication::platformName() == QStringLiteral("cocoa") &&
            !engine.rootObjects().first()->property("integratedTitlebar").toBool())
            return 3;
#endif
        // Exercise startup, real receiver wakeup and shutdown, not just QML construction.
        QTimer::singleShot(500, &application, [&application, &engine] {
#ifdef Q_OS_WIN
            if (QGuiApplication::platformName() == QStringLiteral("windows")) {
                QVariantMap nativeFrame;
                auto *frame =
                    engine.rootObjects().first()->findChild<QObject *>("windowsNativeFrame");
                if (!frame ||
                    !QMetaObject::invokeMethod(frame, "diagnostics",
                                               Q_RETURN_ARG(QVariantMap, nativeFrame)) ||
                    !nativeFrame.value("attached").toBool() ||
                    !nativeFrame.value("resizeStyle").toBool() ||
                    nativeFrame.value("topInsetPixels").toInt() != 1) {
                    QCoreApplication::exit(7);
                    return;
                }
            }
#endif
#ifdef Q_OS_MACOS
            if (!verifyMacDesktopMenus() ||
                !verifyMacTitlebar(qobject_cast<QQuickWindow *>(engine.rootObjects().first()))) {
                QCoreApplication::exit(6);
                return;
            }
#endif
#ifdef Q_OS_MACOS
            if (QGuiApplication::platformName() == QStringLiteral("cocoa")) {
                auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
                window->resize(window->width() + 8, window->height() + 8);
                QTimer::singleShot(400, &application, [&application, window] {
                    if (!verifyMacTitlebar(window)) {
                        QCoreApplication::exit(6);
                        return;
                    }
                    window->requestActivate();
                    QTimer::singleShot(400, &application, [&application, window] {
                        if (!verifyMacTitlebar(window)) {
                            QCoreApplication::exit(6);
                            return;
                        }
                        application.quit();
                    });
                });
                return;
            }
#endif
            application.quit();
        });
        QTimer::singleShot(10000, &application, [] { QCoreApplication::exit(2); });
    }
    // Allow queued platform launch events to join the same startup intent path.
    // Do not wait for archive I/O: large files show the cancellable loading card.
#ifdef Q_OS_MACOS
    showMacWindowAfterLaunch(mainWindow);
#else
    QTimer::singleShot(0, mainWindow, &QWindow::show);
#endif
    const auto result = application.exec();
    instance.release(); // All joins are complete; a restart must elect a new primary.
    // Window close completes the existing task decision and runtime join barrier first.
    // Relaunch only the application, never replay diagnostic flags or a previously opened file.
    if (result == 0 && ((sessions && sessions->restartRequested()) ||
                        (!sessions && !engine.rootObjects().isEmpty() &&
                         engine.rootObjects().first()->property("restartAfterClose").toBool()))) {
        qint64 restartedPid = 0;
        if (!QProcess::startDetached(QCoreApplication::applicationFilePath(), QStringList{},
                                     QString{}, &restartedPid))
            return 6;
#ifdef Q_OS_WIN
        if (restartedPid > 0)
            AllowSetForegroundWindow(static_cast<DWORD>(restartedPid));
#endif
    }
    return result;
}
