#pragma once
#include <QCoreApplication>
#include <QFile>
#ifdef Q_OS_WIN
#include <qt_windows.h>
#elif defined(Q_OS_LINUX)
#include <sys/file.h>
#endif

// Lifetime-only OS handles. No task is terminated. Installers refuse an occupied
// runtime; Linux checks the root-owned install gate while holding the shared lock.
class InstalledRuntimeGuard final {
#ifdef Q_OS_WIN
    HANDLE m_handle = nullptr;
#elif defined(Q_OS_LINUX)
    QFile m_lock;
#endif
  public:
    bool acquire() {
#ifdef Q_OS_WIN
        m_handle = CreateMutexW(nullptr, FALSE, L"Local\\xdzip.desktop.running");
        return m_handle != nullptr;
#elif defined(Q_OS_LINUX)
        if (!QCoreApplication::applicationFilePath().startsWith(
                QStringLiteral("/opt/xdzip/versions/")))
            return true;
        m_lock.setFileName(QStringLiteral("/run/xdzip/usage.lock"));
        return m_lock.open(QIODevice::ReadOnly) && flock(m_lock.handle(), LOCK_SH | LOCK_NB) == 0 &&
               !QFile::exists(QStringLiteral("/run/xdzip/installing"));
#else
        return true;
#endif
    }
    ~InstalledRuntimeGuard() {
#ifdef Q_OS_WIN
        if (m_handle)
            CloseHandle(m_handle);
#endif
        // Closing QFile releases flock, including failed acquisition/startup.
    }
};
