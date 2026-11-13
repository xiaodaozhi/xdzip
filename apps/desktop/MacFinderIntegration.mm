#include "MacFinderIntegration.h"
#include "shell/MacFinderRequest.h"
#include <QCoreApplication>
#include <QCryptographicHash>
#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QJsonDocument>
#include <QJsonObject>
#include <QProcess>
#include <QSaveFile>

bool macFinderIntegrationConfigured() {
    const auto app = QDir(QCoreApplication::applicationDirPath() + "/../..").canonicalPath();
    return !app.isEmpty() && xdzip_shell::finderPreferences(app.toNSString()).valid;
}

bool writeMacFinderConfiguration(const QString &application, const QString &path, bool enabled,
                                 const QString &language, const QString &revision) {
    const QFileInfo parent(QFileInfo(path).absolutePath());
    if ((!parent.exists() && !QDir().mkpath(parent.filePath())) ||
        QFileInfo(parent.filePath()).isSymLink() ||
        QFileInfo(parent.filePath()).ownerId() != getuid())
        return false;
    const QFileInfo existing(path);
    QString handoffToken;
    if (existing.exists() || existing.isSymLink()) {
        if (!existing.isFile() || existing.isSymLink() || existing.ownerId() != getuid() ||
            existing.size() > 8192)
            return false;
        QFile input(path);
        if (!input.open(QIODevice::ReadOnly))
            return false;
        const auto original = QJsonDocument::fromJson(input.readAll()).object();
        if (original.value("schema_version").toInt() != 1 ||
            original.value("product") != "org.xdzip.finder-sync")
            return false;
        // Removing an older copy must not disable a newer, active installation.
        if (!enabled && original.value("application") != application)
            return true;
        if (original.value("application") == application)
            handoffToken = original.value("handoff_token").toString();
    }
    if (!xdzip_shell::validFinderToken(handoffToken.toNSString()))
        handoffToken = QString::fromStdString(xdzip_shell::randomHex() + xdzip_shell::randomHex());
    if (!xdzip_shell::validFinderToken(handoffToken.toNSString()))
        return false;
    const auto bytes = QJsonDocument(QJsonObject{{"schema_version", 1},
                                                 {"product", "org.xdzip.finder-sync"},
                                                 {"application", application},
                                                 {"enabled", enabled},
                                                 {"extension_revision", revision},
                                                 {"handoff_token", handoffToken},
                                                 {"language", language}})
                           .toJson(QJsonDocument::Compact);
    if (bytes.size() > 8192)
        return false;
    // Atomic replacement never falls back to writing through an existing leaf.
    // Only integration metadata and the local handoff secret are shared here.
    QSaveFile output(path);
    output.setDirectWriteFallback(false);
    return output.open(QIODevice::WriteOnly) &&
           output.setPermissions(QFileDevice::ReadOwner | QFileDevice::WriteOwner) &&
           output.write(bytes) == bytes.size() && output.commit();
}
bool updateMacFinderIntegration(bool enabled, const QString &language) {
    const auto application =
        QDir(QCoreApplication::applicationDirPath() + "/../..").canonicalPath();
    const auto extension = application + "/Contents/PlugIns/xdzip-finder-sync.appex";
    if (application.isEmpty() || !QFileInfo::exists(extension + "/Contents/Info.plist"))
        return false;
    const auto path = QString::fromNSString(xdzip_shell::finderConfigurationPath());
    QFile input(path);
    const bool existed = input.exists();
    QByteArray original;
    if (existed) {
        if (!input.open(QIODevice::ReadOnly) || input.size() > 8192)
            return false;
        original = input.readAll();
        input.close();
    }
    QCryptographicHash digest(QCryptographicHash::Sha256);
    for (const auto &leaf : {"/Contents/Info.plist", "/Contents/MacOS/xdzip-finder-sync"}) {
        QFile file(extension + leaf);
        if (!file.open(QIODevice::ReadOnly) || file.size() > 8 * 1024 * 1024 ||
            !digest.addData(&file))
            return false;
    }
    const auto revision = QString::fromLatin1(digest.result().toHex());
    const auto previous = QJsonDocument::fromJson(original).object();
    bool registered = false;
    if (enabled && previous.value("application") == application &&
        previous.value("extension_revision") == revision) {
        QProcess query;
        query.start("/usr/bin/pluginkit", {"-m", "-i", "org.xdzip.finder-sync", "-v"});
        if (!query.waitForFinished(10000) || query.exitStatus() != QProcess::NormalExit ||
            query.exitCode() != 0)
            return false;
        for (const auto &line : QString::fromUtf8(query.readAllStandardOutput()).split('\n'))
            registered |= line.section('\t', -1).trimmed() == extension;
    }
    if (!writeMacFinderConfiguration(application, path, enabled, language, revision))
        return false;
    // Re-adding an unchanged active appex tears down Finder's connection. Normal
    // settings saves update only our configuration; changed or missing bundles
    // still take the explicit registration path.
    if (registered)
        return true;
    QProcess registry;
    registry.start("/usr/bin/pluginkit", {enabled ? "-a" : "-r", extension});
    if (registry.waitForFinished(10000) && registry.exitStatus() == QProcess::NormalExit &&
        registry.exitCode() == 0)
        return true;
    // A failed registration leaves the user's previous menu policy in force.
    if (existed) {
        QSaveFile restore(path);
        restore.setDirectWriteFallback(false);
        if (restore.open(QIODevice::WriteOnly)) {
            restore.setPermissions(QFileDevice::ReadOwner | QFileDevice::WriteOwner);
            restore.write(original);
            restore.commit();
        }
    } else {
        QFile::remove(path);
    }
    return false;
}
