#include "FocusDismissPolicy.h"
#include "UiFont.h"
#include <QGuiApplication>
#include <QIcon>
#include <QQmlApplicationEngine>
#include <QQuickStyle>
#include <QResource>

int main(int argc, char *argv[]) {
    QGuiApplication application(argc, argv);
    FocusDismissPolicy::install(&application);
    application.setApplicationName(QStringLiteral("xdzip"));
    application.setOrganizationName(QStringLiteral("xdzip"));
    QQuickStyle::setStyle(QStringLiteral("Basic"));
    Q_INIT_RESOURCE(ui);
    if (!installUiFont())
        return 5;
    application.setWindowIcon(QIcon(QStringLiteral(":/xdzip/assets/app-icon.png")));
    QQmlApplicationEngine engine;
    QObject::connect(
        &engine, &QQmlApplicationEngine::objectCreationFailed, &application,
        [] { QCoreApplication::exit(1); }, Qt::QueuedConnection);
    engine.load(QUrl(QStringLiteral("qrc:/xdzip/ui/Preview.qml")));
    if (application.arguments().contains(QStringLiteral("--smoke-test"))) {
        if (application.windowIcon().isNull() || application.windowIcon().pixmap(64, 64).isNull())
            return 4;
        return engine.rootObjects().size() == 1 ? 0 : 1;
    }
    return application.exec();
}
