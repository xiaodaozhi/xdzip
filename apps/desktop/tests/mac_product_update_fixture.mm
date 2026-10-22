// apps/desktop/tests/mac_product_update_fixture.mm

// Private build instrumentation. Normal product targets never compile this file.
#include "mac_product_update_fixture.h"
#include "ApplicationSessions.h"
#include "BuildInfo.h"
#include "MacUpdateProvider.h"
#include "UiPreferences.h"
#include "UiText.h"
#include "UpdateController.h"
#import <Foundation/Foundation.h>
#include <QCoreApplication>
#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QGuiApplication>
#include <QJsonArray>
#include <QJsonDocument>
#include <QNetworkAccessManager>
#include <QNetworkReply>
#include <QNetworkRequest>
#include <QQuickItem>
#include <QQuickWindow>
#include <QSaveFile>
#include <QScopedValueRollback>
#include <QSettings>
#include <QSignalSpy>
#include <QTimer>
#include <QUrl>
#include <QtTest/QTest>
#include <memory>

namespace {
QString directory() {
    return QStringLiteral(XDZIP_NATIVE_MAC_UPDATE_FIXTURE_DIRECTORY);
}
bool oldProduct() {
    return BuildInfo::buildNumber() == 200;
}
bool write(const QString &name, const QJsonObject &value) {
    QSaveFile file(directory() + '/' + name);
    const auto bytes = QJsonDocument(value).toJson();
    return file.open(QIODevice::WriteOnly) && file.write(bytes) == bytes.size() && file.commit();
}
QJsonObject configuration() {
    QFile file(directory() + "/configuration.json");
    if (QFileInfo(file).isSymLink() || !file.open(QIODevice::ReadOnly) || file.size() > 65536)
        return {};
    const auto value = QJsonDocument::fromJson(file.readAll()).object();
    if (value.value("marker") != "xdzip-owned-mac-product-update-v1")
        return {};
    return value;
}
void failure(const QString &error) {
    write("fixture-error.json", {{"error", error}});
    QTimer::singleShot(0, qApp, [] { QCoreApplication::exit(50); });
}
class Reply final : public QNetworkReply {
    QFile m_file;

  public:
    Reply(const QNetworkRequest &request, const QString &path, QObject *parent)
        : QNetworkReply(parent), m_file(path) {
        setRequest(request);
        setUrl(request.url());
        const bool found = !QFileInfo(path).isSymLink() && m_file.open(QIODevice::ReadOnly);
        setAttribute(QNetworkRequest::HttpStatusCodeAttribute, found ? 200 : 404);
        if (!found)
            setError(ContentNotFoundError, "Owned fixture route missing");
        else
            setHeader(QNetworkRequest::ContentLengthHeader, m_file.size());
        open(QIODevice::ReadOnly | QIODevice::Unbuffered);
        QTimer::singleShot(0, this, [this] {
            emit readyRead();
            if (!isFinished()) {
                setFinished(true);
                emit finished();
            }
        });
    }
    qint64 bytesAvailable() const override {
        return m_file.bytesAvailable() + QNetworkReply::bytesAvailable();
    }
    void abort() override {
        if (isFinished())
            return;
        setError(OperationCanceledError, "Owned fixture cancelled");
        setFinished(true);
        emit finished();
    }

  protected:
    qint64 readData(char *bytes, qint64 maximum) override {
        return m_file.isOpen() ? m_file.read(bytes, maximum) : -1;
    }
};
class Network final : public QNetworkAccessManager {
  public:
    explicit Network(QObject *parent) : QNetworkAccessManager(parent) {}

