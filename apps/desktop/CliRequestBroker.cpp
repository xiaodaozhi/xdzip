#include "CliRequestBroker.h"
#include "ApplicationSessions.h"
#include "BuildInfo.h"
#include "SingleInstance.h"
#include "UpdateController.h"
#include "xdzip-qt-bridge/src/lib.rs.h"
#include <QDir>
#include <QFileInfo>
#include <QJsonDocument>
#include <QJsonObject>
#include <QStandardPaths>
#include <memory>
#include <utility>

namespace {
rust::Str view(const QByteArray &value) {
    return {value.constData(), size_t(value.size())};
}
QString root() {
    const auto settings = qEnvironmentVariable("XDZIP_SETTINGS_PATH");
    return settings.isEmpty()
               ? QStandardPaths::writableLocation(QStandardPaths::GenericDataLocation) +
                     "/xdzip/cli"
               : QFileInfo(settings).absolutePath() + "/cli";
}
struct State {
    QString expected;
};
} // namespace

void installCliRequests(SingleInstance *instance, UpdateController *updates,
                        ApplicationSessions *sessions) {
    const auto state = std::make_shared<State>();
    QObject::connect(updates, &UpdateController::changed, updates, [updates, state] {
        if (state->expected.isEmpty())
            return;
        if (updates->state() == "available") {
            const auto expected = std::exchange(state->expected, {});
            if (updates->candidateVersion() == expected && updates->trusted() &&
                updates->canInstall())
                updates->downloadAndInstall();
        } else if (updates->state() == "error" || updates->state() == "up_to_date") {
            state->expected.clear();
        }
    });
    instance->attachCli([updates, sessions, state](const QString &directory) {
        const auto cache = root() + "/cache";
        const auto raw =
            xdzip::reader::cli_request_read(view(directory.toUtf8()), view(cache.toUtf8()));
        const auto request =
            QJsonDocument::fromJson(QByteArray(raw.data(), qsizetype(raw.size()))).object();
        QJsonObject result{
            {"schema_version", 1}, {"status", "failed"}, {"error_code", "invalid_request"}};
        const auto candidate = request.value("candidate_version").toString();
        if (request.value("schema_version").toInt() == 1 && request.value("action") == "update" &&
            !candidate.isEmpty() && candidate.size() <= 80) {
            if (updates->busy() || !state->expected.isEmpty()) {
                result.insert("error_code", "update_busy");
            } else {
                state->expected = candidate;
                sessions->command("about");
                updates->check();
                result.insert("status", "handed_off");
                result.insert("error_code", "");
            }
        }
        const auto bytes = QJsonDocument(result).toJson(QJsonDocument::Compact);
        xdzip::reader::cli_request_write(view(directory.toUtf8()), view(cache.toUtf8()),
                                         view(bytes));
    });
}
