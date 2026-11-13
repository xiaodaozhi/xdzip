#pragma once
#include <QLocale>
#include <QStringList>

inline QStringList supportedUiLanguages() {
    return {QStringLiteral("zh-CN"), QStringLiteral("en-US")};
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
        // Use the OS's preferred UI language, independently of our numeric locale
        // and any QLocale::setDefault() performed by a previous manual selection.
        const auto language = QLocale(systemUiLanguages.first()).language();
        for (const auto &candidate : supported)
            if (QLocale(candidate).language() == language)
                return candidate;
    }
    return QStringLiteral("en-US");
}
