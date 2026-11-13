#pragma once
#include <QObject>
#include <QWindow>
#include <QtQml/qqmlregistration.h>

class BuildInfo : public QObject {
    Q_OBJECT
    QML_ELEMENT
    QML_SINGLETON
    Q_PROPERTY(QString version READ version CONSTANT)
    Q_PROPERTY(int buildNumber READ buildNumber CONSTANT)
    Q_PROPERTY(QString revision READ revision CONSTANT)
    Q_PROPERTY(QString channel READ channel CONSTANT)
    Q_PROPERTY(bool dirty READ dirty CONSTANT)
    Q_PROPERTY(QString qtVersion READ qtVersion CONSTANT)
    Q_PROPERTY(QString platform READ platform CONSTANT)
    Q_PROPERTY(QString configuration READ configuration CONSTANT)
    Q_PROPERTY(QString architecture READ architecture CONSTANT)
    Q_PROPERTY(bool packaged READ packaged CONSTANT)
    Q_PROPERTY(bool linuxX11 READ linuxX11 CONSTANT)
  public:
    explicit BuildInfo(QObject *parent = nullptr) : QObject(parent) {}
    static QString version();
    static int buildNumber();
    static QString revision();
    static QString channel();
    static bool dirty();
    static QString qtVersion();
    static QString platform();
    static QString architecture();
    static QString configuration();
    static bool packaged();
    static bool linuxX11();
    static bool printVersionIfRequested(int argc, char **argv);
    Q_INVOKABLE void copyVersionInfo() const;
    Q_INVOKABLE void centerOnScreen(QWindow *window, QWindow *owner = nullptr) const;
};
