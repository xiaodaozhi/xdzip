#include "MacShellService.h"
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
#include <unistd.h>

namespace {
const QString ledgerName = "Contents/Resources/install.json";
const QStringList ownedFiles{"Contents/Info.plist", "Contents/MacOS/xdzip-shell-service",
                             "Contents/Resources/en.lproj/ServicesMenu.strings",
                             "Contents/Resources/zh-Hans.lproj/ServicesMenu.strings",
                             "Contents/Resources/application.txt"};
QByteArray read(const QString &file) {
    QFile input(file);
    return input.open(QIODevice::ReadOnly) && input.size() <= 8 * 1024 * 1024 ? input.readAll()
                                                                              : QByteArray{};
}
bool owned(const QString &path) {
    if (QFileInfo(path).isSymLink())
        return false;
    const auto record = QJsonDocument::fromJson(read(path + '/' + ledgerName)).object();
    if (record.value("product") != "org.xdzip.shell-service" ||
        record.value("schema_version").toInt() != 1)
        return false;
    const auto hashes = record.value("files").toObject();
    for (const auto &file : ownedFiles) {
        const auto info = QFileInfo(path + '/' + file);
        if (!info.isFile() || info.isSymLink() || info.ownerId() != getuid() ||
            QCryptographicHash::hash(read(info.filePath()), QCryptographicHash::Sha256).toHex() !=
                hashes.value(file).toString().toLatin1())
            return false;
    }
    QDirIterator entries(path,
                         QDir::AllEntries | QDir::NoDotAndDotDot | QDir::Hidden | QDir::System,
                         QDirIterator::Subdirectories);
    while (entries.hasNext()) {
        const auto entry = entries.next();
        const auto info = entries.fileInfo();
        const auto relative = QDir(path).relativeFilePath(entry);
        if (info.isSymLink() || info.ownerId() != getuid())
            return false;
        if (info.isDir()) {
            if (std::none_of(ownedFiles.begin(), ownedFiles.end(),
                             [&](const QString &f) { return f.startsWith(relative + '/'); }))
                return false;
        } else if (relative != ledgerName && !ownedFiles.contains(relative))
            return false;
    }
    return true;
}
bool removeOwned(const QString &path) {
    if (!owned(path))
        return false;
    for (const auto &file : ownedFiles)
        if (!QFile::remove(path + '/' + file))
            return false;
    if (!QFile::remove(path + '/' + ledgerName))
        return false;
    for (const auto &dir : {"Contents/Resources/en.lproj", "Contents/Resources/zh-Hans.lproj",
                            "Contents/Resources", "Contents/MacOS", "Contents", ""})
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
    if (QFile::exists(target) && !owned(target))
        return false;
    if (!enabled) {
        const bool result = !QFile::exists(target) || removeOwned(target);
        if (result)
            NSUpdateDynamicServices();
        return result;
    }
    bool identical = QFile::exists(target);
    for (const auto &file : ownedFiles) {
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
    for (const auto &file : ownedFiles) {
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
        !ledger.commit() || !owned(stage))
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
