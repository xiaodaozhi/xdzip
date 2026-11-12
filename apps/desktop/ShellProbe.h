#pragma once
#include "ApplicationSessions.h"
#include "BuildInfo.h"
#include "ShellBatchController.h"
#include <QElapsedTimer>
#include <QJsonArray>
#include <QJsonDocument>
#include <QSaveFile>
#include <QTimer>
#include <algorithm>

// An explicit isolated-settings probe never connects to a real user instance.
// It writes only counts and terminal states, then uses the normal join barrier.
inline void installShellProbe(ApplicationSessions *sessions, const QString &output) {
    auto *timer = new QTimer(sessions);
    timer->setInterval(20);
    const auto elapsed = std::make_shared<QElapsedTimer>();
    elapsed->start();
    QObject::connect(timer, &QTimer::timeout, sessions, [sessions, output, timer, elapsed] {
        const auto batches = sessions->findChildren<ShellBatchController *>();
        const bool timedOut = elapsed->elapsed() > 30000;
        const bool complete = !batches.isEmpty() &&
                              std::all_of(batches.begin(), batches.end(), [](const auto *batch) {
                                  return batch->finished() || batch->choosing();
                              });
        if (!complete && !timedOut)
            return;
        timer->stop();
        QJsonArray statuses;
        for (auto *batch : batches)
            for (const auto &value : batch->results())
                statuses.append(value.toMap().value("status").toString());
        const QJsonObject record{
            {"schema_version", 1},         {"revision", BuildInfo::revision()},
            {"dirty", BuildInfo::dirty()}, {"main_windows", sessions->windows().size()},
            {"batches", batches.size()},   {"statuses", statuses},
            {"timed_out", timedOut}};
        QSaveFile file(output);
        file.setDirectWriteFallback(false);
        if (!file.open(QIODevice::WriteOnly) || file.write(QJsonDocument(record).toJson()) < 0 ||
            !file.commit())
            qFatal("Could not write the isolated shell probe");
        sessions->confirmQuit(false);
    });
    timer->start();
}
