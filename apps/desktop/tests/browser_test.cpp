#include "BrowserController.h"
#include "UiFont.h"
#include "visual_audit.h"
#include <QAbstractItemModelTester>
#include <QClipboard>
#include <QQmlApplicationEngine>
#include <QQuickItem>
#include <QQuickStyle>
#include <QQuickWindow>
#include <QSignalSpy>
#include <QTemporaryDir>
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
        QVERIFY(installUiFont());
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
    void completeNavigationAndGlobalSort() {
        BrowserController controller;
        QAbstractItemModelTester modelCheck(controller.directories(),
                                            QAbstractItemModelTester::FailureReportingMode::QtTest);
        controller.openPath(QStringLiteral(XDZIP_FIXTURES "/pages.zip"));
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        auto *tree = controller.directories();
        QCOMPARE(tree->rowCount(), 1);
        const auto node = tree->index(0, 0);
        QCOMPARE(tree->data(node).toString(), "folder");
        QCOMPARE(tree->data(node, Qt::UserRole + 1).toInt(), 270);
        controller.navigateDirectory(tree->data(node, Qt::UserRole).toString());
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        QCOMPARE(controller.total(), 270);
        QCOMPARE(controller.location(), "folder/");
        controller.sortByColumn(0);
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        controller.selectRow(0);
        controller.sortByColumn(0);
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        QCOMPARE(controller.sortOrder(), 2);
        QCOMPARE(controller.selectedCount(), 0);
        QCOMPARE(
            controller.entries()->data(controller.entries()->index(0, 0), Qt::UserRole).toString(),
            "270");
        controller.fetchPage(128);
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        QCOMPARE(
            controller.entries()->data(controller.entries()->index(0, 0), Qt::UserRole).toString(),
            "142");
        QCOMPARE(tree->rowCount(), 1);
        QCOMPARE(tree->data(tree->index(0, 0), Qt::UserRole + 1).toInt(), 270);
        controller.sortByColumn(-1);
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        QCOMPARE(controller.offset(), 0);
        QCOMPARE(controller.sortOrder(), 0);
        controller.openPath(QStringLiteral(XDZIP_FIXTURES "/navigation-depth.zip"));
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        QVERIFY(controller.ready());
        QVERIFY(controller.navigationLimited());
        QCOMPARE(tree->rowCount(), 0);
        QCOMPARE(controller.total(), 1);
        controller.openPath(QStringLiteral(XDZIP_FIXTURES "/empty.zip"));
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        QVERIFY(controller.ready());
        QVERIFY(!controller.navigationLimited());
        QCOMPARE(tree->rowCount(), 0);
    }
    void contextualSelectionRangeAndClipboard() {
        BrowserController controller;
        controller.openPath(QStringLiteral(XDZIP_FIXTURES "/normal.zip"));
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        QCOMPARE(controller.archiveTotal(), 5);
        controller.selectOnlyRow(2);
        controller.selectRow(3);
        controller.selectForContext(2);
        QCOMPARE(controller.selectedCount(), 2);
        QCOMPARE(controller.detail().value("id").toString(), "3");
        controller.selectForContext(0);
        QCOMPARE(controller.selectedCount(), 1);
        controller.copySelection(false);
        QCOMPARE(QGuiApplication::clipboard()->text(), QString::fromUtf8("旅行"));
        controller.copySelection(true);
        QCOMPARE(QGuiApplication::clipboard()->text(), QString::fromUtf8("旅行/"));
        controller.selectOnlyRow(1);
        controller.selectRangeTo(3);
        QCOMPARE(controller.selectedCount(), 3);
        controller.copySelection(false);
        QCOMPARE(QGuiApplication::clipboard()->text().split('\n').size(), 3);
        controller.selectAllOnPage(true);
        QCOMPARE(controller.selectedCount(), 5);
        controller.selectAllOnPage(false);
        QCOMPARE(controller.selectedCount(), 0);
        QVERIFY(controller.detail().isEmpty());
        controller.search("NOTES");
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        QCOMPARE(controller.archiveTotal(), 5);
        controller.selectRangeTo(1);
        QCOMPARE(controller.selectedCount(), 1);
        controller.selectRangeTo(0);
        QCOMPARE(controller.selectedCount(), 2);
        controller.openPath(QStringLiteral(XDZIP_FIXTURES "/pages.zip"));
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        controller.selectAllOnPage(true);
        QCOMPARE(controller.selectedCount(), 128);
        controller.fetchPage(128);
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        QCOMPARE(controller.selectedCount(), 0);
        controller.selectAllOnPage(true);
        QCOMPARE(controller.selectedCount(), 128);
    }
    void treeExpansionAndColumnInteraction() {
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *controller = window->findChild<BrowserController *>();
        QVERIFY(controller);
        controller->openPath(QStringLiteral(XDZIP_FIXTURES "/navigation.zip"));
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy(), 5000);
        QVERIFY(QTest::qWaitForWindowExposed(window));
        auto *treeView = findVisual(window->contentItem(), "directoryTree");
        QVERIFY(treeView);
        QVERIFY(treeView->property("selectionModel").value<QObject *>());
        auto *model = controller->directories();
        QCOMPARE(model->rowCount(), 3);
        const auto parent = model->index(0, 0);
        QCOMPARE(model->rowCount(parent), 1);
        QCOMPARE(model->data(parent, Qt::UserRole + 1).toInt(), 3);
        QQuickItem *node = nullptr;
        QTRY_VERIFY((node = findVisual(window->contentItem(), "directory-d1")));
        auto *expand = findVisual(window->contentItem(), "directoryDisclosure-d1");
        QVERIFY(expand);
        QTest::mouseClick(
            window, Qt::LeftButton, Qt::NoModifier,
            expand->mapToScene(QPointF(expand->width() / 2, expand->height() / 2)).toPoint());
        QQuickItem *child = nullptr;
        QTRY_VERIFY((child = findVisual(window->contentItem(), "directory-d2")));
        QTest::mouseClick(window, Qt::LeftButton, Qt::NoModifier,
                          child->mapToScene(QPointF(80, child->height() / 2)).toPoint());
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy(), 5000);
        QCOMPARE(controller->directoryId(), "d2");
        QCOMPARE(controller->total(), 2);
        QCOMPARE(controller->location(), "photos/2026/");
        // Arrow keys move the tree current item; Enter activates that item's scope.
        treeView->forceActiveFocus();
        QTest::keyClick(window, Qt::Key_Down);
        QCOMPARE(treeView->property("currentRow").toInt(), 2);
        QTest::keyClick(window, Qt::Key_Return);
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy(), 5000);
        QCOMPARE(controller->location(), "documents/");
        QCOMPARE(controller->total(), 1);
        QTest::keyClick(window, Qt::Key_Up);
        QTest::keyClick(window, Qt::Key_Return);
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy(), 5000);
        QCOMPARE(controller->location(), "photos/2026/");
        auto *header = findVisual(window->contentItem(), "sort-column-1");
        QVERIFY(header);
        QTest::mouseClick(
            window, Qt::LeftButton, Qt::NoModifier,
            header->mapToScene(QPointF(header->width() / 2, header->height() / 2)).toPoint());
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy(), 5000);
        QCOMPARE(controller->sortOrder(), 3);
        header->forceActiveFocus();
        QTest::keyClick(window, Qt::Key_Return);
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy(), 5000);
        QCOMPARE(controller->sortOrder(), 4);
        QCOMPARE(controller->entries()
                     ->data(controller->entries()->index(0, 0), Qt::UserRole)
                     .toString(),
                 "2");
        auto *disclosure = findVisual(window->contentItem(), "directoryDisclosure-d1");
        QVERIFY(disclosure);
        QTest::mouseClick(
            window, Qt::LeftButton, Qt::NoModifier,
            disclosure->mapToScene(QPointF(disclosure->width() / 2, disclosure->height() / 2))
                .toPoint());
        QCOMPARE(controller->directoryId(), "d2");
        QCOMPARE(controller->location(), "photos/2026/");
        auto *marker = findVisual(window->contentItem(), "directoryBranchMarker-d1");
        QVERIFY(marker);
        QTRY_VERIFY(marker->isVisible());
        QTest::mouseClick(
            window, Qt::LeftButton, Qt::NoModifier,
            disclosure->mapToScene(QPointF(disclosure->width() / 2, disclosure->height() / 2))
                .toPoint());
        QTRY_VERIFY(!marker->isVisible());
        // Collapse/reopen reuses delegates. Assert the rendered hierarchy, not merely
        // the expansion flag, before capturing or allowing a later frame to hide drift.
        for (int cycle = 0; cycle < 8; ++cycle) {
            QTRY_VERIFY([&] {
                auto *parentIcon = findVisual(window->contentItem(), "directoryFolder-d1");
                auto *childIcon = findVisual(window->contentItem(), "directoryFolder-d2");
                return parentIcon && childIcon && childIcon->isVisible() &&
                       qAbs(childIcon->mapToScene(QPointF()).x() -
                            parentIcon->mapToScene(QPointF()).x() - 16) < 0.5;
            }());
            if (cycle < 7) {
                // TreeView may pool even the parent delegate during relayout; locate
                // the current identity again instead of clicking a stale QObject.
                auto clickParent = [&] {
                    auto *button = findVisual(window->contentItem(), "directoryDisclosure-d1");
                    if (!button)
                        return false;
                    QTest::mouseClick(
                        window, Qt::LeftButton, Qt::NoModifier,
                        button->mapToScene(QPointF(button->width() / 2, button->height() / 2))
                            .toPoint());
                    return true;
                };
                QVERIFY(clickParent());
                QTRY_VERIFY([&] {
                    auto *collapsed = findVisual(window->contentItem(), "directory-d2");
                    return !collapsed || !collapsed->isVisible();
                }());
                QVERIFY(clickParent());
            }
        }
        if (!qEnvironmentVariableIsEmpty("XDZIP_VISUAL_AUDIT")) {
            const auto directory = qEnvironmentVariable("XDZIP_VISUAL_AUDIT");
            QVERIFY(QDir().mkpath(directory));
            QVERIFY(visual_audit::save(window, directory, "navigation-tree"));
        }
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
    void realContextMenusAndGrid() {
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *controller = window->findChild<BrowserController *>();
        QVERIFY(controller);
        controller->openPath(QStringLiteral(XDZIP_FIXTURES "/normal.zip"));
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy(), 5000);
        QVERIFY(QTest::qWaitForWindowExposed(window));
        controller->selectOnlyRow(2);
        controller->selectRow(3);
        QQuickItem *cell = nullptr;
        QTRY_VERIFY((cell = findVisual(window->contentItem(), "entry-2-0")));
        auto *menu = window->findChild<QObject *>("browserEntryMenu");
        QVERIFY(menu);
        QTest::mouseClick(window, Qt::RightButton, Qt::NoModifier,
                          cell->mapToScene(QPointF(80, cell->height() / 2)).toPoint());
        QTRY_VERIFY(menu->property("visible").toBool());
        QCOMPARE(controller->selectedCount(), 2);
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_VERIFY(!menu->property("visible").toBool());
        cell->forceActiveFocus();
        QTest::keyClick(window, Qt::Key_F10, Qt::ShiftModifier);
        QTRY_VERIFY(menu->property("visible").toBool());
        controller->search("notes");
        QTRY_VERIFY(!menu->property("visible").toBool());
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy(), 5000);
        QCOMPARE(controller->selectedCount(), 0);
        auto *viewButton = findVisual(window->contentItem(), "browserViewMenuButton");
        QVERIFY(viewButton);
        QTest::mouseClick(window, Qt::RightButton, Qt::NoModifier,
                          viewButton->mapToScene(QPointF(12, 12)).toPoint());
        auto *viewMenu = window->findChild<QObject *>("browserViewMenu");
        QVERIFY(viewMenu);
        QTRY_VERIFY(viewMenu->property("visible").toBool());
        QTest::keyClick(window, Qt::Key_Down);
        QTest::keyClick(window, Qt::Key_Down);
        QTest::keyClick(window, Qt::Key_Return);
        QTRY_COMPARE(window->property("viewMode").toString(), "grid");
        QQuickItem *card = nullptr;
        QTRY_VERIFY((card = findVisual(window->contentItem(), "grid-entry-0")));
        QTest::mouseClick(window, Qt::RightButton, Qt::NoModifier,
                          card->mapToScene(QPointF(30, 30)).toPoint());
        QTRY_VERIFY(menu->property("visible").toBool());
        QCOMPARE(controller->selectedCount(), 1);
        if (!qEnvironmentVariableIsEmpty("XDZIP_VISUAL_AUDIT")) {
            QVERIFY(QDir().mkpath(qEnvironmentVariable("XDZIP_VISUAL_AUDIT")));
            QVERIFY(visual_audit::save(window, qEnvironmentVariable("XDZIP_VISUAL_AUDIT"),
                                       "command-context-grid"));
        }
        QTest::keyClick(window, Qt::Key_Escape);
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
    void mainWindowFidelityAndSessionClose() {
        QTemporaryDir directory;
        QVERIFY(directory.isValid());
        const auto archivePath = directory.filePath(QString::fromUtf8("北境旅行.zip"));
        QVERIFY(QFile::copy(QStringLiteral(XDZIP_FIXTURES "/visual-main.zip"), archivePath));
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *controller = window->findChild<BrowserController *>();
        QVERIFY(controller);
        controller->openPath(archivePath);
        QTRY_VERIFY_WITH_TIMEOUT(controller->ready() && !controller->busy(), 5000);
        QCOMPARE(controller->total(), 9);
        controller->selectOnlyRow(3);
        QCOMPARE(controller->detail().value("type").toString(), "PNG");
        QCOMPARE(controller->detail().value("modified").toString(), QString::fromUtf8("—"));
        QVERIFY(controller->selectedBytes().toULongLong() > 0);
        QVERIFY(QTest::qWaitForWindowExposed(window));
        const auto captures = qEnvironmentVariable("XDZIP_VISUAL_AUDIT");
        if (!captures.isEmpty()) {
            QVERIFY(QDir().mkpath(captures));
            QVERIFY(visual_audit::save(window, captures, "main-light"));
            window->setProperty("darkMode", true);
            QVERIFY(visual_audit::save(window, captures, "main-dark"));
            window->setProperty("darkMode", false);
            // Layout evidence only: native Cocoa window buttons are not drawn by Qt.
            window->setProperty("integratedTitlebar", true);
            QVERIFY(visual_audit::save(window, captures, "main-integrated-layout"));
        }
        auto *all = findVisual(window->contentItem(), "selectPage");
        QVERIFY(all);
        const auto selectPoint =
            all->mapToScene(QPointF(all->width() / 2, all->height() / 2)).toPoint();
        QTest::mouseClick(window, Qt::LeftButton, Qt::NoModifier, selectPoint);
        QTRY_COMPARE(controller->selectedCount(), 9);
        QTest::mouseClick(window, Qt::LeftButton, Qt::NoModifier, selectPoint);
        QTRY_COMPARE(controller->selectedCount(), 0);
        const auto session = controller->sessionId();
        auto *close = findVisual(window->contentItem(), "archiveTabClose");
        QVERIFY(close);
        QTest::mouseClick(
            window, Qt::LeftButton, Qt::NoModifier,
            close->mapToScene(QPointF(close->width() / 2, close->height() / 2)).toPoint());
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy() && !controller->ready(), 5000);
        QVERIFY(controller->sessionId().isEmpty());
        QVERIFY(controller->runtimeAvailable());
        QCOMPARE(controller->total(), 0);
        QCOMPARE(controller->selectedBytes(), "0");
        QCOMPARE(controller->entries()->rowCount(), 0);
        QVERIFY(window->isVisible());
        controller->openPath(archivePath);
        QTRY_VERIFY_WITH_TIMEOUT(controller->ready() && !controller->busy(), 5000);
        QVERIFY(controller->sessionId() != session);
        QCOMPARE(controller->total(), 9);
        // Restoring the design's columns must not hide the existing review state.
        controller->openPath(QStringLiteral(XDZIP_FIXTURES "/unsafe-names.zip"));
        QTRY_VERIFY_WITH_TIMEOUT(controller->ready() && !controller->busy(), 5000);
        controller->selectOnlyRow(0);
        QVERIFY(controller->detail().value("attention").toBool());
        QVERIFY(
            controller->detail().value("status").toString().contains(QString::fromUtf8("需检查")));
        QVERIFY(controller->entries()
                    ->data(controller->entries()->index(0, 0), Qt::UserRole + 5)
                    .toBool());
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
        auto *selection = findVisual(cell, "entrySelection");
        QVERIFY(selection);
        const auto checkPoint =
            selection->mapToScene(QPointF(selection->width() / 2, selection->height() / 2))
                .toPoint();
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
        // Hiding/reopening details must preserve the actual archive selection.
        window->resize(1280, 820);
        auto *inspector = findVisual(window->contentItem(), "archiveInspector");
        auto *toggle = findVisual(window->contentItem(), "inspectorToggle");
        QVERIFY(inspector && toggle);
        QTRY_VERIFY(inspector->isVisible());
        auto *closeDetails = findVisual(inspector, "inspectorClose");
        QVERIFY(closeDetails);
        QTest::mouseClick(
            window, Qt::LeftButton, Qt::NoModifier,
            closeDetails->mapToScene(QPointF(closeDetails->width() / 2, closeDetails->height() / 2))
                .toPoint());
        QTRY_VERIFY(!inspector->isVisible());
        QVERIFY(QMetaObject::invokeMethod(toggle, "clicked"));
        QTRY_VERIFY(inspector->isVisible());
        window->resize(1040, 680);
        QTRY_VERIFY(!inspector->isVisible());
        QVERIFY(QMetaObject::invokeMethod(toggle, "clicked"));
        auto *detailsWindow = window->findChild<QQuickWindow *>("inspectorWindow");
        QVERIFY(detailsWindow);
        QTRY_VERIFY(detailsWindow->isVisible());
        QCOMPARE(detailsWindow->transientParent(), window);
        QCOMPARE(detailsWindow->modality(), Qt::NonModal);
        QVERIFY(!detailsWindow->flags().testFlag(Qt::FramelessWindowHint));
        QCOMPARE(window->width(), 1040);
        QCOMPARE(controller->selectedCount(), 1);
        if (!qEnvironmentVariableIsEmpty("XDZIP_VISUAL_AUDIT"))
            QVERIFY(visual_audit::save(detailsWindow, qEnvironmentVariable("XDZIP_VISUAL_AUDIT"),
                                       "inspector-independent"));
        detailsWindow->close();
    }
};
QTEST_MAIN(BrowserTest)
#include "browser_test.moc"
