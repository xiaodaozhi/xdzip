#include "ShellBatchController.h"
#include "shell/MenuModel.h"
#include <QDesktopServices>
#include <QFileInfo>
#include <QTimer>
#include <QUrl>

namespace {
bool archiveCandidate(const QString &path) {
    return xdzip_shell::archiveCandidate(xdzip_shell::fs::u8path(path.toUtf8().toStdString()));
}
QString base(const QStringList &paths) {
    const auto name = paths.size() == 1
                          ? QFileInfo(paths.first()).fileName()
                          : QFileInfo(QFileInfo(paths.first()).absolutePath()).fileName();
    return name.isEmpty() ? QStringLiteral("archive") : name;
}
} // namespace
ShellBatchController::ShellBatchController(QObject *parent) : QObject(parent), m_reader(this) {
    m_reader.shellOperation = true;
    connect(&m_reader, &BrowserController::changed, this, &ShellBatchController::schedule);
    connect(&m_reader, &BrowserController::extractionChanged, this,
            &ShellBatchController::schedule);
    connect(&m_reader, &BrowserController::tasksChanged, this, &ShellBatchController::schedule);
    connect(&m_reader, &BrowserController::creationStarted, this, &ShellBatchController::accepted);
    connect(&m_reader, &BrowserController::extractionStarted, this,
            &ShellBatchController::accepted);
}
bool ShellBatchController::active() const {
    return m_stage != Choosing && m_stage != Configuring && m_stage != Finished;
}
QVariantList ShellBatchController::actions() const {
    QVariantList result;
    if (m_request.paths.isEmpty())
        return result;
    auto append = [&](const QString &action, const QString &label) {
        result.append(QVariantMap{{"action", action}, {"label", label}});
    };
    append("zip", qtTrId("shell.addNamed").arg(base(m_request.paths) + ".zip"));
    append("7z", qtTrId("shell.addNamed").arg(base(m_request.paths) + ".7z"));
    append("create", qtTrId("shell.create"));
    if (m_request.paths.size() > 1)
        append("separate-zip", qtTrId("shell.separateZip"));
    if (m_request.archiveCandidates ||
        std::all_of(m_request.paths.begin(), m_request.paths.end(), archiveCandidate)) {
        append("extract-here", qtTrId("shell.extractHere"));
        append("extract-smart", qtTrId("shell.extractSmart"));
        append("extract-separate", qtTrId("shell.extractSeparate"));
        append("extract-to", qtTrId("shell.extractTo"));
    }
    return result;
}
QString ShellBatchController::summary() const {
    int succeeded = 0, failed = 0, omitted = 0;
    for (const auto &item : m_results) {
        const auto state = item.toMap().value("status").toString();
        succeeded += state == "succeeded";
        failed += state == "failed";
        omitted += state == "not-run" || state == "cancelled";
    }
    return finished()   ? qtTrId("shell.summary").arg(succeeded).arg(failed).arg(omitted)
           : choosing() ? qtTrId("shell.selection").arg(archiveCount())
                        : qtTrId("shell.processing").arg(m_index + 1).arg(m_jobs.size());
}
void ShellBatchController::begin(const ShellActionRequest &request) {
    m_request = request;
    if (request.action != "choose")
        choose(request.action);
    emit changed();
}
void ShellBatchController::choose(const QString &action) {
    if (m_stage != Choosing)
        return;
    const auto available = actions();
    if (std::none_of(available.begin(), available.end(),
                     [&](const QVariant &v) { return v.toMap().value("action") == action; }))
        return;
    m_request.action = action;
    const bool creating = action == "zip" || action == "7z" || action == "create";
    if (creating) {
        m_jobs.append(m_request.paths);
        const auto target = QFileInfo(m_request.paths.first()).absolutePath();
        const bool sameParent =
            std::all_of(m_request.paths.begin(), m_request.paths.end(),
                        [&](const QString &p) { return QFileInfo(p).absolutePath() == target; });
        if (action == "create" || !sameParent) {
            m_stage = Configuring;
            QVariantList inputs;
            for (const auto &path : m_request.paths)
                inputs.append(QVariantMap{{"path", path},
                                          {"name", QFileInfo(path).fileName()},
                                          {"directory", QFileInfo(path).isDir()}});
            emit configureCreate(inputs, target,
                                 base(m_request.paths) + (action == "7z" ? ".7z" : ".zip"),
                                 action == "7z");
            emit changed();
            return;
        }
    } else {
        for (const auto &path : m_request.paths)
            m_jobs.append(QStringList{path});
    }
    m_chooseTarget = action == "extract-to";
    m_layout = action == "extract-here" ? 0 : action == "extract-smart" ? 1 : 2;
    next();
}
void ShellBatchController::configuringCreation() {
    if (m_stage == Configuring) {
        m_stage = Submitting;
        emit changed();
    }
}
void ShellBatchController::configureExtraction(const QString &target, int layout, int conflict,
                                               bool highRatio, bool openAfter) {
    if (m_stage != Configuring || layout < 0 || layout > 2)
        return;
    m_target = target;
    m_layout = layout;
    m_conflict = conflict;
    m_highRatio = highRatio;
    m_openAfter = openAfter;
    m_chooseTarget = false;
    m_stage = Target;
    m_reader.checkTarget(target);
    emit changed();
    schedule();
}
void ShellBatchController::next() {
    if (m_cancelled || m_index >= m_jobs.size()) {
        while (m_index < m_jobs.size()) {
            record("not-run");
            ++m_index;
        }
        m_stage = Finished;
        emit changed();
        emit showSummary();
        return;
    }
    m_task.clear();
    const auto &inputs = m_jobs.at(m_index);
    if (m_request.action == "zip" || m_request.action == "7z" ||
        m_request.action == "separate-zip") {
        m_stage = Submitting;
        m_reader.startCreation(inputs, QFileInfo(inputs.first()).absolutePath(),
                               base(inputs) + (m_request.action == "7z" ? ".7z" : ".zip"),
                               m_request.action == "7z" ? 32 : 8, 6, false);
    } else {
        m_stage = Opening;
        m_reader.openPath(inputs.first());
    }
    emit changed();
    schedule();
}
void ShellBatchController::accepted(const QString &id) {
    m_task = id;
    m_stage = Running;
    if (m_cancelled)
        m_reader.cancelTask(id);
    emit changed();
    emit showTask(id);
    schedule();
}
void ShellBatchController::record(const QString &status, const QString &error,
                                  const QString &target) {
    m_results.append(QVariantMap{{"name", base(m_jobs.at(m_index))},
                                 {"status", status},
                                 {"error", error},
                                 {"target", target}});
}
void ShellBatchController::schedule() {
    if (m_scheduled)
        return;
    m_scheduled = true;
    QTimer::singleShot(0, this, [this] {
        m_scheduled = false;
        step();
    });
}
void ShellBatchController::step() {
    if (!active() || m_reader.busy())
        return;
    if (m_stage == Running) {
        const auto tasks = m_reader.tasks();
        for (const auto &value : tasks) {
            const auto task = value.toMap();
            if (task.value("id") != m_task || !task.value("terminal").toBool())
                continue;
            record(task.value("status").toString(), task.value("error").toString(),
                   task.value("target").toString());
            ++m_index;
            next();
            return;
        }
        return;
    }
    if (m_cancelled) {
        record("cancelled");
        ++m_index;
        next();
        return;
    }
    if (m_stage == Opening) {
        if (!m_reader.passwordPrompt().isEmpty())
            return;
        if (!m_reader.ready()) {
            record("failed", m_reader.message());
            ++m_index;
            next();
            return;
        }
        if (m_groups.contains(m_reader.sourceGroup())) {
            record("duplicate");
            ++m_index;
            next();
            return;
        }
        m_groups.insert(m_reader.sourceGroup());
        m_reader.prepareExtraction(true);
        if (m_chooseTarget) {
            m_stage = Configuring;
            emit changed();
            emit configureExtract();
            return;
        }
        m_stage = Target;
        m_reader.checkTarget(
            m_target.isEmpty() ? QFileInfo(m_jobs.at(m_index).first()).absolutePath() : m_target);
        schedule();
        return;
    }
    if (m_stage == Target) {
        const auto target =
            m_target.isEmpty() ? QFileInfo(m_jobs.at(m_index).first()).absolutePath() : m_target;
        if (!m_reader.targetMatches(target)) {
            record("failed", m_reader.targetCheck().value("error").toString());
            ++m_index;
            next();
            return;
        }
        m_stage = Submitting;
        m_reader.startExtraction(target, true, false, m_conflict, m_highRatio, m_openAfter,
                                 m_layout);
        schedule();
        return;
    }
    if (m_stage == Submitting) {
        // A rejected submission has no task receipt; never fabricate completion.
        record("failed",
               m_reader.creationError().isEmpty() ? m_reader.message() : m_reader.creationError());
        ++m_index;
        next();
    }
}
void ShellBatchController::cancel() {
    if (finished())
        return;
    m_cancelled = true;
    if (m_stage == Choosing || m_stage == Configuring) {
        m_stage = Finished;
        emit changed();
        emit showSummary();
        return;
    }
    if (!m_task.isEmpty())
        m_reader.cancelTask(m_task);
    else if (!m_reader.passwordPrompt().isEmpty())
        m_reader.cancelTask("open");
    else
        m_reader.cancel();
    schedule();
}

void ShellBatchController::openTarget(int index) {
    if (index < 0 || index >= m_results.size())
        return;
    const auto item = m_results.at(index).toMap();
    const auto target = item.value("target").toString();
    if (item.value("status") != "succeeded" || target.isEmpty())
        return;
    const auto folder = QFileInfo(target).isDir() ? target : QFileInfo(target).absolutePath();
    QDesktopServices::openUrl(QUrl::fromLocalFile(folder));
}
