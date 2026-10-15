#include "UiPreferences.h"
#include "UiText.h"
#include <QDir>
#include <QFileInfo>
#include <QGuiApplication>
#include <QJsonArray>
#include <QJsonDocument>
#include <QSaveFile>
#include <QStyleHints>

namespace {
bool validTheme(const QString &theme) {
    return theme == "system" || theme == "light" || theme == "dark";
}
bool validScale(double scale) {
    return scale == 1 || scale == 1.25 || scale == 1.5 || scale == 2;
}
bool validLanguage(const QString &language) {
    return language == "zh-CN" || language == "en-US";
}
} // namespace
UiPreferences::UiPreferences(QObject *parent) : QObject(parent) {
    QSettings settings(uiSettingsPath(), QSettings::IniFormat);
    const auto schema = settings.value("schemaVersion", 1).toInt();
    const auto theme = settings.value("appearance/theme", "system").toString();
    const auto scale = settings.value("appearance/textScale", 1).toDouble();
    const auto language = settings.value("locale/language", "zh-CN").toString();
    if (settings.status() == QSettings::NoError && schema == 1 && validTheme(theme) &&
        validScale(scale) && validLanguage(language)) {
        m_savedTheme = m_theme = theme;
        m_savedScale = m_scale = scale;
        m_language = language;
        const auto density = settings.value("appearance/density", "standard").toString();
        m_savedDensity = m_density = density == "compact" ? density : QStringLiteral("standard");
        m_recentEnabled = settings.value("privacy/recentEnabled", true).toBool();
    } else {
        m_error = qtTrId("ui.056945d3a4fa");
    }
    applyColorScheme();
    connect(QGuiApplication::styleHints(), &QStyleHints::colorSchemeChanged, this, [this] {
        if (m_theme == "system")
            emit appearanceChanged();
    });
}
void UiPreferences::applyColorScheme() {
    auto *hints = QGuiApplication::styleHints();
    if (m_theme == "system")
        hints->unsetColorScheme();
    else
        hints->setColorScheme(m_theme == "dark" ? Qt::ColorScheme::Dark : Qt::ColorScheme::Light);
}
bool UiPreferences::dark() const {
    return m_theme == "dark" ||
           (m_theme == "system" &&
            QGuiApplication::styleHints()->colorScheme() == Qt::ColorScheme::Dark);
}
QString UiPreferences::activeLanguage() const {
    return QCoreApplication::instance()->property("xdzipUiLanguage").toString();
}
void UiPreferences::preview(const QString &theme, double scale) {
    if (!validTheme(theme) || !validScale(scale))
        return;
    if (m_theme == theme && m_scale == scale)
        return;
    m_theme = theme;
    m_scale = scale;
    applyColorScheme();
    emit appearanceChanged();
}
void UiPreferences::cancelPreview() {
    preview(m_savedTheme, m_savedScale);
    previewDensity(m_savedDensity);
}
bool UiPreferences::save(const QString &theme, double scale, const QString &language) {
    if (!validTheme(theme) || !validScale(scale) || !validLanguage(language))
        return false;
    const auto path = uiSettingsPath();
    QSettings settings(path, QSettings::IniFormat);
    // Keep the previous profile recoverable. Failure preserves the draft and the
    // existing file; no archive or task policy is changed by preferences.
    if (!QDir().mkpath(QFileInfo(path).absolutePath())) {
        m_error = qtTrId("ui.c43b29d7cbdf");
        emit errorChanged();
        return false;
    }
    if (QFile::exists(path)) {
        const auto backup = path + QStringLiteral(".bak");
        if ((QFile::exists(backup) && !QFile::remove(backup)) || !QFile::copy(path, backup)) {
            m_error = qtTrId("ui.f9274c70af84");
            emit errorChanged();
            return false;
        }
    }
    settings.clear();
    settings.setValue("schemaVersion", 1);
    settings.setValue("appearance/theme", theme);
    settings.setValue("appearance/textScale", scale);
    settings.setValue("locale/language", language);
    settings.setValue("appearance/density", m_savedDensity);
    settings.setValue("privacy/recentEnabled", m_recentEnabled);
    settings.sync();
    if (settings.status() != QSettings::NoError) {
        m_error = qtTrId("ui.c43b29d7cbdf");
        emit errorChanged();
        return false;
    }
    m_savedTheme = theme;
    m_savedScale = scale;
    m_language = language;
    m_error.clear();
    preview(theme, scale);
    emit savedChanged();
    emit errorChanged();
    return true;
}

void UiPreferences::previewDensity(const QString &density) {
    if ((density != "standard" && density != "compact") || m_density == density)
        return;
    m_density = density;
    emit appearanceChanged();
}
void UiPreferences::rememberArchive(const QString &path) {
    if (!m_recentEnabled || path.isEmpty())
        return;
    const auto filePath = QFileInfo(uiSettingsPath()).dir().filePath("recent.json");
    QFile input(filePath);
    QJsonArray entries;
    if (input.open(QIODevice::ReadOnly) && input.size() <= 65536)
        entries = QJsonDocument::fromJson(input.readAll()).array();
    QJsonArray updated{path};
    for (const auto &entry : entries) {
        if (entry.isString() && entry.toString() != path && updated.size() < 20)
            updated.append(entry);
    }
    QDir().mkpath(QFileInfo(filePath).absolutePath());
    QSaveFile output(filePath);
    if (!output.open(QIODevice::WriteOnly))
        return;
    output.setPermissions(QFileDevice::ReadOwner | QFileDevice::WriteOwner);
    const auto bytes = QJsonDocument(updated).toJson(QJsonDocument::Compact);
    if (output.write(bytes) == bytes.size())
        output.commit();
}
bool UiPreferences::saveDetails(const QString &theme, double scale, const QString &language,
                                const QString &density, bool recentEnabled, bool clearRecent) {
    if (density != "standard" && density != "compact")
        return false;
    const auto oldDensity = m_savedDensity;
    const auto oldRecent = m_recentEnabled;
    m_savedDensity = density;
    m_recentEnabled = recentEnabled;
    if (!save(theme, scale, language)) {
        m_savedDensity = oldDensity;
        m_recentEnabled = oldRecent;
        return false;
    }
    previewDensity(density);
    if (clearRecent) {
        const auto path = QFileInfo(uiSettingsPath()).dir().filePath("recent.json");
        if (QFile::exists(path) && !QFile::remove(path)) {
            m_error = qtTrId("ui.c43b29d7cbdf");
            emit errorChanged();
            return false;
        }
    }
    return true;
}
