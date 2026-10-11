#include "BrowserController.h"
#include <QQmlApplicationEngine>
#include <QQuickStyle>
#include <QQuickWindow>
#include <QSignalSpy>
#include <QtTest>

class BrowserTest : public QObject {
    Q_OBJECT
  private slots:
    void initTestCase() {
        QQuickStyle::setStyle("Basic");
        Q_INIT_RESOURCE(ui);
        qmlRegisterType<BrowserController>("Xdzip.Native", 1, 0, "BrowserController");
    }
    void realPagesAndFailure() {
        BrowserController controller;
        controller.openPath(QStringLiteral(XDZIP_FIXTURES "/pages.zip"));
        QVERIFY(controller.busy());
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        QVERIFY(controller.ready());
        QCOMPARE(controller.total(), 270);
        QCOMPARE(controller.entries()->rowCount(), 128);
        const auto session = controller.sessionId();
        controller.fetchPage(256);
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        QCOMPARE(controller.entries()->rowCount(), 14);
        QCOMPARE(
            controller.entries()->data(controller.entries()->index(0, 0), Qt::UserRole).toString(),
            "257");
        QCOMPARE(controller.sessionId(), session);
        controller.openPath(QStringLiteral(XDZIP_FIXTURES "/truncated.zip"));
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        QVERIFY(!controller.ready());
        QCOMPARE(controller.entries()->rowCount(), 0);
        QVERIFY(!controller.message().isEmpty());
    }
    void unicodeSourcePath() {
        QTemporaryDir directory;
        QVERIFY(directory.isValid());
        const auto path = directory.filePath(QString::fromUtf8("旅行 archive.zip"));
        QVERIFY(QFile::copy(QStringLiteral(XDZIP_FIXTURES "/normal.zip"), path));
        BrowserController controller;
        controller.openPath(path);
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        QVERIFY(controller.ready());
        QCOMPARE(controller.total(), 5);
    }
    void closeDuringReadRemainsAsynchronous() {
        BrowserController controller;
        QSignalSpy closed(&controller, &BrowserController::readyToClose);
        controller.openPath(QStringLiteral(XDZIP_FIXTURES "/pages.zip"));
        QVERIFY(!controller.requestClose());
        QTRY_COMPARE_WITH_TIMEOUT(closed.count(), 1, 5000);
        QVERIFY(!controller.busy());
        QVERIFY(controller.requestClose());
    }
    void renderRealArchive() {
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *controller = window->findChild<BrowserController *>();
        QVERIFY(controller);
        controller->openPath(QStringLiteral(XDZIP_FIXTURES "/normal.zip"));
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy(), 5000);
        QCOMPARE(controller->total(), 5);
        QCOMPARE(controller->entries()
                     ->data(controller->entries()->index(1, 0), Qt::DisplayRole)
                     .toString(),
                 QString::fromUtf8("旅行/说明.txt"));
        QVERIFY(QTest::qWaitForWindowExposed(window));
        QVERIFY(window->grabWindow().save("zip-browser.png"));
    }
};
QTEST_MAIN(BrowserTest)
#include "browser_test.moc"
