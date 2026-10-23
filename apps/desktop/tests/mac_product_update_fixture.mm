// apps/desktop/tests/mac_product_update_fixture.mm

// Private build instrumentation. Normal product targets never compile this file.
#include "mac_product_update_fixture.h"
#include "ApplicationSessions.h"
#include "BrowserController.h"
#include "BuildInfo.h"
#include "MacUpdateProvider.h"
#include "UiPreferences.h"
#include "UiText.h"
#include "UpdateController.h"
#import <Foundation/Foundation.h>
#include <QCoreApplication>
#include <QDir>
#include <QEvent>
#include <QFile>
#include <QFileInfo>
#include <QGuiApplication>
#include <QInputMethodEvent>
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
#include <sys/stat.h>
#include <unistd.h>

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
    QSettings pending(QFileInfo(uiSettingsPath()).dir().filePath("update-state.ini"),
                      QSettings::IniFormat);
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
            {"unknown_key", profile.value("ownedFixture/unknownKey").toString()},
            {"pending_present", pending.contains("update/pending")}};
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
    if (!oldProduct() || configuration().value("scenario") == "recovery")
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
        bool created = false, opened = false, testing = false, waiting = false, cancelled = false;
        bool submitted = false, completed = false, requestedAgain = false;
        bool quitRequested = false, returned = false, cancelQuitRequested = false;
        bool cancelledQuit = false, interrupted = false, joined = false;
        int quitSignals = 0, barriers = 0;
        QString task, prompt, idleSession, workingSession;
        BrowserController *idle = nullptr, *working = nullptr;
    };
    const auto state = std::make_shared<State>();
    auto *initialPage = window->property("activePage").value<QObject *>();
    state->idle = initialPage ? initialPage->findChild<BrowserController *>() : nullptr;
    state->idleSession =
        initialPage ? initialPage->property("sessionIdentity").toString() : QString{};
    const auto prefix = QString::number(BuildInfo::buildNumber());
    const auto scenario = config.value("scenario").toString("success");
    const auto taskRecord = [=] {
        QVariantMap task;
        if (state->working)
            for (const auto &value : state->working->tasks())
                if (value.toMap().value("id").toString() == state->task)
                    task = value.toMap();
        return task;
    };
    QObject::connect(sessions, &ApplicationSessions::readyToQuit, sessions, [=] {
        ++state->quitSignals;
        state->joined = state->idle && state->idle->shutdownComplete() &&
                        (!state->working || state->working->shutdownComplete());
        write(prefix + "-join.json",
              {{"quit_signals", state->quitSignals},
               {"all_rust_joined", state->joined},
               {"active_tasks", sessions->activeTaskCount()},
               {"update_pending", sessions->updatePending()},
               {"password_prompt_cleared",
                !state->working || state->working->passwordPrompt().isEmpty()},
               {"task", QJsonObject::fromVariantMap(taskRecord())}});
    });
    QObject::connect(qApp, &QCoreApplication::aboutToQuit, sessions, [=] {
        write(prefix + "-normal-quit.json",
              {{"quit_signals", state->quitSignals},
               {"all_rust_joined", state->joined},
               {"barriers", state->barriers},
               {"return_clicked", state->returned},
               {"cancel_quit_clicked", state->cancelledQuit},
               {"quit_intent", scenario == "quit-wait"
                                   ? "QEvent::Quit through AppQuitGuard and DesktopIntegration"
                                   : "fixture sessions.requestQuit or native commit"},
               {"native_exit", macUpdateExitStatus()}});
    });
    QObject::connect(updates, &UpdateController::quitForUpdate, sessions, [=] {
        write(prefix + "-native-commit.json",
              {{"committed_native_quit", macUpdateInstallationCommitted()},
               {"active_tasks", sessions->activeTaskCount()}});
    });
    QObject::connect(updates, &UpdateController::exitBarrierReached, sessions, [=] {
        ++state->barriers;
        auto record = snapshot(updates, settings);
        record.insert("update_pending", sessions->updatePending());
        record.insert("active_tasks", sessions->activeTaskCount());
        record.insert("native_exit", macUpdateExitStatus());
        record.insert("about_check_clicked", state->check);
        record.insert("about_install_clicked", state->install);
        QSettings pending(QFileInfo(uiSettingsPath()).dir().filePath("update-state.ini"),
                          QSettings::IniFormat);
        record.insert(
            "pending",
            QJsonDocument::fromJson(pending.value("update/pending").toByteArray()).object());
        write(prefix + "-exit-barrier.json", record);
        if (scenario == "interrupted-handoff") {
            // An owned fault at the real join/commit boundary, never a claim
            // that the OS disk replacement or authorization dialog failed.
            state->interrupted = true;
            cancelMacUpdate();
        } else if (scenario == "immutable-replacement") {
            // Only this fresh installed app root is locked, after Sparkle has
            // prepared its unprivileged helper. Native commit remains unchanged.
            struct stat before {
            }, after{};
            const auto path = QFile::encodeName(installed);
            const bool owned = ::lstat(path.constData(), &before) == 0 && S_ISDIR(before.st_mode) &&
                               before.st_uid == ::geteuid();
            const bool changed =
                owned && ::chflags(path.constData(), before.st_flags | UF_IMMUTABLE) == 0;
            const bool applied = changed && ::lstat(path.constData(), &after) == 0 &&
                                 before.st_dev == after.st_dev && before.st_ino == after.st_ino &&
                                 (after.st_flags & UF_IMMUTABLE);
            write("replacement-fault.json", {{"applied", applied},
                                             {"changed", changed},
                                             {"device", QString::number(qulonglong(before.st_dev))},
                                             {"inode", QString::number(qulonglong(before.st_ino))},
                                             {"flags_before", QString::number(before.st_flags)},
                                             {"flags_after", QString::number(after.st_flags)},
                                             {"fault", "owned UF_IMMUTABLE"}});
            if (!applied) {
                state->interrupted = true;
                cancelMacUpdate();
            }
        }
    });
    auto *timer = new QTimer(sessions);
    timer->setInterval(100);
    QObject::connect(timer, &QTimer::timeout, sessions, [=] {
        if (state->clicking)
            return;
        if (!state->started && window->isVisible()) {
            if (oldProduct() && !config.value("reuse_profile").toBool()) {
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
        if (state->interrupted && updates->state() == "recovery_required" && !state->stopping) {
            state->stopping = true;
            write("interrupted-handoff.json", snapshot(updates, settings));
            sessions->requestQuit();
            return;
        }
        if (QFileInfo::exists(directory() + "/stop.json") && !state->stopping) {
            state->stopping = true;
            sessions->requestQuit();
            return;
        }
        if (!oldProduct() || state->install || !QFileInfo::exists(directory() + "/upgrade.json"))
            return;
        if (scenario == "recovery")
            return;
        auto *page = window->property("activePage").value<QObject *>();
        const bool passwordScenario = scenario == "quit-wait" || scenario == "busy-upgrade";
        if (passwordScenario && state->check && !state->completed) {
            if (!state->created && updates->state() == "available") {
                state->created = true;
                sessions->newSession();
                return; // Observe the new active page in a subsequent event-loop turn.
            }
            if (state->created && !state->opened) {
                const auto identity =
                    page ? page->property("sessionIdentity").toString() : QString{};
                auto *candidate = page ? page->findChild<BrowserController *>() : nullptr;
                if (identity.isEmpty() || identity == state->idleSession || !candidate ||
                    candidate == state->idle || !candidate->runtimeAvailable())
                    return;
                if (sessions->sessionCount() != 2 || state->idleSession.isEmpty()) {
                    failure("password_second_session");
                    return;
                }
                state->workingSession = identity;
                state->working = candidate;
                state->opened = true;
                state->working->openPath(directory() + "/password-retry.zip");
            }
            if (!state->opened || !page ||
                page->property("sessionIdentity").toString() != state->workingSession)
                return;
            auto *controller = state->working;
            if (!controller)
                return;
            if (!state->testing && controller->ready() && !controller->busy()) {
                state->testing = true;
                controller->startArchiveTest();
            }
            const auto prompt = controller->passwordPrompt();
            if (state->testing && !state->waiting && !prompt.isEmpty()) {
                state->waiting = true;
                state->task = prompt.value("taskId").toString();
                state->prompt = prompt.value("promptId").toString();
                auto *password = page ? page->findChild<QQuickWindow *>("passwordWindow") : nullptr;
                if (!password || password->modality() != Qt::ApplicationModal) {
                    failure("password_native_modality");
                    return;
                }
                // A real modal password prompt prevents About pointer input.
                // Exercise its policy through the controller and record the seam.
                updates->downloadAndInstall();
                write("password-waiting.json",
                      {{"task_id", state->task},
                       {"idle_session", state->idleSession},
                       {"working_session", state->workingSession},
                       {"session_count", sessions->sessionCount()},
                       {"prompt_id", state->prompt},
                       {"kind", prompt.value("kind").toString()},
                       {"active_tasks", sessions->activeTaskCount()},
                       {"state", updates->state()},
                       {"install_request", "controller while application-modal"}});
            }
            if (state->waiting && !state->cancelled && updates->state() == "waiting_tasks") {
                updates->cancel();
                const auto after = controller->passwordPrompt();
                if (controller->activeTaskCount() != 1 ||
                    after.value("taskId").toString() != state->task ||
                    after.value("promptId").toString() != state->prompt ||
                    updates->state() != "available") {
                    failure("waiting_cancel_changed_password_task");
                    return;
                }
                state->cancelled = true;
                write("waiting-cancelled.json", {{"method", "controller.cancel"},
                                                 {"state", updates->state()},
                                                 {"task_id", state->task},
                                                 {"prompt_id", state->prompt},
                                                 {"active_tasks", sessions->activeTaskCount()}});
            }
            if (!state->cancelled)
                return;
            if (scenario == "quit-wait") {
                if (!state->requestedAgain) {
                    state->requestedAgain = true;
                    updates->downloadAndInstall();
                    if (updates->state() != "waiting_tasks") {
                        failure("ordinary_quit_not_waiting_update");
                        return;
                    }
                }
                auto *confirmation = page ? page->findChild<QQuickWindow *>("quitWindow") : nullptr;
                if (!state->quitRequested) {
                    state->quitRequested = true;
                    QCoreApplication::postEvent(qApp, new QEvent(QEvent::Quit));
                    return;
                }
                if (!state->returned && confirmation && confirmation->isExposed() &&
                    QGuiApplication::modalWindow() == confirmation) {
                    QScopedValueRollback<bool> pressing(state->clicking, true);
                    state->returned =
                        click(confirmation, confirmation->findChild<QQuickItem *>("quitReturn"));
                    return;
                }
                if (state->returned && !state->cancelQuitRequested && confirmation &&
                    !confirmation->isVisible()) {
                    const auto remaining = controller->passwordPrompt();
                    if (remaining.value("taskId").toString() != state->task ||
                        remaining.value("promptId").toString() != state->prompt ||
                        sessions->activeTaskCount() != 1 || sessions->isClosing() ||
                        sessions->updatePending() || updates->state() != "waiting_tasks") {
                        failure("ordinary_quit_return_changed_task");
                        return;
                    }
                    write("quit-returned.json",
                          {{"task_id", state->task},
                           {"prompt_id", state->prompt},
                           {"state", updates->state()},
                           {"active_tasks", sessions->activeTaskCount()},
                           {"task", QJsonObject::fromVariantMap(taskRecord())}});
                    state->cancelQuitRequested = true;
                    QCoreApplication::postEvent(qApp, new QEvent(QEvent::Quit));
                    return;
                }
                if (state->cancelQuitRequested && !state->cancelledQuit && confirmation &&
                    confirmation->isExposed() && QGuiApplication::modalWindow() == confirmation) {
                    QScopedValueRollback<bool> pressing(state->clicking, true);
                    state->cancelledQuit =
                        click(confirmation, confirmation->findChild<QQuickItem *>("quitCancel"));
                }
                return;
            }
            if (!state->submitted) {
                auto *password = page ? page->findChild<QQuickWindow *>("passwordWindow") : nullptr;
                auto *field =
                    password ? password->findChild<QQuickItem *>("extractionPassword") : nullptr;
                auto *submit =
                    password ? password->findChild<QQuickItem *>("passwordSubmit") : nullptr;
                if (!password || !password->isExposed() || !field || !submit ||
                    QGuiApplication::modalWindow() != password)
                    return;
                QScopedValueRollback<bool> pressing(state->clicking, true);
                field->forceActiveFocus();
                QInputMethodEvent input;
                input.setCommitString(QStringLiteral(" owned AES 密码 "));
                QCoreApplication::sendEvent(field, &input);
                if (field->property("text").toString() != QStringLiteral(" owned AES 密码 ")) {
                    failure("password_ime_commit");
                    return;
                }
                state->submitted = click(password, submit);
                return;
            }
            if (controller->activeTaskCount() == 0) {
                const auto task = taskRecord();
                if (task.value("status").toString() != "succeeded" ||
                    !task.value("terminal").toBool() ||
                    task.value("verifiedFiles").toString() != "1" ||
                    task.value("verifiedBytes").toString() != "18") {
                    failure("password_check_result");
                    return;
                }
                state->completed = true;
                write("password-completed.json",
                      {{"task", QJsonObject::fromVariantMap(task)},
                       {"input", "IME commit and actual confirmation click"}});
            }
            return;
        }
        sessions->showAboutFor(page);
        QQuickWindow *about = nullptr;
        for (auto *candidate : window->findChildren<QQuickWindow *>("aboutWindow")) {
            if (!candidate->isVisible() || !candidate->isExposed() ||
                candidate->transientParent() != window)
                continue;
            if (about) {
                failure("multiple_owned_about_windows");
                return;
            }
            about = candidate;
        }
        if (!about)
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
