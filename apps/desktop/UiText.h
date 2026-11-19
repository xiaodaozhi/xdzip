#pragma once
#include "UiFont.h"
#include "UiLanguage.h"
#include <QCoreApplication>
#include <QFile>
#include <QJsonDocument>
#include <QJsonObject>
#include <QLibraryInfo>
#include <QLocale>
#include <QPointer>
#include <QSettings>
#include <QStandardPaths>
#include <QTranslator>

inline QString uiSettingsPath() {
    const auto overridePath = qEnvironmentVariable("XDZIP_SETTINGS_PATH");
    return overridePath.isEmpty()
               ? QStandardPaths::writableLocation(QStandardPaths::AppConfigLocation) +
                     QStringLiteral("/settings.ini")
               : overridePath;
}

class UiTranslator final : public QTranslator {
    QJsonObject messages;
    xdzip_i18n::PluralRule pluralRule = xdzip_i18n::PluralRule::one;

  public:
    explicit UiTranslator(QObject *parent) : QTranslator(parent) {}
    bool read(const QString &language) {
        const auto *entry = xdzip_i18n::findLanguage(language.toStdString());
        if (!entry)
            return false;
        QFile file(uiCatalogPath(language));
        if (!file.open(QIODevice::ReadOnly))
            return false;
        QJsonParseError error;
        const auto document = QJsonDocument::fromJson(file.readAll(), &error);
        if (error.error != QJsonParseError::NoError || !document.isObject())
            return false;
        pluralRule = entry->plural;
        messages = document.object().value(QStringLiteral("messages")).toObject();
        return !messages.isEmpty();
    }
    bool isEmpty() const override {
        return messages.isEmpty();
    }
    QString translate(const char *context, const char *id, const char *, int n) const override {
        // Native edit-role detection compares labels in QCocoaMenuItem's translation
        // context. Match the app menu's language even when the OS uses another locale;
        // all other Qt contexts continue using the system-locale qtbase catalog.
        if (qstrcmp(context, "QCocoaMenuBar") == 0 && qstrcmp(id, "Edit") == 0)
            return messages.value(QStringLiteral("desktop.edit")).toString();
        if (qstrcmp(context, "QCocoaMenuItem") == 0) {
            const auto command = QString::fromLatin1(id);
            const auto key = command == "Cut"          ? QStringLiteral("desktop.cut")
                             : command == "Copy"       ? QStringLiteral("desktop.copy")
                             : command == "Paste"      ? QStringLiteral("desktop.paste")
                             : command == "Select All" ? QStringLiteral("desktop.selectAll")
                                                       : QString{};
            if (!key.isEmpty())
                return messages.value(key).toString();
        }
        const auto value = messages.value(QString::fromUtf8(id));
        if (value.isString())
            return value.toString();
        if (value.isObject())
            return value.toObject()
                .value(QString::fromLatin1(xdzip_i18n::pluralCategory(pluralRule, n)))
                .toString();
        return {};
    }
};

inline bool loadQtSystemTranslation(QTranslator &translator, const QLocale &locale,
                                    const QString &directory) {
    // SDKs provide qtbase catalogs; windeployqt merges runtime catalogs as qt_*.qm.
    const auto load = [&](const QLocale &candidate) {
        return translator.load(candidate, QStringLiteral("qtbase"), QStringLiteral("_"),
                               directory) ||
               translator.load(candidate, QStringLiteral("qt"), QStringLiteral("_"), directory);
    };
    if (load(locale))
        return true;
    const auto preferred = locale.uiLanguages().value(0);
    const auto fallback = uiLanguageString(xdzip_i18n::systemLanguage(preferred.toStdString()));
    if (fallback == uiLanguageString(xdzip_i18n::defaultLanguage))
        return false;
    // Qt ships a Taiwan catalog but no Hong Kong or Macao catalog.
    const QLocale canonical(fallback);
    return canonical != locale && load(canonical);
}

inline bool initializeUiText(const QString &forcedLanguage = {}) {
    QCoreApplication::setOrganizationName(QStringLiteral("xdzip"));
    QSettings settings(uiSettingsPath(), QSettings::IniFormat);
    auto language =
        forcedLanguage.isEmpty() ? qEnvironmentVariable("XDZIP_LANGUAGE") : forcedLanguage;
    if (language.isEmpty() && settings.value(QStringLiteral("schemaVersion"), 1).toInt() == 1 &&
        settings.status() == QSettings::NoError)
        language = readUiLanguageSelection(settings);
    language = resolveUiLanguage(language.isEmpty() ? QStringLiteral("auto") : language);
    // Qt-owned controls use the OS language, independently of the product locale.
    static QPointer<QTranslator> systemTranslator;
    static QPointer<UiTranslator> activeTranslator;
    if (!systemTranslator) {
        auto *candidate = new QTranslator(QCoreApplication::instance());
        if (loadQtSystemTranslation(*candidate, QLocale::system(),
                                    QLibraryInfo::path(QLibraryInfo::TranslationsPath)) &&
            QCoreApplication::installTranslator(candidate))
            systemTranslator = candidate;
        else
            delete candidate;
    }
    auto *translator = new UiTranslator(QCoreApplication::instance());
    if (!translator->read(language)) {
        delete translator;
        return false;
    }
    if (!QCoreApplication::installTranslator(translator)) {
        delete translator;
        return false;
    }
    if (activeTranslator) {
        QCoreApplication::removeTranslator(activeTranslator);
        delete activeTranslator;
    }
    activeTranslator = translator;
    QLocale::setDefault(QLocale(language));
    QCoreApplication::instance()->setProperty("xdzipUiLanguage", language);
    activateUiFont(language);
    return true;
}

inline QString systemDialogText(const char *id) {
    UiTranslator text(nullptr);
    if (text.read(resolveUiLanguage(QStringLiteral("auto")))) {
        const auto result = text.translate(nullptr, id, nullptr, -1);
        if (!result.isEmpty())
            return result;
    }
    return qtTrId(id);
}
