#pragma once
#include "UiLanguage.h"
#include <QFont>
#include <QFontDatabase>
#include <QGuiApplication>
#include <QVariantMap>

inline QString uiFontFamilyForLanguage(const QString &locale) {
    const auto *language = xdzip_i18n::findLanguage(locale.toStdString());
    if (!language || !QCoreApplication::instance())
        return {};
    const auto families = QCoreApplication::instance()->property("xdzipUiFontFamilies").toMap();
    return families.value(uiLanguageString(language->font)).toString();
}

inline void activateUiFont(const QString &locale) {
    const auto family = uiFontFamilyForLanguage(locale);
    if (family.isEmpty() || QGuiApplication::font().family() == family)
        return;
    auto font = QGuiApplication::font();
    font.setFamily(family);
    QGuiApplication::setFont(font);
}

// Preserve the platform default before installing the shared interface font.
inline bool installUiFont() {
    auto *application = QGuiApplication::instance();
    if (!application->property("xdzipSystemFontFamily").isValid()) {
        auto family = QGuiApplication::font().family();
        if (family.isEmpty())
            family = QFontDatabase::systemFont(QFontDatabase::GeneralFont).family();
        application->setProperty("xdzipSystemFontFamily", family);
    }
    auto registered = application->property("xdzipUiFontFamilies").toMap();
    for (const auto &locale : supportedUiLanguages()) {
        const auto *language = xdzip_i18n::findLanguage(locale.toStdString());
        const auto group = uiLanguageString(language->font);
        if (registered.contains(group))
            continue;
        const auto file =
            QStringLiteral(":/xdzip/fonts/xdzip-design-sans-") + group + QStringLiteral(".ttf");
        const auto families =
            QFontDatabase::applicationFontFamilies(QFontDatabase::addApplicationFont(file));
        if (families.isEmpty())
            return false;
        registered.insert(group, families.first());
    }
    application->setProperty("xdzipUiFontFamilies", registered);
    auto locale = application->property("xdzipUiLanguage").toString();
    if (locale.isEmpty())
        locale = uiLanguageString(xdzip_i18n::defaultLanguage);
    if (uiFontFamilyForLanguage(locale).isEmpty())
        return false;
    activateUiFont(locale);
    return true;
}
