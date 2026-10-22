// apps/desktop/tests/native_update_fixture.cpp

// Compiled only into private native regression copies, never the normal product.
#include "native_update_fixture.h"
#include "ApplicationSessions.h"
#include "BuildInfo.h"
#include "UiPreferences.h"
#include "UiText.h"
#include "UpdateController.h"
#include <QDir>
#include <QFileInfo>
#include <QGuiApplication>
#include <QInputMethodEvent>
#include <QNetworkReply>
#include <QNetworkRequest>
#include <QQuickItem>
#include <QQuickWindow>
#include <QSGRendererInterface>
#include <QSaveFile>
#include <QScopedValueRollback>
#include <QSettings>
#include <QSignalSpy>
#include <QTimer>
#include <QtTest/QTest>
#include <memory>
#ifdef Q_OS_UNIX
#include <unistd.h>
#endif

namespace {
QString directory() {
    return QStringLiteral(XDZIP_NATIVE_UPDATE_FIXTURE_DIRECTORY);
}
bool oldProduct() {
    return BuildInfo::buildNumber() == 200;
}
bool saveRecord(const QString &name, const QJsonObject &value) {
    QSaveFile file(directory() + '/' + name);
    const auto bytes = QJsonDocument(value).toJson();
    return file.open(QIODevice::WriteOnly) && file.write(bytes) == bytes.size() && file.commit();
}
class Reply final : public QNetworkReply {
    QFile m_file;

