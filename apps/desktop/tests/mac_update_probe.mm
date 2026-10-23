// apps/desktop/tests/mac_update_probe.mm

#include "AppQuitGuard.h"
#include "ApplicationSessions.h"
#include "BrowserController.h"
#include "MacUpdateProvider.h"
#include "UiFont.h"
#include "UiPreferences.h"
#include "UiText.h"
#include "UpdateController.h"
#import <Foundation/Foundation.h>
#include <QApplication>
#include <QDateTime>
#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QNetworkAccessManager>
#include <QNetworkProxy>
#include <QNetworkProxyFactory>
#include <QNetworkReply>
#include <QNetworkRequest>
#include <QQmlApplicationEngine>
#include <QQmlComponent>
#include <QQuickItem>
#include <QQuickStyle>
#include <QQuickWindow>
#include <QSaveFile>
#include <QSslSocket>
#include <QTest>
#include <QTimer>
#include <algorithm>
#import <mach-o/dyld.h>

namespace {
bool save(const QString &path, const QJsonObject &value) {
    QSaveFile file(path);
    const auto bytes = QJsonDocument(value).toJson();
    return file.open(QIODevice::WriteOnly) && file.write(bytes) == bytes.size() && file.commit();
}
class HttpsEvidenceNetwork final : public QNetworkAccessManager {
  public:
    int requestCount = 0;
    bool authorizationPresent = false;

