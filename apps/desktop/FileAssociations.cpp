#include "FileAssociations.h"
#include "BuildInfo.h"
#include "UiText.h"
#include "WindowsAssociationSettings.h"

#include <QCoreApplication>
#include <QDesktopServices>
#include <QFileDialog>
#include <QFileInfo>
#include <QGuiApplication>
#include <QProcess>
#include <QSettings>
#include <QStandardPaths>
#include <QTimer>
#include <QUrl>

#ifdef Q_OS_WIN
#include <windows.h>

#include <appmodel.h>
#include <shlwapi.h>
#endif

#ifdef Q_OS_MACOS
QStringList macAssociationExtensions();
bool macAssociationDefault(const QString &extension);
void macAssociationChange(FileAssociations *service, const QString &extension, bool enable);
#endif

namespace {
#ifdef Q_OS_WIN
QString currentApplicationId() {
    UINT32 length = 0;
    if (GetCurrentApplicationUserModelId(&length, nullptr) != ERROR_INSUFFICIENT_BUFFER ||
        length < 2 || length > 4096)
        return {};
    QString value(int(length), Qt::Uninitialized);
    if (GetCurrentApplicationUserModelId(&length, reinterpret_cast<PWSTR>(value.data())) !=
            ERROR_SUCCESS ||
        length < 2 || length > UINT32(value.size()) || value[int(length) - 1] != QChar(0))
        return {};
    value.truncate(int(length) - 1);
    return value.contains(QChar(0)) ? QString() : value;
}
#endif
#ifndef Q_OS_MACOS
const QHash<QString, QStringList> &extensions() {
    static const QHash<QString, QStringList> value{
        {".zip", {"application/zip"}},
        {".zipx", {"application/zip"}},
        {".xpi", {"application/x-xpinstall"}},
        {".apk", {"application/vnd.android.package-archive"}},
        {".ipa", {"application/zip"}},
        {".cbz", {"application/vnd.comicbook+zip", "application/x-cbz"}},
        {".7z", {"application/x-7z-compressed"}},
        {".rar", {"application/vnd.rar", "application/x-rar", "application/x-rar-compressed"}},
        {".tar", {"application/x-tar"}},
        {".tar.gz", {"application/x-compressed-tar"}},
        {".tgz", {"application/x-compressed-tar"}},
        {".gz", {"application/gzip", "application/x-gzip"}},
        {".bz2", {"application/x-bzip2", "application/x-bzip", "application/bzip2"}},
        {".bz", {"application/x-bzip2", "application/x-bzip", "application/bzip2"}},
        {".tar.bz2", {"application/x-bzip2-compressed-tar", "application/x-bzip-compressed-tar"}},
        {".tar.bz", {"application/x-bzip2-compressed-tar", "application/x-bzip-compressed-tar"}},
        {".tbz", {"application/x-bzip2-compressed-tar", "application/x-bzip-compressed-tar"}},
        {".tbz2", {"application/x-bzip2-compressed-tar", "application/x-bzip-compressed-tar"}},
        {".lzma", {"application/x-lzma"}},
        {".tar.lzma", {"application/x-lzma-compressed-tar"}},
        {".tlz", {"application/x-lzma-compressed-tar"}},
        {".lz", {"application/lzip", "application/x-lzip"}},
        {".tar.lz", {"application/x-lzip-compressed-tar"}},
        {".xz", {"application/x-xz"}},
        {".tar.xz", {"application/x-xz-compressed-tar"}},
        {".txz", {"application/x-xz-compressed-tar"}},
        {".cab", {"application/vnd.ms-cab-compressed", "application/x-cab"}},
        {".msi", {"application/x-ole-storage"}},
        {".iso", {"application/x-iso9660-image"}},
        {".udf", {"application/x-udf"}}};
    return value;
}
#endif
} // namespace

