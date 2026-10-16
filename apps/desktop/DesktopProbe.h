#pragma once
#include "BrowserController.h"
#include "BuildInfo.h"
#include <QElapsedTimer>
#include <QFileInfo>
#include <QGuiApplication>
#include <QJsonDocument>
#include <QJsonObject>
#include <QQuickWindow>
#include <QSGRendererInterface>
#include <QTimer>
#include <cstdio>

// Opt-in measurement of real product state transitions. No archive is extracted
// or created. The caller supplies an owned metadata fixture and isolated settings.
class DesktopProbe final : public QObject {
    enum class Phase { FirstFrame, Idle, Opening, Closing, CancelOpening, Cancelling, Done };
    Phase m_phase = Phase::FirstFrame;
    BrowserController *m_controller;
    QQuickWindow *m_window;
    QString m_path;
    QElapsedTimer m_stage;
    void emitEvent(const QString &name, QJsonObject data = {}) {
        data.insert(QStringLiteral("event"), name);
        const auto encoded = QJsonDocument(data).toJson(QJsonDocument::Compact);
        std::printf("XDZIP_PROBE %s\n", encoded.constData());
        std::fflush(stdout);
    }
    void changed() {
        if (m_phase == Phase::Opening && m_controller->ready() && !m_controller->browsingBusy()) {
            m_phase = Phase::Closing;
            emitEvent(QStringLiteral("open_ready"),
                      {{"elapsed_ms", m_stage.elapsed()},
                       {"records", m_controller->archiveTotal()},
                       {"visible_rows", m_controller->entries()->rowCount()}});
            QTimer::singleShot(200, this, [this] {
                m_controller->closeArchive();
                QTimer::singleShot(300, this, [this] {
                    m_phase = Phase::CancelOpening;
                    m_controller->openPath(m_path);
                    QTimer::singleShot(0, this, [this] {
                        const bool pending = m_controller->browsingBusy();
                        m_phase = Phase::Cancelling;
                        m_stage.restart();
                        emitEvent(QStringLiteral("cancel_requested"), {{"pending", pending}});
                        m_controller->cancel();
                        changed();
                    });
                });
            });
        } else if (m_phase == Phase::Cancelling && !m_controller->browsingBusy()) {
            m_phase = Phase::Done;
            emitEvent(QStringLiteral("cancel_settled"),
                      {{"elapsed_ms", m_stage.elapsed()},
                       {"cancelled", !m_controller->ready() && m_controller->runtimeAvailable()}});
            QTimer::singleShot(300, this, [this] {
                if (m_controller->ready() || m_controller->browsingBusy() ||
                    !m_controller->runtimeAvailable()) {
                    emitEvent(QStringLiteral("cancel_rebound"));
                    QCoreApplication::exit(9);
                    return;
                }
                emitEvent(QStringLiteral("quit_requested"));
                m_window->close();
            });
        }
    }

  public:
    DesktopProbe(BrowserController *controller, QQuickWindow *window, const QString &path,
                 QObject *parent)
        : QObject(parent), m_controller(controller), m_window(window), m_path(path) {
        connect(controller, &BrowserController::changed, this, [this] { changed(); });
        connect(window, &QQuickWindow::frameSwapped, this, [this] {
            if (m_phase != Phase::FirstFrame)
                return;
            m_phase = Phase::Idle;
            QString backend = QStringLiteral("unknown");
            switch (m_window->rendererInterface()->graphicsApi()) {
            case QSGRendererInterface::Metal:
                backend = QStringLiteral("metal");
                break;
            case QSGRendererInterface::OpenGL:
                backend = QStringLiteral("opengl");
                break;
            case QSGRendererInterface::Vulkan:
                backend = QStringLiteral("vulkan");
                break;
            case QSGRendererInterface::Direct3D11:
                backend = QStringLiteral("d3d11");
                break;
            case QSGRendererInterface::Direct3D12:
                backend = QStringLiteral("d3d12");
                break;
            case QSGRendererInterface::Software:
                backend = QStringLiteral("software");
                break;
            default:
                break;
            }
            emitEvent(QStringLiteral("first_frame"),
                      {{"version", BuildInfo::version()},
                       {"build", BuildInfo::buildNumber()},
                       {"configuration", BuildInfo::configuration()},
                       {"revision", BuildInfo::revision()},
                       {"dirty", BuildInfo::dirty()},
                       {"backend", backend},
                       {"qt", BuildInfo::qtVersion()},
                       {"runtime_available", m_controller->runtimeAvailable()},
                       {"platform", BuildInfo::platform()},
                       {"architecture", BuildInfo::architecture()}});
            QTimer::singleShot(1500, this, [this] {
                m_phase = Phase::Opening;
                m_stage.start();
                emitEvent(QStringLiteral("open_requested"));
                m_controller->openPath(m_path);
            });
        });
        QTimer::singleShot(30000, this, [this] {
            if (m_phase != Phase::Done) {
                emitEvent(QStringLiteral("timeout"));
                QCoreApplication::exit(8);
            }
        });
    }
};