  protected:
    QNetworkReply *createRequest(Operation operation, const QNetworkRequest &request,
                                 QIODevice *outgoingData = nullptr) override {
        ++requestCount;
        authorizationPresent = authorizationPresent || request.hasRawHeader("Authorization");
        return QNetworkAccessManager::createRequest(operation, request, outgoingData);
    }
};

QJsonValue numericHeader(QNetworkReply *reply, const char *name) {
    if (!reply->hasRawHeader(name))
        return QJsonValue::Null;
    const auto value = reply->rawHeader(name).trimmed();
    if (value.isEmpty() || value.size() > 20 ||
        !std::all_of(value.cbegin(), value.cend(),
                     [](char digit) { return digit >= '0' && digit <= '9'; }))
        return QStringLiteral("invalid");
    return QString::fromLatin1(value);
}

int httpsProbe(QApplication &application, const QString &directory) {
    qputenv("XDZIP_SETTINGS_PATH", (directory + "/https-settings.ini").toUtf8());
    HttpsEvidenceNetwork network;
    UpdateController updates(nullptr, &network);
    int encrypted = 0, httpsResponses = 0;
    QJsonArray responses;
    bool responsesTruncated = false, completed = false;
    QObject::connect(&network, &QNetworkAccessManager::encrypted, &application,
                     [&](QNetworkReply *) { ++encrypted; });
    QObject::connect(
        &network, &QNetworkAccessManager::finished, &application, [&](QNetworkReply *reply) {
            const int status = reply->attribute(QNetworkRequest::HttpStatusCodeAttribute).toInt();
            if (reply->url().scheme() == "https" && reply->error() == QNetworkReply::NoError &&
                status == 200)
                ++httpsResponses;
            if (responses.size() >= 32) {
                responsesTruncated = true;
                return;
            }
            const auto host = reply->url().host().toLower();
            const bool githubApi = host == QStringLiteral("api.github.com");
            QJsonObject headers{{"limit", numericHeader(reply, "x-ratelimit-limit")},
                                {"remaining", numericHeader(reply, "x-ratelimit-remaining")},
                                {"used", numericHeader(reply, "x-ratelimit-used")},
                                {"reset", numericHeader(reply, "x-ratelimit-reset")},
                                {"retry_after", numericHeader(reply, "retry-after")}};
            const auto resource = reply->rawHeader("x-ratelimit-resource").trimmed();
            const bool safeResource =
                resource.size() <= 32 &&
                std::all_of(resource.cbegin(), resource.cend(), [](char value) {
                    return (value >= 'a' && value <= 'z') || value == '_' || value == '-';
                });
            headers.insert("resource", safeResource ? QString::fromLatin1(resource)
                                                    : QStringLiteral("invalid"));
            const auto date = QDateTime::fromString(QString::fromLatin1(reply->rawHeader("date")),
                                                    Qt::RFC2822Date);
            const QString category = reply->error() == QNetworkReply::SslHandshakeFailedError
                                         ? "tls"
                                     : reply->error() == QNetworkReply::TimeoutError ? "timeout"
                                     : status >= 400                                 ? "http_error"
                                     : reply->error() != QNetworkReply::NoError ? "network_error"
                                                                                : "none";
            // Only fixed public host/route categories and bounded numeric
            // response headers are retained. No body, URL/IP or credentials.
            responses.append(QJsonObject{
                {"host", githubApi ? QStringLiteral("api.github.com")
                                   : QStringLiteral("other_allowed_host")},
                {"route", githubApi && reply->url().path() == "/repos/xiaodaozhi/xdzip/releases"
                              ? QStringLiteral("release_list")
                              : QStringLiteral("other")},
                {"status", status},
                {"network_error", int(reply->error())},
                {"error_category", category},
                {"rate_headers", headers},
                {"server_date_epoch", date.isValid() ? QJsonValue(date.toSecsSinceEpoch())
                                                     : QJsonValue(QJsonValue::Null)}});
        });
    const auto finish = [&](bool timeout) {
        if (completed)
            return;
        completed = true;
        QJsonArray bundledQtImages, externalQtImages;
        const auto bundleRoot =
            QFileInfo(QString::fromUtf8(NSBundle.mainBundle.bundlePath.UTF8String))
                .canonicalFilePath();
        for (uint32_t index = 0; index < _dyld_image_count(); ++index) {
            const auto image = QFileInfo(QString::fromUtf8(_dyld_get_image_name(index)));
            // Shared-cache system images may have no filesystem entry. Check
            // normalized dyld paths before the Qt name heuristic, which also
            // matches Apple's libquarantine.dylib.
            const auto imagePath = QDir::cleanPath(image.filePath());
            if (imagePath.startsWith("/usr/lib/") || imagePath.startsWith("/System/Library/"))
                continue;
            const auto name = image.fileName();
            if (!((name.startsWith("Qt") && image.filePath().contains(".framework/")) ||
                  (name.startsWith("libq") && name.endsWith(".dylib"))))
                continue;
            const auto path = image.canonicalFilePath();
            if (!bundleRoot.isEmpty() && path.startsWith(bundleRoot + '/'))
                bundledQtImages.append(path.mid(bundleRoot.size() + 1));
            else
                externalQtImages.append(name);
        }
        const bool passed = !timeout && QSslSocket::supportsSsl() && encrypted > 0 &&
                            httpsResponses > 0 && updates.errorCode().isEmpty() &&
                            !responsesTruncated && !network.authorizationPresent &&
                            !bundledQtImages.isEmpty() && externalQtImages.isEmpty() &&
                            (updates.state() == "available" || updates.state() == "up_to_date");
        const bool written =
            save(directory + "/https.json",
                 {{"success", passed},
                  {"ssl_available", QSslSocket::supportsSsl()},
                  {"ssl_backend", QSslSocket::activeBackend()},
                  {"ssl_library", QSslSocket::sslLibraryVersionString()},
                  {"state", updates.state()},
                  {"error", updates.errorCode()},
                  {"timeout", timeout},
                  {"encrypted_connections", encrypted},
                  {"https_responses", httpsResponses},
                  {"http_diagnostics_schema", 1},
                  {"request_count", network.requestCount},
                  {"http_responses", responses},
                  {"response_records_truncated", responsesTruncated},
                  {"authorization_present", network.authorizationPresent},
                  {"rate_limit_confirmed", updates.errorCode() == "rate_limited"},
                  {"confirmed_from_controller", true},
                  {"system_proxy_configuration", QNetworkProxyFactory::usesSystemConfiguration()},
                  {"proxy_type", int(network.proxy().type())},
                  {"proxy_environment_present", qEnvironmentVariableIsSet("HTTPS_PROXY") ||
                                                    qEnvironmentVariableIsSet("HTTP_PROXY") ||
                                                    qEnvironmentVariableIsSet("ALL_PROXY") ||
                                                    qEnvironmentVariableIsSet("https_proxy") ||
                                                    qEnvironmentVariableIsSet("http_proxy") ||
                                                    qEnvironmentVariableIsSet("all_proxy")},
                  {"bundled_qt_images", bundledQtImages},
                  {"external_qt_images", externalQtImages}});
        application.exit(passed && written ? 0 : 69);
    };
    QObject::connect(&updates, &UpdateController::changed, &application, [&] {
        if (!updates.busy() && updates.state() != "idle")
            // The network manager's finished handlers must all run before the
            // final evidence counts are read, regardless of connection order.
            QTimer::singleShot(0, &application, [&] { finish(false); });
    });
    QTimer::singleShot(180000, &application, [&] { finish(true); });
    QTimer::singleShot(0, &updates, &UpdateController::check);
    return application.exec();
}
} // namespace

