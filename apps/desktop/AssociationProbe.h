#pragma once
#include "BrowserController.h"
#include "BuildInfo.h"
#include <QCoreApplication>
#include <QJsonDocument>
#include <QJsonObject>
#include <QQuickItem>
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
    struct State {
        bool opened = false;
        bool completed = false;
        int frames = 0;
        int welcomeFrames = 0;
    };
    auto state = std::make_shared<State>();
    QObject::connect(controller, &BrowserController::changed, controller,
                     [controller, window, state] {
                         if (state->completed || controller->busy() || !controller->ready())
                             return;
                         state->opened = true;
                         window->update();
                     });
    QObject::connect(
        window, &QQuickWindow::frameSwapped, window, [controller, window, output, state] {
            if (state->completed)
                return;
            auto *welcome = window->findChild<QQuickItem *>("welcomeContent");
            if (!welcome) {
                QCoreApplication::exit(12);
                return;
            }
            ++state->frames;
            if (welcome->isVisible())
                ++state->welcomeFrames;
            if (!state->opened)
                return;
            state->completed = true;
            const QJsonObject result{
                {"event", "association_open"},       {"records", controller->archiveTotal()},
                {"version", BuildInfo::version()},   {"revision", BuildInfo::revision()},
                {"presented_frames", state->frames}, {"welcome_frames", state->welcomeFrames}};
            QSaveFile file(output);
            const auto data = QJsonDocument(result).toJson(QJsonDocument::Compact);
            if (!file.open(QIODevice::WriteOnly) || file.write(data) != data.size() ||
                !file.commit()) {
                QCoreApplication::exit(12);
                return;
            }
            QTimer::singleShot(0, window, &QWindow::close);
        });
    QTimer::singleShot(15000, controller, [state] {
        if (!state->completed)
            QCoreApplication::exit(12);
    });
}
