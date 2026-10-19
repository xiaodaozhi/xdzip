#include "AppQuitGuard.h"
#include "AssociationProbe.h"
#include "BrowserController.h"
#include "BuildInfo.h"
#include "DesktopIntegration.h"
#include "DesktopProbe.h"
#include "FileOpenRouter.h"
#include "FocusDismissPolicy.h"
#include "InstalledRuntimeGuard.h"
#include "UiFont.h"
#include "UiPreferences.h"
#include "UiText.h"
#include <QApplication>
#include <QEvent>
#include <QIcon>
#include <QPointer>
#include <QProcess>
#include <QQmlApplicationEngine>
#include <QQuickStyle>
#include <QQuickWindow>
#include <QResource>
#include <QTimer>
#include <memory>

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
    InstalledRuntimeGuard installedRuntime;
    if (!installedRuntime.acquire())
        return 11;
    FileOpenRouter fileOpens;
    application.installEventFilter(&fileOpens);
    const auto args = application.arguments();
    const bool literalPaths = args.contains(QStringLiteral("--"));
    const auto probeArgument = literalPaths ? -1 : args.indexOf(QStringLiteral("--desktop-probe"));
    const bool smokeTest = !literalPaths && args.contains(QStringLiteral("--smoke-test"));
    if (!smokeTest && probeArgument < 0) {
        const auto start = literalPaths ? args.indexOf(QStringLiteral("--")) + 1 : 1;
        for (int i = start; i < args.size(); ++i) {
            if (literalPaths || !args.at(i).startsWith(QLatin1Char('-')))
                fileOpens.enqueue(args.at(i));
        }
    }
    FocusDismissPolicy::install(&application);
    application.setApplicationName(QStringLiteral("xdzip"));
    application.setApplicationVersion(BuildInfo::version());
    application.setDesktopFileName(QStringLiteral("xdzip"));
    QQuickStyle::setStyle(QStringLiteral("Basic"));
    Q_INIT_RESOURCE(ui);
    if (!initializeUiText())
        return 5;
    if (!installUiFont())
        return 5;
#ifdef Q_OS_MACOS
    application.setWindowIcon(QIcon(QStringLiteral(":/xdzip/assets/app-icon-macos.png")));
#else
    application.setWindowIcon(QIcon(QStringLiteral(":/xdzip/assets/app-icon.png")));
#endif
    QQmlApplicationEngine engine;
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
    fileOpens.attach(controller, mainWindow);
    installAssociationProbe(controller, mainWindow);
    AppQuitGuard guard;
    guard.window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
    guard.controller = controller;
    application.installEventFilter(&guard);
    QObject::connect(controller, &BrowserController::readyToClose, &application,
                     &QApplication::quit, Qt::QueuedConnection);
    auto *settings = engine.singletonInstance<UiPreferences *>("Xdzip.Settings", "AppSettings");
    DesktopIntegration desktop(mainWindow, controller, settings, !smokeTest && probeArgument < 0);
    std::unique_ptr<DesktopProbe> probe;
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
    // Window close completes the existing task decision and runtime join barrier first.
    // Relaunch only the application, never replay diagnostic flags or a previously opened file.
    if (result == 0 && !engine.rootObjects().isEmpty() &&
        engine.rootObjects().first()->property("restartAfterClose").toBool()) {
        if (!QProcess::startDetached(QCoreApplication::applicationFilePath(), QStringList{}))
            return 6;
    }
    return result;
}
