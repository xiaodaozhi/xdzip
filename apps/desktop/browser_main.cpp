#include "BrowserController.h"
#include <QApplication>
#include <QQmlApplicationEngine>
#include <QQuickStyle>
#include <QResource>
#include <QTimer>

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
    if (application.arguments().contains(QStringLiteral("--smoke-test")))
        return 0;
    if (application.arguments().size() == 2) {
        auto *controller = engine.rootObjects().first()->findChild<BrowserController *>();
        const auto path = application.arguments().at(1);
        if (controller)
            QTimer::singleShot(0, controller, [controller, path] { controller->openPath(path); });
    }
    return application.exec();
}
