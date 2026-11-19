#include "UiPreferences.h"
#ifdef XDZIP_CURRENT_PREFS
#include "UiText.h"
#endif
#include <QGuiApplication>
#include <QJsonDocument>
#include <QJsonObject>
#include <cstdio>

int main(int argc, char **argv) {
    QGuiApplication application(argc, argv);
    if (qEnvironmentVariableIsEmpty("XDZIP_SETTINGS_PATH"))
        return 2; // This fixture must never touch a real user profile.
    UiPreferences preferences;
    bool saved = false;
    const auto arguments = application.arguments();
    if (arguments.contains("--write-dark"))
        saved = preferences.saveDetails("dark", 1.25, "en-US", "compact", false, false);
    if (arguments.contains("--write-light"))
        saved = preferences.saveDetails("light", 1.5, "zh-CN", "standard", true, false);
#ifdef XDZIP_CURRENT_PREFS
    if (arguments.size() == 3 && arguments[1] == "--write-language") {
        if (!initializeUiText())
            return 3;
        saved = preferences.applyDetails(preferences.savedTheme(), preferences.savedTextScale(),
                                         arguments[2], preferences.savedDensity(),
                                         preferences.recentEnabled(), preferences.trayEnabled(),
                                         preferences.openMode());
    }
#endif
    const QJsonObject result{
        {"theme", preferences.savedTheme()},     {"scale", preferences.savedTextScale()},
        {"language", preferences.language()},    {"density", preferences.savedDensity()},
        {"recent", preferences.recentEnabled()}, {"saved", saved}};
    const auto bytes = QJsonDocument(result).toJson(QJsonDocument::Compact);
    std::puts(bytes.constData());
    return 0;
}