  public:
    Reply(const QNetworkRequest &request, const QString &path, QObject *parent)
        : QNetworkReply(parent), m_file(path) {
        setRequest(request);
        setUrl(request.url());
        const bool found = m_file.open(QIODevice::ReadOnly);
        setAttribute(QNetworkRequest::HttpStatusCodeAttribute, found ? 200 : 404);
        if (!found)
            setError(ContentNotFoundError, QStringLiteral("Owned fixture route missing"));
        else
            setHeader(QNetworkRequest::ContentLengthHeader, m_file.size());
        open(QIODevice::ReadOnly | QIODevice::Unbuffered);
        const bool taskScenario =
            QFileInfo::exists(directory() + "/password-scenario.json") &&
            (request.url().path().endsWith(".deb") || request.url().path().endsWith("-setup.exe"));
        if (taskScenario)
            saveRecord("download-requested.json", {{"owned", true}});
        QTimer::singleShot(0, this, [this, taskScenario] {
            emit readyRead();
            if (taskScenario && !isFinished()) {
                auto *gate = new QTimer(this);
                gate->setInterval(25);
                connect(gate, &QTimer::timeout, this, [this, gate] {
                    if (isFinished()) {
                        gate->stop();
                    } else if (QFileInfo::exists(directory() + "/release-download.json")) {
                        gate->stop();
                        setFinished(true);
                        emit finished();
                    }
                });
                gate->start();
            } else if (!isFinished()) {
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
        setError(OperationCanceledError, QStringLiteral("Owned fixture cancelled"));
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
        QFile routes(directory() + QStringLiteral("/routes.json"));
        QJsonObject mapping;
        if (operation == GetOperation && routes.open(QIODevice::ReadOnly))
            mapping = QJsonDocument::fromJson(routes.readAll()).object();
        const auto name = mapping.value(request.url().toString()).toString();
        const auto safe = !name.isEmpty() && QFileInfo(name).fileName() == name &&
                          !name.contains('\\') && name != "." && name != "..";
        return new Reply(request, safe ? directory() + '/' + name : QString{}, this);
    }
};
QJsonObject snapshot(UpdateController *updates, UiPreferences *settings) {
    int windows = 0;
    for (auto *window : QGuiApplication::allWindows())
        if (window->isVisible() && window->property("managedWindow").toBool())
            ++windows;
    QJsonObject result{{"version", BuildInfo::version()},
                       {"build", BuildInfo::buildNumber()},
                       {"revision", BuildInfo::revision()},
                       {"pid", QCoreApplication::applicationPid()},
                       {"primary_windows", windows},
                       {"state", updates->state()},
                       {"error", updates->errorCode()},
                       {"trusted", updates->trusted()},
                       {"can_install", updates->canInstall()},
                       {"install_reason", updates->installReason()},
                       {"configuration", BuildInfo::configuration()},
                       {"channel", BuildInfo::channel()},
                       {"dirty", BuildInfo::dirty()},
                       {"executable", QCoreApplication::applicationFilePath()},
                       {"canonical_executable",
                        QFileInfo(QCoreApplication::applicationFilePath()).canonicalFilePath()},
                       {"theme", settings->theme()},
                       {"text_scale", settings->textScale()},
                       {"language", settings->language()},
                       {"active_language", settings->activeLanguage()},
                       {"density", settings->density()},
                       {"settings_path", uiSettingsPath()},
                       {"home", QDir::homePath()}};
#ifdef Q_OS_UNIX
    result.insert("uid", int(getuid()));
    result.insert("minimum_system_gate", "production Ubuntu check");
    for (const auto *name :
         {"XDG_CONFIG_HOME", "XDG_DATA_HOME", "XDG_CACHE_HOME", "XDG_STATE_HOME"})
        result.insert(QString::fromLatin1(name), qEnvironmentVariable(name));
#else
    QSettings registration(QStringLiteral("HKEY_CURRENT_USER\\Software\\xdzip\\Install"),
                           QSettings::NativeFormat);
    const auto root = registration.value("Location").toString();
    result.insert("registered_location", root);
    result.insert("canonical_registered_location", QFileInfo(root).canonicalFilePath());
    result.insert("registered_install_id", registration.value("InstallId").toString());
    result.insert("installed_manifest", QFileInfo(root + "/install-manifest.txt").isFile());
    result.insert("minimum_system_gate",
                  "Windows Server CI override, Windows 11 manual acceptance pending");
#endif
    return result;
}
bool click(QQuickWindow *about, QObject *button) {
    auto *item = qobject_cast<QQuickItem *>(button);
    if (!about || !about->isExposed() || !item || !item->isVisible() || !item->isEnabled())
        return false;
    const auto rectangle = [](QQuickItem *value) {
        const auto rect = value->mapRectToScene(QRectF(0, 0, value->width(), value->height()));
        return QJsonObject{
            {"x", rect.x()}, {"y", rect.y()}, {"width", rect.width()}, {"height", rect.height()}};
    };
    QJsonObject observation{{"button", item->objectName()},
                            {"before_focus", rectangle(item)},
                            {"window_width", about->width()},
                            {"window_height", about->height()}};
    item->forceActiveFocus(Qt::OtherFocusReason);
    // The shared operation window reveals its focused control on the next QML
    // turn. This is essential on short native CI desktops with a scrolled About.
    QTest::qWait(100);
    const auto point = item->mapToScene(QPointF(item->width() / 2, item->height() / 2));
    observation.insert("after_focus", rectangle(item));
    QJsonArray clips;
    bool insideClips = true;
    for (auto *ancestor = item->parentItem(); ancestor; ancestor = ancestor->parentItem()) {
        if (ancestor->clip()) {
            const auto local =
                item->mapToItem(ancestor, QPointF(item->width() / 2, item->height() / 2));
            const bool inside = QRectF(0, 0, ancestor->width(), ancestor->height()).contains(local);
            insideClips &= inside;
            QJsonObject clip{{"name", ancestor->objectName()},
                             {"rect", rectangle(ancestor)},
                             {"contains_center", inside}};
            if (ancestor->property("contentY").isValid())
                clip.insert("content_y", ancestor->property("contentY").toDouble());
            clips.append(clip);
        }
    }
    observation.insert("clipping_ancestors", clips);
    observation.insert("inside_clipping_viewport", insideClips);
    const auto recordName = QStringLiteral("click-") + item->objectName() + ".json";
    if (!insideClips) {
        saveRecord(recordName, observation);
        return false;
    }
    if (!QRectF(0, 0, about->width(), about->height()).contains(point)) {
        saveRecord("fixture-error.json", {{"error", "about_click_outside_window"}});
        return false;
    }
    QSignalSpy pressed(item, SIGNAL(clicked()));
    QTest::mouseClick(about, Qt::LeftButton, Qt::NoModifier, point.toPoint());
    observation.insert("clicked_count", pressed.count());
    saveRecord(recordName, observation);
    return pressed.isValid() && pressed.count() == 1;
}
} // namespace

QNetworkAccessManager *nativeUpdateFixtureTransport(QObject *parent) {
    // The detached Linux restart deliberately does not inherit Qt rendering
    // overrides. Keep this private headless regression on its declared software
    // backend without adding any production environment propagation.
    QQuickWindow::setGraphicsApi(QSGRendererInterface::Software);
    // New ordinary startup must restore the real default-cache helper receipt.
    return oldProduct() ? new Network(parent) : nullptr;
}
void installNativeUpdateFixture(UpdateController *updates, ApplicationSessions *sessions,
                                UiPreferences *settings, QQuickWindow *window) {
    if (!sessions || !settings || !QFileInfo(directory()).isDir()) {
        QCoreApplication::exit(50);
        return;
    }
    struct State {
        bool started = false, check = false, install = false, stopping = false, clicking = false;
        bool duplicateQueued = false, duplicateAttempted = false;
        bool archiveOpened = false, testStarted = false, passwordWaiting = false;
        bool waitCancelled = false, passwordSubmitted = false, passwordCompleted = false;
        QString task, prompt;
    };
    const auto state = std::make_shared<State>();
    const auto prefix = QString::number(BuildInfo::buildNumber());
    QObject::connect(updates, &UpdateController::changed, sessions, [updates, sessions, state] {
        if (!oldProduct() || state->duplicateQueued || updates->state() != "preparing")
            return;
        state->duplicateQueued = true;
        // Queue after the first handoff's synchronous prepare call, so this
        // exercises the real cached-package/preparing reentry boundary.
        QTimer::singleShot(0, sessions, [updates, state] {
            updates->downloadAndInstall();
            state->duplicateAttempted = true;
        });
    });
    QObject::connect(
        updates, &UpdateController::exitBarrierReached, sessions,
        [updates, sessions, settings, state, prefix] {
            auto record = snapshot(updates, settings);
            record.insert("update_pending", sessions->updatePending());
            record.insert("active_tasks", sessions->activeTaskCount());
            record.insert("about_check_clicked", state->check);
            record.insert("about_install_clicked", state->install);
            record.insert("duplicate_install_attempted", state->duplicateAttempted);
            record.insert("password_task_completed", state->passwordCompleted);
            record.insert("waiting_cancel_controller_call", state->waitCancelled);
            QSettings pending(QFileInfo(uiSettingsPath()).dir().filePath("update-state.ini"),
                              QSettings::IniFormat);
            record.insert(
                "pending",
                QJsonDocument::fromJson(pending.value("update/pending").toByteArray()).object());
            saveRecord(prefix + "-exit-barrier.json", record);
        });
    auto *timer = new QTimer(sessions);
    timer->setInterval(100);
    QObject::connect(
        timer, &QTimer::timeout, sessions, [updates, sessions, settings, window, state, prefix] {
            if (state->clicking)
                return;
            if (!state->started && window->isVisible()) {
                if (oldProduct()) {
                    if (QFileInfo::exists(uiSettingsPath()) ||
                        !settings->saveDetails("dark", 1.25, "en-US", "compact", false, false,
                                               false, "tabs")) {
                        saveRecord("fixture-error.json", {{"error", "initial_profile"}});
                        QCoreApplication::exit(51);
                        return;
                    }
                    QSettings profile(uiSettingsPath(), QSettings::IniFormat);
                    profile.setValue("ownedFixture/sentinel", "preserve-me");
                    profile.sync();
                }
                state->started = true;
                saveRecord(prefix + "-started.json", snapshot(updates, settings));
            }
            if (!state->started)
                return;
            saveRecord(prefix + "-state.json", snapshot(updates, settings));
            if (QFileInfo::exists(directory() + "/stop.json") && !state->stopping) {
                state->stopping = true;
                sessions->requestQuit();
                return;
            }
            if (!oldProduct() || !QFileInfo::exists(directory() + "/upgrade.json"))
                return;
            auto *page = window->property("activePage").value<QObject *>();
            const bool passwordScenario =
                QFileInfo::exists(directory() + "/password-scenario.json");
            if ((updates->state() == "available" || updates->state() == "downloaded") &&
                !updates->canInstall()) {
                saveRecord("fixture-error.json", {{"error", "install_ineligible"},
                                                  {"details", snapshot(updates, settings)}});
                QCoreApplication::exit(53);
                return;
            }
            if (passwordScenario && state->install && !state->passwordCompleted) {
                auto *controller = window->findChild<BrowserController *>();
                const auto fail = [](const QString &error) {
                    saveRecord("fixture-error.json", {{"error", error}});
                    QCoreApplication::exit(52);
                };
                if (!controller) {
                    fail("password_controller");
                    return;
                }
                if (!state->archiveOpened &&
                    QFileInfo::exists(directory() + "/download-requested.json")) {
                    state->archiveOpened = true;
                    controller->openPath(directory() + "/password-retry.zip");
                }
                if (state->archiveOpened && !state->testStarted && controller->ready() &&
                    !controller->busy() && controller->title() == "password-retry.zip") {
                    state->testStarted = true;
                    controller->startArchiveTest();
                }
                const auto prompt = controller->passwordPrompt();
                if (state->testStarted && !state->passwordWaiting && !prompt.isEmpty()) {
                    state->task = prompt.value("taskId").toString();
                    state->prompt = prompt.value("promptId").toString();
                    state->passwordWaiting = true;
                    saveRecord("password-waiting.json",
                               {{"task_id", state->task},
                                {"prompt_id", state->prompt},
                                {"active_tasks", controller->activeTaskCount()},
                                {"kind", prompt.value("kind").toString()}});
                    saveRecord("release-download.json", {{"owned", true}});
                }
                if (state->passwordWaiting && !state->waitCancelled &&
                    updates->state() == "waiting_tasks") {
                    auto *password =
                        page ? page->findChild<QQuickWindow *>("passwordWindow") : nullptr;
                    if (!password || !password->isExposed())
                        return;
                    if (password->modality() != Qt::ApplicationModal ||
                        QGuiApplication::modalWindow() != password) {
                        fail("waiting_password_modality");
                        return;
                    }
                    // PasswordWindow remains ApplicationModal. This controller
                    // call tests cancellation policy, never pretends a blocked
                    // About control can receive pointer input through the modal.
                    updates->cancel();
                    const auto after = controller->passwordPrompt();
                    if (controller->activeTaskCount() != 1 ||
                        after.value("taskId").toString() != state->task ||
                        after.value("promptId").toString() != state->prompt ||
                        updates->state() != "downloaded") {
                        fail("waiting_cancel_changed_task");
                        return;
                    }
                    state->waitCancelled = true;
                    saveRecord("waiting-cancelled.json",
                               {{"method", "controller.cancel"},
                                {"password_window_modal", true},
                                {"task_id", state->task},
                                {"prompt_id", state->prompt},
                                {"active_tasks", controller->activeTaskCount()},
                                {"state", updates->state()}});
                }
                if (state->waitCancelled && !state->passwordSubmitted) {
                    auto *password =
                        page ? page->findChild<QQuickWindow *>("passwordWindow") : nullptr;
                    auto *field = password ? password->findChild<QQuickItem *>("extractionPassword")
                                           : nullptr;
                    auto *submit =
                        password ? password->findChild<QObject *>("passwordSubmit") : nullptr;
                    if (password && password->isExposed() && field && submit &&
                        submit->property("enabled").toBool()) {
                        if (password->modality() != Qt::ApplicationModal) {
                            fail("password_modality");
                            return;
                        }
                        QScopedValueRollback<bool> pressing(state->clicking, true);
                        field->forceActiveFocus();
                        QInputMethodEvent input;
                        input.setCommitString(QStringLiteral(" owned AES 密码 "));
                        QCoreApplication::sendEvent(field, &input);
                        if (field->property("text").toString() !=
                            QStringLiteral(" owned AES 密码 ")) {
                            fail("password_input");
                            return;
                        }
                        state->passwordSubmitted = click(password, submit);
                    }
                }
                if (state->passwordSubmitted && controller->activeTaskCount() == 0) {
                    QVariantMap completed;
                    for (const auto &value : controller->tasks()) {
                        const auto candidate = value.toMap();
                        if (candidate.value("id").toString() == state->task)
                            completed = candidate;
                    }
                    if (completed.value("status").toString() != "succeeded" ||
                        !completed.value("terminal").toBool() ||
                        completed.value("verifiedFiles").toString() != "1" ||
                        completed.value("verifiedBytes").toString() != "18") {
                        fail("password_check_result");
                        return;
                    }
                    state->passwordCompleted = true;
                    state->install = false;
                    saveRecord("password-completed.json",
                               {{"task_id", state->task},
                                {"status", "succeeded"},
                                {"verified_files", "1"},
                                {"verified_bytes", "18"},
                                {"input", "IME commit and mouse confirmation"}});
                }
            }
            if (!state->check) {
                sessions->showAboutFor(page);
                auto *about = page ? page->findChild<QQuickWindow *>("aboutWindow") : nullptr;
                auto *button = about ? about->findChild<QObject *>("aboutCheckUpdate") : nullptr;
                if (about && about->isVisible() && button && button->property("enabled").toBool()) {
                    QScopedValueRollback<bool> pressing(state->clicking, true);
                    state->check = click(about, button);
                }
            } else if (!state->install &&
                       (updates->state() == "available" || updates->state() == "downloaded") &&
                       updates->canInstall()) {
                sessions->showAboutFor(page);
                auto *about = page ? page->findChild<QQuickWindow *>("aboutWindow") : nullptr;
                auto *button = about ? about->findChild<QObject *>("aboutInstallUpdate") : nullptr;
                if (button && button->property("visible").toBool() &&
                    button->property("enabled").toBool()) {
                    QScopedValueRollback<bool> pressing(state->clicking, true);
                    state->install = click(about, button);
                }
            }
        });
    timer->start();
}