  protected:
    QNetworkReply *createRequest(Operation operation, const QNetworkRequest &request,
                                 QIODevice *data) override {
        Q_UNUSED(data);
        const auto config = configuration();
        const auto name =
            operation == GetOperation
                ? config.value("routes").toObject().value(request.url().toString()).toString()
                : QString{};
        const bool safe = !name.isEmpty() && QFileInfo(name).fileName() == name &&
                          !name.contains('\\') && name != "." && name != "..";
        return new Reply(request, safe ? directory() + '/' + name : QString{}, this);
    }
};
QJsonObject snapshot(UpdateController *updates, UiPreferences *settings) {
    int windows = 0;
    for (auto *window : QGuiApplication::allWindows())
        if (window->isVisible() && window->property("managedWindow").toBool())
            ++windows;
    QSettings profile(uiSettingsPath(), QSettings::IniFormat);
    return {{"version", BuildInfo::version()},
            {"build", BuildInfo::buildNumber()},
            {"revision", BuildInfo::revision()},
            {"configuration", BuildInfo::configuration()},
            {"dirty", BuildInfo::dirty()},
            {"pid", QCoreApplication::applicationPid()},
            {"primary_windows", windows},
            {"state", updates->state()},
            {"error", updates->errorCode()},
            {"trusted", updates->trusted()},
            {"can_install", updates->canInstall()},
            {"install_root", macUpdateInstallRoot()},
            {"settings_path", uiSettingsPath()},
            {"settings_override", qEnvironmentVariable("XDZIP_SETTINGS_PATH")},
            {"home", QDir::homePath()},
            {"theme", settings->theme()},
            {"text_scale", settings->textScale()},
            {"density", settings->density()},
            {"language", settings->language()},
            {"active_language", settings->activeLanguage()},
            {"unknown_key", profile.value("ownedFixture/unknownKey").toString()}};
}
bool click(QQuickWindow *window, QQuickItem *item) {
    if (!window || !window->isExposed() || !item || !item->isVisible() || !item->isEnabled())
        return false;
    item->forceActiveFocus();
    QTest::qWait(100);
    const auto point = item->mapToScene(QPointF(item->width() / 2, item->height() / 2)).toPoint();
    if (!QRect(0, 0, window->width(), window->height()).contains(point))
        return false;
    QSignalSpy clicked(item, SIGNAL(clicked()));
    QTest::mouseClick(window, Qt::LeftButton, Qt::NoModifier, point);
    return clicked.isValid() && clicked.count() == 1;
}
} // namespace

void macProductUpdateFixtureEvent(const QString &stage, const QJsonObject &details) {
    if (stage.isEmpty() || stage != QFileInfo(stage).fileName() || stage.contains('\\'))
        return;
    const auto bounded = [](const auto &recurse, const QJsonValue &value, int depth) -> QJsonValue {
        if (depth > 5)
            return QStringLiteral("bounded");
        if (value.isString()) {
            auto text = value.toString();
            text.replace(QDir::currentPath(), QStringLiteral("$WORKSPACE"));
            text.replace(QDir::homePath(), QStringLiteral("$HOME"));
            return text.left(1024);
        }
        if (value.isObject()) {
            QJsonObject result;
            const auto input = value.toObject();
            for (auto it = input.begin(); it != input.end() && result.size() < 64; ++it)
                result.insert(it.key(), recurse(recurse, it.value(), depth + 1));
            return result;
        }
        if (value.isArray()) {
            QJsonArray result;
            for (const auto &entry : value.toArray()) {
                if (result.size() == 64)
                    break;
                result.append(recurse(recurse, entry, depth + 1));
            }
            return result;
        }
        return value;
    };
    write("native-" + stage + ".json", bounded(bounded, details, 0).toObject());
}

