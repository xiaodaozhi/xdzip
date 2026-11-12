#include "ShellBatchController.h"
// apps/desktop/tests/update_controller_test.cpp

#include "ApplicationSessions.h"
#include "CliRequestBroker.h"
#include "SingleInstance.h"
#include "UiFont.h"
#include "UiPreferences.h"
#include "UiText.h"
#include "UpdateController.h"
#include "UpdateNetwork.h"
#include "UpdateProbeDiagnostics.h"
#include <QNetworkProxyFactory>
#ifdef Q_OS_MACOS
#include "MacUpdateProvider.h"
#endif
#include <QDesktopServices>
#include <QDir>
#include <QFile>
#include <QJsonArray>
#include <QJsonDocument>
#include <QNetworkReply>
#include <QNetworkRequest>
#include <QQmlApplicationEngine>
#include <QQuickItem>
#include <QQuickStyle>
#include <QScopeGuard>
#include <QSignalSpy>
#include <QSslSocket>
#include <QTcpServer>
#include <QTcpSocket>
#include <QTemporaryDir>
#include <QtTest>
#include <cstring>

namespace {
const QString api =
    QStringLiteral("https://api.github.com/repos/xiaodaozhi/xdzip/releases?per_page=100&page=1");
const QString base =
    QStringLiteral("https://github.com/xiaodaozhi/xdzip/releases/download/v9.0.0/");
QByteArray fixture(const QString &name) {
    QFile file(QStringLiteral(XDZIP_UPDATE_FIXTURES "/") + name);
    return file.open(QIODevice::ReadOnly) ? file.readAll() : QByteArray{};
}
struct Response {
    QByteArray bytes;
    int status = 200;
    QNetworkReply::NetworkError error = QNetworkReply::NoError;
    int delay = 0;
    QUrl redirect{};
    QList<QPair<QByteArray, QByteArray>> headers{};
};
class Reply final : public QNetworkReply {
  public:
    Reply(const QNetworkRequest &request, Response response, QObject *parent)
        : QNetworkReply(parent), m_bytes(std::move(response.bytes)) {
        setRequest(request);
        setUrl(request.url());
        setAttribute(QNetworkRequest::HttpStatusCodeAttribute, response.status);
        setRawHeader("ETag", "\"fixture\"");
        for (const auto &header : response.headers)
            setRawHeader(header.first, header.second);
        if (response.error != NoError)
            setError(response.error, QStringLiteral("fixture failure"));
        open(QIODevice::ReadOnly | QIODevice::Unbuffered);
        QTimer::singleShot(response.delay, this, [this, redirect = response.redirect] {
            if (isFinished())
                return;
            if (!redirect.isEmpty())
                emit redirected(redirect);
            if (isFinished())
                return;
            emit readyRead();
            if (!isFinished()) {
                setFinished(true);
                emit finished();
            }
        });
    }
    void abort() override {
        if (isFinished())
            return;
        setError(OperationCanceledError, QStringLiteral("cancelled"));
        setFinished(true);
        emit finished();
    }
    qint64 bytesAvailable() const override {
        return m_bytes.size() - m_offset + QIODevice::bytesAvailable();
    }

  protected:
    qint64 readData(char *data, qint64 maximum) override {
        const auto size = qMin(maximum, qint64(m_bytes.size()) - m_offset);
        if (!size)
            return -1;
        std::memcpy(data, m_bytes.constData() + m_offset, size_t(size));
        m_offset += size;
        return size;
    }

  private:
    QByteArray m_bytes;
    qint64 m_offset = 0;
};
class Network final : public QNetworkAccessManager {
  public:
    QHash<QString, Response> responses;
    QList<QNetworkRequest> requests;

  protected:
    QNetworkReply *createRequest(Operation operation, const QNetworkRequest &request,
                                 QIODevice *data) override {
        Q_UNUSED(operation);
        Q_UNUSED(data);
        requests.append(request);
        return new Reply(request,
                         responses.value(request.url().toString(),
                                         {{}, 404, QNetworkReply::ContentNotFoundError, 0}),
                         this);
    }
};
QJsonObject release(bool signedManifest) {
    const auto manifest = QJsonDocument::fromJson(fixture("network-manifest.json")).object();
    QJsonArray assets;
    for (const auto &value : manifest.value("assets").toArray()) {
        const auto item = value.toObject();
        assets.append(QJsonObject{{"name", item.value("filename")},
                                  {"size", 4},
                                  {"browser_download_url", item.value("url")}});
    }
    assets.append(
        QJsonObject{{"name", "release.json"}, {"browser_download_url", base + "release.json"}});
    if (signedManifest)
        assets.append(QJsonObject{{"name", "release.json.sig"},
                                  {"browser_download_url", base + "release.json.sig"}});
    return {{"tag_name", "v9.0.0"}, {"draft", false}, {"assets", assets}};
}
void configure(Network &network, bool signedManifest = true) {
    network.responses.insert(api, {QJsonDocument(QJsonArray{release(signedManifest)}).toJson()});
    network.responses.insert(base + "release.json", {fixture("network-manifest.json")});
    network.responses.insert(base + "release.json.sig", {fixture("network-manifest.json.sig")});
    for (const auto &value : QJsonDocument::fromJson(fixture("network-manifest.json"))
                                 .object()
                                 .value("assets")
                                 .toArray())
        network.responses.insert(value.toObject().value("url").toString(),
                                 {QByteArrayLiteral("data")});
}
} // namespace
class UpdateControllerTest final : public QObject {
    Q_OBJECT
    QTemporaryDir m_profile;
    QList<QUrl> m_openedUrls;

