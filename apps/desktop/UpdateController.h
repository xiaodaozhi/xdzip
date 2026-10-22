// apps/desktop/UpdateController.h

#pragma once
#include <QByteArray>
#include <QHash>
#include <QJsonArray>
#include <QJsonObject>
#include <QNetworkAccessManager>
#include <QPointer>
#include <QSaveFile>
#include <QTemporaryDir>
#include <QTimer>
#include <QUrl>
#include <functional>
#include <memory>

class ApplicationSessions;
class QNetworkReply;
class UpdateController final : public QObject {
    Q_OBJECT
    Q_PROPERTY(QString state READ state NOTIFY changed)
    Q_PROPERTY(QString candidateVersion READ candidateVersion NOTIFY changed)
    Q_PROPERTY(QString errorCode READ errorCode NOTIFY changed)
    Q_PROPERTY(qreal progress READ progress NOTIFY changed)
    Q_PROPERTY(bool busy READ busy NOTIFY changed)
    Q_PROPERTY(bool trusted READ trusted NOTIFY changed)
    Q_PROPERTY(bool canInstall READ canInstall NOTIFY changed)
    Q_PROPERTY(bool downloaded READ downloaded NOTIFY changed)
    Q_PROPERTY(QString installReason READ installReason NOTIFY changed)
  public:
    explicit UpdateController(QObject *parent = nullptr, QNetworkAccessManager *transport = nullptr,
                              const QByteArray &trust = {}, const QString &cacheRoot = {},
                              const std::function<bool()> &systemSupport = {});
    ~UpdateController() override;
    void setSessions(ApplicationSessions *sessions);
    void reportNativeProgress(const QString &state, qreal progress);
    QString state() const {
        return m_state;
    }
    QString candidateVersion() const {
        return m_candidate.value("version").toString();
    }
    QString errorCode() const {
        return m_error;
    }
    qreal progress() const {
        return m_progress;
    }
    bool busy() const;
    bool trusted() const {
        return m_trusted;
    }
    bool canInstall() const;
    bool downloaded() const {
        return !m_packagePath.isEmpty();
    }
    QString installReason() const;
    Q_INVOKABLE void check();
    Q_INVOKABLE void cancel();
    Q_INVOKABLE void download();
    Q_INVOKABLE void downloadAndInstall();
    Q_INVOKABLE void openReleasePage();
    Q_INVOKABLE void openDownloadFolder();
  signals:
    void changed();
    void exitBarrierReached();
    void quitForUpdate();

  private:
    struct CachedResponse {
        QByteArray etag, bytes;
    };
    using FetchCallback = std::function<void(QByteArray, QString)>;
    void setState(const QString &state, const QString &error = {});
    void fetch(const QUrl &url, qsizetype limit, FetchCallback callback);
    void fetchPage(int page);
    void inspectNextRelease();
    void acceptManifest(const QJsonObject &release, const QByteArray &manifest);
    void beginDownload(bool install);
    void startInstallation();
    void finishPreparation();
    void restoreInstallationResult();
    bool rememberInstallation();
    void abandonHandoff();
    bool ensureCache();
    QString platform() const;
    QString packageType() const;
    QString installedRoot() const;
    static bool allowedUrl(const QUrl &url);
    QNetworkAccessManager *m_network;
    QByteArray m_trustOverride;
    QString m_cacheRootOverride;
    std::function<bool()> m_systemSupport;
    QPointer<QNetworkReply> m_reply;
    QHash<QString, CachedResponse> m_cache;
    QList<QJsonObject> m_candidates;
    QJsonObject m_candidate;
    QByteArray m_manifest, m_signature;
    QString m_state = QStringLiteral("idle"), m_error, m_packagePath;
    qreal m_progress = -1;
    bool m_trusted = false, m_installAfterDownload = false;
    quint64 m_generation = 0;
    std::unique_ptr<QTemporaryDir> m_directory;
    std::unique_ptr<QSaveFile> m_output;
    QPointer<ApplicationSessions> m_sessions;
    QTimer m_waitTasks;
    std::unique_ptr<class UpdateHandoff> m_handoff;
};
