#pragma once
#include <QEvent>
#include <QFontDatabase>
#include <QFontMetricsF>
#include <QGuiApplication>
#include <QObject>
#include <QtQml/qqmlregistration.h>

class UiFonts : public QObject {
    Q_OBJECT
    QML_NAMED_ELEMENT(UiFonts)
    QML_SINGLETON
    Q_PROPERTY(QString systemFamily READ systemFamily CONSTANT)
    Q_PROPERTY(QString uiFamily READ uiFamily NOTIFY uiFamilyChanged)

  public:
    explicit UiFonts(QObject *parent = nullptr) : QObject(parent) {
        QGuiApplication::instance()->installEventFilter(this);
    }
    QString systemFamily() const {
        const auto family =
            QGuiApplication::instance()->property("xdzipSystemFontFamily").toString();
        return family.isEmpty() ? QFontDatabase::systemFont(QFontDatabase::GeneralFont).family()
                                : family;
    }
    QString uiFamily() const {
        return QGuiApplication::font().family();
    }
    Q_INVOKABLE QFont useFamily(QFont font, const QString &family) const {
        if (!family.isEmpty())
            font.setFamily(family);
        return font;
    }
    Q_INVOKABLE qreal measure(const QString &text, const QFont &font, const QString &family) const {
        return QFontMetricsF(useFamily(font, family)).horizontalAdvance(text);
    }
  signals:
    void uiFamilyChanged();

  protected:
    bool eventFilter(QObject *receiver, QEvent *event) override {
        if (receiver == QGuiApplication::instance() &&
            event->type() == QEvent::ApplicationFontChange)
            emit uiFamilyChanged();
        return QObject::eventFilter(receiver, event);
    }
};
