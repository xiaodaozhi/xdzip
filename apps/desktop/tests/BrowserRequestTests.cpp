#include "BrowserRequestTests.h"

#include "BrowserController.h"
#include "RuntimeConnection.h"
#include <QFile>
#include <QSignalSpy>
#include <QTest>
#include <thread>

void BrowserRequestTests::pagedModelKeepsLogicalRowsAndSelectionWithinItsCacheBudget() {
    ArchiveTableModel model;
    auto page = [](int first, int count) {
        QList<BrowserRow> rows;
        for (int i = 0; i < count; ++i) {
            BrowserRow row;
            row.id = QString::number(first + i + 1);
            row.cells = {QStringLiteral("item-%1.txt").arg(first + i), "0", "0", "TXT", ""};
            rows.append(row);
        }
        return rows;
    };
    model.replacePaged(1000000, page(0, 128), "12.3.1");
    QCOMPARE(model.rowCount(), 1000000);
    QSignalSpy missing(&model, &ArchiveTableModel::pageNeeded);
    QVERIFY(!model.data(model.index(999999, 0), Qt::DisplayRole).isValid());
    QVERIFY(!missing.isEmpty());
    model.installPage(999936, page(999936, 64));
    QCOMPARE(model.data(model.index(999999, 0), Qt::DisplayRole).toString(), "item-999999.txt");
    model.selectOnly(999999);
    QCOMPARE(model.selectedCount(), 1);
    QCOMPARE(model.physicalSelectedIds(), QStringList{"1000000"});
    model.selectAll(true);
    QCOMPARE(model.selectedCount(), 1000000);
    model.select(999999);
    QCOMPARE(model.selectedCount(), 999999);
    QCOMPARE(model.selectedIds(), QStringList{"q12.3.1:0:999998"});
    for (int i = 1; i < 12; ++i)
        model.installPage(i * 128, page(i * 128, 128));
    QVERIFY(model.cachedCount() <= 1024);
    QCOMPARE(model.selectedCount(), 999999);
    model.replace({});
    QCOMPARE(model.rowCount(), 0);
    QCOMPARE(model.selectedCount(), 0);
    QVERIFY(model.selectedIds().isEmpty());
}

