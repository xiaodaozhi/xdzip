#include "../FocusDismissPolicy.h"
#include "../PasswordAccessibility.h"
#include "../UiFont.h"
#include "../UiText.h"
#include "../WindowHoverPolicy.h"
#include <QQmlEngine>
#include <QQuickStyle>
#include <QResource>
#include <QTemporaryDir>
#include <QtQuickTest/quicktest.h>

class Setup : public QObject {
    Q_OBJECT
    QTemporaryDir m_profile;

  public:
    Setup() {
        if (!m_profile.isValid())
            qFatal("Unable to create isolated UI test profile");
        qputenv("XDZIP_SETTINGS_PATH", m_profile.filePath("settings.ini").toUtf8());
        QQuickStyle::setStyle(QStringLiteral("Basic"));
        Q_INIT_RESOURCE(ui);
    }
  public slots:
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
