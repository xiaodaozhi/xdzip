#pragma once
#include <QFontDatabase>
#include <QGuiApplication>
#include <QObject>
#include <QtQml/qqmlregistration.h>

class UiFonts : public QObject {
    Q_OBJECT
    QML_NAMED_ELEMENT(UiFonts)
    QML_SINGLETON
    Q_PROPERTY(QString systemFamily READ systemFamily CONSTANT)
    Q_PROPERTY(QString uiFamily READ uiFamily CONSTANT)

  public:
    explicit UiFonts(QObject *parent = nullptr) : QObject(parent) {}
    QString systemFamily() const {
        const auto family =
            QGuiApplication::instance()->property("xdzipSystemFontFamily").toString();
        return family.isEmpty() ? QFontDatabase::systemFont(QFontDatabase::GeneralFont).family()
                                : family;
    }
    QString uiFamily() const {
        return QGuiApplication::font().family();
    }
};
