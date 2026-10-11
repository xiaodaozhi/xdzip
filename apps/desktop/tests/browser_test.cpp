#include "BrowserController.h"
#include "visual_audit.h"
#include <QQmlApplicationEngine>
#include <QQuickItem>
#include <QQuickStyle>
#include <QQuickWindow>
#include <QSignalSpy>
#include <QTimer>
#include <QtTest>
#include <thread>

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
    void cleanup() {
        QTRY_COMPARE_WITH_TIMEOUT(RuntimeReceiver::activeReceivers(), 0, 5000);
    }
    void mailboxBackpressureShutdownAndIdle() {
        RuntimeConnection connection;
        QSignalSpy disconnected(&connection, &RuntimeConnection::disconnected);
        QSet<quint64> accepted;
        QSet<quint64> finished;
        bool duplicate = false;
        bool probeScheduled = false;
        bool guiYieldedBeforeDrainCompleted = false;
        connection.setConsumer([&](const xdzip::reader::Event &event) {
            if (!probeScheduled) {
                probeScheduled = true;
                // Posted during the first batch, before drain posts its continuation. Unlike a
                // millisecond timer, this tests yielding independently of machine speed.
                QMetaObject::invokeMethod(
                    &connection,
                    [&] { guiYieldedBeforeDrainCompleted = finished.size() < accepted.size(); },
                    Qt::QueuedConnection);
            }
            if (event.kind == xdzip::reader::EventKind::Finished &&
                accepted.contains(event.request_id)) {
                duplicate |= finished.contains(event.request_id);
                finished.insert(event.request_id);
            }
        });
        QElapsedTimer limit;
        limit.start();
        // Deliberately do not drain GUI notifications while filling both bounded transports.
        while (connection.statistics().peakQueued < 256 && limit.elapsed() < 5000) {
            xdzip::reader::Command command{};
            command.kind = xdzip::reader::CommandKind::Snapshot;
            const auto result = connection.submit(std::move(command));
            if (result.status == xdzip::reader::Admission::Accepted)
                accepted.insert(result.request_id);
            else
                QVERIFY(result.status == xdzip::reader::Admission::Busy);
            std::this_thread::yield();
        }
        QCOMPARE(connection.statistics().peakQueued, std::size_t(256));
        QVERIFY(connection.statistics().peakBytes <= 8 * 1024 * 1024);
        connection.shutdown();
        connection.shutdown();
        QTRY_COMPARE_WITH_TIMEOUT(disconnected.count(), 1, 5000);
        QVERIFY(disconnected.first().first().toBool());
        QCOMPARE(finished, accepted);
        QVERIFY(!duplicate);
        QVERIFY(guiYieldedBeforeDrainCompleted);
        QVERIFY(connection.stopped());
        const auto notifications = connection.statistics().notifications;
        QTest::qWait(30);
        QCOMPARE(connection.statistics().notifications, notifications);
    }
    void idleAndDestructionDuringWork() {
        {
            RuntimeConnection connection;
            QTest::qWait(30);
            QCOMPARE(connection.statistics().notifications, std::size_t(0));
            QSignalSpy stopped(&connection, &RuntimeConnection::disconnected);
            connection.shutdown();
            QTRY_COMPARE_WITH_TIMEOUT(stopped.count(), 1, 5000);
            QVERIFY(stopped.first().first().toBool());
        }
        for (int i = 0; i < 20; ++i) {
            auto controller = std::make_unique<BrowserController>();
            controller->openPath(QStringLiteral(XDZIP_FIXTURES "/pages.zip"));
            // Destructor requests shutdown but never waits on a worker or queued GUI notification.
        }
        QTRY_COMPARE_WITH_TIMEOUT(RuntimeReceiver::activeReceivers(), 0, 5000);
    }
    void consumerCanDestroyConnectionAndInvalidInputIsExplicit() {
        auto connection = std::make_unique<RuntimeConnection>();
        connection->setConsumer([&](const xdzip::reader::Event &) { connection.reset(); });
        bool submitted = false;
        QTRY_VERIFY_WITH_TIMEOUT(([&] {
                                     if (submitted)
                                         return true;
                                     xdzip::reader::Command command{};
                                     command.kind = xdzip::reader::CommandKind::Snapshot;
                                     submitted = connection->submit(std::move(command)).status ==
                                                 xdzip::reader::Admission::Accepted;
                                     return submitted;
                                 })(),
                                 5000);
        QTRY_VERIFY_WITH_TIMEOUT(!connection, 5000);
        BrowserController controller;
        controller.openPath(QString(4097, 'x'));
        QVERIFY(!controller.busy());
        QVERIFY(!controller.ready());
        QVERIFY(controller.message().contains(QString::fromUtf8("支持范围")));
        controller.requestClose();
        QTRY_VERIFY_WITH_TIMEOUT(!controller.runtimeAvailable(), 5000);
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
    void preflightPreservesSelectionAndReportsDuplicates() {
        BrowserController controller;
        controller.openPath(QStringLiteral(XDZIP_FIXTURES "/normal.zip"));
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        controller.selectRow(2);
        controller.selectRow(3);
        QSignalSpy completed(&controller, &BrowserController::preflightReady);
        controller.inspectSelection();
        QTRY_COMPARE_WITH_TIMEOUT(completed.count(), 1, 5000);
        QCOMPARE(controller.selectedCount(), 2);
        QCOMPARE(controller.entries()->rowCount(), 5);
        QCOMPARE(controller.preflightReport().value("entries").toString(), "2");
        QCOMPARE(controller.preflightReport().value("findingCount").toString(), "1");
        controller.selectRow(2);
        QVERIFY(controller.preflightReport().isEmpty());
        controller.selectRow(3);
        controller.selectRow(0);
        controller.inspectSelection();
        QTRY_COMPARE_WITH_TIMEOUT(completed.count(), 2, 5000);
        QCOMPARE(controller.preflightReport().value("entries").toString(), "2");
        QCOMPARE(controller.preflightReport().value("bytes").toString(), "12");
        QVERIFY(controller.preflightReport().value("unicodeReview").toBool());
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
        controller->inspectSelection();
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy(), 5000);
        auto *reportWindow = window->findChild<QQuickWindow *>("preflightWindow");
        QVERIFY(reportWindow);
        QTRY_VERIFY(reportWindow->isVisible());
        QCOMPARE(reportWindow->transientParent(), window);
        QCOMPARE(reportWindow->modality(), Qt::NonModal);
        QVERIFY(reportWindow->flags().testFlag(Qt::WindowTitleHint));
        QVERIFY(QTest::qWaitForWindowExposed(reportWindow));
        QVERIFY(reportWindow->grabWindow().save("zip-preflight.png"));
        if (!qEnvironmentVariableIsEmpty("XDZIP_VISUAL_AUDIT")) {
            const auto directory = qEnvironmentVariable("XDZIP_VISUAL_AUDIT");
            QVERIFY(QDir().mkpath(directory));
            QVERIFY(visual_audit::save(reportWindow, directory, "preflight-light"));
        }
        reportWindow->close();
        QVERIFY(window->isVisible());
        QCOMPARE(controller->selectedCount(), 1);
        QVERIFY(visual_audit::capture(engine, window));
    }
};
QTEST_MAIN(BrowserTest)
#include "browser_test.moc"