  public:
    Q_INVOKABLE void recordUrl(const QUrl &url) {
        m_openedUrls.append(url);
    }

  private slots:
    void cliHandoffChecksCandidateWithoutClaimingInstallation() {
        QTemporaryDir directory;
        const auto previous = qgetenv("XDZIP_SETTINGS_PATH");
        const auto restore = qScopeGuard([&] { qputenv("XDZIP_SETTINGS_PATH", previous); });
        qputenv("XDZIP_SETTINGS_PATH", directory.filePath("settings.ini").toUtf8());
        const auto request = directory.filePath("cli/cache/desktop-" + QString(32, 'a'));
        QVERIFY(QDir().mkpath(request));
        QFile input(request + "/request.json");
        QVERIFY(input.open(QIODevice::WriteOnly));
        const QByteArray bytes =
            R"({"schema_version":1,"action":"update","candidate_version":"9.0.0"})";
        QCOMPARE(input.write(bytes), bytes.size());
        input.close();
        Network network;
        configure(network);
        UpdateController updates(nullptr, &network, fixture("network-trust.json"),
                                 directory.filePath("download"), [] { return true; });
        QQmlApplicationEngine engine;
        UiPreferences preferences;
        ApplicationSessions sessions(&engine, &preferences);
        updates.setSessions(&sessions);
        SingleInstance instance;
        QCOMPARE(instance.start({}, directory.path(), 1000, request), SingleInstance::Primary);
        installCliRequests(&instance, &updates, &sessions);
        QFile result(request + "/result.json");
        QTRY_VERIFY(result.exists());
        QVERIFY(result.open(QIODevice::ReadOnly));
        const auto reply = QJsonDocument::fromJson(result.readAll()).object();
        QCOMPARE(reply.value("status").toString(), "handed_off");
        QCOMPARE(reply.value("error_code").toString(), "");
        QTRY_COMPARE(updates.state(), "available");
        QVERIFY(updates.trusted());
        QVERIFY(!updates.canInstall());
        QCOMPARE(network.requests.size(), 3);
        QFile status(directory.filePath("cli/desktop-update-status.json"));
        QVERIFY(status.open(QIODevice::ReadOnly));
        const auto receipt = QJsonDocument::fromJson(status.readAll()).object();
        QCOMPARE(receipt.value("status").toString(), "manual_install");
        QVERIFY(!receipt.value("install_gate").toBool());
    }
    void installationReasonsPreserveTheAutomaticInstallGate() {
        using Controller = UpdateController;
        QCOMPARE(Controller::installationReason(false, true, true, true, true, true), "unsigned");
        QCOMPARE(Controller::installationReason(true, false, true, true, true, true),
                 "manual_install");
        QCOMPARE(Controller::installationReason(true, true, false, true, true, true), "preview");
        QCOMPARE(Controller::installationReason(true, true, true, true, false, true),
                 "macos_missing_appcast");
        QCOMPARE(Controller::installationReason(true, true, true, true, true, false),
                 "macos_proxy");
        for (const bool trusted : {false, true})
            for (const bool installed : {false, true})
                for (const bool sessions : {false, true})
                    for (const bool macos : {false, true})
                        for (const bool feed : {false, true})
                            for (const bool systemProxy : {false, true})
                                QCOMPARE(Controller::installationReason(
                                             trusted, installed, sessions, macos, feed, systemProxy)
                                             .isEmpty(),
                                         trusted && installed && sessions &&
                                             (!macos || (feed && systemProxy)));
    }
    void macosCustomProxyRetainsTheOtherInstallationGates() {
        for (const auto &address :
             {QStringLiteral("http://127.0.0.1:7890"), QStringLiteral("socks5://127.0.0.1:1080")}) {
            const bool supported = UpdateNetwork::macosAutomaticTransport("custom", address);
            QVERIFY(supported);
            QCOMPARE(UpdateController::installationReason(true, true, true, true, true, supported),
                     QString{});
            QCOMPARE(UpdateController::installationReason(false, true, true, true, true, supported),
                     "unsigned");
            QCOMPARE(UpdateController::installationReason(true, false, true, true, true, supported),
                     "manual_install");
            QCOMPARE(UpdateController::installationReason(true, true, true, true, false, supported),
                     "macos_missing_appcast");
        }
        QVERIFY(UpdateNetwork::macosAutomaticTransport("system", {}));
        QVERIFY(UpdateNetwork::macosAutomaticTransport("none", {}));
        QVERIFY(!UpdateNetwork::macosAutomaticTransport("unknown", {}));
        QVERIFY(!UpdateNetwork::macosAutomaticTransport("custom", {}));
        QVERIFY(!UpdateNetwork::macosAutomaticTransport("custom", "http://user:secret@host:42"));
        QVERIFY(!UpdateNetwork::macosAutomaticTransport("custom", "https://host:443"));
    }
    void macosDirectTransportRetainsAllIndependentInstallationGates() {
        const bool supported = UpdateNetwork::macosAutomaticTransport("none", {});
        for (const bool trusted : {false, true})
            for (const bool installed : {false, true})
                for (const bool sessions : {false, true})
                    for (const bool feed : {false, true})
                        QCOMPARE(UpdateController::installationReason(trusted, installed, sessions,
                                                                      true, feed, supported)
                                     .isEmpty(),
                                 trusted && installed && sessions && feed);
    }
#ifdef Q_OS_MACOS
    void nativeDriverCancellationAndOneShotCommit() {
        Network network;
        UpdateController updates(nullptr, &network, fixture("network-trust.json"), {},
                                 [] { return true; });
        QVERIFY(macUpdateLifecycleProbe(&updates));
    }
#endif
    void initTestCase() {
        QVERIFY(m_profile.isValid());
        qputenv("XDZIP_SETTINGS_PATH", m_profile.filePath("settings.ini").toUtf8());
        Q_INIT_RESOURCE(ui);
        QQuickStyle::setStyle("Basic");
        qmlRegisterType<ShellBatchController>("Xdzip.Native", 1, 0, "ShellBatchController");
        qmlRegisterType<BrowserController>("Xdzip.Native", 1, 0, "BrowserController");
        QVERIFY(initializeUiText());
        QVERIFY(installUiFont());
    }
    void updateBarrierKeepsTasksAndWaitsForEveryRuntimeJoin() {
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *preferences =
            engine.singletonInstance<UiPreferences *>("Xdzip.Settings", "AppSettings");
        ApplicationSessions sessions(&engine, preferences);
        sessions.adopt(window);
        auto *first = window->findChild<BrowserController *>();
        sessions.newSession();
        auto *second =
            window->property("activePage").value<QObject *>()->findChild<BrowserController *>();
        QVERIFY(first && second && first != second);
        QTRY_VERIFY(first->runtimeAvailable() && second->runtimeAvailable());
        xdzip::reader::Event event{};
        event.kind = xdzip::reader::EventKind::TaskUpdate;
        event.task.id = 31;
        event.task.kind = rust::String("check");
        event.task.status = rust::String("waiting_password");
        first->receive(event);
        QCOMPARE(sessions.activeTaskCount(), 1);
        QSignalSpy ready(&sessions, &ApplicationSessions::readyForUpdate);
        QSignalSpy quit(&sessions, &ApplicationSessions::readyToQuit);
        QVERIFY(!sessions.prepareForUpdate());
        QVERIFY(!sessions.isClosing());
        QCOMPARE(first->activeTaskCount(), 1);
        event.task.status = rust::String("succeeded");
        first->receive(event);
        QVERIFY(sessions.prepareForUpdate());
        QVERIFY(sessions.updatePending());
        const auto count = sessions.sessionCount();
        sessions.newSession();
        sessions.openPath(QStringLiteral(XDZIP_UPDATE_FIXTURES "/../zip/normal.zip"));
        QCOMPARE(sessions.sessionCount(), count);
        QTRY_COMPARE(ready.count(), 1);
        QVERIFY(first->shutdownComplete() && second->shutdownComplete());
        QVERIFY(quit.isEmpty());
        QVERIFY(!sessions.restartRequested());
        QVERIFY(window->isVisible());
        sessions.requestQuit();
        QCOMPARE(quit.count(), 1);
        QVERIFY(!window->isVisible());
    }
    void toolbarChecksWholeArchiveAndReopensItsHistory() {
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *controller = window->findChild<BrowserController *>();
        controller->openPath(QStringLiteral(XDZIP_UPDATE_FIXTURES "/../zip/normal.zip"));
        QTRY_VERIFY(controller->ready() && !controller->busy());
        controller->search("no-match");
        QTRY_VERIFY(!controller->busy());
        QSignalSpy started(controller, &BrowserController::archiveTestStarted);
        auto *toolbar = window->findChild<QObject *>("archiveTest");
        QVERIFY(toolbar && toolbar->property("enabled").toBool());
        QVERIFY(QMetaObject::invokeMethod(toolbar, "clicked"));
        QTRY_COMPARE(started.count(), 1);
        QTRY_VERIFY(!controller->tasks().isEmpty() &&
                    controller->tasks().first().toMap().value("terminal").toBool());
        const auto task = controller->tasks().first().toMap();
        QCOMPARE(task.value("kind").toString(), QStringLiteral("check"));
        QCOMPARE(task.value("status").toString(), QStringLiteral("succeeded"));
        QVERIFY(task.value("verifiedFiles").toULongLong() > 0);
        QCOMPARE(task.value("writtenBytes").toULongLong(), quint64(0));
        QVERIFY(task.value("target").toString().isEmpty());
        auto *taskWindow = window->findChild<QQuickWindow *>("taskWindow");
        QVERIFY(taskWindow);
        QTRY_VERIFY(taskWindow->isVisible());
        taskWindow->close();
        auto *page = window->property("activePage").value<QObject *>();
        QVERIFY(page);
        QVERIFY(QMetaObject::invokeMethod(page, "showTask",
                                          Q_ARG(QString, task.value("id").toString())));
        QTRY_VERIFY(taskWindow->isVisible());
        QCOMPARE(taskWindow->property("operationKind").toString(), QStringLiteral("check"));
        taskWindow->close();
        controller->requestClose();
        QTRY_VERIFY(controller->shutdownComplete());
    }
    void unsignedReleaseOffersManualUpdate() {
        Network network;
        configure(network, false);
        UpdateController updates(nullptr, &network, {}, {}, [] { return true; });
        updates.check();
        QTRY_COMPARE(updates.state(), QStringLiteral("available"));
        QCOMPARE(updates.candidateVersion(), QStringLiteral("9.0.0"));
        QVERIFY(!updates.trusted());
        QVERIFY(!updates.canInstall());
        QCOMPARE(network.requests.size(), 2);
    }
    void unsupportedCurrentSystemRejectsCandidate() {
        Network network;
        configure(network);
        UpdateController updates(nullptr, &network, {}, {}, [] { return false; });
        updates.check();
        QTRY_COMPARE(updates.state(), QStringLiteral("error"));
        QCOMPARE(updates.errorCode(), QStringLiteral("minimum_system"));
        QVERIFY(!updates.canInstall());
        QCOMPARE(network.requests.size(), 2);
    }
    void mirrorAndProxyPreserveUpdateVerification() {
        const QString mirror = QStringLiteral("https://mirror.example.test/prefix");
        QVERIFY(UpdateNetwork::validMirror(mirror));
        QVERIFY(!UpdateNetwork::validMirror(QStringLiteral("http://mirror.example.test")));
        QVERIFY(
            !UpdateNetwork::validMirror(QStringLiteral("https://user:secret@mirror.example.test")));
        QVERIFY(
            !UpdateNetwork::validMirror(QStringLiteral("https://mirror.example.test/a/%2e%2e/b")));
        QVERIFY(UpdateNetwork::validProxy(QStringLiteral("http://127.0.0.1:7890")));
        QVERIFY(UpdateNetwork::validProxy(QStringLiteral("socks5://127.0.0.1:1080")));
        QVERIFY(!UpdateNetwork::validProxy(QStringLiteral("http://user:secret@127.0.0.1:7890")));
        const auto mapped = UpdateNetwork::mapped(QUrl(api), mirror);
        QVERIFY(UpdateNetwork::isMirrorUrl(mapped, mirror));
        QVERIFY(!UpdateNetwork::isMirrorUrl(
            QUrl(QStringLiteral("https://evil.example.test/prefix/a")), mirror));
        QVERIFY(!UpdateNetwork::isMirrorUrl(
            QUrl(QStringLiteral("https://mirror.example.test/prefix/%2e%2e/evil")), mirror));
        QTemporaryDir directory;
        Network network;
        configure(network);
        const auto originalResponses = network.responses;
        for (auto it = originalResponses.cbegin(); it != originalResponses.cend(); ++it)
            network.responses.insert(UpdateNetwork::mapped(QUrl(it.key()), mirror).toString(),
                                     it.value());
        UpdateController updates(nullptr, &network, fixture("network-trust.json"), directory.path(),
                                 [] { return true; });
        updates.configureNetwork("custom", "http://127.0.0.1:7890", mirror);
        QVERIFY(network.proxyFactory());
        QCOMPARE(network.proxyFactory()->queryProxy(QNetworkProxyQuery(QUrl(api))).first().type(),
                 QNetworkProxy::HttpProxy);
        updates.check();
        QTRY_COMPARE(updates.state(), QStringLiteral("available"));
        QVERIFY(updates.trusted());
        QCOMPARE(network.requests.first().url(), mapped);
        updates.download();
        QTRY_COMPARE(updates.state(), QStringLiteral("downloaded"));
        QVERIFY(updates.downloaded());
        for (const auto &request : network.requests)
            QVERIFY(UpdateNetwork::isMirrorUrl(request.url(), mirror));
        updates.configureNetwork("none", "", "");
        QCOMPARE(network.proxyFactory()->queryProxy(QNetworkProxyQuery(QUrl(api))).first().type(),
                 QNetworkProxy::NoProxy);
    }
    void invalidNetworkSettingsAreNotPersisted() {
        QTemporaryDir directory;
        const auto previous = qgetenv("XDZIP_SETTINGS_PATH");
        const auto restore = qScopeGuard([&] { qputenv("XDZIP_SETTINGS_PATH", previous); });
        qputenv("XDZIP_SETTINGS_PATH", directory.filePath("settings.ini").toUtf8());
        UiPreferences prefs;
        QVERIFY(!prefs.applyNetwork("custom", "http://user:secret@host:42", ""));
        QCOMPARE(prefs.proxyMode(), QStringLiteral("system"));
        QVERIFY(!prefs.applyNetwork("none", "", "http://mirror.example.test"));
        QVERIFY(prefs.applyNetwork("custom", "socks5://127.0.0.1:1080",
                                   "https://mirror.example.test/prefix"));
        UiPreferences restored;
        QCOMPARE(restored.proxyMode(), QStringLiteral("custom"));
        QCOMPARE(restored.proxyAddress(), QStringLiteral("socks5://127.0.0.1:1080"));
        QCOMPARE(restored.mirrorAddress(), QStringLiteral("https://mirror.example.test/prefix"));
    }
    void signedDownloadChecksExactBytesAndHash() {
        QTemporaryDir directory(QDir::tempPath() + "/xdzip-update-network-XXXXXX");
        QVERIFY(directory.isValid());
        Network network;
        configure(network);
        UpdateController updates(nullptr, &network, fixture("network-trust.json"), directory.path(),
                                 [] { return true; });
        updates.check();
        QTRY_COMPARE(updates.state(), QStringLiteral("available"));
        QVERIFY(updates.trusted());
        QVERIFY(!updates.canInstall());
        updates.download();
        QTRY_COMPARE(updates.state(), QStringLiteral("downloaded"));
        QVERIFY(updates.downloaded());
        QCOMPARE(updates.progress(), qreal(1));
    }
    void damagedDownloadNeverBecomesReady() {
        QTemporaryDir directory(QDir::tempPath() + "/xdzip-update-network-XXXXXX");
        Network network;
        configure(network);
        for (auto it = network.responses.begin(); it != network.responses.end(); ++it)
            if (!it.key().endsWith("release.json") && !it.key().endsWith("release.json.sig") &&
                it.key() != api)
                it->bytes = "evil";
        UpdateController updates(nullptr, &network, fixture("network-trust.json"), directory.path(),
                                 [] { return true; });
        updates.check();
        QTRY_COMPARE(updates.state(), QStringLiteral("available"));
        updates.download();
        QTRY_COMPARE(updates.state(), QStringLiteral("error"));
        QCOMPARE(updates.errorCode(), QStringLiteral("download_hash"));
        QVERIFY(!updates.downloaded());
    }
    void modifiedManifestIsRejected() {
        Network network;
        configure(network);
        network.responses[base + "release.json"].bytes.replace(QByteArray(40, 'a'),
                                                               QByteArray(40, 'b'));
        UpdateController updates(nullptr, &network, fixture("network-trust.json"), {},
                                 [] { return true; });
        updates.check();
        QTRY_COMPARE(updates.state(), QStringLiteral("error"));
        QVERIFY(!updates.trusted());
        QVERIFY(!updates.canInstall());
        QCOMPARE(network.requests.size(), 3);
    }
    void malformedUtf8IsRejectedBeforeEnteringRust_data() {
        QTest::addColumn<QString>("source");
        QTest::addColumn<QString>("errorCode");
        QTest::newRow("manifest") << QStringLiteral("manifest")
                                  << QStringLiteral("invalid_manifest_json");
        QTest::newRow("signature")
            << QStringLiteral("signature") << QStringLiteral("invalid_signature_json");
        QTest::newRow("trust") << QStringLiteral("trust") << QStringLiteral("invalid_keyring");
    }
    void malformedUtf8IsRejectedBeforeEnteringRust() {
        QFETCH(QString, source);
        QFETCH(QString, errorCode);
        Network network;
        configure(network);
        const auto invalid = QByteArray::fromHex("ff");
        if (source == "manifest")
            network.responses[base + "release.json"].bytes = invalid;
        if (source == "signature")
            network.responses[base + "release.json.sig"].bytes = invalid;
        UpdateController updates(nullptr, &network,
                                 source == "trust" ? invalid : fixture("network-trust.json"), {},
                                 [] { return true; });
        updates.check();
        QTRY_COMPARE(updates.state(), QStringLiteral("error"));
        QCOMPARE(updates.errorCode(), errorCode);
        QVERIFY(!updates.trusted());
        QCOMPARE(network.requests.size(), source == "manifest" ? 2 : 3);
    }
    void updateProbeNetworkRecordsARealReply() {
        QTcpServer server;
        QVERIFY(server.listen(QHostAddress::LocalHost));
        connect(&server, &QTcpServer::newConnection, &server, [&] {
            auto *client = server.nextPendingConnection();
            connect(client, &QTcpSocket::readyRead, client, [client] {
                client->readAll();
                client->write("HTTP/1.1 429 Too Many Requests\r\nRetry-After: 60\r\n"
                              "Content-Length: 0\r\nConnection: close\r\n\r\n");
                client->disconnectFromHost();
            });
        });
        UpdateProbeNetwork network;
        network.setProxy(QNetworkProxy::NoProxy);
        const QUrl url(QStringLiteral("http://127.0.0.1:%1/owned").arg(server.serverPort()));
        auto *reply = network.get(QNetworkRequest(url));
        QSignalSpy finished(reply, &QNetworkReply::finished);
        QTRY_COMPARE(finished.count(), 1);
        QTRY_COMPARE(network.evidence.report(QStringLiteral("rate_limited"), false)
                         .value("http_responses")
                         .toArray()
                         .size(),
                     1);
        const auto evidence = network.evidence.report(QStringLiteral("rate_limited"), false);
        QCOMPARE(evidence.value("request_count").toInt(), 1);
        const auto response = evidence.value("http_responses").toArray().first().toObject();
        QCOMPARE(response.value("host").toString(), QStringLiteral("other_allowed_host"));
        QVERIFY2(response.value("status").toInt() != 0, qPrintable(reply->errorString()));
        QCOMPARE(response.value("status").toInt(), 429);
        QCOMPARE(response.value("rate_headers").toObject().value("retry_after").toString(),
                 QStringLiteral("60"));
        reply->deleteLater();
    }
    void updateRequestsReachConfiguredHttpProxy() {
        if (!QSslSocket::supportsSsl())
            QSKIP("The active Qt platform has no TLS backend");
        const QList<QPair<QString, QByteArray>> cases{
            {QString{}, QByteArrayLiteral("CONNECT api.github.com:443 HTTP/1.1")},
            {QStringLiteral("https://mirror.example.test/prefix"),
             QByteArrayLiteral("CONNECT mirror.example.test:443 HTTP/1.1")}};
        for (const auto &scenario : cases) {
            QTcpServer proxy;
            QVERIFY(proxy.listen(QHostAddress::LocalHost));
            QByteArray request;
            connect(&proxy, &QTcpServer::newConnection, &proxy, [&] {
                auto *client = proxy.nextPendingConnection();
                connect(client, &QTcpSocket::readyRead, client, [&, client] {
                    request += client->readAll();
                    if (request.contains("\r\n\r\n")) {
                        client->write("HTTP/1.1 502 Bad Gateway\r\nContent-Length: 0\r\n"
                                      "Connection: close\r\n\r\n");
                        client->disconnectFromHost();
                    }
                });
            });
            QNetworkAccessManager network;
            UpdateController updates(nullptr, &network, fixture("network-trust.json"), {},
                                     [] { return true; });
            updates.configureNetwork(QStringLiteral("custom"),
                                     QStringLiteral("http://127.0.0.1:%1").arg(proxy.serverPort()),
                                     scenario.first);
            updates.check();
            QTRY_VERIFY_WITH_TIMEOUT(request.contains("\r\n\r\n"), 10000);
            QCOMPARE(request.left(request.indexOf("\r\n")), scenario.second);
            QTRY_COMPARE_WITH_TIMEOUT(updates.state(), QStringLiteral("error"), 10000);
            QCOMPARE(updates.errorCode(), QStringLiteral("network"));
            QVERIFY(!updates.trusted());
        }
    }
    void updateProbeDiagnosticsRemainBoundedAndDoNotExposeRequests() {
        UpdateProbeDiagnostics diagnostics;
        QNetworkRequest request{QUrl(api + QStringLiteral("&private=owned-secret"))};
        request.setRawHeader("Authorization", "owned-secret");
        diagnostics.recordRequest(request);
        diagnostics.recordEncrypted();
        Response response{{}, 429};
        response.headers = {{"X-RateLimit-Remaining", "0"},
                            {"X-RateLimit-Reset", "1234"},
                            {"Retry-After", "bad-header"}};
        Reply reply(request, response, nullptr);
        for (int i = 0; i < 33; ++i)
            diagnostics.recordResponse(&reply);
        const auto record = diagnostics.report(QStringLiteral("rate_limited"), false);
        QCOMPARE(record.value("request_count").toInt(), 1);
        QCOMPARE(record.value("encrypted_connections").toInt(), 1);
        QVERIFY(record.value("authorization_present").toBool());
        QVERIFY(record.value("response_records_truncated").toBool());
        QVERIFY(record.value("rate_limit_confirmed").toBool());
        const auto responses = record.value("http_responses").toArray();
        QCOMPARE(responses.size(), 32);
        const auto first = responses.first().toObject();
        QCOMPARE(first.value("host").toString(), QStringLiteral("api.github.com"));
        QCOMPARE(first.value("route").toString(), QStringLiteral("release_list"));
        QCOMPARE(first.value("status").toInt(), 429);
        const auto headers = first.value("rate_headers").toObject();
        QCOMPARE(headers.value("remaining").toString(), QStringLiteral("0"));
        QCOMPARE(headers.value("reset").toString(), QStringLiteral("1234"));
        QCOMPARE(headers.value("retry_after").toString(), QStringLiteral("invalid"));
        QVERIFY(!QJsonDocument(record).toJson().contains("owned-secret"));
    }
    void rateLimitIsNotCurrentVersion_data() {
        QTest::addColumn<int>("status");
        QTest::addColumn<QByteArray>("bytes");
        QTest::addColumn<QByteArray>("remaining");
        QTest::addColumn<QByteArray>("retryAfter");
        QTest::addColumn<QString>("errorCode");
        QTest::newRow("429") << 429 << QByteArray{} << QByteArray{} << QByteArray{}
                             << QStringLiteral("rate_limited");
        QTest::newRow("primary") << 403 << QByteArray{} << QByteArray("0") << QByteArray{}
                                 << QStringLiteral("rate_limited");
        QTest::newRow("retry-after") << 403 << QByteArray{} << QByteArray("42") << QByteArray("60")
                                     << QStringLiteral("rate_limited");
        QTest::newRow("secondary-message")
            << 403 << QByteArray("{\"message\":\"You have exceeded a secondary rate limit.\"}")
            << QByteArray("42") << QByteArray{} << QStringLiteral("rate_limited");
        QTest::newRow("primary-message")
            << 403 << QByteArray("{\"message\":\"API rate limit exceeded.\"}") << QByteArray{}
            << QByteArray{} << QStringLiteral("rate_limited");
        QTest::newRow("forbidden") << 403 << QByteArray("{\"message\":\"Resource not accessible\"}")
                                   << QByteArray("42") << QByteArray{} << QStringLiteral("network");
        QTest::newRow("403-empty")
            << 403 << QByteArray{} << QByteArray{} << QByteArray{} << QStringLiteral("network");
        QTest::newRow("invalid-retry-after") << 403 << QByteArray{} << QByteArray("42")
                                             << QByteArray("invalid") << QStringLiteral("network");
        QTest::newRow("negative-retry-after") << 403 << QByteArray{} << QByteArray("42")
                                              << QByteArray("-1") << QStringLiteral("network");
        QTest::newRow("unrelated-message")
            << 403 << QByteArray("{\"documentation_url\":\"https://example.invalid/rate limits\"}")
            << QByteArray("42") << QByteArray{} << QStringLiteral("network");
        QTest::newRow("message-mentions-documentation")
            << 403 << QByteArray("{\"message\":\"Forbidden; see the rate limit documentation.\"}")
            << QByteArray("42") << QByteArray{} << QStringLiteral("network");
        QTest::newRow("message-denies-rate-limit")
            << 403 << QByteArray("{\"message\":\"This is not a rate limit error.\"}")
            << QByteArray("42") << QByteArray{} << QStringLiteral("network");
        QTest::newRow("signed-retry-after") << 403 << QByteArray{} << QByteArray("42")
                                            << QByteArray("+60") << QStringLiteral("network");
    }
    void rateLimitIsNotCurrentVersion() {
        QFETCH(int, status);
        QFETCH(QByteArray, bytes);
        QFETCH(QByteArray, remaining);
        QFETCH(QByteArray, retryAfter);
        QFETCH(QString, errorCode);
        Network network;
        Response response{bytes, status};
        if (!remaining.isEmpty())
            response.headers.append({"X-RateLimit-Remaining", remaining});
        if (!retryAfter.isEmpty())
            response.headers.append({"Retry-After", retryAfter});
        network.responses.insert(api, response);
        UpdateController updates(nullptr, &network, {}, {}, [] { return true; });
        updates.check();
        QTRY_COMPARE(updates.state(), QStringLiteral("error"));
        QCOMPARE(updates.errorCode(), errorCode);
        QVERIFY(!updates.trusted());
        QCOMPARE(network.requests.size(), 1);
        QVERIFY(!network.requests.constFirst().hasRawHeader("Authorization"));
    }
    void repeatedClicksDoNotStartParallelChecks() {
        Network network;
        configure(network);
        network.responses[api].delay = 30;
        UpdateController updates(nullptr, &network, {}, {}, [] { return true; });
        updates.check();
        updates.check();
        updates.check();
        QCOMPARE(network.requests.size(), 1);
        QTRY_COMPARE(updates.state(), QStringLiteral("available"));
        QCOMPARE(network.requests.size(), 3);
    }
    void unavailableCandidateKeepsManualRecoveryLink() {
        Network network;
        UpdateController updates(nullptr, &network);
        m_openedUrls.clear();
        QDesktopServices::setUrlHandler("https", this, "recordUrl");
        const auto restore = qScopeGuard([] { QDesktopServices::unsetUrlHandler("https"); });
        updates.openReleasePage();
        QCOMPARE(m_openedUrls, QList<QUrl>{QUrl("https://github.com/xiaodaozhi/xdzip/releases")});
    }
    void repeatedInstallWhilePreparingDoesNotOpenManualFallback() {
        Network network;
        configure(network);
        UpdateController updates(nullptr, &network, fixture("network-trust.json"), {},
                                 [] { return true; });
        updates.check();
        QTRY_COMPARE(updates.state(), QStringLiteral("available"));
        m_openedUrls.clear();
        QDesktopServices::setUrlHandler("https", this, "recordUrl");
        const auto restore = qScopeGuard([] { QDesktopServices::unsetUrlHandler("https"); });
        updates.reportNativeProgress(QStringLiteral("preparing"), -1);
        updates.downloadAndInstall();
        updates.downloadAndInstall();
        QCOMPARE(updates.state(), QStringLiteral("preparing"));
        QVERIFY(m_openedUrls.isEmpty());
        QCOMPARE(network.requests.size(), 3);
    }
    void redirectToUntrustedHostIsRejected() {
        Network network;
        Response response;
        response.redirect = QUrl(QStringLiteral("https://example.invalid/package"));
        network.responses.insert(api, response);
        UpdateController updates(nullptr, &network, {}, {}, [] { return true; });
        updates.check();
        QTRY_COMPARE(updates.state(), QStringLiteral("error"));
        QVERIFY(!updates.trusted());
        QCOMPARE(network.requests.size(), 1);
    }
    void truncatedDownloadIsRejected() {
        QTemporaryDir directory(QDir::tempPath() + "/xdzip-update-network-XXXXXX");
        Network network;
        configure(network);
        for (auto it = network.responses.begin(); it != network.responses.end(); ++it)
            if (!it.key().endsWith("release.json") && !it.key().endsWith("release.json.sig") &&
                it.key() != api)
                it->bytes = "dat";
        UpdateController updates(nullptr, &network, fixture("network-trust.json"), directory.path(),
                                 [] { return true; });
        updates.check();
        QTRY_COMPARE(updates.state(), QStringLiteral("available"));
        updates.download();
        QTRY_COMPARE(updates.state(), QStringLiteral("error"));
        QVERIFY(!updates.downloaded());
    }
    void cancellationDoesNotCommitPartialDownload() {
        QTemporaryDir directory(QDir::tempPath() + "/xdzip-update-network-XXXXXX");
        Network network;
        configure(network);
        for (auto it = network.responses.begin(); it != network.responses.end(); ++it)
            if (!it.key().endsWith("release.json") && !it.key().endsWith("release.json.sig") &&
                it.key() != api)
                it->delay = 50;
        UpdateController updates(nullptr, &network, fixture("network-trust.json"), directory.path(),
                                 [] { return true; });
        updates.check();
        QTRY_COMPARE(updates.state(), QStringLiteral("available"));
        updates.download();
        updates.cancel();
        QTest::qWait(80);
        QCOMPARE(updates.state(), QStringLiteral("available"));
        QVERIFY(!updates.downloaded());
    }
    void unwritableCacheDoesNotStartDownload() {
        QTemporaryDir directory(QDir::tempPath() + "/xdzip-update-network-XXXXXX");
        QFile file(directory.filePath("file"));
        QVERIFY(file.open(QIODevice::WriteOnly));
        file.close();
        Network network;
        configure(network);
        UpdateController updates(nullptr, &network, fixture("network-trust.json"), file.fileName(),
                                 [] { return true; });
        updates.check();
        QTRY_COMPARE(updates.state(), QStringLiteral("available"));
        updates.download();
        QCOMPARE(updates.state(), QStringLiteral("error"));
        QCOMPARE(updates.errorCode(), QStringLiteral("cache"));
        QCOMPARE(network.requests.size(), 3);
    }
    void networkFailureIsNotCurrentVersion_data() {
        QTest::addColumn<int>("networkError");
        QTest::addColumn<QString>("errorCode");
        QTest::newRow("offline") << int(QNetworkReply::HostNotFoundError)
                                 << QStringLiteral("network");
        QTest::newRow("timeout") << int(QNetworkReply::TimeoutError) << QStringLiteral("timeout");
        QTest::newRow("tls") << int(QNetworkReply::SslHandshakeFailedError)
                             << QStringLiteral("tls");
    }
    void networkFailureIsNotCurrentVersion() {
        QFETCH(int, networkError);
        QFETCH(QString, errorCode);
        Network network;
        network.responses.insert(api, {{}, 0, QNetworkReply::NetworkError(networkError)});
        UpdateController updates(nullptr, &network, {}, {}, [] { return true; });
        updates.check();
        QTRY_COMPARE(updates.state(), QStringLiteral("error"));
        QCOMPARE(updates.errorCode(), errorCode);
        QVERIFY(!updates.busy());
        QVERIFY(!updates.canInstall());
    }
    void metadataDeadlineStopsUnresponsiveServer() {
        Network network;
        network.responses.insert(api, {{}, 200, QNetworkReply::NoError, 60000});
        UpdateController updates(nullptr, &network, {}, {}, [] { return true; });
        updates.check();
        // Exercise the real production deadline without shortening its timer.
        QTRY_COMPARE_WITH_TIMEOUT(updates.state(), QStringLiteral("error"), 32000);
        QCOMPARE(updates.errorCode(), QStringLiteral("timeout"));
        QCOMPARE(network.requests.size(), 1);
        QVERIFY(!updates.busy());
        QVERIFY(!updates.canInstall());
    }
    void cancellationRejectsLateReplies() {
        Network network;
        configure(network);
        network.responses[api].delay = 50;
        UpdateController updates(nullptr, &network, {}, {}, [] { return true; });
        updates.check();
        updates.cancel();
        QTest::qWait(80);
        QCOMPARE(updates.state(), QStringLiteral("idle"));
        QVERIFY(updates.candidateVersion().isEmpty());
        QCOMPARE(network.requests.size(), 1);
    }
    void oversizedResponseIsRejected() {
        Network network;
        network.responses.insert(api, {QByteArray(8 * 1024 * 1024 + 1, 'x')});
        UpdateController updates(nullptr, &network, {}, {}, [] { return true; });
        updates.check();
        QTRY_COMPARE(updates.state(), QStringLiteral("error"));
        QCOMPARE(updates.errorCode(), QStringLiteral("response_limit"));
    }
    void paginationAndConditionalRequests() {
        Network network;
        configure(network);
        QJsonArray page;
        for (int index = 0; index < 100; ++index)
            page.append(QJsonObject{{"draft", true}});
        network.responses[api].bytes = QJsonDocument(page).toJson();
        const auto page2 = api.left(api.size() - 1) + '2';
        network.responses.insert(page2, {QJsonDocument(QJsonArray{release(true)}).toJson()});
        UpdateController updates(nullptr, &network, fixture("network-trust.json"), {},
                                 [] { return true; });
        updates.check();
        QTRY_COMPARE(updates.state(), QStringLiteral("available"));
        QCOMPARE(network.requests.size(), 4);
        for (auto it = network.responses.begin(); it != network.responses.end(); ++it) {
            it->status = 304;
            it->bytes.clear();
        }
        updates.check();
        QTRY_COMPARE(updates.state(), QStringLiteral("available"));
        QVERIFY(updates.trusted());
        QCOMPARE(network.requests.size(), 8);
        for (int index = 4; index < 8; ++index)
            QCOMPARE(network.requests[index].rawHeader("If-None-Match"),
                     QByteArrayLiteral("\"fixture\""));
    }
};
QTEST_MAIN(UpdateControllerTest)
#include "update_controller_test.moc"