FileAssociations::FileAssociations(QObject *parent) : QObject(parent) {
#ifdef Q_OS_MACOS
    m_supportedExtensions = macAssociationExtensions();
    m_available = !m_supportedExtensions.isEmpty();
#elif defined(Q_OS_WIN)
    QSettings installed(QStringLiteral("HKEY_CURRENT_USER\\Software\\RegisteredApplications"),
                        QSettings::NativeFormat);
    m_systemManaged = BuildInfo::packaged();
    m_available = m_systemManaged || installed.value(QStringLiteral("xdzip")).isValid();
#elif defined(Q_OS_UNIX)
    m_available = !QStandardPaths::findExecutable(QStringLiteral("xdg-mime")).isEmpty() &&
                  !QStandardPaths::locate(QStandardPaths::ApplicationsLocation,
                                          QStringLiteral("xdzip.desktop"))
                       .isEmpty();
#endif
    connect(qGuiApp, &QGuiApplication::applicationStateChanged, this,
            [this](Qt::ApplicationState state) {
                if (state == Qt::ApplicationActive && m_available && !m_busy)
                    refresh();
            });
}

bool FileAssociations::isDefault(const QString &extension) const {
    return m_defaults.value(extension, false);
}
bool FileAssociations::supportsExtension(const QString &extension) const {
#ifdef Q_OS_MACOS
    return m_supportedExtensions.contains(extension);
#else
    Q_UNUSED(extension);
    return true;
#endif
}
void FileAssociations::notify() {
    ++m_revision;
    emit changed();
}
void FileAssociations::setBusy(bool busy) {
    m_busy = busy;
    notify();
}
void FileAssociations::nativeFinished(const QString &error, const QString &extension) {
    m_error = error;
    m_errorExtension = error.isEmpty() ? QString() : extension;
    m_busy = false;
    refresh();
}
void FileAssociations::refresh() {
    if (m_busy)
        return;
#if defined(Q_OS_UNIX) && !defined(Q_OS_MACOS)
    if (QStandardPaths::findExecutable(QStringLiteral("xdg-mime")).isEmpty())
        return;
#endif
    m_pendingDefaults.clear();
    if (m_systemManaged) {
        m_defaults.clear();
        m_ready = true;
        notify();
        return;
    }
#ifdef Q_OS_MACOS
    for (const auto &extension : m_supportedExtensions)
        m_pendingDefaults.insert(extension, macAssociationDefault(extension));
    m_defaults = m_pendingDefaults;
    m_ready = true;
    notify();
#elif defined(Q_OS_WIN)
    for (auto it = extensions().cbegin(); it != extensions().cend(); ++it) {
        const QString ext = QStringLiteral(".") + it.key().section('.', -1);
        DWORD size = 0;
        AssocQueryStringW(ASSOCF_NONE, ASSOCSTR_PROGID, reinterpret_cast<LPCWSTR>(ext.utf16()),
                          nullptr, nullptr, &size);
        QString id(int(size), Qt::Uninitialized);
        const HRESULT result =
            size && size < 4096 ? AssocQueryStringW(ASSOCF_NONE, ASSOCSTR_PROGID,
                                                    reinterpret_cast<LPCWSTR>(ext.utf16()), nullptr,
                                                    reinterpret_cast<LPWSTR>(id.data()), &size)
                                : E_FAIL;
        if (SUCCEEDED(result))
            id.truncate(int(size) - 1);
        m_pendingDefaults.insert(
            it.key(), SUCCEEDED(result) &&
                          (id.compare(QStringLiteral("xdzip.Archive"), Qt::CaseInsensitive) == 0 ||
                           id.startsWith(QStringLiteral("xdzip.Archive."), Qt::CaseInsensitive)));
    }
    m_defaults = m_pendingDefaults;
    m_ready = true;
    notify();
#elif defined(Q_OS_UNIX)
    setBusy(true);
    m_pending = 0;
    m_queryFailed = false;
    for (auto it = extensions().cbegin(); it != extensions().cend(); ++it) {
        m_pendingDefaults.insert(it.key(), true);
        for (const QString &mime : it.value()) {
            ++m_pending;
            const QString ext = it.key();
            auto *process = new QProcess(this);
            connect(process, &QProcess::finished, this,
                    [this, process, ext](int code, QProcess::ExitStatus status) {
                        m_queryFailed =
                            m_queryFailed || status != QProcess::NormalExit || code != 0;
                        const bool selected =
                            status == QProcess::NormalExit && code == 0 &&
                            QString::fromUtf8(process->readAllStandardOutput()).trimmed() ==
                                QStringLiteral("xdzip.desktop");
                        m_pendingDefaults[ext] = m_pendingDefaults.value(ext) && selected;
                        process->deleteLater();
                        if (--m_pending == 0) {
                            m_ready = !m_queryFailed;
                            if (m_ready)
                                m_defaults = m_pendingDefaults;
                            else
                                m_error = QStringLiteral("settings.associationFailed");
                            setBusy(false);
                        }
                    });
            connect(process, &QProcess::errorOccurred, this,
                    [process](QProcess::ProcessError error) {
                        if (error == QProcess::FailedToStart)
                            emit process->finished(-1, QProcess::CrashExit);
                    });
            process->start(QStringLiteral("xdg-mime"),
                           {QStringLiteral("query"), QStringLiteral("default"), mime});
            QTimer::singleShot(5000, process, [process] {
                if (process->state() != QProcess::NotRunning)
                    process->kill();
            });
        }
    }
#endif
}
void FileAssociations::requestChange(const QString &extension, bool enable) {
    if (!m_available || !m_ready || m_busy)
        return;
#ifdef Q_OS_MACOS
    if (!supportsExtension(extension))
        return;
#else
    if (!extensions().contains(extension))
        return;
#endif
    m_error.clear();
    m_errorExtension.clear();
#ifdef Q_OS_MACOS
    setBusy(true);
    macAssociationChange(this, extension, enable);
#elif defined(Q_OS_WIN)
    Q_UNUSED(enable);
    openSystemSettings();
#elif defined(Q_OS_UNIX)
    QString desktopId = QStringLiteral("xdzip.desktop");
    setBusy(true);
    if (!enable) {
        const QString choice =
            QFileDialog::getOpenFileName(nullptr, systemDialogText("settings.chooseDefaultApp"),
                                         QStringLiteral("/usr/share/applications"),
                                         systemDialogText("settings.desktopApplications"));
        if (choice.isEmpty()) {
            setBusy(false);
            return;
        }
        desktopId = QFileInfo(choice).fileName();
        const auto registered =
            QStandardPaths::locate(QStandardPaths::ApplicationsLocation, desktopId);
        if (desktopId == QStringLiteral("xdzip.desktop") || registered.isEmpty() ||
            QFileInfo(choice).canonicalFilePath() != QFileInfo(registered).canonicalFilePath()) {
            m_error = QStringLiteral("settings.associationFailed");
            setBusy(false);
            return;
        }
    }
    auto *process = new QProcess(this);
    connect(process, &QProcess::finished, this,
            [this, process](int code, QProcess::ExitStatus status) {
                process->deleteLater();
                nativeFinished(status == QProcess::NormalExit && code == 0
                                   ? QString()
                                   : QStringLiteral("settings.associationFailed"));
            });
    connect(process, &QProcess::errorOccurred, this, [process](QProcess::ProcessError error) {
        if (error == QProcess::FailedToStart)
            emit process->finished(-1, QProcess::CrashExit);
    });
    QStringList args{QStringLiteral("default"), desktopId};
    args.append(extensions().value(extension));
    process->start(QStringLiteral("xdg-mime"), args);
    QTimer::singleShot(5000, process, [process] {
        if (process->state() != QProcess::NotRunning)
            process->kill();
    });
#endif
}
void FileAssociations::openSystemSettings() {
#ifdef Q_OS_WIN
    if (!m_available || m_busy)
        return;
    m_error.clear();
    const auto url = file_associations::windowsSettingsUrl(
        m_systemManaged, m_systemManaged ? currentApplicationId() : QString());
    if (!QDesktopServices::openUrl(url))
        m_error = QStringLiteral("settings.associationFailed");
    notify();
#endif
}
