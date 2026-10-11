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
};
QUICK_TEST_MAIN_WITH_SETUP(xdzip_ui, Setup)
#include "main.moc"
