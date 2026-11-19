#pragma once
#include "LanguageSelection.h"
#include <QFile>
#include <QLocale>
#include <QSettings>
#include <QStringList>

inline QString uiLanguageString(std::string_view value) {
    return QString::fromUtf8(value.data(), qsizetype(value.size()));
}

inline QString uiCatalogPath(const QString &language) {
    return QStringLiteral(":/xdzip/i18n/") + language + QStringLiteral(".json");
}

inline QStringList supportedUiLanguages() {
    QStringList available;
    for (const auto &entry : xdzip_i18n::languages) {
        const auto language = uiLanguageString(entry.id);
        if (QFile::exists(uiCatalogPath(language)))
            available.append(language);
    }
    return available;
}

inline bool validUiLanguageSelection(const QString &selection) {
    return selection == QStringLiteral("auto") || supportedUiLanguages().contains(selection);
}

inline QString
resolveUiLanguage(const QString &selection,
                  const QStringList &systemUiLanguages = QLocale::system().uiLanguages()) {
    const auto supported = supportedUiLanguages();
    if (supported.contains(selection))
        return selection;
    if (selection == QStringLiteral("auto") && !systemUiLanguages.isEmpty()) {
        // Use the OS's UI preference independently of the current numeric locale.
        const auto id = xdzip_i18n::systemLanguage(systemUiLanguages.first().toStdString());
        const auto candidate = uiLanguageString(id);
        if (supported.contains(candidate))
            return candidate;
        if (const auto *language = xdzip_i18n::findLanguage(id))
            return uiLanguageString(language->legacy);
    }
    return uiLanguageString(xdzip_i18n::defaultLanguage);
}

inline QString readUiLanguageSelection(const QSettings &settings) {
    const auto legacy =
        settings.value(QStringLiteral("locale/language"), QStringLiteral("auto")).toString();
    const auto extended = settings.value(QStringLiteral("locale/extendedLanguage")).toString();
    if (!extended.isEmpty() && supportedUiLanguages().contains(extended))
        return uiLanguageString(
            xdzip_i18n::storedLanguage(legacy.toStdString(), extended.toStdString()));
    return legacy;
}

inline QString legacyUiLanguage(const QString &selection) {
    const auto *language = xdzip_i18n::findLanguage(selection.toStdString());
    return language ? uiLanguageString(language->legacy) : selection;
}

inline void writeUiLanguageSelection(QSettings &settings, const QString &selection) {
    const auto legacy = legacyUiLanguage(selection);
    settings.setValue(QStringLiteral("locale/language"), legacy);
    if (legacy != selection)
        settings.setValue(QStringLiteral("locale/extendedLanguage"), selection);
    else
        settings.remove(QStringLiteral("locale/extendedLanguage"));
}
