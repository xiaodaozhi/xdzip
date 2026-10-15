#include "AppQuitGuard.h"
#include "BrowserController.h"
#include "FocusDismissPolicy.h"
#include "UiFont.h"
#include <QApplication>
#include <QEvent>
#include <QIcon>
#include <QPointer>
#include <QQmlApplicationEngine>
#include <QQuickStyle>
#include <QQuickWindow>
#include <QResource>
#include <QTimer>

#ifdef Q_OS_MACOS
void integrateMacTitlebar(QQuickWindow *window);
bool verifyMacTitlebar(QQuickWindow *window);
#endif

int main(int argc, char *argv[]) {
    QApplication application(argc, argv);
    FocusDismissPolicy::install(&application);
    application.setApplicationName(QStringLiteral("xdzip"));
    application.setDesktopFileName(QStringLiteral("xdzip"));
    QQuickStyle::setStyle(QStringLiteral("Basic"));
    Q_INIT_RESOURCE(ui);
    if (!installUiFont())
        return 5;
    application.setWindowIcon(QIcon(QStringLiteral(":/xdzip/assets/app-icon.png")));
    QQmlApplicationEngine engine;
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
    AppQuitGuard guard;
    guard.window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
    guard.controller = controller;
    application.installEventFilter(&guard);
    QObject::connect(controller, &BrowserController::readyToClose, &application,
                     &QApplication::quit, Qt::QueuedConnection);
    if (application.arguments().contains(QStringLiteral("--smoke-test"))) {
        if (application.windowIcon().isNull() || application.windowIcon().pixmap(64, 64).isNull())
            return 4;
#ifdef Q_OS_MACOS
        if (QGuiApplication::platformName() == QStringLiteral("cocoa") &&
            !engine.rootObjects().first()->property("integratedTitlebar").toBool())
            return 3;
#endif
        // Exercise startup, real receiver wakeup and shutdown, not just QML construction.
        QTimer::singleShot(500, &application, [&application, &engine] {
#ifdef Q_OS_MACOS
            if (!verifyMacTitlebar(qobject_cast<QQuickWindow *>(engine.rootObjects().first()))) {
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
    } else if (application.arguments().size() == 2) {
        const auto path = application.arguments().at(1);
        QTimer::singleShot(0, controller, [controller, path] { controller->openPath(path); });
    }
    return application.exec();
}
