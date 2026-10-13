#pragma once
#include <QFont>
#include <QFontDatabase>
#include <QGuiApplication>

// The same licensed subset provides stable UI metrics in HTML and Qt. Archive names
// outside its character coverage still use Qt's platform font fallback.
inline bool installUiFont() {
    const int id =
        QFontDatabase::addApplicationFont(QStringLiteral(":/xdzip/fonts/xdzip-design-sans-sc.ttf"));
    const auto families = QFontDatabase::applicationFontFamilies(id);
    if (families.isEmpty())
        return false;
    auto font = QGuiApplication::font();
    font.setFamily(families.first());
    QGuiApplication::setFont(font);
    return true;
}
