#include "../FocusDismissPolicy.h"
#include "../PasswordAccessibility.h"
#include "../UiFont.h"
#include "../UiText.h"
#include "../WindowHoverPolicy.h"
#include "settings_storage_test.h"
#include "test_environment.h"
#include <QDir>
#include <QQmlContext>
#include <QQmlEngine>
#include <QQuickStyle>
#include <QResource>
#include <QTemporaryDir>
#include <QtQuickTest/quicktest.h>

class Setup : public QObject {
    Q_OBJECT
    std::unique_ptr<QTemporaryDir> m_profile;
    std::unique_ptr<test_environment::ScopedVariable> m_settingsPath;

  public:
    Setup() {
        const QString root = QString::fromUtf8(XDZIP_TEST_TEMP_ROOT);
        if (!QDir().mkpath(root))
            qFatal("Unable to create UI fixture root");
        m_profile =
            std::make_unique<QTemporaryDir>(root + QStringLiteral("/qml-\u914d\u7f6e-XXXXXX"));
        if (!m_profile->isValid())
            qFatal("Unable to create isolated UI test profile");
        const auto profilePath = m_profile->filePath("settings.ini");
        m_settingsPath =
            std::make_unique<test_environment::ScopedVariable>("XDZIP_SETTINGS_PATH", profilePath);
        if (!m_settingsPath->valid())
            qFatal("UI fixture precondition: Unicode environment setup failed");
        QString error;
        if (!test_environment::checkStorage(m_profile->path(), &error))
            qFatal("%s", qPrintable(error));
        QQuickStyle::setStyle(QStringLiteral("Basic"));
        Q_INIT_RESOURCE(ui);
    }
    Q_INVOKABLE bool profileMatchesEnvironment() const {
        const auto path = m_profile->filePath("settings.ini");
        return qEnvironmentVariable("XDZIP_SETTINGS_PATH") == path &&
               test_environment::processValue("XDZIP_SETTINGS_PATH") == path;
    }
    Q_INVOKABLE bool storedBoolean(const QString &control) const {
        QSettings settings(m_profile->filePath("settings.ini"), QSettings::IniFormat);
        settings.sync();
        return settings.value(control == "Tray" ? "desktop/trayEnabled" : "privacy/recentEnabled")
            .toBool();
    }
    Q_INVOKABLE QVariantMap contendSettingsWrite(const QString &target, bool transient) const {
        return settings_storage_test::contend(m_profile->path(), target, transient);
    }
  public slots:
    void cleanupTestCase() {
        m_settingsPath.reset();
        m_profile.reset();
    }
    void qmlEngineAvailable(QQmlEngine *engine) {
        engine->rootContext()->setContextProperty(QStringLiteral("TestFixture"), this);
    }
    void applicationAvailable() {
        installPasswordAccessibility();
        FocusDismissPolicy::install(QCoreApplication::instance());
        WindowHoverPolicy::install(QCoreApplication::instance());
        if (!initializeUiText(QStringLiteral("zh-CN")))
            qFatal("Unable to load UI language");
        if (!installUiFont())
            qFatal("Unable to load UI font");
    }
};
QUICK_TEST_MAIN_WITH_SETUP(xdzip_ui, Setup)
#include "main.moc"
