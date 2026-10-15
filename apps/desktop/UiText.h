#pragma once
#include <QCoreApplication>
#include <QFile>
#include <QJsonDocument>
#include <QJsonObject>
#include <QLocale>
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

  public:
    explicit UiTranslator(QObject *parent) : QTranslator(parent) {}
    bool read(const QString &language) {
        QFile file(QStringLiteral(":/xdzip/i18n/") + language + QStringLiteral(".json"));
        if (!file.open(QIODevice::ReadOnly))
            return false;
        QJsonParseError error;
        const auto document = QJsonDocument::fromJson(file.readAll(), &error);
        if (error.error != QJsonParseError::NoError || !document.isObject())
            return false;
        messages = document.object().value(QStringLiteral("messages")).toObject();
        return !messages.isEmpty();
    }
    bool isEmpty() const override {
        return messages.isEmpty();
    }
    QString translate(const char *, const char *id, const char *, int n) const override {
        const auto value = messages.value(QString::fromUtf8(id));
        if (value.isString())
            return value.toString();
        if (value.isObject())
            return value.toObject()
                .value(n == 1 ? QStringLiteral("one") : QStringLiteral("other"))
                .toString();
        return {};
    }
};

inline bool initializeUiText(const QString &forcedLanguage = {}) {
    QCoreApplication::setOrganizationName(QStringLiteral("xdzip"));
    QSettings settings(uiSettingsPath(), QSettings::IniFormat);
    auto language =
        forcedLanguage.isEmpty() ? qEnvironmentVariable("XDZIP_LANGUAGE") : forcedLanguage;
    if (language.isEmpty() && settings.value(QStringLiteral("schemaVersion"), 1).toInt() == 1 &&
        settings.status() == QSettings::NoError)
        language =
            settings.value(QStringLiteral("locale/language"), QStringLiteral("zh-CN")).toString();
    if (language != QStringLiteral("en-US"))
        language = QStringLiteral("zh-CN");
    auto *translator = new UiTranslator(QCoreApplication::instance());
    if (!translator->read(language)) {
        delete translator;
        return false;
    }
    QLocale::setDefault(QLocale(language));
    QCoreApplication::instance()->setProperty("xdzipUiLanguage", language);
    return QCoreApplication::installTranslator(translator);
}