void BrowserRequestTests::realLargeArchiveFetchesDistantRowsAndReplacesItsQuery() {
    const auto fixture = qEnvironmentVariable("XDZIP_LARGE_INDEX_FIXTURE");
    if (fixture.isEmpty())
        QSKIP("Owned large-index fixture was not requested");
    QVERIFY(QFile::exists(fixture));
    BrowserController controller;
    controller.openPath(fixture);
    QTRY_VERIFY_WITH_TIMEOUT(controller.ready() && !controller.busy(), 15000);
    QCOMPARE(controller.archiveTotal(), 1000000);
    auto *model = qobject_cast<ArchiveTableModel *>(controller.entries());
    QVERIFY(model && model->paged());
    QVERIFY(model->cachedCount() <= 1024);
    const int finalRow = model->rowCount() - 1;
    model->data(model->index(finalRow, 0), Qt::DisplayRole);
    QTRY_VERIFY_WITH_TIMEOUT(!controller.busy() && model->rowAt(finalRow), 10000);
    const int last = model->rowAt(finalRow)->directory ? finalRow - 1 : finalRow;
    QVERIFY(model->rowAt(last));
    QVERIFY(model->cachedCount() <= 1024);
    QCOMPARE(model->rowAt(last)->id, "1000000");
    for (int offset = 128; offset <= 1024; offset += 128) {
        model->data(model->index(offset, 0), Qt::DisplayRole);
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy() && model->rowAt(offset), 10000);
    }
    QVERIFY(!model->rowAt(finalRow));
    controller.selectAll(true);     // Starts an uncached detail lookup for the last row.
    controller.selectOnlyRow(1024); // A newer user intent must win that race.
    const auto selectedId = model->rowAt(1024)->id;
    QTRY_VERIFY_WITH_TIMEOUT(!controller.busy() && model->rowAt(finalRow), 10000);
    QCOMPARE(controller.detail().value("id").toString(), selectedId);
    model->data(model->index(last, 0), Qt::DisplayRole);
    QTRY_VERIFY_WITH_TIMEOUT(!controller.busy() && model->rowAt(last), 10000);
    controller.selectOnlyRow(last);
    QCOMPARE(controller.selectedCount(), 1);
    QVERIFY(controller.canPreflight());
    QSignalSpy extraction(&controller, &BrowserController::extractionReady);
    controller.prepareExtraction(false);
    QTRY_VERIFY(!controller.busy());
    QCOMPARE(extraction.count(), 1);
    QCOMPARE(controller.m_extractionSelectors, QStringList{"1000000"});
    // A query smaller than the model cache can still span several transport batches.
    // Its continuation must never emit the idle state of a viewport page fetch.
    const auto previousQuery = controller.committedQueryId();
    bool reportedIdleBeforeCommit = false;
    const auto observation = connect(&controller, &BrowserController::changed, this, [&] {
        if (controller.committedQueryId() == previousQuery && !controller.browsingBusy())
            reportedIdleBeforeCommit = true;
    });
    controller.search("item-000");
    QTRY_VERIFY_WITH_TIMEOUT(!controller.busy() && controller.total() == 990, 10000);
    disconnect(observation);
    QVERIFY(!reportedIdleBeforeCommit);
    QVERIFY(!model->paged());
    QCOMPARE(model->rowCount(), 990);
    QCOMPARE(controller.selectedCount(), 0);
    const auto searchQuery = controller.committedQueryId();
    controller.sortByColumn(1);
    QTRY_VERIFY_WITH_TIMEOUT(!controller.busy() && controller.committedQueryId() > searchQuery,
                             10000);
    QCOMPARE(model->rowCount(), 990);
    controller.search(QString());
    QTRY_VERIFY_WITH_TIMEOUT(!controller.busy() && model->paged(), 10000);
    controller.search("099999");
    QTRY_VERIFY_WITH_TIMEOUT(!controller.busy() && controller.total() == 1, 10000);
    QCOMPARE(model->rowCount(), 1);
    QCOMPARE(controller.selectedCount(), 0);
    QCOMPARE(controller.m_extractionSelectors, QStringList{"1000000"});
}

void BrowserRequestTests::taskAdmissionCountsOriginatingRequestOnce_data() {
    QTest::addColumn<int>("kind");
    QTest::addColumn<int>("ordering");
    for (int kind = 0; kind < 3; ++kind)
        for (int ordering = 0; ordering < 4; ++ordering)
            QTest::newRow(qPrintable(QString("kind-%1-order-%2").arg(kind).arg(ordering)))
                << kind << ordering;
}

