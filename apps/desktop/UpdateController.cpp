// apps/desktop/UpdateController.cpp

#include "UpdateController.h"
#include "ApplicationSessions.h"
#include "BuildInfo.h"
#include "UiText.h"
#include "UpdateHandoff.h"
#include "xdzip-qt-bridge/src/update.rs.h"
#include <QCoreApplication>
#include <QCryptographicHash>
#include <QDateTime>
#include <QDesktopServices>
#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QJsonDocument>
#include <QNetworkReply>
#include <QNetworkRequest>
#include <QOperatingSystemVersion>
#include <QSettings>
#include <QStandardPaths>
#include <QSysInfo>
#include <QVersionNumber>
#include <algorithm>
#ifdef Q_OS_MACOS
#include "MacUpdateProvider.h"
#endif

namespace {
constexpr qsizetype listLimit = 8 * 1024 * 1024;
constexpr qsizetype manifestLimit = 256 * 1024;
constexpr qint64 packageLimit = 2LL * 1024 * 1024 * 1024;
QString fromRust(const rust::String &value) {
    return QString::fromUtf8(value.data(), qsizetype(value.size()));
}
rust::Str view(const QByteArray &value) {
    return {value.constData(), size_t(value.size())};
}
QUrl manifestUrl(const QJsonObject &release, const QString &name) {
    for (const auto &value : release.value("assets").toArray()) {
        const auto asset = value.toObject();
        if (asset.value("name").toString() == name)
            return QUrl(asset.value("browser_download_url").toString());
    }
    return {};
}
QByteArray trustKeys() {
    QFile input(QStringLiteral(":/xdzip/update-trust.json"));
    return input.open(QIODevice::ReadOnly) ? input.read(16 * 1024) : QByteArray{};
}
bool supportedSystem() {
#ifdef Q_OS_WIN
    const auto system = QOperatingSystemVersion::current();
    return system.majorVersion() >= 10 && system.microVersion() >= 22000;
#elif defined(Q_OS_MACOS)
    return QOperatingSystemVersion::current() >=
           QOperatingSystemVersion(QOperatingSystemVersion::MacOS, 14);
#elif defined(Q_OS_LINUX)
    return QSysInfo::productType() == "ubuntu" &&
           QVersionNumber::fromString(QSysInfo::productVersion()) >= QVersionNumber(24, 4);
#else
    return false;
#endif
}
QString updateCacheRoot() {
    return QStandardPaths::writableLocation(QStandardPaths::AppLocalDataLocation) + "/updates";
}
QString updateStatePath() {
    return QFileInfo(uiSettingsPath()).dir().filePath("update-state.ini");
}
} // namespace