// This isolated fixture supplies a loopback candidate to the native provider.
// It uses actual About controls, session joins and Sparkle's signed update path.
class MacProbeUpdate final : public QObject {
    Q_OBJECT
    Q_PROPERTY(QString state READ state NOTIFY changed)
    Q_PROPERTY(QString errorCode READ errorCode NOTIFY changed)
    Q_PROPERTY(QString candidateVersion READ candidateVersion CONSTANT)
    Q_PROPERTY(qreal progress READ progress NOTIFY changed)
    Q_PROPERTY(bool busy READ busy NOTIFY changed)
    Q_PROPERTY(bool trusted READ trusted CONSTANT)
    Q_PROPERTY(bool canInstall READ trusted CONSTANT)
    Q_PROPERTY(bool downloaded READ downloaded CONSTANT)
  public:
    MacProbeUpdate(UpdateController *updates, ApplicationSessions *sessions, QJsonObject candidate,
                   QString directory, bool cancel)
        : m_updates(updates), m_sessions(sessions), m_candidate(std::move(candidate)),
          m_directory(std::move(directory)), m_cancel(cancel) {
        connect(updates, &UpdateController::changed, this, [this] {
            emit changed();
            if (state() == "error")
                finish(false, m_updates->errorCode());
        });
        connect(sessions, &ApplicationSessions::readyForUpdate, this, [this] {
            m_events["barrier_before_install"] = true;
            save(m_directory + "/events.json", m_events);
            if (!commitMacUpdate())
                finish(false, "commit_failed");
        });
    }
    QString state() const {
        return m_updates->state();
    }
    QString errorCode() const {
        return m_updates->errorCode();
    }
    QString candidateVersion() const {
        return m_candidate.value("version").toString();
    }
    qreal progress() const {
        return m_updates->progress();
    }
    bool busy() const {
        return m_updates->busy();
    }
    bool trusted() const {
        return true;
    }
    bool downloaded() const {
        return false;
    }
    std::function<void()> cancelControl;
    std::function<void()> releaseExitGuard;
    void releaseCommittedQuitGuard() {
        m_events["guard_release_after_commit"] = macUpdateInstallationCommitted();
        m_events["guard_release_after_join"] = m_events.value("barrier_before_install").toBool();
        save(m_directory + "/events.json", m_events);
        if (releaseExitGuard)
            releaseExitGuard();
    }
    Q_INVOKABLE void check() {
        m_updates->reportNativeProgress("available", -1);
    }
    Q_INVOKABLE void downloadAndInstall() {
        m_events["about_install_clicked"] = true;
        save(m_directory + "/events.json", m_events);
        startMacUpdate(m_updates, m_candidate, [this](QString error) {
            if (!error.isEmpty()) {
                finish(false, error);
                return;
            }
            if (m_cancel) {
                if (cancelControl)
                    cancelControl();
                else
                    finish(false, "cancel_button_missing");
                return;
            }
            if (!m_sessions->prepareForUpdate()) {
                cancelMacUpdate();
                finish(false, "busy");
            }
        });
    }
    Q_INVOKABLE void download() {
        downloadAndInstall();
    }
    Q_INVOKABLE void cancel() {
        m_events["about_cancel_clicked"] = true;
        save(m_directory + "/events.json", m_events);
        cancelMacUpdate();
        finish(false, "cancelled");
    }
    Q_INVOKABLE void openReleasePage() {
        finish(false, "unexpected_fallback");
    }
    Q_INVOKABLE void openDownloadFolder() {}
    void finish(bool success, const QString &error) {
        if (m_finished)
            return;
        m_finished = true;
        const auto quit = [this, success, error] {
            m_events["native_cycle_settled_before_exit"] = true;
            m_events["native_exit_status"] = macUpdateExitStatus();
            save(m_directory + "/events.json", m_events);
            save(m_directory + "/result.json",
                 {{"success", success}, {"error", error}, {"events", m_events}});
            if (releaseExitGuard)
                releaseExitGuard();
            QTimer::singleShot(0, qApp, &QCoreApplication::quit);
        };
        const bool deferred = deferMacUpdateQuit(quit);
        m_events["native_quit_deferred"] = deferred;
        if (!deferred)
            quit();
    }
  signals:
    void changed();

  private:
    UpdateController *m_updates;
    ApplicationSessions *m_sessions;
    QJsonObject m_candidate, m_events;
    QString m_directory;
    bool m_cancel, m_finished = false;
};

