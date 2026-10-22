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
#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QJsonDocument>
#include <QJsonObject>
#include <QNetworkAccessManager>
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

namespace {
bool save(const QString &path, const QJsonObject &value) {
    QSaveFile file(path);
    const auto bytes = QJsonDocument(value).toJson();
    return file.open(QIODevice::WriteOnly) && file.write(bytes) == bytes.size() && file.commit();
}
int httpsProbe(QApplication &application, const QString &directory) {
    qputenv("XDZIP_SETTINGS_PATH", (directory + "/https-settings.ini").toUtf8());
    QNetworkAccessManager network;
    UpdateController updates(nullptr, &network);
    int encrypted = 0, httpsResponses = 0;
    QObject::connect(&network, &QNetworkAccessManager::encrypted, &application,
                     [&](QNetworkReply *) { ++encrypted; });
    QObject::connect(
        &network, &QNetworkAccessManager::finished, &application, [&](QNetworkReply *reply) {
            if (reply->url().scheme() == "https" && reply->error() == QNetworkReply::NoError &&
                reply->attribute(QNetworkRequest::HttpStatusCodeAttribute).toInt() == 200)
                ++httpsResponses;
        });
    const auto finish = [&](bool timeout) {
        const bool passed = !timeout && QSslSocket::supportsSsl() && encrypted > 0 &&
                            httpsResponses > 0 && updates.errorCode().isEmpty() &&
                            (updates.state() == "available" || updates.state() == "up_to_date");
        const bool written =
            save(directory + "/https.json", {{"success", passed},
                                             {"ssl_available", QSslSocket::supportsSsl()},
                                             {"ssl_backend", QSslSocket::activeBackend()},
                                             {"ssl_library", QSslSocket::sslLibraryVersionString()},
                                             {"state", updates.state()},
                                             {"error", updates.errorCode()},
                                             {"timeout", timeout},
                                             {"encrypted_connections", encrypted},
                                             {"https_responses", httpsResponses}});
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
            if (releaseExitGuard)
                releaseExitGuard();
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
        save(m_directory + "/result.json",
             {{"success", success}, {"error", error}, {"events", m_events}});
        if (releaseExitGuard)
            releaseExitGuard();
        QTimer::singleShot(0, qApp, &QCoreApplication::quit);
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
                             evidence.value("barrier_before_install").toBool();
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
    return application.exec();
}

#include "mac_update_probe.moc"
