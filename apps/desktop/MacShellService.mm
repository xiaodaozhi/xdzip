#include "MacShellService.h"
#include "UiLanguage.h"
#import <AppKit/AppKit.h>
#include <QCoreApplication>
#include <QCryptographicHash>
#include <QDir>
#include <QDirIterator>
#include <QFile>
#include <QFileInfo>
#include <QJsonDocument>
#include <QJsonObject>
#include <QSaveFile>
#include <QUuid>
#include <algorithm>
#include <optional>
#include <unistd.h>

namespace {
const QString ledgerName = "Contents/Resources/install.json";
const QStringList requiredFiles{"Contents/Info.plist", "Contents/MacOS/xdzip-shell-service",
                                "Contents/Resources/en.lproj/ServicesMenu.strings",
                                "Contents/Resources/zh-Hans.lproj/ServicesMenu.strings",
                                "Contents/Resources/application.txt"};
QStringList allowedFiles() {
    QStringList files = requiredFiles;
    for (const auto &language : xdzip_i18n::languages) {
        const QString file = "Contents/Resources/" + uiLanguageString(language.macos) +
                             ".lproj/ServicesMenu.strings";
        if (!files.contains(file))
            files.append(file);
    }
    return files;
}
QByteArray read(const QString &file) {
    QFile input(file);
    return input.open(QIODevice::ReadOnly) && input.size() <= 8 * 1024 * 1024 ? input.readAll()
                                                                              : QByteArray{};
}
std::optional<QStringList> ownedFiles(const QString &path) {
    if (QFileInfo(path).isSymLink())
        return std::nullopt;
    const auto record = QJsonDocument::fromJson(read(path + '/' + ledgerName)).object();
    if (record.value("product") != "org.xdzip.shell-service" ||
        record.value("schema_version").toInt() != 1)
        return std::nullopt;
    const auto hashes = record.value("files").toObject();
    const auto files = hashes.keys();
    const auto allowed = allowedFiles();
    for (const auto &required : requiredFiles)
        if (!files.contains(required))
            return std::nullopt;
    for (const auto &file : files) {
        if (!allowed.contains(file))
            return std::nullopt;
        const auto info = QFileInfo(path + '/' + file);
        if (!info.isFile() || info.isSymLink() || info.ownerId() != getuid() ||
            QCryptographicHash::hash(read(info.filePath()), QCryptographicHash::Sha256).toHex() !=
                hashes.value(file).toString().toLatin1())
            return std::nullopt;
    }
    QDirIterator entries(path,
                         QDir::AllEntries | QDir::NoDotAndDotDot | QDir::Hidden | QDir::System,
                         QDirIterator::Subdirectories);
    while (entries.hasNext()) {
        const auto entry = entries.next();
        const auto info = entries.fileInfo();
        const auto relative = QDir(path).relativeFilePath(entry);
        if (info.isSymLink() || info.ownerId() != getuid())
            return std::nullopt;
        if (info.isDir()) {
            if (std::none_of(files.begin(), files.end(),
                             [&](const QString &f) { return f.startsWith(relative + '/'); }))
                return std::nullopt;
        } else if (relative != ledgerName && !files.contains(relative))
            return std::nullopt;
    }
    return files;
}
bool removeOwned(const QString &path) {
    const auto files = ownedFiles(path);
    if (!files)
        return false;
    for (const auto &file : *files)
        if (!QFile::remove(path + '/' + file))
            return false;
    if (!QFile::remove(path + '/' + ledgerName))
        return false;
    for (const auto &file : *files)
        if (file.endsWith("/ServicesMenu.strings") &&
            !QDir().rmdir(path + '/' + QFileInfo(file).path()))
            return false;
    for (const auto &dir : {"Contents/Resources", "Contents/MacOS", "Contents", ""})
        if (!QDir().rmdir(path + '/' + dir))
            return false;
    return true;
}
} // namespace
bool updateMacShellService(bool enabled) {
    const auto app = QDir::cleanPath(QCoreApplication::applicationDirPath() + "/../..");
    const auto source = app + "/Contents/Helpers/xdzip.service";
    const auto parent = QDir::homePath() + "/Library/Services";
    const auto target = parent + "/xdzip.service";
    const bool targetExists = QFile::exists(target);
    const auto installedFiles = targetExists ? ownedFiles(target) : std::nullopt;
    if (targetExists && !installedFiles)
        return false;
    if (!enabled) {
        const bool result = !QFile::exists(target) || removeOwned(target);
        if (result)
            NSUpdateDynamicServices();
        return result;
    }
    QStringList sourceFiles;
    for (const auto &file : allowedFiles())
        if (requiredFiles.contains(file) || QFileInfo(source + '/' + file).isFile())
            sourceFiles.append(file);
    auto listing = sourceFiles;
    listing.sort();
    bool identical = installedFiles && *installedFiles == listing;
    for (const auto &file : sourceFiles) {
        const auto expected =
            file.endsWith("application.txt") ? app.toUtf8() + '\n' : read(source + '/' + file);
        identical &= !expected.isEmpty() && expected == read(target + '/' + file);
    }
    if (identical)
        return true;
    const auto stage =
        parent + "/.xdzip-service-stage-" + QUuid::createUuid().toString(QUuid::WithoutBraces);
    if (!QDir().mkpath(stage))
        return false;
    QJsonObject hashes;
    for (const auto &file : sourceFiles) {
        const auto bytes =
            file.endsWith("application.txt") ? app.toUtf8() + '\n' : read(source + '/' + file);
        if (bytes.isEmpty() || !QDir().mkpath(QFileInfo(stage + '/' + file).absolutePath()))
            return false;
        QSaveFile output(stage + '/' + file);
        output.setDirectWriteFallback(false);
        if (!output.open(QIODevice::WriteOnly) || output.write(bytes) != bytes.size() ||
            !output.commit())
            return false;
        if (file.endsWith("xdzip-shell-service"))
            QFile::setPermissions(stage + '/' + file,
                                  QFileDevice::ReadOwner | QFileDevice::WriteOwner |
                                      QFileDevice::ExeOwner | QFileDevice::ReadGroup |
                                      QFileDevice::ExeGroup | QFileDevice::ReadOther |
                                      QFileDevice::ExeOther);
        hashes.insert(file,
                      QString::fromLatin1(
                          QCryptographicHash::hash(bytes, QCryptographicHash::Sha256).toHex()));
    }
    QSaveFile ledger(stage + '/' + ledgerName);
    ledger.setDirectWriteFallback(false);
    const auto bytes = QJsonDocument(QJsonObject{{"schema_version", 1},
                                                 {"product", "org.xdzip.shell-service"},
                                                 {"files", hashes}})
                           .toJson();
    if (!ledger.open(QIODevice::WriteOnly) || ledger.write(bytes) != bytes.size() ||
        !ledger.commit() || !ownedFiles(stage))
        return false;
    const auto previous =
        parent + "/.xdzip-service-old-" + QUuid::createUuid().toString(QUuid::WithoutBraces);
    const bool existed = QFile::exists(target);
    if (existed && !QDir().rename(target, previous)) {
        removeOwned(stage);
        return false;
    }
    if (!QDir().rename(stage, target)) {
        if (existed)
            QDir().rename(previous, target);
        removeOwned(stage);
        return false;
    }
    const bool cleaned = !existed || removeOwned(previous);
    NSUpdateDynamicServices();
    return cleaned;
}
