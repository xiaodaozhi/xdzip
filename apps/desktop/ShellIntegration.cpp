#include "ShellIntegration.h"
#include "BuildInfo.h"
#include "UiLanguage.h"
#include "UiPreferences.h"
#include <QCoreApplication>
#include <QCryptographicHash>
#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QJsonDocument>
#include <QJsonObject>
#include <QSaveFile>
#include <QSettings>
#include <QStandardPaths>
#ifdef Q_OS_WIN
#include "shell/PackageMenu.h"
#include <shlobj.h>
#endif
#ifdef Q_OS_MACOS
#include "MacFinderIntegration.h"
#include "MacShellService.h"
#endif
namespace {
#ifdef Q_OS_WIN
void writeWindowsMenuPreferences(QSettings &options, bool enabled, const QString &language) {
    const auto legacy = legacyUiLanguage(language);
    options.setValue("Enabled", enabled ? 1 : 0);
    options.setValue("Language", legacy);
    if (legacy != language)
        options.setValue("ExtendedLanguage", language);
    else
        options.remove("ExtendedLanguage");
}
#endif

bool installed() {
    if (!qEnvironmentVariableIsEmpty("XDZIP_SETTINGS_PATH") ||
        !qEnvironmentVariableIsEmpty("XDZIP_LANGUAGE"))
        return false;
#ifdef Q_OS_MACOS
    const auto app = QDir::cleanPath(QCoreApplication::applicationDirPath() + "/../..");
    QFile registered(QDir::homePath() +
                     "/Library/Services/xdzip.service/Contents/Resources/application.txt");
    const bool matches = registered.open(QIODevice::ReadOnly) &&
                         QString::fromUtf8(registered.readAll()).trimmed() == app;
    return matches || macFinderIntegrationConfigured() || app == "/Applications/xdzip.app" ||
           app == QDir::homePath() + "/Applications/xdzip.app";
#elif defined(Q_OS_WIN)
    if (BuildInfo::packaged())
        return xdzip_shell::readPackageMenuContext(
                   QCoreApplication::applicationFilePath().toStdWString())
            .registered;
    QSettings install("HKEY_CURRENT_USER\\Software\\xdzip\\Install", QSettings::NativeFormat);
    const auto directory = QFileInfo(QCoreApplication::applicationDirPath()).canonicalFilePath();
    const auto location = install.value("Location").toString();
    return !directory.isEmpty() && !location.isEmpty() && QFileInfo(location).isAbsolute() &&
           install.value("InstallId")
                   .toString()
                   .compare("56DF0D26-2B26-40C4-9E85-FF0F8C8937CA", Qt::CaseInsensitive) == 0 &&
           QFileInfo(location).canonicalFilePath().compare(directory, Qt::CaseInsensitive) == 0 &&
           QFile::exists(directory + "/install-manifest.txt");
#else
    const auto dir = QCoreApplication::applicationDirPath();
    return dir.startsWith("/opt/xdzip/versions/") &&
           QFile::exists(dir + "/../share/xdzip-install.json");
#endif
}
} // namespace
ShellIntegration::ShellIntegration(UiPreferences *settings, QObject *parent)
    : QObject(parent), m_settings(settings) {
    connect(settings, &UiPreferences::savedChanged, this, &ShellIntegration::refresh);
}
bool ShellIntegration::available() const {
    return m_availableOverride.value_or(installed());
}
bool ShellIntegration::enabled() const {
    return m_settings->shellEnabled();
}
void ShellIntegration::setEnabled(bool value) {
    if (!available() || value == enabled())
        return;
    if (!apply(value)) {
        m_error = qtTrId("shell.integrationFailed");
        emit changed();
        return;
    }
    if (!m_settings->setShellEnabled(value)) {
        apply(enabled());
        m_error = qtTrId("shell.integrationFailed");
        emit changed();
        return;
    }
    m_error.clear();
    emit changed();
}
void ShellIntegration::refresh() {
    if (!available())
        return;
    m_error = apply(enabled()) ? QString{} : qtTrId("shell.integrationFailed");
    emit changed();
}
bool ShellIntegration::apply(bool value) {
    if (m_applyOverride)
        return m_applyOverride(value);
#ifdef Q_OS_WIN
    if (BuildInfo::packaged()) {
        const auto context = xdzip_shell::readPackageMenuContext(
            QCoreApplication::applicationFilePath().toStdWString());
        if (!context.registered)
            return false;
        QSettings options(QStringLiteral("HKEY_CURRENT_USER\\") +
                              QString::fromStdWString(context.settingsKey),
                          QSettings::NativeFormat);
        writeWindowsMenuPreferences(options, value, m_settings->activeLanguage());
        options.sync();
        return options.status() == QSettings::NoError;
    }
    const auto root = QCoreApplication::applicationDirPath();
    QFile manifest(root + "/shell/integration.json");
    if (!manifest.open(QIODevice::ReadOnly) || manifest.size() > 4096)
        return false;
    const auto info = QJsonDocument::fromJson(manifest.readAll()).object();
    const auto name = info.value("file").toString();
    if (!name.startsWith("xdzip-shell-") || !name.endsWith(".dll") || name.contains('/') ||
        name.contains('\\'))
        return false;
    QFile library(root + "/shell/" + name);
    if (!library.open(QIODevice::ReadOnly) ||
        QCryptographicHash::hash(library.readAll(), QCryptographicHash::Sha256).toHex() !=
            info.value("sha256").toString().toLatin1())
        return false;
    const auto clsid = QStringLiteral("{54B70170-1072-49B5-9BD2-6DFB5ECBF1B1}");
    const auto legacyClsid = QStringLiteral("{731D9A38-549E-4CDE-B35B-6EB3A2C88B71}");
    QSettings classes("HKEY_CURRENT_USER\\Software\\Classes", QSettings::NativeFormat);
    const auto classKey = "CLSID/" + clsid + "/InprocServer32";
    const auto menuKey = QStringLiteral("AllFilesystemObjects/shell/xdzip");
    const auto legacyMenuKey =
        QStringLiteral("AllFilesystemObjects/shellex/ContextMenuHandlers/xdzip");
    const auto dll = QDir::toNativeSeparators(library.fileName());
    const auto ownedClass = [&](const QString &id) {
        const auto previous = classes.value("CLSID/" + id + "/InprocServer32/Default").toString();
        if (previous.isEmpty())
            return true;
        const auto file = QFileInfo(previous);
        return file.absolutePath() == QFileInfo(dll).absolutePath() &&
               file.fileName().startsWith("xdzip-shell-") && file.fileName().endsWith(".dll");
    };
    if (!ownedClass(clsid) || !ownedClass(legacyClsid))
        return false;
    const auto handler = classes.value(menuKey + "/ExplorerCommandHandler").toString();
    if (!handler.isEmpty() && handler != clsid && handler != legacyClsid)
        return false;
    if (value) {
        classes.setValue(classKey + "/Default", dll);
        classes.setValue(classKey + "/ThreadingModel", "Apartment");
        classes.setValue(menuKey + "/ExplorerCommandHandler", clsid);
        classes.setValue(menuKey + "/Position", "Top");
        classes.setValue(menuKey + "/MultiSelectModel", "Player");
    } else {
        if (handler == clsid || handler == legacyClsid)
            classes.remove(menuKey);
        classes.remove("CLSID/" + clsid);
    }
    // Retire only our old dynamic entry when enabling or disabling integration.
    if (classes.value(legacyMenuKey + "/Default") == legacyClsid)
        classes.remove(legacyMenuKey);
    classes.remove("CLSID/" + legacyClsid);
    classes.sync();
    QSettings options("HKEY_CURRENT_USER\\Software\\xdzip\\Shell", QSettings::NativeFormat);
    writeWindowsMenuPreferences(options, value, m_settings->activeLanguage());
    options.sync();
    const bool applied =
        classes.status() == QSettings::NoError && options.status() == QSettings::NoError;
    if (applied)
        SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST, nullptr, nullptr);
    return applied;
#elif defined(Q_OS_MACOS)
    if (!updateMacShellService(value))
        return false;
    if (updateMacFinderIntegration(value, m_settings->activeLanguage()))
        return true;
    updateMacShellService(enabled());
    return false;
#else
    Q_UNUSED(value)
    return true; // Nautilus reads the same per-user persisted setting at menu creation.
#endif
}

bool ShellIntegration::registerExplicitly(bool value) {
#ifdef Q_OS_WIN
    if (BuildInfo::packaged())
        return false;
#endif
#if !defined(Q_OS_MACOS) && !defined(Q_OS_WIN)
    if (!available())
        return false;
#endif
    return apply(value);
}
