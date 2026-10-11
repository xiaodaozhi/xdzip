#include "BrowserController.h"
#include <QApplication>
#include <QEvent>
#include <QPointer>
#include <QQmlApplicationEngine>
#include <QQuickStyle>
#include <QQuickWindow>
#include <QResource>
#include <QTimer>

// Platform Quit (including Cmd+Q) must use the same asynchronous shutdown as window close.
class QuitGuard final : public QObject {
  public:
    QPointer<BrowserController> controller;
    bool eventFilter(QObject *, QEvent *event) override {
        return event->type() == QEvent::Quit && controller && !controller->requestClose();
    }
};
int main(int argc, char *argv[]) {
    QApplication application(argc, argv);
    application.setApplicationName(QStringLiteral("xdzip"));
    QQuickStyle::setStyle(QStringLiteral("Basic"));
    Q_INIT_RESOURCE(ui);
    QQmlApplicationEngine engine;
    QObject::connect(
        &engine, &QQmlApplicationEngine::objectCreationFailed, &application,
        [] { QCoreApplication::exit(1); }, Qt::QueuedConnection);
    engine.load(QUrl(QStringLiteral("qrc:/xdzip/ui/Browser.qml")));
    if (engine.rootObjects().isEmpty())
        return 1;
    auto *controller = engine.rootObjects().first()->findChild<BrowserController *>();
    if (!controller)
        return 1;
    QuitGuard guard;
    guard.controller = controller;
    application.installEventFilter(&guard);
    QObject::connect(controller, &BrowserController::readyToClose, &application,
                     &QApplication::quit, Qt::QueuedConnection);
    if (application.arguments().contains(QStringLiteral("--smoke-test"))) {
        // Exercise startup, real receiver wakeup and shutdown, not just QML construction.
        QTimer::singleShot(0, &application, &QApplication::quit);
        QTimer::singleShot(10000, &application, [] { QCoreApplication::exit(2); });
    } else if (application.arguments().size() == 2) {
        const auto path = application.arguments().at(1);
        QTimer::singleShot(0, controller, [controller, path] { controller->openPath(path); });
    }
    return application.exec();
}
