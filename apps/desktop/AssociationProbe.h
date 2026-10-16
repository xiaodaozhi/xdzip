#pragma once
#include "BrowserController.h"
#include "BuildInfo.h"
#include <QCoreApplication>
#include <QJsonDocument>
#include <QJsonObject>
#include <QQuickWindow>
#include <QSaveFile>
#include <QTimer>
#include <memory>

// Explicit CI opt-in only. Observe the normal OS/argv open route, write no
// source path, and exit through the normal close barrier after a real read.
inline void installAssociationProbe(BrowserController *controller, QQuickWindow *window) {
    const auto output = qEnvironmentVariable("XDZIP_ASSOCIATION_PROBE");
    if (output.isEmpty())
        return;
    if (qEnvironmentVariableIsEmpty("XDZIP_SETTINGS_PATH")) {
        QTimer::singleShot(0, [] { QCoreApplication::exit(12); });
        return;
    }
    auto completed = std::make_shared<bool>(false);
    QObject::connect(controller, &BrowserController::changed, controller,
                     [controller, window, output, completed] {
                         if (*completed || controller->busy() || !controller->ready())
                             return;
                         *completed = true;
                         const QJsonObject result{{"event", "association_open"},
                                                  {"records", controller->archiveTotal()},
                                                  {"version", BuildInfo::version()},
                                                  {"revision", BuildInfo::revision()}};
                         QSaveFile file(output);
                         const auto data = QJsonDocument(result).toJson(QJsonDocument::Compact);
                         if (!file.open(QIODevice::WriteOnly) || file.write(data) != data.size() ||
                             !file.commit()) {
                             QCoreApplication::exit(12);
                             return;
                         }
                         QTimer::singleShot(0, window, &QWindow::close);
                     });
    QTimer::singleShot(15000, controller, [completed] {
        if (!*completed)
            QCoreApplication::exit(12);
    });
}
