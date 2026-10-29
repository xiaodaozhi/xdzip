#pragma once
#include <QFont>
#include <QFontDatabase>
#include <QGuiApplication>

// Preserve the platform default before installing the shared interface font.
inline bool installUiFont() {
    auto *application = QGuiApplication::instance();
    if (!application->property("xdzipSystemFontFamily").isValid()) {
        auto family = QGuiApplication::font().family();
        if (family.isEmpty())
            family = QFontDatabase::systemFont(QFontDatabase::GeneralFont).family();
        application->setProperty("xdzipSystemFontFamily", family);
    }
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