QNetworkAccessManager *macProductUpdateFixtureTransport(QObject *parent) {
    if (!qEnvironmentVariableIsEmpty("XDZIP_SETTINGS_PATH") || configuration().isEmpty()) {
        failure("fixture_configuration_or_profile_override");
        return nullptr;
    }
    if (!oldProduct())
        return nullptr; // Normal constructor must restore the real persisted receipt.
    return new Network(parent);
}
QString macProductUpdateFixtureFeed(const QJsonObject &candidate) {
    const auto config = configuration();
    const auto target = config.value("loopback_appcast_url").toString();
    const QUrl url(target);
    if (!oldProduct() || candidate.value("appcast_url") != config.value("fixed_appcast_url") ||
        candidate.value("asset_url") != config.value("fixed_asset_url") ||
        candidate.value("version") != "0.2.0-alpha.2" || candidate.value("build").toInt() != 201 ||
        url.scheme() != "http" || url.host() != "127.0.0.1" || !url.path().startsWith("/owned/"))
        return {};
    write("feed-redirect.json", {{"fixed_appcast_url", candidate.value("appcast_url")},
                                 {"loopback_appcast_url", target}});
    return target;
}
QString macProductUpdateFixtureAsset(const QJsonObject &candidate, const QString &requestUrl) {
    const auto config = configuration();
    const auto target = config.value("loopback_asset_url").toString();
    const QUrl url(target);
    if (!oldProduct() || candidate.value("appcast_url") != config.value("fixed_appcast_url") ||
        candidate.value("asset_url") != config.value("fixed_asset_url") ||
        requestUrl != candidate.value("asset_url").toString() ||
        candidate.value("version") != "0.2.0-alpha.2" || candidate.value("build").toInt() != 201 ||
        url.scheme() != "http" || url.host() != "127.0.0.1" || url.port() <= 0 ||
        !url.userInfo().isEmpty() || url.hasQuery() || url.hasFragment() ||
        url.path() != "/owned/" + QUrl(requestUrl).fileName()) {
        failure("authenticated_asset_request_mapping");
        return {};
    }
    write("asset-redirect.json", {{"fixed_asset_url", requestUrl},
                                  {"loopback_asset_url", target},
                                  {"transport", "Sparkle public willDownloadUpdate delegate"}});
    return target;
}
void installMacProductUpdateFixture(UpdateController *updates, ApplicationSessions *sessions,
                                    UiPreferences *settings, QQuickWindow *window) {
    const auto config = configuration();
    const auto installed = macUpdateInstallRoot();
    if (!sessions || !settings || config.isEmpty() || installed.isEmpty() ||
        installed != config.value("install_root").toString() ||
        !qEnvironmentVariableIsEmpty("XDZIP_SETTINGS_PATH")) {
        failure("real_install_identity_or_profile");
        return;
    }
    struct State {
        bool started = false, check = false, install = false, clicking = false, stopping = false;
    };
    const auto state = std::make_shared<State>();
    const auto prefix = QString::number(BuildInfo::buildNumber());
    QObject::connect(updates, &UpdateController::exitBarrierReached, sessions, [=] {
        auto record = snapshot(updates, settings);
        record.insert("update_pending", sessions->updatePending());
        record.insert("active_tasks", sessions->activeTaskCount());
        record.insert("about_check_clicked", state->check);
        record.insert("about_install_clicked", state->install);
        QSettings pending(QFileInfo(uiSettingsPath()).dir().filePath("update-state.ini"),
                          QSettings::IniFormat);
        record.insert(
            "pending",
            QJsonDocument::fromJson(pending.value("update/pending").toByteArray()).object());
        write(prefix + "-exit-barrier.json", record);
    });
    auto *timer = new QTimer(sessions);
    timer->setInterval(100);
    QObject::connect(timer, &QTimer::timeout, sessions, [=] {
        if (state->clicking)
            return;
        if (!state->started && window->isVisible()) {
            if (oldProduct()) {
                if (QFileInfo::exists(uiSettingsPath()) ||
                    !settings->saveDetails("dark", 1.25, "en-US", "compact", false, false, false,
                                           "tabs")) {
                    failure("initial_default_profile");
                    return;
                }
                QSettings profile(uiSettingsPath(), QSettings::IniFormat);
                profile.setValue("ownedFixture/unknownKey", "preserve-across-native-Sparkle");
                profile.sync();
                if (profile.status() != QSettings::NoError) {
                    failure("profile_write");
                    return;
                }
            }
            state->started = true;
            write(prefix + "-started.json", snapshot(updates, settings));
        }
        if (!state->started)
            return;
        write(prefix + "-state.json", snapshot(updates, settings));
        if (QFileInfo::exists(directory() + "/stop.json") && !state->stopping) {
            state->stopping = true;
            sessions->requestQuit();
            return;
        }
        if (!oldProduct() || state->install || !QFileInfo::exists(directory() + "/upgrade.json"))
            return;
        auto *page = window->property("activePage").value<QObject *>();
        sessions->showAboutFor(page);
        auto *about = page ? page->findChild<QQuickWindow *>("aboutWindow") : nullptr;
        if (!about || !about->isVisible())
            return;
        QScopedValueRollback<bool> pressing(state->clicking, true);
        if (!state->check) {
            state->check = click(about, about->findChild<QQuickItem *>("aboutCheckUpdate"));
            return;
        }
        if (!state->install && updates->state() == "available") {
            if (!updates->trusted() || !updates->canInstall()) {
                failure("authenticated_real_installability");
                return;
            }
            state->install = click(about, about->findChild<QQuickItem *>("aboutInstallUpdate"));
        }
    });
    timer->start();
    QTimer::singleShot(240000, sessions, [] { failure("product_update_timeout"); });
}
