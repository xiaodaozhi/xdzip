#pragma once
#include "../UiPreferences.h"
#include "test_environment.h"
#include <QFile>
#include <QGuiApplication>
#include <QScopeGuard>
#include <QSettings>
#include <QStyleHints>
#include <QTemporaryDir>
#include <QVariantMap>
#ifdef Q_OS_WIN
#include <QThread>
#include <qt_windows.h>
#include <thread>
#endif

namespace settings_storage_test {
inline QByteArray read(const QString &path) {
    QFile file(path);
    return file.open(QIODevice::ReadOnly) ? file.readAll() : QByteArray{};
}

inline QVariantMap contend(const QString &root, const QString &target, bool transient) {
#ifdef Q_OS_WIN
    QVariantMap result{{"supported", true}};
    QTemporaryDir directory(root + QStringLiteral("/storage-XXXXXX"));
    const auto path = directory.filePath("settings.ini");
    test_environment::ScopedVariable profile("XDZIP_SETTINGS_PATH", path);
    if (!directory.isValid() || !profile.valid()) {
        result.insert("fixtureError", "Cannot isolate profile");
        return result;
    }
    auto *hints = QGuiApplication::styleHints();
    const auto oldScheme = hints->colorScheme();
    const auto restoreScheme = qScopeGuard([hints, oldScheme] {
        if (oldScheme == Qt::ColorScheme::Unknown)
            hints->unsetColorScheme();
        else
            hints->setColorScheme(oldScheme);
    });
    UiPreferences preferences;
    if (!preferences.save("light", 1, "auto") || !preferences.save("dark", 1.25, "en-US")) {
        result.insert("fixtureError", "Cannot seed profile");
        return result;
    }
    const auto previous = read(path);
    const auto oldBackup = read(path + ".bak");
    const auto lockedPath = target == "backup" ? path + ".bak" : path;
    // Only owned fixture files are opened. Omitting FILE_SHARE_DELETE reproduces
    // the observed Qt/Windows rename denial without changing file permissions.
    HANDLE handle = CreateFileW(reinterpret_cast<LPCWSTR>(lockedPath.utf16()), GENERIC_READ,
                                FILE_SHARE_READ | FILE_SHARE_WRITE, nullptr, OPEN_EXISTING,
                                FILE_ATTRIBUTE_NORMAL, nullptr);
    if (handle == INVALID_HANDLE_VALUE) {
        result.insert("fixtureError", "Cannot establish read handle");
        return result;
    }
    std::thread release;
    if (transient) {
        release = std::thread([handle] {
            QThread::msleep(30);
            CloseHandle(handle);
        });
    }
    const bool saved = preferences.save("light", 1.5, "zh-CN");
    // Join before restoring the environment or removing the owned directory.
    if (release.joinable())
        release.join();
    else
        CloseHandle(handle);
    QSettings written(path, QSettings::IniFormat);
    written.sync();
    result.insert("saved", saved);
    result.insert("error", preferences.error());
    result.insert("profileUnchanged", read(path) == previous);
    result.insert("backupUnchanged", read(path + ".bak") == oldBackup);
    result.insert("backupMatchesPrevious", read(path + ".bak") == previous);
    result.insert("modelUnchanged", preferences.savedTheme() == "dark" &&
                                        preferences.savedTextScale() == 1.25 &&
                                        preferences.language() == "en-US");
    result.insert("profileCommitted", written.value("appearance/theme") == "light" &&
                                          written.value("appearance/textScale").toDouble() == 1.5 &&
                                          written.value("locale/language") == "zh-CN");
    return result;
#else
    Q_UNUSED(root);
    Q_UNUSED(target);
    Q_UNUSED(transient);
    return {{"supported", false}};
#endif
}
} // namespace settings_storage_test
