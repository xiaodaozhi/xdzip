#include "../FocusDismissPolicy.h"
#include "../UiFont.h"
#include "../UiText.h"
#include "../WindowHoverPolicy.h"
#include <QQmlEngine>
#include <QQuickStyle>
#include <QResource>
#include <QtQuickTest/quicktest.h>

class Setup : public QObject {
    Q_OBJECT
  public:
    Setup() {
        QQuickStyle::setStyle(QStringLiteral("Basic"));
        Q_INIT_RESOURCE(ui);
    }
  public slots:
    void applicationAvailable() {
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