UpdateController::UpdateController(QObject *parent, QNetworkAccessManager *transport,
                                   const QByteArray &trust, const QString &cacheRoot,
                                   const std::function<bool()> &systemSupport)
    : QObject(parent), m_network(transport ? transport : new QNetworkAccessManager(this)),
      m_trustOverride(trust), m_cacheRootOverride(cacheRoot),
      m_systemSupport(systemSupport ? systemSupport : supportedSystem) {
    m_waitTasks.setInterval(200);
    connect(&m_waitTasks, &QTimer::timeout, this, &UpdateController::startInstallation);
    if (!transport && cacheRoot.isEmpty())
        restoreInstallationResult();
}
UpdateController::~UpdateController() {
    ++m_generation;
    if (m_reply)
        m_reply->abort();
#ifdef Q_OS_MACOS
    cancelMacUpdate();
#endif
}
void UpdateController::reportNativeProgress(const QString &state, qreal progress) {
    m_progress = progress;
    setState(state == "error" && m_sessions && m_sessions->updatePending()
                 ? QStringLiteral("recovery_required")
                 : state,
             state == "error" ? QStringLiteral("native_update") : QString{});
}
void UpdateController::setSessions(ApplicationSessions *sessions) {
    m_sessions = sessions;
    connect(sessions, &ApplicationSessions::readyForUpdate, this,
            &UpdateController::finishPreparation);
}
bool UpdateController::busy() const {
    return m_state == "checking" || m_state == "downloading" || m_state == "verifying" ||
           m_state == "waiting_tasks" || m_state == "preparing" || m_state == "installing";
}
QString UpdateController::platform() const {
#ifdef Q_OS_WIN
    return QStringLiteral("windows");
#elif defined(Q_OS_MACOS)
    return QStringLiteral("macos");
#elif defined(Q_OS_LINUX)
    return QStringLiteral("linux");
#else
    return {};
#endif
}
QString UpdateController::installedRoot() const {
    const auto revision = BuildInfo::revision();
    const bool knownRevision =
        revision.size() == 12 && std::all_of(revision.cbegin(), revision.cend(), [](QChar value) {
            return (value >= u'0' && value <= u'9') || (value >= u'a' && value <= u'f');
        });
    if (BuildInfo::dirty() || BuildInfo::channel() == "development" || !knownRevision ||
        BuildInfo::configuration() != "Release")
        return {};
    const auto current = QFileInfo(QCoreApplication::applicationFilePath()).canonicalFilePath();
#ifdef Q_OS_WIN
    QSettings install(QStringLiteral("HKEY_CURRENT_USER\\Software\\xdzip\\Install"),
                      QSettings::NativeFormat);
    const auto root = QFileInfo(install.value("Location").toString()).canonicalFilePath();
    if (install.value("InstallId").toString() == "56DF0D26-2B26-40C4-9E85-FF0F8C8937CA" &&
        !root.isEmpty() &&
        QString::compare(current, root + "/xdzip.exe", Qt::CaseInsensitive) == 0 &&
        QFileInfo(root + "/install-manifest.txt").isFile())
        return root;
#elif defined(Q_OS_LINUX)
    auto root = QFileInfo(current).absolutePath();
    if (QFileInfo(root).fileName() == "bin")
        root = QFileInfo(root).absolutePath();
    if (root.startsWith("/opt/xdzip/versions/") && QFileInfo(current).ownerId() == 0 &&
        QFileInfo("/usr/libexec/xdzip-update-helper").ownerId() == 0 &&
        QFileInfo(root + "/share/xdzip-install.json").isFile())
        return root;
#elif defined(Q_OS_MACOS)
    return macUpdateInstallRoot();
#endif
    return {};
}
QString UpdateController::packageType() const {
    if (platform() == "windows")
        return installedRoot().isEmpty() ? "zip" : "exe";
    if (platform() == "linux")
        return installedRoot().isEmpty() ? "tar.gz" : "deb";
    return QStringLiteral("app.zip");
}
bool UpdateController::canInstall() const {
    return m_trusted && !installedRoot().isEmpty() && m_sessions &&
           (platform() != "macos" || !m_candidate.value("appcast_url").toString().isEmpty());
}
QString UpdateController::installReason() const {
    if (!m_trusted)
        return QStringLiteral("unsigned");
    if (installedRoot().isEmpty())
        return QStringLiteral("manual_install");
    if (!m_sessions)
        return QStringLiteral("preview");
    if (platform() == "macos" && m_candidate.value("appcast_url").toString().isEmpty())
        return QStringLiteral("manual_install");
    return {};
}
void UpdateController::setState(const QString &state, const QString &error) {
    m_state = state;
    m_error = error;
    emit changed();
}
bool UpdateController::allowedUrl(const QUrl &url) {
    if (url.scheme() != "https" || !url.userInfo().isEmpty() ||
        (url.port(-1) != -1 && url.port() != 443))
        return false;
    const auto host = url.host().toLower();
    return host == "api.github.com" || host == "github.com" ||
           host == "release-assets.githubusercontent.com" ||
           host == "objects.githubusercontent.com" ||
           host == "github-releases.githubusercontent.com";
}
void UpdateController::fetch(const QUrl &url, qsizetype limit, FetchCallback callback) {
    if (!allowedUrl(url)) {
        callback({}, "invalid_url");
        return;
    }
    QNetworkRequest request(url);
    request.setRawHeader("User-Agent", "xdzip/" + BuildInfo::version().toUtf8());
    request.setRawHeader("Accept", "application/vnd.github+json");
    request.setTransferTimeout(15000);
    request.setMaximumRedirectsAllowed(5);
    request.setAttribute(QNetworkRequest::RedirectPolicyAttribute,
                         QNetworkRequest::UserVerifiedRedirectPolicy);
    const auto cacheKey = url.toString();
    if (m_cache.contains(cacheKey))
        request.setRawHeader("If-None-Match", m_cache[cacheKey].etag);
    auto *reply = m_network->get(request);
    m_reply = reply;
    const auto generation = m_generation;
    auto bytes = std::make_shared<QByteArray>();
    auto overflow = std::make_shared<bool>(false);
    auto timedOut = std::make_shared<bool>(false);
    QTimer::singleShot(30000, reply, [reply, timedOut] {
        if (!reply->isFinished()) {
            *timedOut = true;
            reply->abort();
        }
    });
    connect(reply, &QNetworkReply::redirected, reply, [reply](const QUrl &target) {
        if (allowedUrl(target))
            reply->redirectAllowed();
        else
            reply->abort();
    });
    connect(reply, &QIODevice::readyRead, reply, [reply, bytes, overflow, limit] {
        const auto chunk = reply->readAll();
        if (bytes->size() > limit - chunk.size()) {
            *overflow = true;
            reply->abort();
        } else
            bytes->append(chunk);
    });
    connect(reply, &QNetworkReply::finished, this,
            [this, reply, bytes, overflow, timedOut, generation, callback = std::move(callback),
             cacheKey] {
                reply->deleteLater();
                if (generation != m_generation)
                    return;
                m_reply = nullptr;
                const auto status =
                    reply->attribute(QNetworkRequest::HttpStatusCodeAttribute).toInt();
                if (status == 304 && m_cache.contains(cacheKey)) {
                    callback(m_cache[cacheKey].bytes, {});
                    return;
                }
                if (*overflow) {
                    callback({}, "response_limit");
                    return;
                }
                if (*timedOut) {
                    callback({}, "timeout");
                    return;
                }
                if (status == 404) {
                    callback({}, "not_found");
                    return;
                }
                if (status == 403 || status == 429) {
                    callback({}, "rate_limited");
                    return;
                }
                if (reply->error() != QNetworkReply::NoError || status != 200) {
                    callback({}, reply->error() == QNetworkReply::SslHandshakeFailedError ? "tls"
                                 : reply->error() == QNetworkReply::TimeoutError ? "timeout"
                                                                                 : "network");
                    return;
                }
                if (m_cache.size() >= 24)
                    m_cache.clear();
                m_cache.insert(cacheKey, {reply->rawHeader("ETag"), *bytes});
                callback(*bytes, {});
            });
}
void UpdateController::check() {
    if (busy())
        return;
    ++m_generation;
    m_candidate = {};
    m_manifest.clear();
    m_signature.clear();
    m_packagePath.clear();
    m_trusted = false;
    m_progress = -1;
    m_candidates.clear();
    setState("checking");
    fetchPage(1);
}
void UpdateController::fetchPage(int page) {
    fetch(QUrl(QStringLiteral(
                   "https://api.github.com/repos/xiaodaozhi/xdzip/releases?per_page=100&page=%1")
                   .arg(page)),
          listLimit, [this, page](QByteArray bytes, QString error) {
              if (!error.isEmpty()) {
                  setState("error", error);
                  return;
              }
              QJsonParseError parse;
              const auto document = QJsonDocument::fromJson(bytes, &parse);
              if (parse.error != QJsonParseError::NoError || !document.isArray()) {
                  setState("error", "invalid_response");
                  return;
              }
              const auto values = document.array();
              for (const auto &value : values) {
                  const auto release = value.toObject();
                  if (!release.value("draft").toBool() &&
                      xdzip::updates::release_tag_eligible(
                          view(release.value("tag_name").toString().toUtf8()),
                          view(BuildInfo::version().toUtf8()), view(BuildInfo::channel().toUtf8())))
                      m_candidates.append(release);
              }
              if (values.size() == 100) {
                  if (page >= 10) {
                      setState("error", "page_limit");
                      return;
                  }
                  fetchPage(page + 1);
                  return;
              }
              std::sort(m_candidates.begin(), m_candidates.end(), [](const auto &a, const auto &b) {
                  return xdzip::updates::compare_versions(
                             view(a.value("tag_name").toString().mid(1).toUtf8()),
                             view(b.value("tag_name").toString().mid(1).toUtf8())) == 1;
              });
              inspectNextRelease();
          });
}
void UpdateController::inspectNextRelease() {
    if (m_candidates.isEmpty()) {
        setState("up_to_date");
        return;
    }
    const auto release = m_candidates.takeFirst();
    const auto url = manifestUrl(release, "release.json");
    const auto tag = release.value("tag_name").toString();
    const auto expected =
        "https://github.com/xiaodaozhi/xdzip/releases/download/" + tag + "/release.json";
    if (url.toString() != expected) {
        setState("error", "missing_manifest");
        return;
    }
    fetch(url, manifestLimit, [this, release](QByteArray bytes, QString error) {
        if (!error.isEmpty()) {
            setState("error", error);
            return;
        }
        acceptManifest(release, bytes);
    });
}
void UpdateController::acceptManifest(const QJsonObject &release, const QByteArray &manifest) {
    const auto inspection = xdzip::updates::inspect_release(
        view(manifest), view(BuildInfo::version().toUtf8()), quint32(BuildInfo::buildNumber()),
        view(BuildInfo::channel().toUtf8()), view(platform().toUtf8()),
        view(QSysInfo::currentCpuArchitecture().toUtf8()), view(packageType().toUtf8()));
    if (!inspection.valid) {
        setState("error", fromRust(inspection.error));
        return;
    }
    if (!inspection.newer) {
        inspectNextRelease();
        return;
    }
    if (!m_systemSupport()) {
        setState("error", "minimum_system");
        return;
    }
    if (QStringLiteral("v") + fromRust(inspection.version) !=
        release.value("tag_name").toString()) {
        setState("error", "release_identity");
        return;
    }
    const auto asset = fromRust(inspection.asset_url);
    bool sameAsset = false;
    for (const auto &value : release.value("assets").toArray()) {
        const auto file = value.toObject();
        if (file.value("name").toString() == fromRust(inspection.filename) &&
            file.value("browser_download_url").toString() == asset &&
            quint64(file.value("size").toDouble()) == inspection.bytes)
            sameAsset = true;
    }
    if (!sameAsset || inspection.bytes == 0 || inspection.bytes > quint64(packageLimit)) {
        setState("error", "release_asset");
        return;
    }
    m_candidate = {{"version", fromRust(inspection.version)},
                   {"build", int(inspection.build)},
                   {"channel", fromRust(inspection.channel)},
                   {"release_url", fromRust(inspection.release_url)},
                   {"asset_url", asset},
                   {"filename", fromRust(inspection.filename)},
                   {"bytes", double(inspection.bytes)},
                   {"sha256", fromRust(inspection.sha256)},
                   {"appcast_url", fromRust(inspection.appcast_url)}};
    m_manifest = manifest;
    const auto signatureUrl = manifestUrl(release, "release.json.sig");
    if (signatureUrl.isEmpty()) {
        setState("available");
        return;
    }
    const auto expected = "https://github.com/xiaodaozhi/xdzip/releases/download/" +
                          release.value("tag_name").toString() + "/release.json.sig";
    if (signatureUrl.toString() != expected) {
        setState("error", "invalid_signature_url");
        return;
    }
    fetch(signatureUrl, 4096, [this](QByteArray signature, QString error) {
        if (!error.isEmpty()) {
            setState("error", error);
            return;
        }
        m_signature = signature;
        const auto trust = m_trustOverride.isEmpty() ? trustKeys() : m_trustOverride;
        const auto verification = fromRust(xdzip::updates::verify_manifest(
            rust::Slice<const uint8_t>(reinterpret_cast<const uint8_t *>(m_manifest.constData()),
                                       size_t(m_manifest.size())),
            view(signature), view(trust)));
        if (!verification.isEmpty()) {
            // A new production key is an explicit release prerequisite. An unsigned
            // or unknown-key release remains available through its official page.
            if (verification == "unknown_key" || verification == "empty_keyring" ||
                verification == "untrusted_signing_key") {
                setState("available");
                return;
            }
            setState("error", verification);
            return;
        }
        m_trusted = true;
        setState("available");
    });
}
bool UpdateController::ensureCache() {
    if (m_directory && m_directory->isValid())
        return true;
    const auto base = m_cacheRootOverride.isEmpty() ? updateCacheRoot() : m_cacheRootOverride;
    if (!QDir().mkpath(base))
        return false;
    m_directory = std::make_unique<QTemporaryDir>(base + "/job-XXXXXX");
    return m_directory->isValid();
}
void UpdateController::download() {
    beginDownload(false);
}
void UpdateController::downloadAndInstall() {
    if (!canInstall()) {
        openReleasePage();
        return;
    }
    m_installAfterDownload = true;
    if (downloaded())
        startInstallation();
    else
        beginDownload(true);
}
void UpdateController::beginDownload(bool install) {
    if (busy() || m_candidate.isEmpty())
        return;
    if (!m_trusted) {
        openReleasePage();
        return;
    }
    m_installAfterDownload = install;
#ifdef Q_OS_MACOS
    if (install) {
        startInstallation();
        return;
    }
#endif
    if (!ensureCache()) {
        setState("error", "cache");
        return;
    }
    const auto file = m_candidate.value("filename").toString();
    m_output = std::make_unique<QSaveFile>(m_directory->filePath(file));
    if (!m_output->open(QIODevice::WriteOnly)) {
        setState("error", "disk");
        return;
    }
    QNetworkRequest request(QUrl(m_candidate.value("asset_url").toString()));
    request.setRawHeader("User-Agent", "xdzip/" + BuildInfo::version().toUtf8());
    request.setTransferTimeout(30000);
    request.setMaximumRedirectsAllowed(5);
    request.setAttribute(QNetworkRequest::RedirectPolicyAttribute,
                         QNetworkRequest::UserVerifiedRedirectPolicy);
    auto *reply = m_network->get(request);
    m_reply = reply;
    const auto generation = ++m_generation;
    const auto expected = qint64(m_candidate.value("bytes").toDouble());
    auto count = std::make_shared<qint64>(0);
    auto hash = std::make_shared<QCryptographicHash>(QCryptographicHash::Sha256);
    auto failure = std::make_shared<QString>();
    QTimer::singleShot(3600000, reply, [reply, failure] {
        if (!reply->isFinished()) {
            *failure = "timeout";
            reply->abort();
        }
    });
    m_progress = 0;
    setState("downloading");
    connect(reply, &QNetworkReply::redirected, reply, [reply](const QUrl &target) {
        if (allowedUrl(target))
            reply->redirectAllowed();
        else
            reply->abort();
    });
    connect(reply, &QIODevice::readyRead, this,
            [this, reply, generation, count, expected, hash, failure] {
                if (generation != m_generation)
                    return;
                const auto bytes = reply->readAll();
                if (bytes.size() > expected - *count) {
                    *failure = "download_size";
                    reply->abort();
                    return;
                }
                if (m_output->write(bytes) != bytes.size()) {
                    *failure = "disk";
                    reply->abort();
                    return;
                }
                hash->addData(bytes);
                *count += bytes.size();
                m_progress = qreal(*count) / qreal(expected);
                emit changed();
            });
    connect(reply, &QNetworkReply::finished, this,
            [this, reply, generation, count, expected, hash, failure, file] {
                reply->deleteLater();
                if (generation != m_generation)
                    return;
                m_reply = nullptr;
                if (!failure->isEmpty() || reply->error() != QNetworkReply::NoError ||
                    reply->attribute(QNetworkRequest::HttpStatusCodeAttribute).toInt() != 200 ||
                    *count != expected) {
                    m_output->cancelWriting();
                    m_output.reset();
                    setState("error", failure->isEmpty() ? "download" : *failure);
                    return;
                }
                setState("verifying");
                if (hash->result().toHex() != m_candidate.value("sha256").toString().toLatin1()) {
                    m_output->cancelWriting();
                    m_output.reset();
                    setState("error", "download_hash");
                    return;
                }
                if (!m_output->commit()) {
                    m_output.reset();
                    setState("error", "disk");
                    return;
                }
                m_output.reset();
                m_packagePath = m_directory->filePath(file);
                m_directory->setAutoRemove(false);
                m_progress = 1;
                setState("downloaded");
                if (m_installAfterDownload)
                    startInstallation();
            });
}
void UpdateController::startInstallation() {
    if (!canInstall() || !m_installAfterDownload) {
        m_waitTasks.stop();
        return;
    }
    if (!m_sessions || m_sessions->isClosing()) {
        m_waitTasks.stop();
        setState("error", "closing");
        return;
    }
    if (m_sessions->activeTaskCount()) {
        setState("waiting_tasks");
        m_waitTasks.start();
        return;
    }
    m_waitTasks.stop();
#ifdef Q_OS_MACOS
    setState("preparing");
    startMacUpdate(this, m_candidate, [this](QString error) {
        if (!error.isEmpty()) {
            setState("error", error);
            return;
        }
        if (!rememberInstallation()) {
            cancelMacUpdate();
            setState("error", "handoff_receipt");
            return;
        }
        if (!m_sessions->prepareForUpdate()) {
            QSettings settings(updateStatePath(), QSettings::IniFormat);
            settings.remove("update/pending");
            settings.sync();
            cancelMacUpdate();
            setState("error", "busy");
            return;
        }
    });
#else
    if (!ensureCache() || m_packagePath.isEmpty()) {
        setState("error", "cache");
        return;
    }
    m_handoff = std::make_unique<UpdateHandoff>(this);
    connect(m_handoff.get(), &UpdateHandoff::ready, this, [this] {
        if (!rememberInstallation()) {
            abandonHandoff();
            setState("error", "handoff_receipt");
            return;
        }
        if (!m_sessions || !m_sessions->prepareForUpdate()) {
            QSettings settings(updateStatePath(), QSettings::IniFormat);
            settings.remove("update/pending");
            settings.sync();
            abandonHandoff();
            setState("available", "busy");
        }
    });
    connect(m_handoff.get(), &UpdateHandoff::failed, this, [this](const QString &error) {
        abandonHandoff();
        setState("error", error);
    });
    m_directory->setAutoRemove(false);
    setState("preparing");
    m_handoff->prepare(m_directory->path(), installedRoot(), m_manifest, m_signature,
                       m_packagePath);
#endif
}
void UpdateController::finishPreparation() {
    setState("installing");
    // Release the application's normal quit interceptor only after every Rust
    // connection joined, before a native installer can synchronously terminate it.
    emit exitBarrierReached();
#ifdef Q_OS_MACOS
    if (!commitMacUpdate())
        setState("recovery_required", "handoff");
#else
    if (!m_handoff || !m_handoff->commit()) {
        setState("recovery_required", "handoff");
        return;
    }
    emit quitForUpdate();
#endif
}
void UpdateController::abandonHandoff() {
    if (!m_handoff)
        return;
    m_handoff->cancel();
    m_handoff.release()->deleteLater();
    // A running helper may still own files in this job. Preserve it as evidence
    // and use a new private directory and transaction for the next attempt.
    if (m_directory)
        m_directory->setAutoRemove(false);
    m_directory.reset();
    m_packagePath.clear();
}
bool UpdateController::rememberInstallation() {
    QJsonObject pending{{"version", candidateVersion()},
                        {"build", m_candidate.value("build")},
                        {"release_url", m_candidate.value("release_url")}};
#ifdef Q_OS_MACOS
    pending.insert("kind", "sparkle");
#else
    if (!m_directory || !m_handoff)
        return false;
    pending.insert("kind", "helper");
    pending.insert("directory", m_directory->path());
    pending.insert("transaction", m_handoff->transaction());
    pending.insert("filename", m_candidate.value("filename"));
#endif
    QSettings settings(updateStatePath(), QSettings::IniFormat);
    settings.setValue("update/pending", QJsonDocument(pending).toJson(QJsonDocument::Compact));
    settings.sync();
    return settings.status() == QSettings::NoError;
}
void UpdateController::restoreInstallationResult() {
    QSettings settings(updateStatePath(), QSettings::IniFormat);
    const auto bytes = settings.value("update/pending").toByteArray();
    if (bytes.isEmpty())
        return;
    settings.remove("update/pending");
    settings.sync();
    if (bytes.size() > 8192) {
        setState("recovery_required", "handoff_receipt");
        return;
    }
    const auto pending = QJsonDocument::fromJson(bytes).object();
    m_candidate = {{"version", pending.value("version")},
                   {"release_url", pending.value("release_url")}};
#ifdef Q_OS_MACOS
    if (pending.value("kind") == "sparkle") {
        const bool installed = BuildInfo::version() == pending.value("version").toString() &&
                               BuildInfo::buildNumber() == pending.value("build").toInt() &&
                               !installedRoot().isEmpty();
        setState(installed ? "updated" : "recovery_required",
                 installed ? "" : "installation_interrupted");
        return;
    }
#endif
    if (pending.value("kind") != "helper") {
        setState("recovery_required", "handoff_receipt");
        return;
    }
    const auto directory = pending.value("directory").toString();
    const auto base = QFileInfo(updateCacheRoot()).canonicalFilePath();
    const auto canonical = QFileInfo(directory).canonicalFilePath();
    if (base.isEmpty() || canonical.isEmpty() || QFileInfo(directory).isSymLink() ||
        !canonical.startsWith(base + '/') || !QFileInfo(canonical).fileName().startsWith("job-")) {
        setState("recovery_required", "handoff_receipt");
        return;
    }
    const auto filename = pending.value("filename").toString();
    const QFileInfo package(canonical + '/' + filename);
    if (!filename.isEmpty() && QFileInfo(filename).fileName() == filename && !package.isSymLink() &&
        package.isFile() && package.canonicalPath() == canonical)
        m_packagePath = package.canonicalFilePath();
    QFile result(canonical + "/result.json");
    if (QFileInfo(result).isSymLink() || !result.open(QIODevice::ReadOnly) ||
        result.size() > 16384) {
        setState("recovery_required", "installation_interrupted");
        return;
    }
    const auto receipt = QJsonDocument::fromJson(result.readAll()).object();
    if (receipt.value("transaction") != pending.value("transaction") ||
        pending.value("transaction").toString().size() != 32) {
        setState("recovery_required", "handoff_receipt");
        return;
    }
    const auto error = receipt.value("error").toString("installation_failed");
    if (!error.isEmpty() || BuildInfo::version() != pending.value("version").toString() ||
        BuildInfo::buildNumber() != pending.value("build").toInt() || installedRoot().isEmpty()) {
        setState("recovery_required", error.isEmpty() ? "installed_identity" : error);
        return;
    }
    setState("updated");
}
void UpdateController::cancel() {
    if (m_state == "installing" || (m_sessions && m_sessions->updatePending()))
        return;
    ++m_generation;
    m_waitTasks.stop();
    abandonHandoff();
#ifdef Q_OS_MACOS
    cancelMacUpdate();
#endif
    if (m_reply) {
        m_reply->abort();
        m_reply = nullptr;
    }
    if (m_output) {
        m_output->cancelWriting();
        m_output.reset();
    }
    m_installAfterDownload = false;
    m_progress = downloaded() ? 1 : -1;
    setState(m_candidate.isEmpty() ? "idle" : downloaded() ? "downloaded" : "available");
}
void UpdateController::openReleasePage() {
    const auto url = QUrl(m_candidate.value("release_url").toString());
    if (url.scheme() == "https" && url.host() == "github.com" &&
        url.path().startsWith("/xiaodaozhi/xdzip/releases/tag/"))
        QDesktopServices::openUrl(url);
}
void UpdateController::openDownloadFolder() {
    if (downloaded())
        QDesktopServices::openUrl(QUrl::fromLocalFile(QFileInfo(m_packagePath).absolutePath()));
}
