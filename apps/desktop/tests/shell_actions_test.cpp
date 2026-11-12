#include "ApplicationSessions.h"
#include "PasswordAccessibility.h"
#include "ShellBatchController.h"
#include "ShellIntegration.h"
#include "SingleInstance.h"
#include "UiFont.h"
#include "UiPreferences.h"
#include "UiText.h"
#include "shell/MenuModel.h"
#include "test_environment.h"
#include <QAccessible>
#include <QApplication>
#include <QDesktopServices>
#include <QFile>
#include <QJsonArray>
#include <QJsonDocument>
#include <QProcess>
#include <QProcessEnvironment>
#include <QQmlApplicationEngine>
#include <QQmlComponent>
#include <QQmlContext>
#include <QQuickItem>
#include <QQuickStyle>
#include <QScopeGuard>
#include <QSignalSpy>
#include <QTemporaryDir>
#include <QtTest>

class UrlRecorder : public QObject {
    Q_OBJECT
  public:
    int count = 0;
  public slots:
    void opened(const QUrl &) {
        ++count;
    }
};
class ShellTest : public QObject {
    Q_OBJECT
    static void write(const QString &path, const QByteArray &bytes) {
        QFile file(path);
        QVERIFY(file.open(QIODevice::WriteOnly));
        QCOMPARE(file.write(bytes), bytes.size());
    }
    static ShellActionRequest request(const QString &action, const QStringList &paths) {
        return ShellActionRequest::parse(
            {{"schema_version", 1},
             {"id", QUuid::createUuid().toString(QUuid::WithoutBraces)},
             {"action", action},
             {"paths", QJsonArray::fromStringList(paths)}});
    }
  private slots:
    void initTestCase() {
        Q_INIT_RESOURCE(ui);
        QQuickStyle::setStyle("Basic");
        QVERIFY(initializeUiText("zh-CN"));
        QVERIFY(installUiFont());
        qmlRegisterType<BrowserController>("Xdzip.Native", 1, 0, "BrowserController");
        qmlRegisterType<ShellBatchController>("Xdzip.Native", 1, 0, "ShellBatchController");
    }
    void boundedRequestAndNativeHandoff() {
        QTemporaryDir source;
        const auto path = source.filePath(QString::fromUtf8("中文 & notes.txt"));
        write(path, "payload");
        auto req = request("zip", {path});
        QVERIFY(req.valid());
        auto bad = req.json();
        bad.insert("action", "exec");
        QVERIFY(!ShellActionRequest::parse(bad).valid());
        bad = req.json();
        bad.insert("paths", QJsonArray{"relative.txt"});
        QVERIFY(!ShellActionRequest::parse(bad).valid());
        QStringList tooMany;
        for (int i = 0; i < 129; ++i)
            tooMany << source.filePath(QString::number(i));
        QVERIFY(!request("zip", tooMany).valid());
        const auto native =
            xdzip_shell::requestFile("zip", {xdzip_shell::fs::u8path(path.toUtf8().toStdString())});
        QVERIFY(!native.empty());
        const auto read =
            ShellActionRequest::readFile(QString::fromUtf8(native.u8string().c_str()));
        QCOMPARE(ShellActionRequest::parse(read).paths, QStringList{path});
        QVERIFY(!QFile::exists(QString::fromUtf8(native.u8string().c_str())));
    }
    void quickCreationAndPartialBatchFailure() {
        UrlRecorder opened;
        QDesktopServices::setUrlHandler("file", &opened, "opened");
        const auto restore = qScopeGuard([] { QDesktopServices::unsetUrlHandler("file"); });
        QTemporaryDir source;
        const auto one = source.filePath("one.txt"), two = source.filePath("two.txt");
        write(one, "first");
        write(two, "second");
        ShellBatchController batch;
        batch.begin(request("separate-zip", {source.filePath("missing.txt"), one, two}));
        QTRY_VERIFY_WITH_TIMEOUT(batch.finished(), 15000);
        QCOMPARE(batch.results().size(), 3);
        QCOMPARE(batch.results()[0].toMap().value("status").toString(), QString("failed"));
        QCOMPARE(batch.results()[1].toMap().value("status").toString(), QString("succeeded"));
        QVERIFY(QFile::exists(one + ".zip"));
        QVERIFY(QFile::exists(two + ".zip"));
        QVERIFY(!batch.reader()->ready()); // shell creation never auto-opens its output.
        ShellBatchController extract;
        QTemporaryDir target;
        connect(&extract, &ShellBatchController::configureExtract, &extract,
                [&] { extract.configureExtraction(target.path(), 2, 0, false, false); });
        extract.begin(request("extract-to", {one + ".zip", two + ".zip"}));
        QTRY_VERIFY_WITH_TIMEOUT(extract.finished(), 15000);
        QCOMPARE(extract.results()[0].toMap().value("status").toString(), QString("succeeded"));
        QVERIFY(QFile::exists(target.filePath("one.txt/one.txt")));
        QVERIFY(QFile::exists(target.filePath("two.txt/two.txt")));
        QCOMPARE(opened.count, 0);
        extract.openTarget(0);
        QCOMPARE(opened.count, 1);
    }
    void forwardsShellRequestsToExistingInstance() {
        const auto space = "shell-test-" + QUuid::createUuid().toString(QUuid::WithoutBraces);
        SingleInstance primary;
        QCOMPARE(primary.start({}, space), SingleInstance::Primary);
        int received = 0;
        primary.attachShell([&](const QJsonObject &value) {
            ++received;
            return ShellActionRequest::parse(value).valid();
        });
        const auto manifest =
            xdzip_shell::requestFile("choose", {xdzip_shell::fs::temp_directory_path()});
        QVERIFY(!manifest.empty());
        QProcess client;
        client.start(
            QCoreApplication::applicationFilePath(),
            {"--shell-instance-client", space, QString::fromUtf8(manifest.u8string().c_str())});
        QVERIFY(client.waitForStarted());
        QTRY_COMPARE_WITH_TIMEOUT(client.state(), QProcess::NotRunning, 10000);
        QCOMPARE(client.exitCode(), 0);
        QCOMPARE(received, 1);
    }
    void cancellationStopsRemainingJobs() {
        QTemporaryDir source;
        QStringList paths;
        for (int i = 0; i < 8; ++i) {
            paths << source.filePath(QString::number(i) + ".txt");
            write(paths.last(), QByteArray(262144, 'x'));
        }
        ShellBatchController batch;
        batch.begin(request("separate-zip", paths));
        batch.cancel();
        QTRY_VERIFY_WITH_TIMEOUT(batch.finished(), 15000);
        QCOMPARE(batch.results().size(), 8);
        for (int i = 1; i < 8; ++i)
            QCOMPARE(batch.results()[i].toMap().value("status").toString(), QString("not-run"));
    }
    void realProductColdLaunchHasNoMainWindow() {
        QTemporaryDir source;
        const auto path = source.filePath("payload.txt");
        write(path, "actual product");
        const auto manifest =
            xdzip_shell::requestFile("zip", {xdzip_shell::fs::u8path(path.toUtf8().toStdString())});
        QVERIFY(!manifest.empty());
        const auto output = source.filePath("probe.json");
        auto environment = QProcessEnvironment::systemEnvironment();
        environment.insert("XDZIP_SHELL_PROBE", output);
        environment.insert("XDZIP_SHELL_NAMESPACE",
                           QUuid::createUuid().toString(QUuid::WithoutBraces));
        QProcess product;
        product.setProcessEnvironment(environment);
        product.start(QString::fromUtf8(XDZIP_SHELL_PRODUCT),
                      {"--shell-request", QString::fromUtf8(manifest.u8string().c_str())});
        QVERIFY(product.waitForStarted());
        QVERIFY(product.waitForFinished(40000));
        QCOMPARE(product.exitStatus(), QProcess::NormalExit);
        QVERIFY2(product.exitCode() == 0, product.readAllStandardError().constData());
        QFile record(output);
        QVERIFY(record.open(QIODevice::ReadOnly));
        const auto result = QJsonDocument::fromJson(record.readAll()).object();
        QCOMPARE(result.value("main_windows").toInt(), 0);
        QCOMPARE(result.value("batches").toInt(), 1);
        QCOMPARE(result.value("timed_out").toBool(), false);
        QCOMPARE(result.value("statuses").toArray(), QJsonArray{"succeeded"});
        QVERIFY(QFile::exists(path + ".zip"));
    }
    void shellSettingsAccessibleActionsUpdateTheRealModel() {
        QQmlApplicationEngine engine;
        auto *settings = engine.singletonInstance<UiPreferences *>("Xdzip.Settings", "AppSettings");
        QVERIFY(settings);
        ShellIntegration integration(settings);
        integration.m_availableOverride = true;
        bool nativeEnabled = settings->shellEnabled();
        integration.m_applyOverride = [&](bool value) {
            nativeEnabled = value;
            return true;
        };
        engine.rootContext()->setContextProperty("ShellIntegrationService", &integration);
        QQmlComponent component(&engine, QUrl("qrc:/xdzip/ui/SettingsWindow.qml"));
        std::unique_ptr<QObject> object(component.create());
        QVERIFY2(object, qPrintable(component.errorString()));
        QVERIFY(QMetaObject::invokeMethod(object.get(), "configureTab", Q_ARG(QVariant, 1)));
        auto *control = object->findChild<QQuickItem *>("settingsShellIntegration");
        QTRY_VERIFY(control = object->findChild<QQuickItem *>("settingsShellIntegration"));
        QVERIFY(control->isEnabled());
        auto *accessible = QAccessible::queryAccessibleInterface(control);
        QVERIFY(accessible);
        QVERIFY(accessible->actionInterface());
        QVERIFY(!accessible->text(QAccessible::Name).isEmpty());
        const auto before = settings->shellEnabled();
        accessible->actionInterface()->doAction(QAccessibleActionInterface::pressAction());
        QCOMPARE(settings->shellEnabled(), !before);
        QCOMPARE(nativeEnabled, !before);
        QCOMPARE(control->property("checked").toBool(), !before);
        accessible->actionInterface()->doAction(QAccessibleActionInterface::toggleAction());
        QCOMPARE(settings->shellEnabled(), before);
        QCOMPARE(nativeEnabled, before);
        integration.m_applyOverride = [](bool) { return false; };
        accessible->actionInterface()->doAction(QAccessibleActionInterface::pressAction());
        QCOMPARE(settings->shellEnabled(), before);
        QCOMPARE(control->property("checked").toBool(), before);
        QVERIFY(!integration.error().isEmpty());
    }
    void shellLayoutMatrix_data() {
        QTest::addColumn<QString>("language");
        QTest::addColumn<QString>("theme");
        QTest::addColumn<double>("scale");
        for (const auto &language : {"zh-CN", "en-US"})
            for (const auto &theme : {"light", "dark"})
                for (const auto scale : {1., 1.25, 1.5})
                    QTest::newRow(qPrintable(QString("%1-%2-%3").arg(language, theme).arg(scale)))
                        << QString(language) << QString(theme) << scale;
    }
    void shellLayoutMatrix() {
        QFETCH(QString, language);
        QFETCH(QString, theme);
        QFETCH(double, scale);
        QQmlApplicationEngine engine;
        auto *settings = engine.singletonInstance<UiPreferences *>("Xdzip.Settings", "AppSettings");
        QVERIFY(settings);
        QVERIFY(settings->applyDetails(theme, scale, language, "standard", true, false, "tabs"));
        ApplicationSessions sessions(&engine, settings);
        QTemporaryDir source;
        QVERIFY(sessions.shellAction(
            request("choose", {source.filePath("one.zip"), source.filePath("two.7z")}).json()));
        auto *panel = sessions.findChild<QQuickWindow *>("shellActionsWindow");
        QVERIFY(panel);
        QTRY_VERIFY(panel->isVisible());
        QTRY_VERIFY(panel->isExposed());
        QCOMPARE(panel->width(), 491);
        QVERIFY(panel->height() <= 600);
        const auto commandItems = [&] {
            QList<QQuickItem *> result;
            std::function<void(QQuickItem *)> visit = [&](QQuickItem *item) {
                if (item->objectName().startsWith("shellAction-") &&
                    !item->objectName().endsWith("Background"))
                    result.append(item);
                for (auto *child : item->childItems())
                    visit(child);
            };
            visit(panel->contentItem());
            return result;
        };
        QTRY_COMPARE(commandItems().size(), 8);
        for (auto *item : commandItems()) {
            QVERIFY(item->property("wrapLabel").toBool());
            QCOMPARE(item->window(), panel);
            QVERIFY(item->width() <= panel->width());
            QVERIFY(item->height() >= 28);
        }
        const auto captures = qEnvironmentVariable("XDZIP_SHELL_CAPTURE_DIR");
        if (!captures.isEmpty()) {
            QVERIFY(QDir().mkpath(captures));
            QTRY_VERIFY(panel->isExposed());
            QTest::qWait(40);
            const auto image = panel->grabWindow();
            QVERIFY(!image.isNull());
            const auto stem = language + '-' + theme + '-' + QString::number(int(scale * 100));
            QVERIFY(image.save(captures + '/' + stem + ".png"));
            write(captures + '/' + stem + ".json",
                  QJsonDocument(QJsonObject{{"platform", QGuiApplication::platformName()},
                                            {"width", panel->width()},
                                            {"height", panel->height()},
                                            {"dpr", panel->devicePixelRatio()},
                                            {"language", language},
                                            {"theme", theme},
                                            {"text_scale", scale}})
                      .toJson());
        }
        QSignalSpy joined(&sessions, &ApplicationSessions::readyToQuit);
        sessions.confirmQuit(false);
        QTRY_COMPARE_WITH_TIMEOUT(joined.count(), 1, 10000);
    }
    void hostHasNoMainWindowAndDeduplicates() {
        QQmlApplicationEngine engine;
        UiPreferences settings;
        ApplicationSessions sessions(&engine, &settings);
        QTemporaryDir source;
        const auto file = source.filePath("one.txt");
        write(file, "one");
        const auto req = request("choose", {file});
        QVERIFY(sessions.shellAction(req.json()));
        QVERIFY(sessions.shellAction(req.json()));
        QCOMPARE(sessions.sessionCount(), 1);
        QCOMPARE(sessions.windows().size(), 0);
        auto *panel = sessions.findChild<QQuickWindow *>("shellActionsWindow");
        QVERIFY(panel);
        QVERIFY(panel->isVisible());
        QCOMPARE(panel->width(), 491);
        QVERIFY(!panel->transientParent());
        auto *batch = sessions.findChild<ShellBatchController *>();
        QVERIFY(batch);
        batch->choose("create");
        auto *create = sessions.findChild<QQuickWindow *>("createWindow");
        QVERIFY(create);
        QVERIFY(create->isVisible());
        QCOMPARE(create->width(), 491);
        QCOMPARE(create->property("inputs").toList().size(), 1);
        auto changed = req.json();
        changed.insert("action", "zip");
        QVERIFY(!sessions.shellAction(changed));
        QSignalSpy joined(&sessions, &ApplicationSessions::readyToQuit);
        sessions.confirmQuit(false);
        QTRY_COMPARE_WITH_TIMEOUT(joined.count(), 1, 10000);
    }
};
int main(int argc, char **argv) {
    test_environment::Runtime fixtures(QString::fromUtf8(XDZIP_TEST_TEMP_ROOT));
    if (!fixtures.valid())
        return 2;
    QApplication app(argc, argv);
    app.setQuitOnLastWindowClosed(false);
    if (app.arguments().value(1) == "--shell-instance-client") {
        SingleInstance instance;
        const auto req = ShellActionRequest::readFile(app.arguments().value(3));
        return instance.start({}, app.arguments().value(2), 15000, {}, req) ==
                       SingleInstance::Forwarded
                   ? 0
                   : 1;
    }
    installPasswordAccessibility();
    ShellTest test;
    const auto result = QTest::qExec(&test, argc, argv);
    fixtures.close();
    return result;
}
#include "shell_actions_test.moc"
