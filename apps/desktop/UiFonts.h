#pragma once
#include <QEvent>
#include <QFontDatabase>
#include <QFontMetricsF>
#include <QGlyphRun>
#include <QGuiApplication>
#include <QObject>
#include <QRawFont>
#include <QTextLayout>
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
    Q_INVOKABLE QRectF visualTextBounds(const QString &text, const QFont &font, qreal width) const {
        if (text.trimmed().isEmpty() || width <= 0)
            return {};

        QTextLayout layout(text, font);
        QTextOption option;
        option.setWrapMode(QTextOption::WrapAtWordBoundaryOrAnywhere);
        layout.setTextOption(option);
        layout.beginLayout();
        qreal y = 0;
        while (true) {
            auto line = layout.createLine();
            if (!line.isValid())
                break;
            line.setLineWidth(width);
            line.setPosition(QPointF(0, y));
            y += line.height();
        }
        layout.endLayout();

        QRectF bounds;
        for (const auto &run : layout.glyphRuns()) {
            const auto indexes = run.glyphIndexes();
            const auto positions = run.positions();
            const auto rawFont = run.rawFont();
            for (qsizetype i = 0; i < indexes.size() && i < positions.size(); ++i) {
                const auto glyph = rawFont.boundingRect(indexes[i]).translated(positions[i]);
                if (!glyph.isEmpty())
                    bounds = bounds.isEmpty() ? glyph : bounds.united(glyph);
            }
        }
        return bounds;
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
