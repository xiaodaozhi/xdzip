#pragma once
#include <QByteArray>
#include <QObject>
#include <QString>
#include <QtQml/qqmlregistration.h>
class QQmlEngine;
class QJSEngine;

// The QML singleton installs a provider in its own engine. URLs contain random
// tokens only; encoded user content and filesystem paths never enter a URL/log.
class PreviewImages final : public QObject {
    Q_OBJECT
    QML_ELEMENT
    QML_SINGLETON
  public:
    static PreviewImages *create(QQmlEngine *engine, QJSEngine *);
    static QString store(QByteArray bytes, int kind);
    static void remove(const QString &url);
    Q_INVOKABLE bool ready() const {
        return true;
    }

  private:
    explicit PreviewImages(QObject *parent = nullptr) : QObject(parent) {}
};