int main(int argc, char **argv) {
    QApplication application(argc, argv);
    QApplication::setQuitOnLastWindowClosed(false);
    const auto bundle = NSBundle.mainBundle;
    const auto path = QString::fromUtf8(bundle.bundleURL.path.UTF8String);
    if (![bundle.bundleIdentifier isEqualToString:@"org.xdzip.update-regression"] ||
        !path.contains("/build/"))
        return 60;
    const auto directory = QFileInfo(path).absolutePath();
    Q_INIT_RESOURCE(ui);
    if (application.arguments().contains("--update-https-probe"))
        return httpsProbe(application, directory);
    QFile configuration(directory + "/probe.json");
    if (!configuration.open(QIODevice::ReadOnly) || configuration.size() > 65536)
        return 61;
    const auto config = QJsonDocument::fromJson(configuration.readAll()).object();
    if (config.value("marker") != "xdzip-owned-mac-update-regression-v1")
        return 62;
    const auto nativeBuild = [[bundle objectForInfoDictionaryKey:@"CFBundleVersion"] intValue];
    if (nativeBuild == config.value("target_build").toInt()) {
        QFile events(directory + "/events.json");
        if (!events.open(QIODevice::ReadOnly))
            return 63;
        const auto evidence = QJsonDocument::fromJson(events.readAll()).object();
        const bool success = evidence.value("about_install_clicked").toBool() &&
                             evidence.value("barrier_before_install").toBool() &&
                             evidence.value("guard_release_after_commit").toBool() &&
                             evidence.value("guard_release_after_join").toBool();
        return save(directory + "/result.json", {{"success", success},
                                                 {"relaunched_build", nativeBuild},
                                                 {"events", evidence}}) &&
                       success
                   ? 0
                   : 64;
    }
    qputenv("XDZIP_SETTINGS_PATH", (directory + "/settings.ini").toUtf8());
    QQuickStyle::setStyle("Basic");
    qmlRegisterType<BrowserController>("Xdzip.Native", 1, 0, "BrowserController");
    if (!initializeUiText() || !installUiFont())
        return 65;
    QQmlApplicationEngine engine;
    engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
    if (engine.rootObjects().isEmpty())
        return 66;
    auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
    auto *settings = engine.singletonInstance<UiPreferences *>("Xdzip.Settings", "AppSettings");
    ApplicationSessions sessions(&engine, settings);
    sessions.adopt(window);
    sessions.newSession();
    if (sessions.sessionCount() != 2)
        return 68;
    QNetworkAccessManager network;
    UpdateController updates(nullptr, &network, {}, directory + "/cache");
    MacProbeUpdate facade(&updates, &sessions, config.value("candidate").toObject(), directory,
                          config.value("mode") == "cancel");
    AppQuitGuard guard;
    guard.controller = window->findChild<BrowserController *>();
    guard.window = window;
    guard.quitHandler = [&] { sessions.requestQuit(); };
    application.installEventFilter(&guard);
    facade.releaseExitGuard = [&] {
        guard.quitHandler = {};
        guard.controller = nullptr;
        for (auto *sessionWindow : sessions.windows())
            sessionWindow->setProperty("allowManagedClose", true);
    };
    setMacUpdateCommitCallback(&guard, [&] { facade.releaseCommittedQuitGuard(); });
    QObject::connect(&updates, &UpdateController::quitForUpdate, &application, &QApplication::quit);
    QQmlComponent component(&engine, QUrl("qrc:/xdzip/ui/AboutWindow.qml"));
    std::unique_ptr<QObject> aboutObject(component.createWithInitialProperties(
        {{"owner", QVariant::fromValue(window)}, {"updater", QVariant::fromValue(&facade)}}));
    auto *about = qobject_cast<QQuickWindow *>(aboutObject.get());
    if (!about)
        return 67;
    facade.cancelControl = [&] {
        auto *button = about->findChild<QQuickItem *>("aboutCancelUpdate");
        if (!button || !button->isVisible() || !button->isEnabled()) {
            facade.finish(false, "cancel_button_missing");
            return;
        }
        button->forceActiveFocus();
        QTest::qWait(100);
        QTest::mouseClick(
            about, Qt::LeftButton, Qt::NoModifier,
            button->mapToScene(QPointF(button->width() / 2, button->height() / 2)).toPoint());
    };
    window->show();
    about->show();
    QTimer::singleShot(0, &facade, [&] {
        if (!QTest::qWaitForWindowExposed(about)) {
            facade.finish(false, "not_exposed");
            return;
        }
        facade.check();
        auto *button = about->findChild<QQuickItem *>("aboutInstallUpdate");
        if (!button) {
            facade.finish(false, "about_button_missing");
            return;
        }
        button->forceActiveFocus();
        QTest::qWait(100);
        QTest::mouseClick(
            about, Qt::LeftButton, Qt::NoModifier,
            button->mapToScene(QPointF(button->width() / 2, button->height() / 2)).toPoint());
    });
    QTimer::singleShot(120000, &facade, [&] {
        cancelMacUpdate();
        facade.finish(false, "timeout");
    });
    const auto result = application.exec();
    setMacUpdateCommitCallback(nullptr, {});
    return result;
}

#include "mac_update_probe.moc"
