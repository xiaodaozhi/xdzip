#include "AppQuitGuard.h"
#include "BrowserController.h"
#include "UiFont.h"
#include "visual_audit.h"
#include <QAbstractItemModelTester>
#include <QClipboard>
#include <QElapsedTimer>
#include <QFile>
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
    if (item->property("inReusePool").toBool())
        return nullptr;
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
    void m3ExtractionWindowsAndRealOutput() {
        QTemporaryDir target;
        QVERIFY(target.isValid());
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *controller = window->findChild<BrowserController *>();
        QVERIFY(controller);
        controller->openPath(QStringLiteral(XDZIP_FIXTURES "/../extract/tree.zip"));
        QTRY_VERIFY(controller->ready() && !controller->busy());
        QCOMPARE(controller->entries()->rowCount(), 3);
        controller->selectAll(true);
        controller->prepareExtraction(false);
        auto *extract = window->findChild<QQuickWindow *>("extractWindow");
        QVERIFY(extract);
        QTRY_VERIFY(extract->isVisible());
        QCOMPARE(extract->width(), 614);
        auto *targetField = findVisual(extract->contentItem(), "extractTarget");
        QVERIFY(targetField);
        targetField->setProperty("text", target.path());
        QTRY_VERIFY(!controller->busy());
        controller->checkTarget(target.path());
        QTRY_VERIFY(!controller->busy() && controller->targetCheck().value("valid").toBool());
        auto *start = findVisual(extract->contentItem(), "startExtract");
        QVERIFY(start);
        QTRY_VERIFY(start->isEnabled());
        // Test the explicit action without opening a desktop file manager on CI.
        controller->startExtraction(target.path(), false, false, 0, false, false);
        QTRY_VERIFY_WITH_TIMEOUT(!controller->tasks().isEmpty() &&
                                     controller->tasks().first().toMap().value("terminal").toBool(),
                                 10000);
        auto result = controller->tasks().first().toMap();
        QCOMPARE(result.value("status").toString(), "succeeded");
        QCOMPARE(result.value("committedFiles").toString(), "3");
        QCOMPARE(result.value("skippedFiles").toString(), "0");
        QVERIFY(QFile::exists(target.filePath(QStringLiteral("照片/说明.txt"))));
        QFile binary(target.filePath("data.bin"));
        QVERIFY(binary.open(QIODevice::ReadOnly));
        const auto data = binary.readAll();
        QCOMPARE(data.size(), 262144);
        for (qsizetype i = 0; i < data.size(); ++i)
            QCOMPARE(static_cast<unsigned char>(data[i]), static_cast<unsigned char>(i % 256));
        binary.close();
        QCOMPARE(controller->entries()->rowCount(), 3);
        QCOMPARE(controller->selectedCount(), 3);
        auto *task = window->findChild<QQuickWindow *>("taskWindow");
        QVERIFY(task);
        QTRY_VERIFY(task->isVisible());
        QCOMPARE(task->width(), 846);
        QVERIFY(!(task->flags() & Qt::FramelessWindowHint));
        QCOMPARE(task->transientParent(), window);
        const auto button = findVisual(task->contentItem(), "cancelButton");
        QVERIFY(button);
        QVERIFY(!button->isVisible());
        const auto capture = [&](QQuickWindow *w, const QString &name) {
            if (qEnvironmentVariableIsEmpty("XDZIP_M3_VISUAL_AUDIT"))
                return true;
            const auto directory = qEnvironmentVariable("XDZIP_M3_VISUAL_AUDIT");
            return QDir().mkpath(directory) && visual_audit::save(w, directory, name);
        };
        QVERIFY(capture(task, "task-success"));
        controller->prepareExtraction(true);
        QTRY_VERIFY(extract->isVisible());
        QTRY_VERIFY(!controller->busy());
        targetField->setProperty("text", target.path());
        controller->checkTarget(target.path());
        QTRY_VERIFY(!controller->busy() && controller->targetCheck().value("valid").toBool());
        QVERIFY(capture(extract, "extract-options"));
        controller->startExtraction(target.path(), true, false, 0, false, false);
        QTRY_VERIFY_WITH_TIMEOUT(controller->activeTaskCount() == 1 &&
                                     controller->tasks().first().toMap().value("status") ==
                                         "waiting",
                                 10000);
        result = controller->tasks().first().toMap();
        const auto taskId = result.value("id").toString();
        const auto prompt = result.value("promptId").toString();
        QVERIFY(
            !controller->decideTask(taskId, QString::number(prompt.toULongLong() + 1), 3, false));
        auto *conflict = window->findChild<QQuickWindow *>("conflictWindow");
        QVERIFY(conflict);
        QTRY_VERIFY(conflict->isVisible());
        QCOMPARE(conflict->width(), 612);
        QVERIFY(capture(conflict, "conflict"));
        QVERIFY(capture(task, "task-waiting"));
        conflict->close();
        task->close();
        QCOMPARE(controller->activeTaskCount(), 1);
        AppQuitGuard guard;
        guard.controller = controller;
        guard.window = window;
        QEvent quitEvent(QEvent::Quit);
        QVERIFY(guard.eventFilter(nullptr, &quitEvent));
        QVERIFY(window->isVisible());
        QCOMPARE(controller->activeTaskCount(), 1);
        auto *quit = window->findChild<QQuickWindow *>("quitWindow");
        QVERIFY(quit);
        QTRY_VERIFY(quit->isVisible());
        QVERIFY(capture(quit, "quit"));
        quit->close();
        controller->closeArchive();
        QTRY_VERIFY(!controller->ready() && !controller->busy());
        // A task retains its input authority after its browser session is closed.
        QVERIFY(controller->decideTask(taskId, prompt, 1, true));
        QVERIFY(!controller->decideTask(taskId, prompt, 3, false));
        QTRY_COMPARE_WITH_TIMEOUT(controller->activeTaskCount(), 0, 10000);
        QCOMPARE(controller->tasks().first().toMap().value("skippedFiles").toString(), "3");
        QCOMPARE(controller->tasks().first().toMap().value("committedFiles").toString(), "0");
        controller->requestClose();
    }
    void m3IntegrityFailurePreservesDestinationAndCancelEndsWaiting() {
        QTemporaryDir target;
        QVERIFY(target.isValid());
        const auto leaf = target.filePath(QStringLiteral("说明.txt"));
        QFile old(leaf);
        QVERIFY(old.open(QIODevice::WriteOnly));
        old.write("old important file");
        old.close();
        BrowserController controller;
        const auto start = [&](const QString &fixture, int policy) {
            controller.openPath(QStringLiteral(XDZIP_FIXTURES "/../extract/") + fixture);
            QTRY_VERIFY(controller.ready() && !controller.busy());
            controller.prepareExtraction(true);
            controller.checkTarget(target.path());
            QTRY_VERIFY(!controller.busy() && controller.targetCheck().value("valid").toBool());
            controller.startExtraction(target.path(), true, false, policy, false, false);
        };
        start("crc_bad.zip", 3);
        QTRY_VERIFY(!controller.tasks().isEmpty() &&
                    controller.tasks().first().toMap().value("terminal").toBool());
        QCOMPARE(controller.tasks().first().toMap().value("errorCode").toString(),
                 "integrity_error");
        QVERIFY(old.open(QIODevice::ReadOnly));
        QCOMPARE(old.readAll(), QByteArray("old important file"));
        old.close();
        start("deflate.zip", 0);
        QTRY_VERIFY(controller.activeTaskCount() == 1 &&
                    controller.tasks().first().toMap().value("status") == "waiting");
        controller.cancelTask(controller.tasks().first().toMap().value("id").toString());
        QTRY_COMPARE(controller.activeTaskCount(), 0);
        QCOMPARE(controller.tasks().first().toMap().value("status").toString(), "cancelled");
        QVERIFY(old.open(QIODevice::ReadOnly));
        QCOMPARE(old.readAll(), QByteArray("old important file"));
        old.close();
        QCOMPARE(QDir(target.path())
                     .entryList(QDir::Files | QDir::Dirs | QDir::NoDotAndDotDot | QDir::Hidden)
                     .size(),
                 1);
        controller.requestClose();
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
        QCOMPARE(controller.total(), 1);
        QCOMPARE(controller.archiveTotal(), 270);
        QCOMPARE(controller.entries()->data(controller.entries()->index(0, 0)).toString(),
                 "folder");
        controller.selectRow(0);
        QVERIFY(!controller.canPreflight());
        controller.enterDirectory(0);
        QTRY_VERIFY(!controller.busy());
        QCOMPARE(controller.total(), 270);
        QCOMPARE(controller.entries()->rowCount(), 270);
        const auto session = controller.sessionId();
        controller.selectRow(0);
        QCOMPARE(controller.selectedCount(), 1);
        QCOMPARE(controller.entries()->rowCount(), 270);
        QCOMPARE(controller.entries()
                     ->data(controller.entries()->index(256, 0), Qt::UserRole)
                     .toString(),
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
        QCOMPARE(controller.detail().value("id").toString(), "5");
        controller.selectRow(2);
        QCOMPARE(controller.selectedCount(), 1);
        controller.enterDirectory(0);
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        QCOMPARE(controller.total(), 1);
        QCOMPARE(controller.selectedCount(), 0);
        QVERIFY(controller.detail().isEmpty());
        QCOMPARE(controller.location(), QString::fromUtf8("旅行/"));
        controller.showRoot();
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
        QCOMPARE(controller.total(), 4);
    }
    void typeHeaderSortGroupsDirectoriesBeforeExtensionlessFiles() {
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *controller = window->findChild<BrowserController *>();
        controller->openPath(QStringLiteral(XDZIP_FIXTURES "/type-order.zip"));
        QTRY_VERIFY(controller->ready() && !controller->busy());
        QVERIFY(QTest::qWaitForWindowExposed(window));
        auto *header = findVisual(window->contentItem(), "sort-column-3");
        QVERIFY(header);
        const auto click = [window, header] {
            QTest::mouseClick(
                window, Qt::LeftButton, Qt::NoModifier,
                header->mapToScene(QPointF(header->width() / 2, header->height() / 2)).toPoint());
        };
        const auto names = [controller] {
            QStringList result;
            for (int row = 0; row < controller->entries()->rowCount(); ++row)
                result.append(
                    controller->entries()->data(controller->entries()->index(row, 0)).toString());
            return result;
        };
        click();
        QTRY_VERIFY(controller->sortOrder() == 9 && !controller->busy());
        QCOMPARE(names(),
                 QStringList({"item-alpha", "item-folder.ZIP", "item-implicit", "item-README",
                              "item-LICENSE", "item-duplicate", "item-duplicate", "item-image.JPG",
                              "item-image.jpeg", "item-last.ZIP"}));
        QCOMPARE(controller->entries()->data(controller->entries()->index(2, 3)).toString(),
                 QString::fromUtf8("文件夹"));
        QCOMPARE(controller->entries()->data(controller->entries()->index(3, 3)).toString(),
                 QString::fromUtf8("文件"));
        click();
        QTRY_VERIFY(controller->sortOrder() == 10 && !controller->busy());
        QCOMPARE(names(),
                 QStringList({"item-alpha", "item-folder.ZIP", "item-implicit", "item-last.ZIP",
                              "item-image.JPG", "item-image.jpeg", "item-README", "item-LICENSE",
                              "item-duplicate", "item-duplicate"}));
        controller->search("item");
        QTRY_VERIFY(!controller->busy());
        QCOMPARE(names(),
                 QStringList({"item-alpha/", "item-folder.ZIP/", "item-last.ZIP", "item-image.JPG",
                              "item-image.jpeg", "item-README", "item-LICENSE", "item-duplicate",
                              "item-duplicate", "item-implicit/item-child"}));
        click();
        QTRY_VERIFY(controller->sortOrder() == 9 && !controller->busy());
        QCOMPARE(names(),
                 QStringList({"item-alpha/", "item-folder.ZIP/", "item-README", "item-LICENSE",
                              "item-duplicate", "item-duplicate", "item-implicit/item-child",
                              "item-image.JPG", "item-image.jpeg", "item-last.ZIP"}));
        controller->requestClose();
        QTRY_VERIFY(!controller->runtimeAvailable());
    }
    void sourceChangeDuringPreflightInvalidatesSnapshot() {
        QTemporaryDir owned;
        const auto path = owned.filePath("preflight.zip");
        QVERIFY(QFile::copy(QStringLiteral(XDZIP_FIXTURES "/normal.zip"), path));
        BrowserController controller;
        QSignalSpy report(&controller, &BrowserController::preflightReady);
        controller.openPath(path);
        QTRY_VERIFY(controller.ready() && !controller.busy());
        controller.selectOnlyRow(1);
        QVERIFY(controller.canPreflight());
        QFile changed(path);
        QVERIFY(changed.open(QIODevice::Append));
        QCOMPARE(changed.write("changed"), 7);
        changed.close();
        controller.inspectSelection();
        QTRY_COMPARE(report.count(), 1);
        QVERIFY(!controller.busy());
        QVERIFY(!controller.ready());
        QCOMPARE(controller.total(), 0);
        QCOMPARE(controller.archiveTotal(), 0);
        QCOMPARE(controller.entries()->rowCount(), 0);
        QCOMPARE(controller.directories()->rowCount(), 0);
        QCOMPARE(controller.selectedCount(), 0);
        QVERIFY(controller.detail().isEmpty());
        QVERIFY(controller.preflightReport().contains("error"));
        controller.requestClose();
        QTRY_VERIFY(!controller.runtimeAvailable());
    }
    void m2LargeArchiveLatestQueryAndSourceChange() {
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *controller = window->findChild<BrowserController *>();
        QVERIFY(controller);
        QElapsedTimer elapsed;
        elapsed.start();
        controller->openPath(QStringLiteral(XDZIP_M2_FIXTURE));
        QTRY_VERIFY_WITH_TIMEOUT(controller->ready() && !controller->busy(), 20000);
        const auto openMilliseconds = elapsed.elapsed();
        const auto capturePath = qEnvironmentVariable("XDZIP_VISUAL_AUDIT");
        if (!capturePath.isEmpty()) {
            QVERIFY(QDir().mkpath(capturePath));
            QVERIFY(visual_audit::save(window, capturePath, "m2-large-flat"));
        }
        QCOMPARE(controller->total(), 100000);
        QCOMPARE(controller->archiveTotal(), 100000);
        QCOMPARE(controller->entries()->rowCount(), 100000);
        QCOMPARE(controller->entries()->data(controller->entries()->index(99999, 0)).toString(),
                 "item-099999.txt");
        QVERIFY(QTest::qWaitForWindowExposed(window));
        QTest::qWait(50);
        int delegates = 0;
        const auto countDelegates = [&delegates](auto &&self, QQuickItem *item) -> void {
            if (item->objectName().startsWith("entry-") && item->objectName().count('-') == 2)
                ++delegates;
            for (auto *child : item->childItems())
                self(self, child);
        };
        countDelegates(countDelegates, window->contentItem());
        QVERIFY(delegates > 0 && delegates < 512);
        controller->selectAll(true);
        QCOMPARE(controller->selectedCount(), 100000);
        QVERIFY(!controller->canPreflight());
        QCOMPARE(controller->selectedBytes(), "0");
        elapsed.restart();
        controller->sortByColumn(0);
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy(), 20000);
        const auto sortMilliseconds = elapsed.elapsed();
        QCOMPARE(controller->total(), 100000);
        QCOMPARE(controller->selectedCount(), 0);
        controller->selectAll(true);
        controller->search("099998");
        controller->search("099997");
        controller->search("099999");
        controller->sortByColumn(0);
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy(), 10000);
        QCOMPARE(controller->total(), 1);
        QCOMPARE(controller->selectedCount(), 0);
        QCOMPARE(controller->sortOrder(), 2);
        QCOMPARE(controller->entries()->data(controller->entries()->index(0, 0)).toString(),
                 "item-099999.txt");
        if (!capturePath.isEmpty())
            QVERIFY(visual_audit::save(window, capturePath, "m2-latest-query"));
        controller->openPath(QStringLiteral(XDZIP_M2_HIERARCHICAL));
        QTRY_VERIFY_WITH_TIMEOUT(controller->ready() && !controller->busy(), 20000);
        QCOMPARE(controller->total(), 100);
        QCOMPARE(controller->archiveTotal(), 100000);
        controller->navigateDirectory("d1");
        controller->navigateDirectory("d2");
        controller->showRoot();
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy(), 10000);
        QCOMPARE(controller->total(), 100);
        QCOMPARE(controller->location(), "");
        controller->navigateDirectory("d100");
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy(), 10000);
        QCOMPARE(controller->total(), 1000);
        if (!capturePath.isEmpty())
            QVERIFY(visual_audit::save(window, capturePath, "m2-directory"));
        QCOMPARE(controller->entries()->data(controller->entries()->index(0, 0)).toString(),
                 "item-099000.txt");
        controller->search("099999");
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy(), 10000);
        QCOMPARE(controller->entries()->data(controller->entries()->index(0, 0)).toString(),
                 "folder-099/item-099999.txt");
        qInfo().noquote()
            << QString("M2_QT_METRICS open_ms=%1 rows=100000 delegates=%2 full_sort_ms=%3")
                   .arg(openMilliseconds)
                   .arg(delegates)
                   .arg(sortMilliseconds);
        QTemporaryDir owned;
        QVERIFY(owned.isValid());
        const auto path = owned.filePath("source.zip");
        QVERIFY(QFile::copy(QStringLiteral(XDZIP_FIXTURES "/normal.zip"), path));
        controller->openPath(path);
        QTRY_VERIFY(controller->ready() && !controller->busy());
        QFile changed(path);
        QVERIFY(changed.open(QIODevice::Append));
        QCOMPARE(changed.write("x"), 1);
        changed.close();
        controller->search("notes");
        QTRY_VERIFY(!controller->busy());
        QVERIFY(!controller->ready());
        QCOMPARE(controller->entries()->rowCount(), 0);
        QVERIFY(controller->message().contains(QString::fromUtf8("源文件")));
        controller->requestClose();
        QTRY_VERIFY(!controller->runtimeAvailable());
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
        QCOMPARE(controller.entries()->rowCount(), 270);
        QCOMPARE(controller.entries()
                     ->data(controller.entries()->index(128, 0), Qt::UserRole)
                     .toString(),
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
        QCOMPARE(controller.total(), 0);
        controller.search("file");
        QTRY_VERIFY(!controller.busy());
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
        QCOMPARE(controller.detail().value("id").toString(), "4");
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
        controller.selectAll(true);
        QCOMPARE(controller.selectedCount(), 4);
        controller.selectAll(false);
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
        controller.enterDirectory(0);
        QTRY_VERIFY(!controller.busy());
        controller.selectAll(true);
        QCOMPARE(controller.selectedCount(), 270);
        controller.selectAll(false);
        QCOMPARE(controller.selectedCount(), 0);
        controller.selectRangeTo(269);
        QVERIFY(controller.selectedCount() > 0);
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
        QTRY_VERIFY(treeView->property("contentWidth").toDouble() <= treeView->width() + 1);
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
        // Any relayout can reuse even the parent delegate for another model identity.
        auto clickParent = [&] {
            auto *button = findVisual(window->contentItem(), "directoryDisclosure-d1");
            if (!button)
                return false;
            QTest::mouseClick(
                window, Qt::LeftButton, Qt::NoModifier,
                button->mapToScene(QPointF(button->width() / 2, button->height() / 2)).toPoint());
            return true;
        };
        QVERIFY(clickParent());
        QTRY_COMPARE(treeView->property("rows").toInt(), 3);
        QCOMPARE(controller->directoryId(), "d2");
        QCOMPARE(controller->location(), "photos/2026/");
        auto branchMarkerVisible = [&] {
            auto *marker = findVisual(window->contentItem(), "directoryBranchMarker-d1");
            return marker && marker->isVisible();
        };
        QTRY_VERIFY(branchMarkerVisible());
        QVERIFY(clickParent());
        QTRY_COMPARE(treeView->property("rows").toInt(), 4);
        QTRY_VERIFY(!branchMarkerVisible());
        // Collapse/reopen reuses delegates. Assert the rendered hierarchy, not merely
        // the expansion flag, before capturing or allowing a later frame to hide drift.
        for (int cycle = 0; cycle < 8; ++cycle) {
            QTRY_VERIFY([&] {
                auto *parentIcon = findVisual(window->contentItem(), "directoryFolder-d1");
                auto *childIcon = findVisual(window->contentItem(), "directoryFolder-d2");
                return parentIcon && childIcon && childIcon->isVisible() &&
                       qAbs(childIcon->mapToScene(QPointF()).x() -
                            parentIcon->mapToScene(QPointF()).x() - 12.8) < 0.5;
            }());
            if (cycle < 7) {
                QVERIFY(clickParent());
                QTRY_COMPARE(treeView->property("rows").toInt(), 3);
                QTRY_VERIFY([&] {
                    auto *collapsed = findVisual(window->contentItem(), "directory-d2");
                    return !collapsed || !collapsed->isVisible();
                }());
                QVERIFY(clickParent());
                QTRY_COMPARE(treeView->property("rows").toInt(), 4);
            }
        }
        if (!qEnvironmentVariableIsEmpty("XDZIP_VISUAL_AUDIT")) {
            const auto directory = qEnvironmentVariable("XDZIP_VISUAL_AUDIT");
            QVERIFY(QDir().mkpath(directory));
            QVERIFY(visual_audit::save(window, directory, "navigation-tree"));
        }
    }
    void treeWidthTracksOnlyExpandedNodes() {
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *controller = window->findChild<BrowserController *>();
        controller->openPath(QStringLiteral(XDZIP_FIXTURES "/tree-layout.zip"));
        QTRY_VERIFY(controller->ready() && !controller->busy());
        auto *tree = findVisual(window->contentItem(), "directoryTree");
        QVERIFY(tree);
        QTRY_COMPARE(tree->property("rows").toInt(), 2);
        QTRY_VERIFY(tree->property("contentWidth").toDouble() <= tree->width() + 1);
        QVERIFY(
            QMetaObject::invokeMethod(tree, "expandRecursively", Q_ARG(int, 0), Q_ARG(int, -1)));
        QTRY_VERIFY(tree->property("rows").toInt() > 12);
        QTRY_VERIFY(tree->property("contentWidth").toDouble() > tree->width());
        QVERIFY(QMetaObject::invokeMethod(tree, "collapseRecursively", Q_ARG(int, 0)));
        QTRY_COMPARE(tree->property("rows").toInt(), 2);
        QTRY_VERIFY(tree->property("contentWidth").toDouble() <= tree->width() + 1);
    }
    void preflightPreservesSelectionAndReportsDuplicates() {
        BrowserController controller;
        controller.openPath(QStringLiteral(XDZIP_FIXTURES "/normal.zip"));
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        controller.selectRow(1);
        controller.selectRow(2);
        QSignalSpy completed(&controller, &BrowserController::preflightReady);
        controller.inspectSelection();
        QTRY_COMPARE_WITH_TIMEOUT(completed.count(), 1, 5000);
        QCOMPARE(controller.selectedCount(), 2);
        QCOMPARE(controller.entries()->rowCount(), 4);
        QCOMPARE(controller.preflightReport().value("entries").toString(), "2");
        QCOMPARE(controller.preflightReport().value("findingCount").toString(), "1");
        controller.selectRow(1);
        QVERIFY(controller.preflightReport().isEmpty());
        controller.selectRow(2);
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
        auto *enterDirectory = window->findChild<QQuickItem *>("contextEnterDirectory");
        auto *properties = window->findChild<QQuickItem *>("contextProperties");
        QVERIFY(enterDirectory);
        QVERIFY(properties);
        QVERIFY(!enterDirectory->isVisible());
        QTRY_COMPARE(enterDirectory->height(), 0);
        QTRY_COMPARE(properties->y(), 0);
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
        QCOMPARE(controller.total(), 4);
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
    void rootListingAndReviewedInteractions() {
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *controller = window->findChild<BrowserController *>();
        QVERIFY(controller);
        controller->openPath(QStringLiteral(XDZIP_FIXTURES "/tree-layout.zip"));
        QTRY_VERIFY(!controller->busy());
        QCOMPARE(controller->total(), 2);
        QCOMPARE(controller->archiveTotal(), 2);
        QCOMPARE(controller->entries()->data(controller->entries()->index(0, 0)).toString(),
                 "GitHub Desktop.app");
        controller->selectOnlyRow(0);
        QVERIFY(!controller->canPreflight());
        controller->copySelection(true);
        QCOMPARE(QGuiApplication::clipboard()->text(), "GitHub Desktop.app/");
        controller->selectAll(false);
        QVERIFY(QTest::qWaitForWindowExposed(window));
        const auto item = [window](const char *name) {
            return findVisual(window->contentItem(), QString::fromLatin1(name));
        };
        const auto click = [window](QQuickItem *target) {
            QTest::mouseClick(
                window, Qt::LeftButton, Qt::NoModifier,
                target->mapToScene(QPointF(target->width() / 2, target->height() / 2)).toPoint());
        };
        auto *caption = item("archiveCaption");
        auto *title = item("filePaneTitle");
        QCOMPARE(caption->property("text").toString(), QString::fromUtf8("档案文件"));
        QVERIFY(qAbs(caption->mapToScene(QPointF(0, caption->height() / 2)).y() -
                     title->mapToScene(QPointF(0, title->height() / 2)).y()) < .5);
        auto *cell = item("entry-0-0");
        QTest::mouseMove(window, cell->mapToScene(QPointF(100, cell->height() / 2)).toPoint());
        for (int column = 0; column < 5; ++column) {
            auto *band = findVisual(window->contentItem(), QString("entryBand-0-%1").arg(column));
            QVERIFY(band);
            QTRY_COMPARE(band->property("color").value<QColor>(), QColor("#EDF2FF"));
        }
        const auto captures = qEnvironmentVariable("XDZIP_VISUAL_AUDIT");
        if (!captures.isEmpty())
            QVERIFY(QDir().mkpath(captures));
        if (!captures.isEmpty())
            QVERIFY(visual_audit::save(window, captures, "root-row-hover"));
        auto *tree = item("directoryTree");
        QVERIFY(tree);
        QVERIFY(QMetaObject::invokeMethod(tree, "expandRecursively", Q_ARG(int, 0), Q_ARG(int, 2)));
        QQuickItem *guide = nullptr, *disclosure = nullptr;
        QTRY_VERIFY((guide = item("directoryParentGuide-d2")));
        QTRY_VERIFY((disclosure = item("directoryDisclosure-d1")));
        QVERIFY(qAbs(guide->mapToScene(QPointF(guide->width() / 2, 0)).x() -
                     disclosure->mapToScene(QPointF(disclosure->width() / 2, 0)).x()) < .01);
        QQmlComponent scaleSettings(&engine);
        scaleSettings.setData(
            "import QtQuick\nimport \"qrc:/xdzip/ui\" as Ui\n"
            "QtObject { property real scale: 1; onScaleChanged: Ui.Theme.fontScale = scale }",
            QUrl());
        std::unique_ptr<QObject> scaling(scaleSettings.create());
        QVERIFY(scaling);
        scaling->setProperty("scale", 2);
        QTRY_VERIFY(qAbs(caption->mapToScene(QPointF(0, caption->height() / 2)).y() -
                         title->mapToScene(QPointF(0, title->height() / 2)).y()) < .5);
        QTRY_VERIFY(qAbs(guide->mapToScene(QPointF(guide->width() / 2, 0)).x() -
                         disclosure->mapToScene(QPointF(disclosure->width() / 2, 0)).x()) < .01);
        scaling->setProperty("scale", 1);
        click(item("inspectorClose"));
        QQuickItem *toggle = nullptr;
        QTRY_VERIFY((toggle = item("inspectorToggle")) && toggle->isVisible());
        QCOMPARE(toggle->width(), 28.0);
        QCOMPARE(toggle->height(), 28.0);
        click(item("browserGridButton"));
        QQuickItem *card = nullptr, *check = nullptr;
        QTRY_VERIFY((card = item("grid-entry-0")));
        QTRY_VERIFY((check = item("gridSelection-0")));
        auto *grid = item("archiveGrid");
        QVERIFY(qAbs(grid->x() - 16) < .01);
        QTest::mouseMove(window, item("archiveOpen")->mapToScene(QPointF(10, 10)).toPoint());
        QTRY_VERIFY(!check->isVisible());
        QTest::mouseMove(window, card->mapToScene(QPointF(40, 40)).toPoint());
        QTRY_VERIFY(check->isVisible());
        click(check);
        QTRY_COMPARE(controller->selectedCount(), 1);
        QTest::mouseMove(window, item("archiveOpen")->mapToScene(QPointF(10, 10)).toPoint());
        QTRY_VERIFY(check->isVisible());
        if (!captures.isEmpty())
            QVERIFY(visual_audit::save(window, captures, "root-grid-selected"));
        window->setProperty("darkMode", true);
        QTRY_COMPARE(item("actionSplitDivider")->property("color").value<QColor>(),
                     QColor("#363D4B"));
        window->setProperty("darkMode", false);
        click(item("browserViewMenuButton"));
        auto *more = item("archiveMore");
        QTest::mouseMove(window, more->mapToScene(QPointF(10, 10)).toPoint());
        const QImage before = window->grabWindow();
        click(more);
        auto *menu = window->findChild<QObject *>("archiveMoreMenu");
        QVERIFY(menu);
        QTRY_VERIFY(menu->property("visible").toBool());
        auto *content = menu->property("contentItem").value<QQuickItem *>();
        auto *background = menu->property("background").value<QQuickItem *>();
        QVERIFY(background);
        auto *shadow = findVisual(background, "menuShadowLayer-0");
        QVERIFY(shadow && shadow->x() < 0 && shadow->width() > background->width());
        const auto top =
            content->mapToScene(QPointF()).y() - menu->property("topPadding").toDouble();
        QVERIFY(qAbs(top - more->mapToScene(QPointF(0, more->height())).y() - 4) < .1);
        const auto center = background->mapToScene(QPointF(-4, background->height() / 2));
        const QImage after = window->grabWindow();
        const auto pixel = QPoint(qRound(center.x() * after.width() / window->width()),
                                  qRound(center.y() * after.height() / window->height()));
        QVERIFY(after.valid(pixel));
        QVERIFY(after.pixelColor(pixel) != before.pixelColor(pixel));
        if (!captures.isEmpty())
            QVERIFY(visual_audit::save(window, captures, "root-menu-shadow"));
        QTest::keyClick(window, Qt::Key_Escape);
        controller->enterDirectory(0);
        QTRY_VERIFY(!controller->busy());
        QCOMPARE(controller->entries()->data(controller->entries()->index(0, 0)).toString(),
                 "Contents");
        controller->showRoot();
        QTRY_VERIFY(!controller->busy());
        controller->search("file.txt");
        QTRY_VERIFY(!controller->busy());
        QCOMPARE(controller->total(), 1);
        QVERIFY(controller->entries()
                    ->data(controller->entries()->index(0, 0))
                    .toString()
                    .contains("GitHub Desktop.app/Contents/Resources/"));
        controller->requestClose();
        QTRY_VERIFY(!controller->runtimeAvailable());
    }
    void reviewedMainControlsAndMenus() {
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *controller = window->findChild<BrowserController *>();
        controller->openPath(QStringLiteral(XDZIP_FIXTURES "/visual-main.zip"));
        QTRY_VERIFY(controller->ready() && !controller->busy());
        QVERIFY(QTest::qWaitForWindowExposed(window));
        const auto item = [window](const char *name) {
            return findVisual(window->contentItem(), QString::fromLatin1(name));
        };
        const auto click = [window](QQuickItem *target) {
            const auto point =
                target->mapToScene(QPointF(target->width() / 2, target->height() / 2)).toPoint();
            // Move before pressing, as a real pointer does. This lets hovered
            // controls dismiss their tooltips before the next hit test.
            QTest::mouseMove(window, point);
            QCoreApplication::processEvents();
            QTest::mouseClick(window, Qt::LeftButton, Qt::NoModifier, point);
        };
        for (const auto *name : {"archiveNewTab", "archiveSettings", "browserViewMenuButton",
                                 "browserGridButton", "browserSortMenuButton", "inspectorClose"}) {
            auto *button = item(name);
            QVERIFY(button);
            auto *icon = findVisual(button, "actionIcon");
            QVERIFY(icon);
            QTRY_VERIFY(
                qAbs(icon->mapToItem(button, QPointF(icon->width() / 2, icon->height() / 2)).x() -
                     button->width() / 2) < .5);
            QVERIFY(
                qAbs(icon->mapToItem(button, QPointF(icon->width() / 2, icon->height() / 2)).y() -
                     button->height() / 2) < .5);
        }
        for (const auto *name : {"archiveOpen", "archiveNew", "archiveTest"}) {
            auto *button = item(name);
            auto *icon = findVisual(button, "actionIcon");
            auto *label = findVisual(button, "actionLabel");
            QVERIFY(qAbs(label->mapToItem(button, QPointF()).x() -
                         icon->mapToItem(button, QPointF(icon->width(), 0)).x() - 6) < .01);
        }
        auto *close = item("inspectorClose");
        QTest::mouseMove(
            window, close->mapToScene(QPointF(close->width() / 2, close->height() / 2)).toPoint());
        QTRY_VERIFY(close->property("hovered").toBool());
        QVERIFY(findVisual(close, "actionBackground")->property("radius").toDouble() > 0);
        auto *extract = item("archiveExtract");
        QCOMPARE(findVisual(extract, "actionBackground")->property("color").value<QColor>(),
                 QColor("#1956ff"));
        for (const auto *name :
             {"browserViewMenuButton", "browserGridButton", "browserSortMenuButton"}) {
            QCOMPARE(item(name)->width(), 28.0);
            QCOMPARE(item(name)->height(), 28.0);
        }
        auto *grid = item("browserGridButton");
        const auto fixedGrid = grid->mapToScene(QPointF());
        controller->enterDirectory(0);
        QTRY_VERIFY(!controller->busy());
        QTRY_COMPARE(grid->mapToScene(QPointF()), fixedGrid);
        controller->showRoot();
        QTRY_VERIFY(!controller->busy());
        click(grid);
        QTRY_VERIFY(!item("fileTableHeader")->isVisible());
        const auto captures = qEnvironmentVariable("XDZIP_VISUAL_AUDIT");
        if (!captures.isEmpty())
            QVERIFY(QDir().mkpath(captures));
        if (!captures.isEmpty())
            QVERIFY(visual_audit::save(window, captures, "review-grid"));
        QSignalSpy frames(window, &QQuickWindow::afterAnimating);
        click(item("browserViewMenuButton"));
        // Visibility changes synchronously; the layout and hit-test tree update on the next frame.
        QTRY_VERIFY(item("fileTableHeader")->isVisible());
        frames.clear();
        window->requestUpdate();
        QTRY_VERIFY(frames.count() > 0);
        for (int column : {3, 4}) {
            auto *header = item(column == 3 ? "sort-column-3" : "sort-column-4");
            QVERIFY(header && header->isVisible() && header->width() > 0);
            const auto point =
                header->mapToScene(QPointF(header->width() / 2, header->height() / 2));
            auto *bar = item("fileTableHeader");
            QVERIFY(bar->contains(bar->mapFromScene(point)));
            click(header);
            QTRY_COMPARE(controller->sortOrder(), column == 3 ? 9 : 11);
            QTRY_VERIFY(!controller->busy());
            click(header);
            QTRY_COMPARE(controller->sortOrder(), column == 3 ? 10 : 12);
            QTRY_VERIFY(!controller->busy());
        }
        for (const auto &[buttonName, menuName] :
             {std::pair{"browserSortMenuButton", "browserSortMenu"},
              std::pair{"browserTasks", "browserTaskMenu"}}) {
            auto *button = item(buttonName);
            click(button);
            auto *menu = window->findChild<QObject *>(QString::fromLatin1(menuName));
            QVERIFY(menu);
            QTRY_VERIFY(menu->property("visible").toBool());
            // Popup coordinates are relative to its QML parent, which differs for the two menus.
            auto *popup = menu->property("contentItem").value<QQuickItem *>();
            QVERIFY(popup);
            const auto top =
                popup->mapToScene(QPointF()).y() - menu->property("topPadding").toDouble();
            const auto bottom = top + menu->property("height").toDouble();
            const auto buttonTop = button->mapToScene(QPointF()).y();
            if (QString::fromLatin1(buttonName) == "browserTasks")
                QVERIFY(bottom <= buttonTop - 3);
            else
                QVERIFY(top >= buttonTop + button->height() + 3);
            if (!captures.isEmpty())
                QVERIFY(visual_audit::save(window, captures, QString::fromLatin1(buttonName)));
            QTest::keyClick(window, Qt::Key_Escape);
        }
        auto *rootNode = item("directoryRoot");
        QTest::mouseClick(window, Qt::RightButton, Qt::NoModifier,
                          rootNode->mapToScene(QPointF(40, rootNode->height() / 2)).toPoint());
        auto *directoryMenu = window->findChild<QObject *>("browserDirectoryMenu");
        QTRY_VERIFY(directoryMenu->property("visible").toBool());
        auto *menuContent = directoryMenu->property("contentItem").value<QQuickItem *>();
        auto *separator = findVisual(menuContent, "commandSeparator");
        QVERIFY(separator);
        QVERIFY(separator->width() > 200);
        if (!captures.isEmpty())
            QVERIFY(visual_audit::save(window, captures, "review-directory-menu"));
        QTest::keyClick(window, Qt::Key_Escape);
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
        QCOMPARE(controller->detail().value("modified").toString(), QStringLiteral("2026/10/01"));
        QVERIFY(controller->selectedBytes().toULongLong() > 0);
        QVERIFY(QTest::qWaitForWindowExposed(window));
        const auto captures = qEnvironmentVariable("XDZIP_VISUAL_AUDIT");
        auto *more = findVisual(window->contentItem(), "archiveMore");
        QVERIFY(more);
        QTest::mouseClick(
            window, Qt::LeftButton, Qt::NoModifier,
            more->mapToScene(QPointF(more->width() / 2, more->height() / 2)).toPoint());
        auto *dropdown = window->findChild<QObject *>("archiveMoreMenu");
        QVERIFY(dropdown);
        QTRY_VERIFY(dropdown->property("visible").toBool());
        QCOMPARE(dropdown->property("width").toDouble(), 240.0);
        QCOMPARE(dropdown->property("font").value<QFont>().pixelSize(), 13);
        if (!captures.isEmpty()) {
            QVERIFY(QDir().mkpath(captures));
            QVERIFY(visual_audit::save(window, captures, "main-dropdown"));
        }
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_VERIFY(!dropdown->property("visible").toBool());
        auto *entry = findVisual(window->contentItem(), "entry-3-0");
        QVERIFY(entry);
        QTest::mouseClick(window, Qt::RightButton, Qt::NoModifier,
                          entry->mapToScene(QPointF(100, entry->height() / 2)).toPoint());
        auto *context = window->findChild<QObject *>("browserEntryMenu");
        QVERIFY(context);
        QTRY_VERIFY(context->property("visible").toBool());
        QCOMPARE(context->property("width").toDouble(), 240.0);
        auto *properties = window->findChild<QQuickItem *>("contextProperties");
        QVERIFY(properties);
        QCOMPARE(properties->property("font").value<QFont>().pixelSize(), 13);
        if (!captures.isEmpty())
            QVERIFY(visual_audit::save(window, captures, "main-context"));
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_VERIFY(!context->property("visible").toBool());
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
        QCOMPARE(controller->total(), 4);
        QCOMPARE(controller->entries()
                     ->data(controller->entries()->index(1, 0), Qt::DisplayRole)
                     .toString(),
                 QString::fromUtf8("notes.txt"));
        QVERIFY(QTest::qWaitForWindowExposed(window));
        QQuickItem *cell = nullptr;
        QTRY_VERIFY((cell = findVisual(window->contentItem(), "entry-1-0")));
        const auto point = cell->mapToScene(QPointF(120, 24)).toPoint();
        QTest::mouseClick(window, Qt::LeftButton, Qt::NoModifier, point);
        QTRY_COMPARE(controller->selectedCount(), 1);
        QCOMPARE(controller->detail().value("id").toString(), "3");
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
        window->resize(1024, 622);
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
        window->resize(832, 544);
        QTRY_VERIFY(!inspector->isVisible());
        QVERIFY(QMetaObject::invokeMethod(toggle, "clicked"));
        auto *detailsWindow = window->findChild<QQuickWindow *>("inspectorWindow");
        QVERIFY(detailsWindow);
        QTRY_VERIFY(detailsWindow->isVisible());
        QCOMPARE(detailsWindow->transientParent(), window);
        QCOMPARE(detailsWindow->modality(), Qt::NonModal);
        QVERIFY(!detailsWindow->flags().testFlag(Qt::FramelessWindowHint));
        QCOMPARE(window->width(), 832);
        QCOMPARE(controller->selectedCount(), 1);
        if (!qEnvironmentVariableIsEmpty("XDZIP_VISUAL_AUDIT"))
            QVERIFY(visual_audit::save(detailsWindow, qEnvironmentVariable("XDZIP_VISUAL_AUDIT"),
                                       "inspector-independent"));
        detailsWindow->close();
    }
};
QTEST_MAIN(BrowserTest)
#include "browser_test.moc"
