#include "ShellActionRequest.h"
#include "xdzip-qt-bridge/src/lib.rs.h"
#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QJsonArray>
#include <QJsonDocument>
#include <QUuid>

ShellActionRequest ShellActionRequest::parse(const QJsonObject &value) {
    ShellActionRequest result;
    const QStringList actions{"choose",
                              "zip",
                              "7z",
                              "create",
                              "separate-zip",
                              "extract-here",
                              "extract-smart",
                              "extract-separate",
                              "extract-to"};
    const auto id = value.value("id").toString();
    const auto action = value.value("action").toString();
    const auto paths = value.value("paths");
    if (value.value("schema_version").toInt() != 1 || QUuid(id).isNull() || id.size() != 36 ||
        !actions.contains(action) || !paths.isArray() || paths.toArray().isEmpty() ||
        paths.toArray().size() > 128 ||
        QJsonDocument(value).toJson(QJsonDocument::Compact).size() > 256 * 1024)
        return {};
    for (const auto &item : paths.toArray()) {
        const auto path = item.toString();
        if (!item.isString() || path.isEmpty() || path.contains(QChar::Null) ||
            !QFileInfo(path).isAbsolute() || path.toUtf8().size() > 4096 ||
            QString::fromUtf8(path.toUtf8()) != path)
            return {};
        const auto clean = QDir::cleanPath(path);
        if (!result.paths.contains(clean))
            result.paths.append(clean);
    }
    result.id = id;
    result.action = action;
    return result;
}
QJsonObject ShellActionRequest::json() const {
    return {{"schema_version", 1},
            {"id", id},
            {"action", action},
            {"paths", QJsonArray::fromStringList(paths)}};
}
QJsonObject ShellActionRequest::readFile(const QString &file) {
    const QFileInfo info(file);
    if (info.fileName() != "request.json")
        return {};
    const auto directory = info.absolutePath().toUtf8();
    const auto root = QDir::tempPath().toUtf8();
    const auto raw = xdzip::reader::shell_request_read(
        {directory.constData(), size_t(directory.size())}, {root.constData(), size_t(root.size())});
    const auto request =
        QJsonDocument::fromJson(QByteArray(raw.data(), qsizetype(raw.size()))).object();
    if (!parse(request).valid())
        return {};
    // Rust has verified an owned private, no-follow request. Consume only these
    // two exact objects; never recursively clean a caller-selected directory.
    QFile::remove(file);
    QDir().rmdir(info.absolutePath());
    return request;
}
