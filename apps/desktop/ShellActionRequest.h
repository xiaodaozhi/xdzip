#pragma once
#include <QJsonObject>
#include <QStringList>

struct ShellActionRequest {
    QString id, action;
    QStringList paths;
    static ShellActionRequest parse(const QJsonObject &value);
    static QJsonObject readFile(const QString &file);
    bool valid() const {
        return !id.isEmpty();
    }
    QJsonObject json() const;
};
