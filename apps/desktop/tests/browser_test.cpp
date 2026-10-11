#include "BrowserController.h"
#include <QQmlApplicationEngine>
#include <QQuickItem>
#include <QQuickStyle>
#include <QQuickWindow>
#include <QSignalSpy>
#include <QtTest>

// TableView delegates belong to its visual tree, not necessarily its QObject tree.
static QQuickItem *findVisual(QQuickItem *item, const QString &name) {
    if (item->objectName() == name)
        return item;
    for (auto *child : item->childItems()) {
        if (auto *match = findVisual(child, name))
            return match;
    }
    return nullptr;
}

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
        controller.selectRow(0);
        QCOMPARE(controller.selectedCount(), 1);
        controller.fetchPage(256);
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        QCOMPARE(controller.entries()->rowCount(), 14);
        QCOMPARE(controller.selectedCount(), 0);
        QVERIFY(controller.detail().isEmpty());
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
    void selectionSearchAndDirectoryNavigation() {
        BrowserController controller;
        controller.openPath(QStringLiteral(XDZIP_FIXTURES "/normal.zip"));
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        controller.selectRow(2);
        controller.selectRow(3);
        QCOMPARE(controller.selectedCount(), 2);
        QCOMPARE(controller.detail().value("id").toString(), "4");
        controller.selectRow(2);
        QCOMPARE(controller.selectedCount(), 1);
        controller.enterDirectory(0);
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        QCOMPARE(controller.total(), 1);
        QCOMPARE(controller.selectedCount(), 0);
        QVERIFY(controller.detail().isEmpty());
        QCOMPARE(controller.location(), QString::fromUtf8("旅行/"));
        controller.showAll();
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        controller.search("NOTES");
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        QCOMPARE(controller.total(), 2);
        QCOMPARE(
            controller.entries()->data(controller.entries()->index(0, 0), Qt::UserRole).toString(),
            "3");
        controller.search("missing");
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        QVERIFY(controller.ready());
        QCOMPARE(controller.total(), 0);
        controller.search("");
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        QCOMPARE(controller.total(), 5);
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
        QCOMPARE(window->property("accentTextColor").value<QColor>(), QColor(Qt::white));
        controller->openPath(QStringLiteral(XDZIP_FIXTURES "/normal.zip"));
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy(), 5000);
        QCOMPARE(controller->total(), 5);
        QCOMPARE(controller->entries()
                     ->data(controller->entries()->index(1, 0), Qt::DisplayRole)
                     .toString(),
                 QString::fromUtf8("旅行/说明.txt"));
        QVERIFY(QTest::qWaitForWindowExposed(window));
        QQuickItem *cell = nullptr;
        QTRY_VERIFY((cell = findVisual(window->contentItem(), "entry-1-0")));
        const auto point = cell->mapToScene(QPointF(120, 24)).toPoint();
        QTest::mouseClick(window, Qt::LeftButton, Qt::NoModifier, point);
        QTRY_COMPARE(controller->selectedCount(), 1);
        QCOMPARE(controller->detail().value("id").toString(), "2");
        const auto checkPoint = cell->mapToScene(QPointF(20, 24)).toPoint();
        QTest::mouseClick(window, Qt::LeftButton, Qt::NoModifier, checkPoint);
        QTRY_COMPARE(controller->selectedCount(), 0);
        QTest::mouseClick(window, Qt::LeftButton, Qt::NoModifier, checkPoint);
        QTRY_COMPARE(controller->selectedCount(), 1);
        QTest::qWait(30);
        QVERIFY(window->grabWindow().save("zip-browser.png"));
        window->setProperty("darkMode", true);
        QTest::qWait(30);
        QVERIFY(window->grabWindow().save("zip-browser-dark.png"));
        window->setProperty("darkMode", false);
    }
};
QTEST_MAIN(BrowserTest)
#include "browser_test.moc"
