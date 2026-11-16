#include "UiFont.h"
#include "UiPreferences.h"
#include "UiText.h"
#include "test_environment.h"
#include <QApplication>
#include <QCursor>
#include <QDir>
#include <QQmlApplicationEngine>
#include <QQmlComponent>
#include <QQuickStyle>
#include <QQuickWindow>
#include <QScopeGuard>
#include <QScreen>
#include <QtTest>
#include <memory>

class SettingsPlacementTest : public QObject {
    Q_OBJECT
  private slots:
    void trayWindowStaysInWorkArea_data() {
        QTest::addColumn<int>("screenIndex");
        QTest::addColumn<QString>("language");
        QTest::addColumn<bool>("dark");
        QTest::addColumn<double>("scale");
        QTest::addColumn<int>("tab");
        for (int screen = 0; screen < QGuiApplication::screens().size(); ++screen)
            for (const auto &language : {QString("zh-CN"), QString("en-US")})
                for (const bool dark : {false, true})
                    for (const double scale : {1., 1.25, 1.5})
                        for (const int tab : {0, 5}) {
                            const auto name = QString("screen%1-%2-%3-%4-tab%5")
                                                  .arg(screen)
                                                  .arg(language)
                                                  .arg(dark)
                                                  .arg(scale)
                                                  .arg(tab);
                            QTest::newRow(qPrintable(name))
                                << screen << language << dark << scale << tab;
                        }
    }
    void trayWindowStaysInWorkArea() {
        QFETCH(int, screenIndex);
        QFETCH(QString, language);
        QFETCH(bool, dark);
        QFETCH(double, scale);
        QFETCH(int, tab);
        auto *screen = QGuiApplication::screens().at(screenIndex);
        const auto area = screen->availableGeometry();
        const auto previousCursor = QCursor::pos();
        const auto restoreCursor = qScopeGuard([&] { QCursor::setPos(previousCursor); });
        QQmlApplicationEngine engine;
        auto *settings = engine.singletonInstance<UiPreferences *>("Xdzip.Settings", "AppSettings");
        QVERIFY(settings);
        QVERIFY2(settings->applyDetails(dark ? "dark" : "light", scale, language, "standard", false,
                                        true, "tabs"),
                 qPrintable(settings->error()));
        QQmlComponent themeComponent(&engine);
        themeComponent.setData("import QtQml\nimport \"qrc:/xdzip/ui\" as Ui\n"
                               "QtObject { property QtObject theme: Ui.Theme }",
                               QUrl());
        std::unique_ptr<QObject> themeHolder(themeComponent.create());
        QVERIFY2(themeHolder, qPrintable(themeComponent.errorString()));
        auto *theme = themeHolder->property("theme").value<QObject *>();
        QVERIFY(theme);
        theme->setProperty("dark", dark);
        theme->setProperty("fontScale", scale);
        QQmlComponent component(&engine, QUrl("qrc:/xdzip/ui/SettingsWindow.qml"));
        std::unique_ptr<QObject> object(component.create());
        QVERIFY2(object, qPrintable(component.errorString()));
        auto *window = qobject_cast<QQuickWindow *>(object.get());
        QVERIFY(window);
        QCOMPARE(window->transientParent(), nullptr);
        // Tray activation can leave the previous native position beside the menu.
        // Check the actual frame and QScreen work area after creation and reopening.
        for (const int page : {tab, 5 - tab}) {
            window->setPosition(area.bottomRight() - QPoint(10, 10));
            if (QGuiApplication::platformName() != "offscreen") {
                const auto cursorPosition = area.bottomRight() - QPoint(2, 2);
                QCursor::setPos(cursorPosition);
                QTRY_COMPARE(QCursor::pos(), cursorPosition);
                QCOMPARE(QGuiApplication::screenAt(QCursor::pos()), screen);
            }
            QVERIFY(QMetaObject::invokeMethod(window, "configureTab", Q_ARG(QVariant, page)));
            QTRY_VERIFY(window->isVisible());
            QTRY_COMPARE(window->screen(), screen);
            QTRY_VERIFY2(area.contains(window->frameGeometry()),
                         qPrintable(QString("frame %1,%2 %3x%4 outside %5,%6 %7x%8")
                                        .arg(window->frameGeometry().x())
                                        .arg(window->frameGeometry().y())
                                        .arg(window->frameGeometry().width())
                                        .arg(window->frameGeometry().height())
                                        .arg(area.x())
                                        .arg(area.y())
                                        .arg(area.width())
                                        .arg(area.height())));
            QTRY_VERIFY(qAbs(window->frameGeometry().center().x() - area.center().x()) <= 2);
            QTRY_VERIFY(qAbs(window->frameGeometry().center().y() - area.center().y()) <= 2);
            const auto audit = qEnvironmentVariable("XDZIP_SETTINGS_PLACEMENT_AUDIT");
            if (!audit.isEmpty()) {
                QVERIFY(QDir().mkpath(audit));
                QImage image;
                QTRY_VERIFY(!(image = window->grabWindow()).isNull());
                QVERIFY(image.save(QDir(audit).filePath(
                    QString("%1-page%2.png").arg(QTest::currentDataTag()).arg(page))));
            }
            window->close();
        }
    }
};

int main(int argc, char **argv) {
    test_environment::Runtime fixtures(QString::fromUtf8(XDZIP_TEST_TEMP_ROOT));
    if (!fixtures.valid()) {
        qWarning().noquote() << fixtures.error();
        return 2;
    }
    QApplication app(argc, argv);
    app.setQuitOnLastWindowClosed(false);
    QQuickStyle::setStyle(QStringLiteral("Basic"));
    Q_INIT_RESOURCE(ui);
    if (!initializeUiText(QStringLiteral("zh-CN")) || !installUiFont())
        return 2;
    SettingsPlacementTest test;
    const auto result = QTest::qExec(&test, argc, argv);
    fixtures.close();
    return result;
}
#include "settings_placement_test.moc"