void BrowserRequestTests::taskAdmissionCountsOriginatingRequestOnce() {
    QFETCH(int, kind);
    QFETCH(int, ordering);
    BrowserController controller;
    controller.m_pending = 10001;
    controller.m_busy = true;
    controller.m_result = {};
    using RequestKind = BrowserController::RequestKind;
    controller.m_result.kind = kind == 0   ? RequestKind::Extraction
                               : kind == 1 ? RequestKind::Creation
                                           : RequestKind::ArchiveCheck;
    controller.m_result.taskSession = kind == 1 ? 0 : 23;
    const auto session = controller.m_result.taskSession;
    QCOMPARE(controller.activeTaskCount(), 1);
    QSignalSpy started(&controller, kind == 0   ? &BrowserController::extractionStarted
                                    : kind == 1 ? &BrowserController::creationStarted
                                                : &BrowserController::archiveTestStarted);
    xdzip::reader::Event event{};
    event.kind = xdzip::reader::EventKind::TaskUpdate;
    event.request_id = 10000;
    event.task.id = 7;
    event.task.session = session;
    event.task.status = rust::String("running");
    controller.receive(event);
    QCOMPARE(controller.activeTaskCount(), 2);
    QVERIFY(controller.m_result.taskId.isEmpty());
    event.request_id = 10001;
    event.task.id = 8;
    event.task.session = session + 1;
    controller.receive(event);
    QCOMPARE(controller.activeTaskCount(), 3);
    QVERIFY(controller.m_result.taskId.isEmpty());
    // An uncorrelated snapshot cannot consume a pending request's placeholder.
    event.request_id = 0;
    event.task.id = 10;
    event.task.session = session;
    controller.receive(event);
    QCOMPARE(controller.activeTaskCount(), 4);
    QVERIFY(controller.m_result.taskId.isEmpty());
    const auto accept = [&] {
        xdzip::reader::Event accepted{};
        accepted.kind = xdzip::reader::EventKind::TaskAccepted;
        accepted.request_id = 10001;
        accepted.task.id = 9;
        controller.receive(accepted);
    };
    if (ordering == 0)
        accept();
    event.request_id = 10001;
    event.task.id = 9;
    event.task.status = rust::String("waiting_password");
    event.task.password_prompt_id = 4;
    controller.receive(event);
    QCOMPARE(controller.activeTaskCount(), 4);
    QCOMPARE(controller.tasks().size(), 4);
    QCOMPARE(controller.m_result.observedTask, quint64(9));
    QCOMPARE(controller.m_result.taskId, ordering == 0 ? QStringLiteral("9") : QString{});
    QCOMPARE(controller.passwordPrompt().value("taskId").toString(), QStringLiteral("9"));
    QVERIFY(controller.taskStarting());
    QVERIFY(started.isEmpty());
    QCOMPARE(controller.m_hasPayload, ordering == 0);
    for (const auto *status : {"running", "waiting", "cancelling", "cleaning"}) {
        event.task.status = rust::String(status);
        event.task.password_prompt_id = 0;
        controller.receive(event);
        QCOMPARE(controller.activeTaskCount(), 4);
    }
    if (ordering == 2) {
        event.task.status = rust::String("cancelled");
        controller.receive(event);
        QCOMPARE(controller.activeTaskCount(), 3);
    }
    if (ordering == 1 || ordering == 2)
        accept();
    QCOMPARE(controller.activeTaskCount(), ordering == 2 ? 3 : 4);
    event.kind = xdzip::reader::EventKind::Finished;
    if (ordering == 3)
        event.error = rust::String("cancelled");
    controller.receive(event);
    QVERIFY(!controller.taskStarting());
    QCOMPARE(started.count(), ordering == 3 ? 0 : 1);
    if (ordering != 3)
        QCOMPARE(started.first().first().toString(), QStringLiteral("9"));
    QCOMPARE(controller.activeTaskCount(), ordering == 2 ? 3 : 4);
    event.kind = xdzip::reader::EventKind::TaskUpdate;
    event.task.status = rust::String("cancelled");
    controller.receive(event);
    QCOMPARE(controller.activeTaskCount(), 3);
    for (const quint64 id : {7, 8, 10}) {
        event.task.id = id;
        controller.receive(event);
    }
    QCOMPARE(controller.activeTaskCount(), 0);
    controller.requestClose();
    QTRY_VERIFY(controller.requestClose());
}

