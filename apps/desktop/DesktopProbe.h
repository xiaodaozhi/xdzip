#pragma once
#include "BrowserController.h"
#include "BuildInfo.h"
#include <QCryptographicHash>
#include <QDir>
#include <QDirIterator>
#include <QElapsedTimer>
#include <QFile>
#include <QFileInfo>
#include <QGuiApplication>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QQuickWindow>
#include <QSGRendererInterface>
#include <QTimer>
#include <cstdio>
#ifdef Q_OS_WIN
#ifndef NOMINMAX
#define NOMINMAX
#endif
#include <windows.h>
#else
#include <dlfcn.h>
#endif

// Opt-in measurement of real product state transitions. The caller supplies an
// owned fixture and isolated settings. Explicit extraction mode additionally
// requires an existing empty output directory; ordinary product startup is unaffected.
class DesktopProbe final : public QObject {
    enum class Phase {
        FirstFrame,
        Idle,
        Opening,
        Search,
        SearchFrame,
        Sort,
        SortFrame,
        Reset,
        ResetFrame,
        Target,
        Extract,
        Closing,
        CancelOpening,
        Cancelling,
        Done
    };
    Phase m_phase = Phase::FirstFrame;
    BrowserController *m_controller;
    QQuickWindow *m_window;
    QString m_path, m_extractTarget;
    QElapsedTimer m_stage;
    void emitEvent(const QString &name, QJsonObject data = {}) {
        data.insert(QStringLiteral("event"), name);
        const auto encoded = QJsonDocument(data).toJson(QJsonDocument::Compact);
        std::printf("XDZIP_PROBE %s\n", encoded.constData());
        std::fflush(stdout);
    }
    int m_idleMs = 1500, m_browseMs = 0, m_targetAttempts = 0;
    void closeForCancel() {
        m_phase = Phase::Closing;
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
    }
    void queryFrame() {
        if (m_phase == Phase::SearchFrame) {
            emitEvent(QStringLiteral("search_frame"),
                      {{"elapsed_ms", m_stage.nsecsElapsed() / 1e6},
                       {"rows", m_controller->entries()->rowCount()}});
            m_phase = Phase::Sort;
            m_stage.restart();
            m_controller->sortByColumn(1);
        } else if (m_phase == Phase::SortFrame) {
            emitEvent(QStringLiteral("sort_frame"), {{"elapsed_ms", m_stage.nsecsElapsed() / 1e6}});
            m_phase = Phase::Reset;
            m_controller->search(QString());
        } else if (m_phase == Phase::ResetFrame) {
            m_phase = Phase::Closing;
            emitEvent(QStringLiteral("browse_stable"));
            QTimer::singleShot(m_browseMs, this, [this] {
                emitEvent(QStringLiteral("browse_finished"));
                if (m_extractTarget.isEmpty()) {
                    closeForCancel();
                } else {
                    m_phase = Phase::Target;
                    m_controller->prepareExtraction(true);
                    m_controller->checkTarget(m_extractTarget);
                }
            });
        }
    }
    void changed() {
        if (m_phase == Phase::Opening && m_controller->ready() && !m_controller->browsingBusy()) {
            m_phase = Phase::Closing;
            emitEvent(QStringLiteral("open_ready"),
                      {{"elapsed_ms", m_stage.elapsed()},
                       {"records", m_controller->archiveTotal()},
                       {"visible_rows", m_controller->entries()->rowCount()}});
            m_phase = Phase::Search;
            m_stage.restart();
            m_controller->search(QStringLiteral("item-000"));
        } else if ((m_phase == Phase::Search || m_phase == Phase::Sort ||
                    m_phase == Phase::Reset) &&
                   m_controller->ready() && !m_controller->browsingBusy()) {
            m_phase = m_phase == Phase::Search ? Phase::SearchFrame
                      : m_phase == Phase::Sort ? Phase::SortFrame
                                               : Phase::ResetFrame;
            m_window->update();
        } else if (m_phase == Phase::Target && !m_controller->busy()) {
            if (!m_controller->targetMatches(m_extractTarget)) {
                if (++m_targetAttempts > 3) {
                    emitEvent(QStringLiteral("target_check_failed"));
                    QCoreApplication::exit(6);
                } else {
                    m_controller->checkTarget(m_extractTarget);
                }
                return;
            }
            m_phase = Phase::Extract;
            m_stage.restart();
            emitEvent(QStringLiteral("extract_requested"));
            m_controller->startExtraction(m_extractTarget, true, false, 0, false, false);
        } else if (m_phase == Phase::Extract && !m_controller->tasks().isEmpty() &&
                   m_controller->tasks().first().toMap().value("terminal").toBool()) {
            const auto task = m_controller->tasks().first().toMap();
            m_phase = Phase::Done;
            emitEvent(QStringLiteral("extract_finished"),
                      {{"elapsed_ms", m_stage.nsecsElapsed() / 1e6},
                       {"status", task.value("status").toString()},
                       {"committed_bytes", task.value("committedBytes").toString()}});
            if (task.value("status").toString() != QStringLiteral("succeeded")) {
                QCoreApplication::exit(7);
                return;
            }
            emitEvent(QStringLiteral("quit_requested"));
            QMetaObject::invokeMethod(m_window, "requestDesktopExit");
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
                QMetaObject::invokeMethod(m_window, "requestDesktopExit");
            });
        }
    }

  public:
    DesktopProbe(BrowserController *controller, QQuickWindow *window, const QString &path,
                 QObject *parent)
        : QObject(parent), m_controller(controller), m_window(window), m_path(path) {
        const auto duration = [](const char *name, int fallback, int maximum) {
            bool ok = false;
            int value = qEnvironmentVariableIntValue(name, &ok);
            return ok && value >= 0 && value <= maximum ? value : fallback;
        };
        if (qEnvironmentVariable("XDZIP_PROBE_OPERATION") == QStringLiteral("extract")) {
            m_extractTarget = qEnvironmentVariable("XDZIP_PROBE_EXTRACT_TARGET");
            const QDir target(m_extractTarget);
            if (m_extractTarget.isEmpty() || !QDir::isAbsolutePath(m_extractTarget) ||
                !target.exists() ||
                !target.entryList(QDir::NoDotAndDotDot | QDir::AllEntries).isEmpty()) {
                QTimer::singleShot(0, this, [] { QCoreApplication::exit(6); });
                return;
            }
        }
        m_idleMs = duration("XDZIP_PROBE_IDLE_MS", 1500, 90000);
        m_browseMs = duration("XDZIP_PROBE_BROWSE_MS", 0, 60000);
        connect(controller, &BrowserController::changed, this, [this] { changed(); });
        connect(controller, &BrowserController::tasksChanged, this, [this] { changed(); });
        connect(window, &QQuickWindow::frameSwapped, this, [this] {
            if (m_phase != Phase::FirstFrame) {
                queryFrame();
                return;
            }
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
            if (qEnvironmentVariableIsSet("XDZIP_PROBE_RESOURCE_MANIFEST")) {
                QJsonArray resources;
                QDirIterator iterator(QStringLiteral(":/xdzip"), QDir::Files,
                                      QDirIterator::Subdirectories);
                while (iterator.hasNext()) {
                    QFile file(iterator.next());
                    if (!file.open(QIODevice::ReadOnly) || file.size() > 32 * 1024 * 1024) {
                        QCoreApplication::exit(11);
                        return;
                    }
                    resources.append(QJsonObject{
                        {"resource", file.fileName()},
                        {"sha256",
                         QString::fromLatin1(
                             QCryptographicHash::hash(file.readAll(), QCryptographicHash::Sha256)
                                 .toHex())}});
                }
                emitEvent(QStringLiteral("resource_manifest"), {{"files", resources}});
            }
            const auto expectedProof = qEnvironmentVariable("XDZIP_EXPECT_QT_SOURCE_PROOF");
            if (!expectedProof.isEmpty()) {
                // The literal symbol/ABI is exported only by our recorded QtCore
                // source patch. Resolve in the already loaded process, never a
                // caller-selected library. Missing/mismatched proof fails closed.
                using Proof = const char *(*)();
#ifdef Q_OS_WIN
                auto proof = reinterpret_cast<Proof>(GetProcAddress(
                    GetModuleHandleW(L"Qt6Core.dll"), "xdzip_qt_source_rebuild_proof"));
#else
                auto proof = reinterpret_cast<Proof>(dlsym(RTLD_DEFAULT, "xdzip_qt_source_rebuild_proof"));
#endif
                const auto actual = proof ? QString::fromUtf8(proof()) : QString();
                emitEvent(QStringLiteral("qt_source_proof"), {{"value", actual}});
                if (actual != expectedProof) {
                    QCoreApplication::exit(10);
                    return;
                }
            }
            QTimer::singleShot(m_idleMs, this, [this] {
                m_phase = Phase::Opening;
                m_stage.start();
                emitEvent(QStringLiteral("open_requested"));
                m_controller->openPath(m_path);
            });
        });
        QTimer::singleShot((m_extractTarget.isEmpty() ? 30000 : 180000) + m_idleMs + m_browseMs,
                           this, [this] {
                               if (m_phase != Phase::Done) {
                                   emitEvent(QStringLiteral("timeout"));
                                   QCoreApplication::exit(8);
                               }
                           });
    }
};
