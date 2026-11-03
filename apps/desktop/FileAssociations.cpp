#include "FileAssociations.h"
#include "UiText.h"

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

#include <shlwapi.h>
#endif

#ifdef Q_OS_MACOS
bool macAssociationsAvailable();
bool macAssociationDefault(const QString &extension);
void macAssociationChange(FileAssociations *service, const QString &extension, bool enable);
#endif

namespace {
const QHash<QString, QStringList> &extensions() {
    static const QHash<QString, QStringList> value{
        {".zip", {"application/zip"}},
        {".7z", {"application/x-7z-compressed"}},
        {".rar", {"application/vnd.rar", "application/x-rar"}},
        {".tar", {"application/x-tar"}},
        {".tar.gz", {"application/x-compressed-tar"}},
        {".tgz", {"application/x-compressed-tar"}},
        {".cab", {"application/vnd.ms-cab-compressed"}},
        {".iso", {"application/x-iso9660-image"}},
        {".udf", {"application/x-udf"}}};
    return value;
}
} // namespace

FileAssociations::FileAssociations(QObject *parent) : QObject(parent) {
#ifdef Q_OS_MACOS
    m_available = macAssociationsAvailable();
#elif defined(Q_OS_WIN)
    QSettings installed(QStringLiteral("HKEY_CURRENT_USER\\Software\\RegisteredApplications"),
                        QSettings::NativeFormat);
    m_available = installed.value(QStringLiteral("xdzip")).isValid();
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
void FileAssociations::notify() {
    ++m_revision;
    emit changed();
}
void FileAssociations::setBusy(bool busy) {
    m_busy = busy;
    notify();
}
void FileAssociations::nativeFinished(bool success) {
    m_error = success ? QString() : QStringLiteral("settings.associationFailed");
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
#ifdef Q_OS_MACOS
    for (auto it = extensions().cbegin(); it != extensions().cend(); ++it)
        m_pendingDefaults.insert(it.key(), macAssociationDefault(it.key()));
    m_defaults = m_pendingDefaults;
    m_ready = true;
    notify();
#elif defined(Q_OS_WIN)
    for (auto it = extensions().cbegin(); it != extensions().cend(); ++it) {
        const QString ext =
            it.key() == QStringLiteral(".tar.gz") ? QStringLiteral(".gz") : it.key();
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
    if (!m_available || !m_ready || m_busy || !extensions().contains(extension))
        return;
    m_error.clear();
#ifdef Q_OS_MACOS
    setBusy(true);
    macAssociationChange(this, extension, enable);
#elif defined(Q_OS_WIN)
    Q_UNUSED(enable);
    if (!QDesktopServices::openUrl(
            QUrl(QStringLiteral("ms-settings:defaultapps?registeredAppUser=xdzip"))))
        m_error = QStringLiteral("settings.associationFailed");
    notify();
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
                nativeFinished(status == QProcess::NormalExit && code == 0);
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