void BrowserRequestTests::unknownSizesRemainDistinctFromEmptyFilesAndNavigationSelection() {
    BrowserController controller;
    controller.m_pending = 10001;
    xdzip::reader::Event page{};
    page.kind = xdzip::reader::EventKind::Page;
    page.request_id = 10001;
    page.query = page.revision = page.session = 1;
    page.total = page.archive_total = 4;
    page.format = rust::String("RAR");
    for (int i = 0; i < 4; ++i) {
        xdzip::reader::Row row{};
        row.id = rust::String(i == 3 ? "d1" : std::to_string(i + 1));
        row.name = rust::String(i == 3 ? "folder/" : "file" + std::to_string(i + 1));
        row.size = i == 2 ? 7 : 0;
        row.size_known = i == 1 || i == 2;
        if (i == 1)
            row.modified_dos =
                ((2026 - 1980) << 25) | (10 << 21) | (15 << 16) | (12 << 11) | (34 << 5) | (56 / 2);
        row.physical = i != 3;
        row.directory = i == 3;
        page.rows.push_back(std::move(row));
    }
    controller.m_result.kind = BrowserController::RequestKind::Preflight;
    controller.receive(page);
    QCOMPARE(controller.m_result.error, QStringLiteral("stale_query"));
    QVERIFY(controller.m_result.rows.isEmpty());
    controller.m_result = {};
    controller.m_result.kind = BrowserController::RequestKind::Query;
    controller.receive(page);
    QVERIFY(controller.m_result.error.isEmpty());
    QCOMPARE(controller.m_result.rows.size(), 4);
    controller.m_model.replace(std::move(controller.m_result.rows));
    controller.m_pending = 0;
    QVERIFY(controller.m_model.rowAt(0)->cells[1].isEmpty());
    QVERIFY(controller.m_model.rowAt(0)->status.contains(qtTrId("archive.unknownSize")));
    QVERIFY(!controller.m_model.rowAt(3)->status.contains(qtTrId("archive.unknownSize")));
    QCOMPARE(controller.m_model.rowAt(1)->cells[1], QStringLiteral("0"));
    controller.m_model.select(1);
    QCOMPARE(controller.selectedBytes(), QStringLiteral("0"));
    controller.showRowDetail(1);
    QCOMPARE(controller.detail().value("modified").toString(),
             QStringLiteral("2026/10/15 12:34:56"));
    QCOMPARE(controller.m_model.rowAt(1)->cells[4], QStringLiteral("2026/10/15"));
    QVERIFY(controller.canPreflight());
    controller.m_model.select(0);
    QVERIFY(controller.selectedBytes().isEmpty());
    controller.showRowDetail(0);
    QVERIFY(controller.detail().value("modified").toString().isEmpty());
    QVERIFY(controller.m_model.rowAt(0)->cells[4].isEmpty());
    QVERIFY(controller.canPreflight());
    // A later navigation row must still disable physical preflight after an unknown file.
    controller.m_model.select(3);
    QVERIFY(!controller.canPreflight());
    controller.m_model.select(0);
    controller.m_model.select(3);
    controller.m_model.select(2);
    QCOMPARE(controller.selectedBytes(), QStringLiteral("7"));
    QVERIFY(controller.canPreflight());
}

void BrowserRequestTests::
    encryptedOpenAmbiguityReusesThePasswordPromptWithoutClaimingVerifierFailure() {
    BrowserController controller;
    quint64 previous = 0;
    for (const auto *code : {"password_required", "wrong_password", "password_or_integrity"}) {
        controller.m_pending = 10001;
        controller.m_busy = true;
        controller.m_title = QStringLiteral("owned-encrypted-header.7z");
        controller.m_result = {};
        controller.m_result.kind = BrowserController::RequestKind::Open;
        xdzip::reader::Event event{};
        event.kind = xdzip::reader::EventKind::Finished;
        event.request_id = 10001;
        event.error = rust::String(code);
        controller.receive(event);
        const auto prompt = controller.passwordPrompt();
        QCOMPARE(prompt.value("kind").toString(), QStringLiteral("open"));
        QCOMPARE(prompt.value("retry").toBool(), QString::fromLatin1(code) != "password_required");
        const auto token = prompt.value("promptId").toString().toULongLong();
        QVERIFY(token > previous);
        previous = token;
        QVERIFY(!controller.submitPassword("open", QString::number(token + 1), "ignored"));
        controller.m_openPasswordPrompt.clear();
    }
    QCOMPARE(BrowserController::explain("password_or_integrity"), qtTrId("password.ambiguous"));
}

void BrowserRequestTests::mailboxBackpressureShutdownAndIdle() {
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
    QVERIFY(connection.statistics().peakBytes <= xdzip::reader::mailbox_byte_limit());
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
