#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QQuickStyle>
#include <QResource>

int main(int argc, char *argv[]) {
    QGuiApplication application(argc, argv);
    application.setApplicationName(QStringLiteral("xdzip"));
    application.setOrganizationName(QStringLiteral("xdzip"));
    QQuickStyle::setStyle(QStringLiteral("Basic"));
    Q_INIT_RESOURCE(ui);
    QQmlApplicationEngine engine;
    QObject::connect(
        &engine, &QQmlApplicationEngine::objectCreationFailed, &application,
        [] { QCoreApplication::exit(1); }, Qt::QueuedConnection);
    engine.load(QUrl(QStringLiteral("qrc:/xdzip/ui/Preview.qml")));
    if (application.arguments().contains(QStringLiteral("--smoke-test"))) {
        return engine.rootObjects().size() == 1 ? 0 : 1;
    }
    return application.exec();
}
