// apps/desktop/ui/SessionPage.qml

pragma ComponentBehavior: Bound
import QtQuick
import Xdzip.Settings 1.0
import QtQuick.Controls
import QtQuick.Layouts
import QtQml.Models
import Xdzip.Native

Item {
    id: root
    required property var windowHost
    readonly property var sessionManager: windowHost.sessionManager
    property string sessionIdentity: ""
    property bool retainShellPage: false
    property bool closedPage: false
    property bool sessionClosed: false
    readonly property bool archiveInternalDragging: moveDrag.Drag.active
    readonly property bool archiveTransferActive: moveDrag.Drag.active || moveDrag.promoting || reader.nativeDragActive
    readonly property var contentItem: root
    readonly property int visibility: windowHost.visibility
    readonly property bool windowsTitlebar: windowHost.windowsTitlebar
    readonly property bool integratedTitlebar: windowHost.integratedTitlebar
    width: windowHost.contentItem.width
    height: windowHost.contentItem.height
    HoverHandler { enabled: moveDrag.Drag.active; cursorShape: Qt.ForbiddenCursor }
    Shortcut { sequence: "Escape"; enabled: moveDrag.Drag.active; onActivated: root.cancelTransfer() }
    // Keep a gesture internal until it leaves this window. Crossing the window
    // hands the same frozen selection to the existing native copy/export path.
    Item {
        id: moveDrag
        objectName: "archiveMoveDrag"
        width: 1; height: 1
        property string token: ""
        property string message: ""
        property string targetId: ""
        property var feedbackOwner: null
        property bool permitted: false
        property bool promoting: false
        property var preview: ({})
        Drag.source: moveDrag
        Drag.keys: ["xdzip-internal-move"]
        Drag.supportedActions: Qt.MoveAction
        Drag.proposedAction: Qt.MoveAction
        function feedback(owner, target, state) {
            feedbackOwner = owner; targetId = target;
            message = state.message; permitted = state.permitted;
        }
        function clearFeedback(owner) {
            if (feedbackOwner === owner) { feedbackOwner = null; targetId = ""; message = ""; permitted = false; }
        }
    }
    ArchiveDragPreview {
        objectName: "archiveMoveFeedback"
        z: 1000
        visible: moveDrag.Drag.active
        fileName: moveDrag.preview.name || ""
        directory: moveDrag.preview.directory || false
        itemCount: moveDrag.preview.count || 0
        message: moveDrag.message
        permitted: moveDrag.permitted
        availableWidth: root.width - Theme.mainGridInset * 2
        x: Math.max(Theme.mainGridInset, Math.min(moveDrag.x + Theme.dragPreviewPointerOffset, root.width - width - Theme.mainGridInset))
        y: Math.max(Theme.mainGridInset, Math.min(moveDrag.y + Theme.dragPreviewPointerOffset, root.height - height - Theme.mainGridInset))
    }
    function beginTransfer(item, row, mouse) {
        const point = item.mapToItem(root, mouse.x, mouse.y);
        moveDrag.token = reader.beginInternalDrag(row, root.windowHost);
        if (!moveDrag.token.length) return;
        moveDrag.preview = reader.internalDragPreview(moveDrag.token);
        moveDrag.x = point.x; moveDrag.y = point.y;
        moveDrag.Drag.start(Qt.MoveAction);
    }
    function continueTransfer(item, mouse) {
        if (!moveDrag.Drag.active) return;
        const point = item.mapToItem(root, mouse.x, mouse.y);
        moveDrag.x = point.x; moveDrag.y = point.y;
        if (point.x < 0 || point.y < 0 || point.x >= root.width || point.y >= root.height || !root.windowHost.active) {
            root.promoteTransfer();
        }
    }
    Connections {
        target: root.windowHost
        function onActiveChanged() {
            if (!root.windowHost.active && moveDrag.Drag.active) { if (root.visible && !root.sessionClosed) root.promoteTransfer(); else root.cancelTransfer(); }
        }
    }
    function promoteTransfer() {
        if (!moveDrag.Drag.active) return;
        const token = moveDrag.token;
        moveDrag.promoting = true;
        moveDrag.Drag.cancel();
        moveDrag.clearFeedback(moveDrag.feedbackOwner);
        reader.exportInternalDrag(token, root.windowHost);
        moveDrag.token = "";
        moveDrag.preview = ({});
        moveDrag.promoting = false;
    }
    function deferCancelledTransfer() {
        Qt.callLater(function () {
            if (!moveDrag.Drag.active) return;
            if (!root.windowHost.active && root.visible && !root.sessionClosed) root.promoteTransfer();
            else root.cancelTransfer();
        });
    }
    function finishTransfer() {
        if (moveDrag.Drag.active) moveDrag.Drag.drop();
        cancelTransfer();
    }
    function cancelTransfer() {
        if (moveDrag.promoting) return;
        moveDrag.Drag.cancel();
        reader.cancelInternalDrag(moveDrag.token);
        moveDrag.token = "";
        moveDrag.preview = ({});
        moveDrag.clearFeedback(moveDrag.feedbackOwner);
    }
    function close() {
        windowHost.close();
    }
    function raise() {
        windowHost.raise();
    }
    function requestActivate() {
        windowHost.requestActivate();
    }
    function startSystemMove() {
        windowHost.startSystemMove();
    }
    function showMinimized() {
        windowHost.showMinimized();
    }
    function toggleMaximized() {
        windowHost.toggleMaximized();
    }
    function hideSessionWindows() {
        createWindow.close();
        updateWindow.close();
        extractWindow.close();
        taskWindow.close();
        preflightWindow.close();
        inspectorWindow.close();
        conflictWindow.hide();
        passwordWindow.clearSecret();
        passwordWindow.hide();
    }
    function closeWindowDialogs() {
        hideSessionWindows();
        if (aboutWindow.visible) aboutWindow.close();
        if (settingsWindow.visible) settingsWindow.close();
        if (restartWindow.visible) restartWindow.close();
        if (quitWindow.visible) quitWindow.close();
        taskMenu.close();
        entryMenu.close();
        directoryMenu.close();
        emptyMenu.close();
        sortMenu.close();
    }
    function presentQuit() {
        waitBeforeExit = false;
        quitWindow.present();
    }
    function presentAbout() {
        aboutWindow.present();
    }
    readonly property var globalTasks: sessionManager ? (visible ? sessionManager.tasks : []) : reader.tasks
    readonly property var globalTaskModel: sessionManager ? (visible ? sessionManager.taskModel : null) : reader.taskModel
    readonly property int globalActiveTaskCount: sessionManager ? sessionManager.activeTaskCount : reader.activeTaskCount
    onVisibleChanged: if (!visible) {
        if (taskMenu) taskMenu.close();
        if (moveDrag && moveDrag.Drag.active) root.cancelTransfer();
    }
    readonly property var controller: reader
    readonly property color accentTextColor: Theme.accentText
    property bool uiReady: false
    Component.onCompleted: {
        uiReady = true;
        Theme.dark = AppSettings.dark;
        Theme.fontScale = AppSettings.textScale;
    }
    readonly property bool archiveOperationVisible: reader.archiveOperation.length > 0
    readonly property string archiveOperationTitle: reader.archiveOperation === "move" ? qsTrId("archive.operation.moving") : qsTrId("archive.operation.adding")
    readonly property bool welcomeVisible: closedPage || (!reader.ready && !archiveOperationVisible)
    property bool externalFileOpening: false
    property bool welcomeWasBusy: false
    onWelcomeVisibleChanged: if (uiReady && welcomeVisible) {
        inspectorWindow.close();
        Qt.callLater(() => {
            if (root.welcomeVisible && welcomePage.canOpen)
                welcomePage.focusOpen();
        });
    }
    property bool inspectorRequested: true
    readonly property bool inspectorVisible: inspectorRequested && width >= Theme.inspectorCollapseWidth
    property string searchSession: ""
    property bool restartAfterClose: false
    property bool darkMode: AppSettings.dark
    property string activeTaskId: ""
    property var taskSnapshot: ({})
    property bool waitBeforeExit: false
    property bool allowExit: false
    readonly property var desktopShell: windowHost.desktopShell
    property bool desktopExitRequested: false
    readonly property bool desktopInspectorVisible: root.inspectorVisible || inspectorWindow.visible
    readonly property bool nativeMenuAvailable: desktopShell ? desktopShell.nativeMenuAvailable : false
    function requestDesktopExit() {
        if (sessionManager) {
            sessionManager.requestQuit();
            return;
        }
        desktopExitRequested = true;
        root.close();
    }
    function dispatchDesktopCommand(command) {
        if (desktopShell)
            desktopShell.execute(command);
        else
            runDesktopCommand(command);
    }
    function runDesktopCommand(command) {
        const ready = !closedPage && reader.ready && !reader.browsingBusy;
        switch (command) {
        case "open":
            if (reader.runtimeAvailable && !reader.browsingBusy)
                reader.chooseArchive(windowHost);
            break;
        case "new":
            if (reader.runtimeAvailable && !reader.browsingBusy)
                createWindow.configure();
            break;
        case "testArchive":
            if (ready && !reader.taskStarting)
                reader.startArchiveTest();
            break;
        case "settings":
            settingsWindow.configure();
            break;
        case "about":
            aboutWindow.present();
            break;
        case "closeArchive":
            if (sessionManager)
                sessionManager.closeSession(sessionIdentity);
            else if (ready)
                reader.closeArchive();
            break;
        case "copy":
            if (ready && reader.selectedCount > 0) {
                entryMenu.close();
                reader.copySelection(false);
            }
            break;
        case "selectAll":
            if (ready)
                reader.selectAll(true);
            break;
        case "find":
            if (reader.browserQueryAvailable)
                toolbar.focusSearch();
            break;
        case "list":
            root.viewMode = "list";
            break;
        case "grid":
            root.viewMode = "grid";
            break;
        case "inspector":
            root.toggleInspector();
            break;
        case "extractAll":
            if (ready)
                reader.prepareExtraction(true);
            break;
        case "extractSelected":
            if (ready && reader.selectedCount > 0)
                reader.prepareExtraction(false);
            break;
        case "preflight":
            if (ready && reader.canPreflight)
                reader.inspectSelection();
            break;
        case "minimize":
            root.showMinimized();
            break;
        case "zoom":
            root.toggleMaximized();
            break;
        }
    }
    function phaseLabel(status: string): string {
        return ({
                queued: qsTrId("ui.03f07c12d5fa"),
                preparing: qsTrId("ui.a1b9ba49c8a2"),
                running: taskSnapshot.creating ? qsTrId("ui.f066ff44699e") : qsTrId("ui.8865f5103610"),
                verifying: qsTrId("ui.b87ef7da7959"),
                waiting: qsTrId("ui.9a6cfa6abc53"),
                waiting_password: qsTrId("password.waiting"),
                committing: qsTrId("ui.f5312f1e339a"),
                cancelling: qsTrId("ui.684398ae6c51"),
                cleaning: qsTrId("ui.b4966391542e"),
                succeeded: qsTrId("ui.c0b3fbff51cc"),
                failed: qsTrId("ui.28384d7afd2e"),
                cancelled: qsTrId("ui.a37778f17c5f")
            })[status] || qsTrId("ui.b53be09930f4");
    }
    function taskStateLabel(status: string): string {
        return ({
                queued: qsTrId("ui.d6f766f2adf5"),
                preparing: qsTrId("ui.b53be09930f4"),
                running: qsTrId("ui.dc9591e56d50"),
                verifying: qsTrId("ui.784f9b54ae2b"),
                waiting: qsTrId("ui.8fb3a3261b43"),
                waiting_password: qsTrId("password.waiting"),
                committing: qsTrId("ui.a1a1720d3eb5"),
                cancelling: qsTrId("ui.72eea71385de"),
                cleaning: qsTrId("ui.718b6372d7fb"),
                succeeded: qsTrId("ui.f28461bb49c8"),
                failed: qsTrId("ui.6707de42c29d"),
                cancelled: qsTrId("ui.a37778f17c5f")
            })[status] || qsTrId("ui.b53be09930f4");
    }
    function taskStatus(task: var): string {
        return task.cancelRequested && task.status !== "cleaning" ? "cancelling" : task.status || "queued";
    }
    function taskLabel(task: var): string {
        const label = task.kind === "check" ? qsTrId("tasks.check")
            : task.kind === "move" ? qsTrId("move.task")
            : task.kind === "update" ? qsTrId("tasks.add")
            : ["export", "export-copy"].includes(task.kind) ? qsTrId("tasks.export")
            : task.creating ? qsTrId("ui.9a63e2cb4339") : qsTrId("ui.fb4b66729449");
        return label.arg(task.backendId || task.id);
    }
    function showTask(id: string) {
        const task = reader.tasks.find(t => t.id === id);
        if (!task) return;
        if (id !== activeTaskId) conflictWindow.close();
        activeTaskId = id;
        taskSnapshot = task;
        taskWindow.present();
        if (reader.runtimeAvailable && !taskSnapshot.cancelRequested && taskSnapshot.status === "waiting" && taskSnapshot.promptId)
            conflictWindow.presentPrompt(true);
        if (taskSnapshot.status === "waiting_password" && taskSnapshot.passwordPromptId)
            passwordWindow.revealPrompt();
    }
    function taskFraction(): real {
        const t = taskSnapshot;
        if (t.status === "succeeded")
            return 1;
        const bytes = Number(t.declaredBytes || 0);
        const files = Number(t.files || 0);
        if (t.kind === "check")
            return Math.min(.99, bytes ? Number(t.verifiedBytes || 0) / bytes : files ? Number(t.verifiedFiles || 0) / files : 0);
        return Math.min(.99, bytes ? Number(t.decodedBytes || 0) / bytes : files ? (Number(t.committedFiles || 0) + Number(t.skippedFiles || 0)) / files : 0);
    }
    function averageSpeed(): string {
        const seconds = Number(taskSnapshot.elapsedMs || 0) / 1000;
        if (taskSnapshot.terminal && seconds < 2)
            return "—";
        const processed = Number(taskSnapshot.kind === "check" ? taskSnapshot.readBytes || 0 : taskSnapshot.writtenBytes || 0);
        return seconds >= 2 && processed > 0 ? formatSize(String(Math.round(processed / seconds))) + "/s" : qsTrId("ui.f4199761edc9");
    }
    function remainingTime(): string {
        const t = taskSnapshot;
        const seconds = Number(t.elapsedMs || 0) / 1000;
        if (t.kind === "check" && !t.totalKnown)
            return "—";
        if (t.kind === "check") {
            const verified = Number(t.verifiedBytes || 0);
            return seconds >= 2 && verified > 0 && t.status === "running" ? qsTrId("ui.d6c40aaea130").arg(Math.ceil(Math.max(0, Number(t.declaredBytes || 0) - verified) / (verified / seconds))) : "—";
        }
        const written = Number(t.creating ? t.decodedBytes || 0 : t.writtenBytes || 0);
        if (seconds < 2 || !written || t.status !== "running")
            return "—";
        return qsTrId("ui.d6c40aaea130").arg(Math.ceil(Math.max(0, Number(t.declaredBytes || 0) - Number(t.decodedBytes || 0)) / (written / seconds)));
    }
    property string viewMode: "list"
    property var contextEntry: ({})
    property string contextDirectoryId: ""
    property int contextDirectoryRow: -1
    property bool contextDirectoryHasChildren: false
    property bool contextDirectoryExpanded: false
    property bool pendingDirectorySearchFocus: false
    readonly property bool contextAvailable: reader.ready && !reader.browsingBusy && contextEntry.session === reader.sessionId
    // File and directory icons share one logical slot; artwork retains its aspect ratio.
    readonly property real entryIconSize: Theme.mainFileIconSize
    function toggleInspector() {
        if (root.width < Theme.inspectorCollapseWidth) {
            inspectorWindow.show();
            inspectorWindow.raise();
            inspectorWindow.requestActivate();
        } else {
            inspectorWindow.close();
            extractWindow.close();
            taskWindow.close();
            conflictWindow.close();
            quitWindow.close();
            root.inspectorRequested = !root.inspectorVisible;
        }
    }
    function selectWithModifiers(row, modifiers) {
        if (modifiers & Qt.ShiftModifier)
            reader.selectRangeTo(row);
        else if (modifiers & (Qt.ControlModifier | Qt.MetaModifier))
            reader.selectRow(row);
        else
            reader.selectOnlyRow(row);
    }
    function entryMenuAt(item, row, entryId, directory, navigable, x, y) {
        if (!reader.ready || reader.browsingBusy)
            return;
        root.contextEntry = {
            row: row,
            id: entryId,
            directory: directory,
            navigable: navigable,
            session: reader.sessionId
        };
        reader.selectForContext(row);
        item.forceActiveFocus(Qt.MouseFocusReason);
        const point = item.mapToItem(root.contentItem, x, y);
        entryMenu.popup(point.x, point.y);
    }
    onDarkModeChanged: Theme.dark = darkMode
    visible: true
    function handleClosing(close) {
        root.desktopExitRequested = true;
        if (reader.activeTaskCount > 0 && !root.allowExit) {
            close.accepted = false;
            quitWindow.present();
            return;
        }
        close.accepted = reader.requestClose();
        if (close.accepted) {
            preflightWindow.close();
            inspectorWindow.close();
        }
    }

    function fileIcon(name: string, directory: bool): string {
        return AssetLibrary.fileIcon(name, directory, false);
    }
    // Human-readable display only; exact integer strings remain in the model.
    function formatSize(value: string): string {
        if (!/^\d+$/.test(value))
            return "—";
        const size = Number(value);
        if (size < 1024)
            return value + " B";
        if (size < 1048576)
            return (size / 1024).toLocaleString(Qt.locale(AppSettings.activeLanguage), "f", 1) + " KiB";
        if (size < 1073741824)
            return (size / 1048576).toLocaleString(Qt.locale(AppSettings.activeLanguage), "f", 1) + " MiB";
        return (size / 1073741824).toLocaleString(Qt.locale(AppSettings.activeLanguage), "f", 1) + " GiB";
    }
    BrowserController {
        id: reader
        objectName: "browserController"
        onReadyToClose: if (!root.sessionManager)
            root.close()
        onArchiveOpened: path => AppSettings.rememberArchive(path)
        onPreflightReady: if (!root.sessionClosed)
            preflightWindow.present()
        onExtractionReady: if (!root.sessionClosed)
            extractWindow.configure()
        onAdditionRequested: paths => { if (!root.sessionClosed) updateWindow.configure(paths); }
        onUpdateStarted: id => { updateWindow.close(); if (!root.sessionClosed) root.showTask(id); }
        onCreationStarted: id => {
            createWindow.close();
            if (!root.sessionClosed)
                root.showTask(id);
        }
        onExtractionStarted: id => {
            extractWindow.close();
            const task = reader.tasks.find(t => t.id === id);
            if (!root.sessionClosed && !(task && task.openAccepted))
                root.showTask(id);
        }
        onArchiveTestStarted: id => {
            if (!root.sessionClosed)
                root.showTask(id);
        }
        onTasksChanged: {
            const previous = root.taskSnapshot;
            root.taskSnapshot = tasks.find(t => t.id === root.activeTaskId) || {};
            if (root.activeTaskId.length && !root.taskSnapshot.id) {
                taskWindow.close();
                root.activeTaskId = "";
            }
            if (root.taskSnapshot.openAccepted && (!previous.openAccepted || previous.id !== root.taskSnapshot.id))
                taskWindow.close();
            if (!root.sessionClosed && reader.runtimeAvailable && !root.taskSnapshot.cancelRequested && root.taskSnapshot.status === "waiting" && root.taskSnapshot.promptId && !conflictWindow.visible)
                conflictWindow.presentPrompt(false);
            if (!reader.runtimeAvailable || root.taskSnapshot.cancelRequested || root.taskSnapshot.status !== "waiting" || !root.taskSnapshot.promptId)
                conflictWindow.close();
            if (!root.sessionManager && root.waitBeforeExit && activeTaskCount === 0) {
                root.allowExit = true;
                root.close();
            }
        }
        onChanged: {
            if (moveDrag.Drag.active && !reader.internalDragCurrent(moveDrag.token)) Qt.callLater(function () { if (moveDrag.Drag.active && !reader.internalDragCurrent(moveDrag.token)) root.cancelTransfer(); });
            if (root.welcomeVisible && !busy && root.welcomeWasBusy)
                Qt.callLater(() => {
                    if (root.welcomeVisible && welcomePage.canOpen)
                        welcomePage.focusOpen();
                });
            root.welcomeWasBusy = root.welcomeVisible && busy && title.length > 0;
            if (browsingBusy || root.contextEntry.session !== sessionId) {
                entryMenu.close();
                directoryMenu.close();
            }
            if (!busy && root.pendingDirectorySearchFocus) {
                root.pendingDirectorySearchFocus = false;
                toolbar.focusSearch();
            }
            if (!preflightReport.entries && !preflightReport.error)
                preflightWindow.close();
            if (sessionId !== root.searchSession) {
                root.searchSession = sessionId;
                toolbar.clearSearch();
            }
        }
    }
    UpdateWindow { id: updateWindow; owner: root.windowHost; controller: reader }
    CreateWindow {
        id: createWindow
        owner: root.windowHost
        controller: reader
    }
    ExtractWindow {
        id: extractWindow
        owner: root.windowHost
        controller: reader
    }
    TaskWindow {
        id: taskWindow
        owner: root.windowHost
        archiveName: root.taskSnapshot.archive || reader.extractionIntent.archive || ""
        destination: taskWindow.checking ? root.taskSnapshot.source || "" : root.taskSnapshot.target || reader.extractionIntent.target || ""
        currentFile: root.taskSnapshot.current || qsTrId("ui.f07335163e36")
        fraction: root.taskFraction()
        creating: root.taskSnapshot.creating || false
        operationKind: root.taskSnapshot.kind || (creating ? "create" : "extract")
        totalKnown: root.taskSnapshot.totalKnown || false
        stopping: root.taskSnapshot.cancelRequested === true || ["cancelling", "cleaning"].includes(root.taskSnapshot.status)
        taskStatus: root.taskStatus(root.taskSnapshot)
        phaseSummary: taskWindow.checking && taskStatus === "running" ? qsTrId("integrity.running") : root.phaseLabel(taskStatus)
        errorSummary: root.taskSnapshot.error || ""
        cleanupWarning: root.taskSnapshot.cleanupWarning || false
        outcomeUncertain: root.taskSnapshot.outcomeUncertain || false
        processedSummary: taskWindow.checking ? qsTrId("integrity.processed").arg(root.formatSize(root.taskSnapshot.readBytes || "0")).arg(root.formatSize(root.taskSnapshot.verifiedBytes || "0")) : taskWindow.creating ? qsTrId("ui.a1c9c21cfbef").arg(root.formatSize(root.taskSnapshot.decodedBytes || "0")).arg(root.formatSize(root.taskSnapshot.writtenBytes || "0")) : qsTrId("ui.8bce13df4816").arg(root.formatSize(root.taskSnapshot.writtenBytes || "0")).arg(root.formatSize(root.taskSnapshot.committedBytes || "0"))
        fileCountSummary: taskWindow.checking ? qsTrId("integrity.files").arg(root.taskSnapshot.verifiedFiles || "0").arg(root.taskSnapshot.files || "0") : taskWindow.creating ? qsTrId("ui.1f43814c240b").arg(root.taskSnapshot.files || "0").arg(root.taskSnapshot.terminal ? qsTrId("ui.b61825c80ed6") : qsTrId("ui.483f8dd01321")) : qsTrId("ui.ed52971e9256").arg(root.taskSnapshot.committedFiles || "0").arg(root.taskSnapshot.files || "0").arg(root.taskSnapshot.skippedFiles || "0")
        resultSummary: taskWindow.checking ? qsTrId("integrity.result").arg(root.taskSnapshot.verifiedFiles || "0").arg(Math.max(0, Number(root.taskSnapshot.files || 0) - Number(root.taskSnapshot.verifiedFiles || 0))) : qsTrId("ui.51fe7c80c764").arg(root.taskSnapshot.committedFiles || "0").arg(root.taskSnapshot.skippedFiles || "0").arg(root.taskSnapshot.createdDirectories || "0")
        resultTask: root.taskSnapshot
        speedSummary: root.averageSpeed()
        remainingSummary: root.remainingTime()
        onCancelRequested: reader.cancelTask(root.activeTaskId)
        onOpenOutputRequested: reader.openOutput(root.activeTaskId)
        onConflictRequested: conflictWindow.presentPrompt(true)
        onPasswordRequested: passwordWindow.revealPrompt()
    }
    ConflictWindow {
        id: conflictWindow
        owner: root.windowHost
        task: root.taskSnapshot
        decisionsAvailable: reader.runtimeAvailable
        sizeFormatter: root.formatSize
        onDecision: (taskId, promptId, policy, all) => {
            if (reader.decideTask(taskId, promptId, policy, all))
                conflictWindow.close();
        }
    }
    PasswordWindow {
        id: passwordWindow
        owner: root.windowHost
        controller: reader
        automaticPresentation: !root.sessionClosed
        presentationBlocked: root.sessionManager ? root.sessionManager.quitDecisionPending : quitWindow.visible
    }
    QuitWindow {
        id: quitWindow
        owner: root.windowHost
        onClosing: if (!stopping && !root.waitBeforeExit) {
            if (root.sessionManager)
                root.sessionManager.cancelQuit();
            root.restartAfterClose = false;
            root.desktopExitRequested = false;
        }
        onWaitRequested: {
            root.waitBeforeExit = true;
            if (root.sessionManager)
                root.sessionManager.confirmQuit(true);
            close();
        }
        onCancelAndQuitRequested: {
            stopping = true;
            if (root.sessionManager)
                root.sessionManager.confirmQuit(false);
            else {
                root.allowExit = true;
                reader.requestClose();
            }
        }
    }
    PreflightWindow {
        id: preflightWindow
        owner: root.windowHost
        report: reader.preflightReport
    }
    Window {
        id: inspectorWindow
        objectName: "inspectorWindow"
        transientParent: root.windowHost
        title: qsTrId("ui.801948f7646f")
        width: 360
        height: 600
        minimumWidth: 320
        minimumHeight: 400
        modality: Qt.NonModal
        color: Theme.inspector
        Shortcut {
            sequence: "Escape"
            enabled: inspectorWindow.visible && inspectorWindow.active
            onActivated: inspectorWindow.close()
        }
        InspectorPanel {
            showCloseAction: false
            anchors.fill: parent
            detail: reader.detail
            selectionCount: reader.selectedCount
            sizeFormatter: root.formatSize
            iconForFile: root.fileIcon
            canExtract: reader.ready && !reader.browsingBusy
            onExtractRequested: id => reader.prepareExtraction(false, [id])
            onCloseRequested: inspectorWindow.close()
        }
    }
    Connections {
        target: createWindow
        function onVisibleChanged() {
            if (!createWindow.visible && root.welcomeVisible)
                welcomePage.focusCreate();
        }
    }
    AboutWindow {
        id: aboutWindow
        owner: settingsWindow.visible ? settingsWindow : root.windowHost
    }
    SettingsWindow {
        id: settingsWindow
        owner: root.windowHost
        onAboutRequested: root.sessionManager ? root.sessionManager.showAboutFor(root) : root.presentAbout()
        onRestartConfirmationRequested: restartWindow.present()
    }
    RestartWindow {
        id: restartWindow
        owner: root.windowHost
        onRestartRequested: {
            close();
            if (root.sessionManager)
                root.sessionManager.requestRestart();
            else {
                root.restartAfterClose = true;
                root.requestDesktopExit();
            }
        }
    }
    Connections {
        target: AppSettings
        function onAppearanceChanged() {
            root.darkMode = AppSettings.dark;
            Theme.fontScale = AppSettings.textScale;
        }
    }
    Shortcut {
        sequence: StandardKey.Preferences
        enabled: root.visible && !root.nativeMenuAvailable
        onActivated: root.dispatchDesktopCommand("settings")
    }
    Shortcut {
        sequence: StandardKey.Open
        enabled: root.visible && !root.nativeMenuAvailable && reader.runtimeAvailable && !reader.browsingBusy
        onActivated: root.dispatchDesktopCommand("open")
    }
    Shortcut {
        sequence: StandardKey.New
        enabled: root.visible && !root.nativeMenuAvailable && reader.runtimeAvailable && !reader.browsingBusy
        onActivated: root.dispatchDesktopCommand("new")
    }
    Shortcut {
        sequence: StandardKey.Find
        enabled: root.visible && !root.nativeMenuAvailable && reader.browserQueryAvailable
        onActivated: root.dispatchDesktopCommand("find")
    }
    Shortcut {
        sequences: [StandardKey.Close]
        enabled: root.visible && !root.nativeMenuAvailable && reader.ready && !reader.browsingBusy
        onActivated: root.dispatchDesktopCommand("closeArchive")
    }
    Shortcut {
        sequence: StandardKey.SelectAll
        enabled: root.visible && !root.nativeMenuAvailable && reader.ready && !reader.browsingBusy
        onActivated: root.dispatchDesktopCommand("selectAll")
    }
    Shortcut {
        sequences: [StandardKey.Copy]
        enabled: root.visible && !root.nativeMenuAvailable && reader.ready && !reader.browsingBusy && reader.selectedCount > 0
        onActivated: {
            root.dispatchDesktopCommand("copy");
        }
    }
    CommandMenu {
        id: directoryMenu
        objectName: "browserDirectoryMenu"
        CommandMenuItem {
            text: root.contextDirectoryId.length ? qsTrId("ui.fb1eccfe14ea") : qsTrId("ui.42ee2863b6d7")
            enabled: reader.ready && !reader.browsingBusy
            onTriggered: {
                if (root.contextDirectoryId.length)
                    reader.navigateDirectory(root.contextDirectoryId);
                else
                    reader.showRoot();
            }
        }
        CommandMenuItem {
            objectName: "directoryExpansionCommand"
            text: root.contextDirectoryExpanded ? qsTrId("ui.9c9c21cc25c0") : qsTrId("ui.521dbd520244")
            enabled: root.contextDirectoryId.length > 0 && reader.ready && !reader.browsingBusy && root.contextDirectoryHasChildren
            disabledReason: root.contextDirectoryId.length ? qsTrId("ui.fa58108c4823") : qsTrId("directory.rootAlwaysExpanded")
            onTriggered: directoryTree.toggleExpanded(root.contextDirectoryRow)
        }
        CommandMenuItem {
            text: qsTrId("ui.2a7934774a06")
            enabled: reader.ready && !reader.browsingBusy
            onTriggered: reader.prepareExtraction(!root.contextDirectoryId.length, root.contextDirectoryId.length ? [root.contextDirectoryId] : [])
        }
        CommandMenuSeparator {}
        CommandMenuItem {
            text: qsTrId("ui.6050037c48c3")
            asset: "search"
            enabled: reader.ready && !reader.browsingBusy
            onTriggered: {
                root.pendingDirectorySearchFocus = true;
                if (root.contextDirectoryId.length)
                    reader.navigateDirectory(root.contextDirectoryId);
                else
                    reader.showRoot();
            }
        }
    }
    CommandMenu {
        id: sortMenu
        objectName: "browserSortMenu"
        CommandMenuItem {
            text: qsTrId("ui.d44e9b3d3b31")
            checkable: true
            checked: reader.sortOrder === 1 || reader.sortOrder === 2
            enabled: reader.ready && !reader.browsingBusy
            onTriggered: reader.sortByColumn(0)
        }
        CommandMenuItem {
            text: qsTrId("ui.301a69c74dfc")
            checkable: true
            checked: reader.sortOrder === 3 || reader.sortOrder === 4
            enabled: reader.ready && !reader.browsingBusy
            onTriggered: reader.sortByColumn(1)
        }
        CommandMenuItem {
            text: qsTrId("ui.2582e6a2c9e2")
            checkable: true
            checked: reader.sortOrder === 5 || reader.sortOrder === 6
            enabled: reader.ready && !reader.browsingBusy
            onTriggered: reader.sortByColumn(2)
        }
        CommandMenuItem {
            text: qsTrId("ui.ba40014ff496")
            checkable: true
            checked: reader.sortOrder === 9 || reader.sortOrder === 10
            enabled: reader.ready && !reader.browsingBusy
            onTriggered: reader.sortByColumn(3)
        }
        CommandMenuItem {
            text: qsTrId("ui.2cbced881b2d")
            checkable: true
            checked: reader.sortOrder === 11 || reader.sortOrder === 12
            enabled: reader.ready && !reader.browsingBusy
            onTriggered: reader.sortByColumn(4)
        }
        CommandMenuSeparator {}
        CommandMenuItem {
            text: qsTrId("ui.897c9681c945")
            checkable: true
            checked: reader.sortOrder === 0
            enabled: reader.ready && !reader.browsingBusy
            onTriggered: reader.sortByColumn(-1)
        }
    }
    CommandMenu {
        id: viewMenu
        objectName: "browserViewMenu"
        title: qsTrId("ui.1c5c06713870")
        CommandMenuItem {
            text: qsTrId("ui.d56220435194")
            checkable: true
            checked: root.viewMode === "list"
            onTriggered: root.viewMode = "list"
        }
        CommandMenuItem {
            text: qsTrId("ui.a35441db5cd1")
            checkable: true
            checked: root.viewMode === "grid"
            onTriggered: root.viewMode = "grid"
        }
        CommandMenuSeparator {}
        CommandMenuItem {
            text: qsTrId("ui.27b6154b3fd3")
            checkable: true
            checked: root.inspectorVisible || inspectorWindow.visible
            onTriggered: root.toggleInspector()
        }
    }
    CommandMenu {
        id: entryMenu
        objectName: "browserEntryMenu"
        CommandMenuItem {
            objectName: "contextOpenEntry"
            text: qsTrId("entry.open")
            enabled: root.contextAvailable && !reader.taskStarting
            onTriggered: reader.openContextEntry(root.contextEntry.session, root.contextEntry.id)
        }
        CommandMenuItem {
            objectName: "contextEnterDirectory"
            text: qsTrId("ui.fb1eccfe14ea")
            asset: "folder"
            commandVisible: Boolean(root.contextEntry.directory)
            enabled: root.contextAvailable && Boolean(root.contextEntry.navigable)
            disabledReason: qsTrId("ui.613797f41e6c")
            onTriggered: reader.enterDirectory(root.contextEntry.row)
        }
        CommandMenuItem {
            objectName: "contextProperties"
            text: qsTrId("ui.19ca3df6c82f")
            enabled: root.contextAvailable
            onTriggered: {
                reader.selectForContext(root.contextEntry.row);
                if (!root.inspectorVisible)
                    root.toggleInspector();
            }
        }
        CommandMenuItem {
            text: qsTrId("ui.994e3891e9d5")
            enabled: root.contextAvailable && reader.canPreflight
            onTriggered: reader.inspectSelection()
        }
        CommandMenuSeparator {}
        CommandMenuItem {
            objectName: "contextCopyName"
            text: qsTrId("ui.a77a6cca3ce2")
            shortcutLabel: Qt.platform.os === "osx" ? "⌘C" : "Ctrl+C"
            enabled: root.contextAvailable && reader.selectedCount > 0
            onTriggered: reader.copySelection(false)
        }
        CommandMenuItem {
            text: qsTrId("ui.567254b2f2a5")
            enabled: root.contextAvailable && reader.selectedCount > 0
            onTriggered: reader.copySelection(true)
        }
        CommandMenuSeparator {}
        CommandMenuItem {
            text: qsTrId("ui.a1fca7bde5d2")
            enabled: root.contextAvailable && reader.selectedCount > 0
            onTriggered: reader.prepareExtraction(false)
        }
    }
    CommandMenu {
        id: emptyMenu
        objectName: "browserBackgroundMenu"
        CommandMenuItem {
            text: qsTrId("ui.64f51f30dabc")
            enabled: reader.ready && !reader.browsingBusy
            onTriggered: reader.selectAll(true)
        }
        CommandMenuItem {
            text: qsTrId("ui.74966e20df2e")
            enabled: reader.ready && !reader.browsingBusy && reader.selectedCount > 0
            onTriggered: reader.selectAll(false)
        }
        CommandMenuSeparator {}
        CommandMenu {
            title: qsTrId("ui.1c5c06713870")
            CommandMenuItem {
                text: qsTrId("ui.d56220435194")
                checkable: true
                checked: root.viewMode === "list"
                onTriggered: root.viewMode = "list"
            }
            CommandMenuItem {
                text: qsTrId("ui.a35441db5cd1")
                checkable: true
                checked: root.viewMode === "grid"
                onTriggered: root.viewMode = "grid"
            }
        }
        CommandMenuItem {
            text: qsTrId("ui.27b6154b3fd3")
            checkable: true
            checked: root.inspectorVisible || inspectorWindow.visible
            onTriggered: root.toggleInspector()
        }
    }
    ColumnLayout {
        id: pageColumn
        anchors.fill: parent
        spacing: 0
        ArchiveTabs {
            Layout.fillWidth: true
            archiveTitle: root.closedPage ? "" : reader.title
            tabs: root.sessionManager && root.visible ? root.windowHost.sessionTabs : []
            sessionControls: root.sessionManager !== null
            onTabSelected: identity => root.sessionManager.activateId(identity)
            onTabClosed: identity => root.sessionManager.closeSession(identity)
            onOtherTabsClosed: identity => root.sessionManager.closeOtherSessions(identity)
            onRightTabsClosed: identity => root.sessionManager.closeSessionsToRight(identity)
            onTabMoved: (identity, index) => root.sessionManager.moveTab(identity, index)
            integratedTitlebar: root.integratedTitlebar
            windowsControls: root.windowsTitlebar
            maximized: root.visibility === Window.Maximized
            canOpen: root.sessionManager !== null || (reader.runtimeAvailable && !reader.browsingBusy)
            canClose: reader.ready && !reader.browsingBusy
            onOpenRequested: root.sessionManager ? root.sessionManager.newSession() : root.dispatchDesktopCommand("open")
            onCloseRequested: root.dispatchDesktopCommand("closeArchive")
            onSettingsRequested: root.dispatchDesktopCommand("settings")
            onDragRequested: root.startSystemMove()
            onMinimizeRequested: root.showMinimized()
            onMaximizeRequested: root.toggleMaximized()
            onWindowCloseRequested: root.close()
        }
        ArchiveToolbar {
            id: toolbar
            objectName: "archiveToolbar"
            visible: !root.welcomeVisible
            Layout.fillWidth: true
            canOpen: reader.runtimeAvailable && !reader.browsingBusy
            canSelect: reader.ready && !reader.browsingBusy
            canSearch: reader.browserQueryAvailable
            canTest: reader.ready && !reader.browsingBusy && !reader.taskStarting
            canInspect: reader.ready && !reader.browsingBusy && reader.canPreflight
            hasSelection: reader.selectedCount > 0
            onNewRequested: root.dispatchDesktopCommand("new")
            onOpenRequested: root.dispatchDesktopCommand("open")
            onTestRequested: root.dispatchDesktopCommand("testArchive")
            onInspectRequested: root.dispatchDesktopCommand("preflight")
            onExtractRequested: all => root.dispatchDesktopCommand(all ? "extractAll" : "extractSelected")
            onResetSortRequested: reader.sortByColumn(-1)
            onSearchRequested: query => reader.search(query)
            onSelectAllRequested: root.dispatchDesktopCommand("selectAll")
            onClearSelectionRequested: reader.selectAll(false)
            onDetailsRequested: root.dispatchDesktopCommand("inspector")
            onSettingsRequested: root.dispatchDesktopCommand("settings")
        }
        WelcomePage {
            id: welcomePage
            visible: root.welcomeVisible
            Layout.fillWidth: true
            Layout.fillHeight: true
            canOpen: reader.runtimeAvailable && !reader.browsingBusy
            canCreate: reader.runtimeAvailable && !reader.browsingBusy
            busy: !root.closedPage && reader.browsingBusy && reader.title.length > 0
            externalOpening: !root.closedPage && root.externalFileOpening
            openingName: reader.title
            message: root.closedPage || reader.browsingBusy ? "" : reader.message || AppSettings.error
            onOpenRequested: root.dispatchDesktopCommand("open")
            onCreateRequested: createWindow.configure()
            onCancelRequested: reader.cancel()
            onDragEntered: urls => {
                dropSupported = reader.canOpenDroppedUrls(urls);
                dropMessage = dropSupported ? qsTrId("ui.dc3e93fbf81a") : qsTrId("ui.a3c9bc4e63f7");
            }
            onArchiveDropped: urls => {
                if (reader.openDroppedUrls(urls)) {
                    // Activate after DropArea has accepted the drag and Finder has ended it.
                    Qt.callLater(function () {
                        root.raise();
                        root.requestActivate();
                    });
                }
            }
        }
        RowLayout {
            id: archiveWorkspace
            objectName: "archiveWorkspace"
            visible: !root.welcomeVisible
            enabled: !root.archiveOperationVisible
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: 0
            Rectangle {
                objectName: "archiveSidebar"
                Layout.preferredWidth: Math.round(Theme.sidebarWidth)
                Layout.minimumWidth: Math.round(Theme.sidebarWidth)
                Layout.maximumWidth: Math.round(Theme.sidebarWidth)
                Layout.fillHeight: true
                color: Theme.sidebar
                Rectangle {
                    anchors.right: parent.right
                    width: 1
                    height: parent.height
                    color: Theme.border
                }
                ColumnLayout {
                    anchors.fill: parent
                    anchors.leftMargin: Theme.mainTreeOuterInset
                    anchors.rightMargin: Theme.mainTreeOuterInset
                    anchors.bottomMargin: Theme.mainContentInset
                    spacing: 0
                    Item {
                        id: archiveCard
                        Layout.fillWidth: true
                        implicitHeight: Math.max(Theme.mainArchiveCardHeight, archiveSummary.y + archiveSummary.height + Theme.mainUnits(32))
                        Text {
                            id: archiveCaption
                            x: Theme.mainUnits(8)
                            objectName: "archiveCaption"
                            y: (fileHeader.height - height) / 2
                            text: qsTrId("ui.1674a0c33f35")
                            color: Theme.textSecondary
                            font.pixelSize: Theme.mainArchiveCaptionSize
                        }
                        Text {
                            id: archiveTitle
                            objectName: "sidebarArchiveName"
                            x: Theme.mainUnits(8)
                            y: archiveCaption.y + archiveCaption.height + Theme.mainUnits(7)
                            width: parent.width - Theme.mainUnits(16)
                            text: reader.title.length ? reader.title : qsTrId("ui.365d4b7cdd5c")
                            textFormat: Text.PlainText
                            elide: Text.ElideMiddle
                            font.pixelSize: Theme.mainArchiveTitleSize
                            font.weight: Font.DemiBold
                            color: Theme.text
                        }
                        RowLayout {
                            id: archiveSummary
                            x: Theme.mainUnits(8)
                            y: archiveTitle.y + archiveTitle.height + Theme.mainUnits(8)
                            spacing: Theme.mainUnits(12)
                            Rectangle {
                                objectName: "archiveFormatBadge"
                                implicitWidth: Math.max(Theme.mainFormatBadgeMinimumWidth, formatLabel.implicitWidth + 2 * Theme.mainFormatBadgeInset)
                                implicitHeight: Math.max(Theme.mainUnits(24), Theme.mainSummarySize * 1.6)
                                radius: Theme.mainUnits(5)
                                color: Theme.selection
                                Text {
                                    id: formatLabel
                                    objectName: "archiveFormatLabel"
                                    anchors.centerIn: parent
                                    text: reader.formatLabel || "ZIP"
                                    font.pixelSize: Theme.mainSummarySize
                                    color: Theme.accentInk
                                }
                            }
                            Text {
                                text: reader.ready || root.archiveOperationVisible ? qsTrId("ui.569488175a99", Number(reader.archiveTotal)) : qsTrId("ui.7fe646004f2a")
                                color: Theme.textSecondary
                                font.pixelSize: Theme.mainSummarySize
                            }
                        }
                        Rectangle {
                            x: Theme.mainUnits(8)
                            width: parent.width - Theme.mainUnits(16)
                            y: parent.height - Theme.mainUnits(16)
                            height: 1
                            color: Theme.border
                        }
                    }
                    Button {
                        id: allFiles
                        objectName: "directoryRoot"
                        ArchiveMoveTarget { objectName: "moveTarget-root"; anchors.fill: parent; z: 20; controller: reader; dragSource: moveDrag; directoryId: "" }
                        Layout.fillWidth: true
                        implicitHeight: Math.ceil(Math.max(Theme.treeRowMinHeight, Theme.treeBodyFontSize * Theme.fontScale * 1.6 + Theme.treeRowVerticalPadding, Theme.treeDisclosureHitMinHeight * Theme.fontScale + Theme.treeRowVerticalPadding))
                        leftPadding: Theme.mainUnits(12)
                        rightPadding: Theme.mainUnits(12)
                        enabled: reader.browserQueryAvailable
                        onClicked: reader.showRoot()
                        Accessible.name: qsTrId("ui.42ee2863b6d7")
                        background: Rectangle {
                            radius: Theme.mainControlRadius
                            topLeftRadius: 0
                            bottomLeftRadius: 0
                            color: (moveDrag.Drag.active && moveDrag.permitted && moveDrag.targetId === "" && moveDrag.feedbackOwner !== null) || reader.location.length === 0 ? Theme.treeSelectionColor : allFiles.hovered ? Theme.treeHoverColor : "transparent"
                            border.width: 0
                            border.color: Theme.accent
                            Rectangle {
                                visible: reader.location.length === 0
                                width: Theme.mainUnits(3)
                                height: parent.height
                                y: 0
                                radius: 2
                                color: Theme.accent
                            }
                        }
                        contentItem: RowLayout {
                            spacing: Theme.treeIconLabelGap
                            Image {
                                source: root.fileIcon("", true)
                                Layout.preferredWidth: Theme.treeFolderCanvas * Theme.fontScale
                                Layout.preferredHeight: Theme.treeFolderCanvas * Theme.fontScale
                                fillMode: Image.PreserveAspectFit
                            }
                            Text {
                                Layout.fillWidth: true
                                text: qsTrId("ui.42ee2863b6d7")
                                elide: Text.ElideRight
                                color: Theme.text
                                font.pixelSize: Theme.mainBodySize
                                font.bold: true
                            }
                        }
                        TapHandler {
                            acceptedButtons: Qt.RightButton
                            onTapped: function (eventPoint) {
                                root.contextDirectoryId = "";
                                root.contextDirectoryRow = -1;
                                root.contextDirectoryHasChildren = false;
                                root.contextDirectoryExpanded = true;
                                const position = allFiles.mapToItem(root.contentItem, eventPoint.position.x, eventPoint.position.y);
                                directoryMenu.popup(position.x, position.y);
                            }
                        }
                        Keys.onPressed: function (event) {
                            if (event.key === Qt.Key_Menu || event.key === Qt.Key_F10 && event.modifiers & Qt.ShiftModifier) {
                                root.contextDirectoryId = "";
                                root.contextDirectoryRow = -1;
                                root.contextDirectoryHasChildren = false;
                                root.contextDirectoryExpanded = true;
                                const position = allFiles.mapToItem(root.contentItem, 0, allFiles.height);
                                directoryMenu.popup(position.x, position.y);
                                event.accepted = true;
                            }
                        }
                    }
                    TreeView {
                        id: directoryTree
                        objectName: "directoryTree"
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        clip: true
                        model: reader.directories
                        selectionModel: ItemSelectionModel {}
                        selectionMode: TableView.SingleSelection
                        function activateCurrentDirectory() {
                            reader.navigateDirectory(reader.directories.data(selectionModel.currentIndex, Qt.UserRole));
                        }
                        Keys.onReturnPressed: activateCurrentDirectory()
                        Keys.onEnterPressed: activateCurrentDirectory()
                        Keys.onPressed: function (event) {
                            if (event.key === Qt.Key_Menu || event.key === Qt.Key_F10 && event.modifiers & Qt.ShiftModifier) {
                                const row = directoryTree.currentRow;
                                const index = directoryTree.index(row, 0);
                                if (row < 0 || !reader.ready || reader.browsingBusy)
                                    return;
                                root.contextDirectoryId = reader.directories.data(index, Qt.UserRole);
                                root.contextDirectoryRow = row;
                                root.contextDirectoryHasChildren = reader.directories.rowCount(index) > 0;
                                root.contextDirectoryExpanded = directoryTree.isExpanded(row);
                                const item = directoryTree.itemAtCell(Qt.point(0, row));
                                const position = (item || directoryTree).mapToItem(root.contentItem, 0, item ? item.height : 0);
                                directoryMenu.popup(position.x, position.y);
                                event.accepted = true;
                            }
                        }
                        enabled: reader.browserQueryAvailable
                        columnWidthProvider: function (column) {
                            let extent = width;
                            for (let row = 0; row < rows; ++row) {
                                const node = index(row, 0);
                                const depth = reader.directories.data(node, Qt.UserRole + 5);
                                const name = reader.directories.data(node, Qt.DisplayRole);
                                extent = Math.max(extent, Theme.treeContentLeftInset + depth * Theme.treeDepthIndent + Theme.treeDisclosureHitWidth * Theme.fontScale + Theme.treeFolderCanvas * Theme.fontScale + Theme.treeIconLabelGap + treeTextMetrics.advanceWidth(name) + Theme.mainTreeTrailingInset);
                            }
                            return Math.ceil(extent);
                        }
                        FontMetrics {
                            id: treeTextMetrics
                            font.family: "Xdzip Design Sans SC"
                            font.pixelSize: Theme.mainBodySize
                        }
                        onExpanded: forceLayout()
                        onCollapsed: forceLayout()
                        onWidthChanged: forceLayout()
                        Connections {
                            target: Theme
                            function onFontScaleChanged() {
                                directoryTree.forceLayout();
                            }
                        }
                        Connections {
                            target: reader
                            function onChanged() {
                                directoryTree.forceLayout();
                            }
                        }
                        ScrollBar.vertical: ScrollBar {}
                        ScrollBar.horizontal: ScrollBar {}
                        delegate: TreeViewDelegate {
                            id: directoryNode
                            required property string nodeName
                            required property string nodeId
                            required property int entryCount
                            required property string fullPath
                            required property var continuationFlags
                            required property bool lastSibling
                            required property int nodeDepth
                            property bool inReusePool: false
                            TableView.onPooled: inReusePool = true
                            TableView.onReused: inReusePool = false
                            readonly property bool hiddenCurrentBranch: reader.directoryId !== nodeId && !expanded && reader.directoryAncestors(reader.directoryId).includes(nodeId)
                            objectName: "directory-" + nodeId
                            ArchiveMoveTarget { objectName: "moveTarget-tree-" + directoryNode.nodeId; anchors.fill: parent; z: 20; controller: reader; dragSource: moveDrag; directoryId: directoryNode.nodeId }
                            implicitHeight: Math.ceil(Math.max(Theme.treeRowMinHeight, Theme.treeBodyFontSize * Theme.fontScale * 1.6 + Theme.treeRowVerticalPadding, Theme.treeDisclosureHitMinHeight * Theme.fontScale + Theme.treeRowVerticalPadding))
                            implicitWidth: directoryTree.width
                            indentation: Theme.treeDepthIndent
                            leftMargin: 0
                            leftPadding: Theme.treeContentLeftInset + nodeDepth * Theme.treeDepthIndent + Theme.treeDisclosureHitWidth * Theme.fontScale
                            topPadding: 0
                            bottomPadding: 0
                            rightPadding: Theme.mainUnits(8)
                            palette.windowText: Theme.textSecondary
                            indicator: ToolButton {
                                id: directoryDisclosure
                                objectName: "directoryDisclosure-" + directoryNode.nodeId
                                x: Theme.treeContentLeftInset + directoryNode.nodeDepth * Theme.treeDepthIndent
                                y: (directoryNode.height - height) / 2
                                width: Theme.treeDisclosureHitWidth * Theme.fontScale
                                height: Theme.treeDisclosureHitMinHeight * Theme.fontScale
                                padding: (Theme.treeDisclosureHitWidth - Theme.treeDisclosureGlyphCanvas) / 2 * Theme.fontScale
                                visible: directoryNode.hasChildren
                                Accessible.name: directoryNode.expanded ? qsTrId("ui.47c91fadcc3e") : qsTrId("ui.a49f5287fc9e")
                                contentItem: Image {
                                    source: "qrc:/xdzip/assets/" + (Theme.dark ? "dark/" : "light/") + (directoryNode.expanded ? "chevron-down.png" : "chevron-right.png")
                                    fillMode: Image.PreserveAspectFit
                                }
                                background: Rectangle {
                                    radius: Theme.mainUnits(4)
                                    color: directoryDisclosure.down ? Theme.controlPressed : directoryDisclosure.hovered ? Theme.controlHover : "transparent"
                                }
                                onClicked: {
                                    const index = directoryTree.index(directoryNode.row, 0);
                                    directoryTree.toggleExpanded(directoryNode.row);
                                    directoryTree.selectionModel.setCurrentIndex(index, ItemSelectionModel.NoUpdate);
                                    directoryTree.forceActiveFocus(Qt.MouseFocusReason);
                                }
                            }
                            onClicked: reader.navigateDirectory(nodeId)
                            Keys.onReturnPressed: reader.navigateDirectory(nodeId)
                            Keys.onEnterPressed: reader.navigateDirectory(nodeId)
                            TapHandler {
                                acceptedButtons: Qt.RightButton
                                onTapped: function (eventPoint) {
                                    root.contextDirectoryId = directoryNode.nodeId;
                                    root.contextDirectoryRow = directoryNode.row;
                                    root.contextDirectoryHasChildren = directoryNode.hasChildren;
                                    root.contextDirectoryExpanded = directoryNode.expanded;
                                    const position = directoryNode.mapToItem(root.contentItem, eventPoint.position.x, eventPoint.position.y);
                                    directoryMenu.popup(position.x, position.y);
                                }
                            }
                            Accessible.name: nodeName + qsTrId("ui.81d50cef7f67", Number(entryCount))
                            Accessible.description: directoryNode.fullPath
                            background: Rectangle {
                                radius: Theme.mainControlRadius
                                topLeftRadius: 0
                                bottomLeftRadius: 0
                                color: (moveDrag.Drag.active && moveDrag.permitted && moveDrag.targetId === directoryNode.nodeId) || reader.directoryId === directoryNode.nodeId ? Theme.treeSelectionColor : directoryNode.hovered ? Theme.treeHoverColor : "transparent"
                                border.width: 0
                                border.color: Theme.accent
                                Rectangle {
                                    visible: reader.directoryId === directoryNode.nodeId
                                    x: directoryTree.contentX
                                    width: Theme.mainUnits(3)
                                    height: parent.height
                                    y: 0
                                    radius: 2
                                    color: Theme.accent
                                }
                                Repeater {
                                    model: directoryNode.continuationFlags
                                    Rectangle {
                                        required property bool modelData
                                        required property int index
                                        visible: modelData
                                        x: Theme.treeContentLeftInset + Theme.treeDisclosureHitWidth * Theme.fontScale / 2 + index * Theme.treeDepthIndent - width / 2
                                        width: 1
                                        height: directoryNode.height
                                        color: Theme.treeGuideColor
                                    }
                                }
                                Rectangle {
                                    objectName: "directoryParentGuide-" + directoryNode.nodeId
                                    visible: directoryNode.nodeDepth > 0
                                    x: Theme.treeContentLeftInset + Theme.treeDisclosureHitWidth * Theme.fontScale / 2 + (directoryNode.nodeDepth - 1) * Theme.treeDepthIndent - width / 2
                                    width: 1
                                    height: directoryNode.lastSibling ? directoryNode.height / 2 : directoryNode.height
                                    color: Theme.treeGuideColor
                                    Rectangle {
                                        y: directoryNode.lastSibling ? parent.height : directoryNode.height / 2
                                        width: Theme.mainUnits(10)
                                        height: 1
                                        color: Theme.treeGuideColor
                                    }
                                }
                            }
                            contentItem: RowLayout {
                                spacing: Theme.mainUnits(8)
                                Image {
                                    objectName: "directoryFolder-" + directoryNode.nodeId
                                    source: root.fileIcon("", true)
                                    Layout.preferredWidth: Theme.treeFolderCanvas * Theme.fontScale
                                    Layout.preferredHeight: Theme.treeFolderCanvas * Theme.fontScale
                                    fillMode: Image.PreserveAspectFit
                                }
                                Text {
                                    Layout.fillWidth: true
                                    Layout.rightMargin: directoryNode.hiddenCurrentBranch ? Theme.treeBranchMarkerCanvas : 0
                                    text: directoryNode.nodeName
                                    textFormat: Text.PlainText
                                    elide: Text.ElideRight
                                    color: Theme.text
                                    font.pixelSize: Theme.mainBodySize
                                }
                            }
                            Image {
                                objectName: "directoryBranchMarker-" + directoryNode.nodeId
                                anchors.right: parent.right
                                anchors.verticalCenter: parent.verticalCenter
                                width: Theme.treeBranchMarkerCanvas
                                height: width
                                visible: directoryNode.hiddenCurrentBranch
                                source: "qrc:/xdzip/assets/" + (Theme.dark ? "dark/" : "light/") + "tree-active-branch.png"
                            }
                        }
                    }
                    Text {
                        Layout.fillWidth: true
                        visible: reader.navigationLimited
                        text: qsTrId("ui.a2bec6b91db6")
                        wrapMode: Text.Wrap
                        color: Theme.textSecondary
                        font.pixelSize: Theme.mainSecondarySize
                    }
                }
            }
            ColumnLayout {
                Layout.fillWidth: true
                Layout.fillHeight: true
                spacing: 0
                PaneHeader {
                    id: fileHeader
                    rightInset: root.inspectorVisible ? Theme.mainPaneInset : Theme.mainInspectorRightInset
                    actionInset: fileActions.width + Theme.mainFileActionGap
                    objectName: "filePaneHeader"
                    Layout.fillWidth: true
                    Item {
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        Layout.minimumWidth: 0
                        Breadcrumb {
                            anchors.fill: parent
                            available: reader.browserQueryAvailable
                            entries: reader.title.length ? [
                                {
                                    id: "",
                                    label: reader.title,
                                    current: false
                                },
                                ...(reader.breadcrumbs.length ? reader.breadcrumbs : [
                                        {
                                            id: "",
                                            label: qsTrId("ui.42ee2863b6d7"),
                                            current: true
                                        }
                                    ])] : [
                                {
                                    id: "",
                                    label: qsTrId("ui.989ea71171f9"),
                                    current: true
                                }
                            ]
                            onActivated: function (nodeId) {
                                if (nodeId.length)
                                    reader.navigateDirectory(nodeId);
                                else
                                    reader.showRoot();
                            }
                        }
                    }
                    Row {
                        id: fileActions
                        objectName: "filePaneActions"
                        parent: fileHeader
                        anchors.right: parent.right
                        anchors.rightMargin: fileHeader.rightInset
                        anchors.verticalCenter: parent.verticalCenter
                        width: implicitWidth
                        height: Theme.mainIconButtonSize
                        spacing: Theme.mainFileActionGap
                        Text {
                            height: Theme.mainIconButtonSize
                            verticalAlignment: Text.AlignVCenter
                            text: reader.ready || root.archiveOperationVisible ? qsTrId("ui.569488175a99", Number(reader.total)) : ""
                            color: Theme.textSecondary
                            font.pixelSize: Theme.mainSummarySize
                        }
                        ArchiveAction {
                            id: viewButton
                            width: Theme.mainIconButtonSize
                            height: Theme.mainIconButtonSize
                            objectName: "browserViewMenuButton"
                            Layout.minimumWidth: Theme.mainIconButtonSize
                            Layout.preferredWidth: Theme.mainIconButtonSize
                            Layout.maximumWidth: Theme.mainIconButtonSize
                            Layout.minimumHeight: Theme.mainIconButtonSize
                            Layout.preferredHeight: Theme.mainIconButtonSize
                            Layout.maximumHeight: Theme.mainIconButtonSize
                            Layout.alignment: Qt.AlignVCenter
                            asset: root.viewMode === "list" ? "view-list-selected" : "view-list"
                            leftPadding: Theme.mainUnits(8)
                            rightPadding: Theme.mainUnits(8)
                            selected: root.viewMode === "list"
                            Accessible.name: qsTrId("ui.d56220435194")
                            explanation: Accessible.name
                            onClicked: root.viewMode = "list"
                            TapHandler {
                                acceptedButtons: Qt.RightButton
                                onTapped: viewMenu.popup(viewButton, 0, viewButton.height + Theme.mainPopupGap)
                            }
                        }
                        ArchiveAction {
                            objectName: "browserGridButton"
                            width: Theme.mainIconButtonSize
                            height: Theme.mainIconButtonSize
                            Layout.minimumWidth: Theme.mainIconButtonSize
                            Layout.preferredWidth: Theme.mainIconButtonSize
                            Layout.maximumWidth: Theme.mainIconButtonSize
                            Layout.minimumHeight: Theme.mainIconButtonSize
                            Layout.preferredHeight: Theme.mainIconButtonSize
                            Layout.maximumHeight: Theme.mainIconButtonSize
                            Layout.alignment: Qt.AlignVCenter
                            asset: root.viewMode === "grid" ? "view-grid-selected" : "view-grid"
                            leftPadding: Theme.mainUnits(5)
                            rightPadding: Theme.mainUnits(5)
                            selected: root.viewMode === "grid"
                            Accessible.name: qsTrId("ui.a35441db5cd1")
                            explanation: Accessible.name
                            onClicked: root.viewMode = "grid"
                        }
                        ArchiveAction {
                            objectName: "browserSortMenuButton"
                            width: Theme.mainIconButtonSize
                            height: Theme.mainIconButtonSize
                            Layout.minimumWidth: Theme.mainIconButtonSize
                            Layout.preferredWidth: Theme.mainIconButtonSize
                            Layout.maximumWidth: Theme.mainIconButtonSize
                            Layout.minimumHeight: Theme.mainIconButtonSize
                            Layout.preferredHeight: Theme.mainIconButtonSize
                            Layout.maximumHeight: Theme.mainIconButtonSize
                            Layout.alignment: Qt.AlignVCenter
                            asset: "sort"
                            leftPadding: 2
                            rightPadding: 2
                            glyphSize: Theme.mainUnits(28)
                            Accessible.name: qsTrId("ui.1ce6b1d7c95c")
                            explanation: Accessible.name
                            enabled: reader.browserQueryAvailable
                            onClicked: {
                                const p = mapToItem(root.contentItem, width, height);
                                sortMenu.popup(root.contentItem, p.x - sortMenu.width, p.y + Theme.mainPopupGap);
                            }
                        }
                        ArchiveAction {
                            objectName: "inspectorToggle"
                            width: Theme.mainIconButtonSize
                            height: Theme.mainIconButtonSize
                            visible: !root.inspectorVisible
                            asset: "panel"
                            Accessible.name: qsTrId("ui.749ca86aef0e")
                            explanation: Accessible.name
                            onClicked: root.toggleInspector()
                        }
                    }
                }
                Rectangle {
                    objectName: "fileTableHeader"
                    visible: root.viewMode === "list"
                    Layout.fillWidth: true
                    implicitHeight: Math.round(Math.max(Theme.mainTableHeaderHeight, Theme.mainHeaderSize * 1.6 + Theme.mainUnits(12)))
                    color: Theme.canvas
                    clip: true
                    Row {
                        x: -table.contentX
                        Repeater {
                            model: [qsTrId("ui.d44e9b3d3b31"), qsTrId("ui.301a69c74dfc"), qsTrId("ui.2582e6a2c9e2"), qsTrId("ui.ba40014ff496"), qsTrId("ui.2cbced881b2d")]
                            Text {
                                id: columnHeader
                                required property string modelData
                                required property int index
                                width: table.columnSize(index)
                                height: parent && parent.parent ? parent.parent.height : 0
                                readonly property bool sorted: reader.sortOrder > 0 && (reader.sortOrder < 9 ? Math.floor((reader.sortOrder - 1) / 2) : Math.floor((reader.sortOrder - 3) / 2)) === index
                                readonly property bool rightAligned: index === 1 || index === 2
                                readonly property real displayedTextWidth: Math.min(contentWidth, Math.max(0, width - leftPadding - rightPadding))
                                leftPadding: Math.max(table.columnLeftInset(index, true), sorted && rightAligned ? sortIndicator.implicitWidth + Theme.mainSortIndicatorGap + Theme.mainUnits(4) : 0)
                                rightPadding: Math.max(table.columnRightInset(index), sorted && !rightAligned ? sortIndicator.implicitWidth + Theme.mainSortIndicatorGap + Theme.mainUnits(4) : 0)
                                horizontalAlignment: rightAligned ? Text.AlignRight : Text.AlignLeft
                                verticalAlignment: Text.AlignVCenter
                                objectName: "sort-column-" + index
                                text: modelData
                                elide: Text.ElideRight
                                color: Theme.text
                                font.pixelSize: Theme.mainHeaderSize
                                activeFocusOnTab: true
                                enabled: reader.browserQueryAvailable
                                Accessible.role: Accessible.Button
                                Accessible.name: modelData + (reader.sortOrder === (index < 3 ? index * 2 + 1 : index * 2 + 3) ? qsTrId("ui.e5a9c120c1e7") : reader.sortOrder === (index < 3 ? index * 2 + 2 : index * 2 + 4) ? qsTrId("ui.8abeaa2c751b") : "")
                                Accessible.onPressAction: if (reader.browserQueryAvailable)
                                    reader.sortByColumn(index)
                                Keys.onReturnPressed: if (reader.browserQueryAvailable)
                                    reader.sortByColumn(index)
                                Keys.onEnterPressed: if (reader.browserQueryAvailable)
                                    reader.sortByColumn(index)
                                Keys.onSpacePressed: if (reader.browserQueryAvailable)
                                    reader.sortByColumn(index)
                                TapHandler {
                                    enabled: reader.browserQueryAvailable
                                    onTapped: eventPoint => {
                                        if (eventPoint.position.x < columnHeader.width - Theme.mainColumnResizeHitWidth)
                                            reader.sortByColumn(columnHeader.index);
                                    }
                                }
                                TapHandler {
                                    acceptedButtons: Qt.RightButton
                                    enabled: reader.browserQueryAvailable
                                    onTapped: function (eventPoint) {
                                        const position = columnHeader.mapToItem(root.contentItem, eventPoint.position.x, eventPoint.position.y);
                                        sortMenu.popup(position.x, position.y);
                                    }
                                }
                                Rectangle {
                                    anchors.fill: parent
                                    color: "transparent"
                                    border.width: 0
                                    border.color: Theme.accent
                                }
                                Text {
                                    id: sortIndicator
                                    objectName: "sort-indicator-" + columnHeader.index
                                    anchors.verticalCenter: parent.verticalCenter
                                    x: columnHeader.rightAligned
                                        ? columnHeader.width - columnHeader.rightPadding - columnHeader.displayedTextWidth - width - Theme.mainSortIndicatorGap
                                        : columnHeader.leftPadding + columnHeader.displayedTextWidth + Theme.mainSortIndicatorGap
                                    text: reader.sortOrder % 2 === 1 ? "↑" : "↓"
                                    visible: columnHeader.sorted
                                    color: Theme.accentInk
                                    font.pixelSize: Theme.mainHeaderSize
                                }
                                Rectangle {
                                    anchors.right: parent.right
                                    anchors.verticalCenter: parent.verticalCenter
                                    width: 1
                                    height: Theme.mainUnits(22)
                                    visible: columnHeader.index < 4
                                    color: Theme.border
                                }
                                MouseArea {
                                    id: resizeHandle
                                    objectName: "column-resize-" + columnHeader.index
                                    x: parent.width - width
                                    width: Theme.mainColumnResizeHitWidth
                                    height: parent.height
                                    z: 3
                                    cursorShape: Qt.SplitHCursor
                                    acceptedButtons: Qt.LeftButton
                                    preventStealing: true
                                    property real originX: 0
                                    property real originWidth: 0
                                    property bool resetting: false
                                    onPressed: mouse => {
                                        resetting = false;
                                        originX = mapToItem(root, mouse.x, mouse.y).x;
                                        originWidth = table.columnSize(columnHeader.index);
                                    }
                                    onPositionChanged: mouse => {
                                        if (!pressed || resetting)
                                            return;
                                        const width = originWidth + mapToItem(root, mouse.x, mouse.y).x - originX;
                                        BrowserLayout.resizeColumn(columnHeader.index, Math.max(table.minimumColumnWidth(columnHeader.index), width) / Math.max(1, Theme.fontScale));
                                    }
                                    onDoubleClicked: {
                                        resetting = true;
                                        BrowserLayout.resetColumn(columnHeader.index);
                                    }
                                }
                                SelectionCheck {
                                    objectName: "selectPage"
                                    visible: columnHeader.index === 0
                                    x: Theme.mainCheckboxInset
                                    anchors.verticalCenter: parent.verticalCenter
                                    enabled: reader.ready && !reader.browsingBusy && reader.entries.rowCount() > 0
                                    checkState: reader.selectedCount === 0 ? Qt.Unchecked : reader.selectedCount === reader.entries.rowCount() ? Qt.Checked : Qt.PartiallyChecked
                                    nextCheckState: () => checkState === Qt.Checked ? Qt.Unchecked : Qt.Checked
                                    Accessible.name: qsTrId("ui.6f0c67b2ca91")
                                    onClicked: reader.selectAll(checkState === Qt.Checked)
                                }
                            }
                        }
                    }
                }
                Item {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    MouseArea {
                        anchors.fill: parent
                        acceptedButtons: Qt.RightButton
                        onClicked: function (mouse) {
                            const point = mapToItem(root.contentItem, mouse.x, mouse.y);
                            emptyMenu.popup(point.x, point.y);
                        }
                    }
                    DropArea {
                        id: archiveDrop
                        objectName: "archiveAddDropArea"
                        anchors.fill: parent
                        z: 20
                        property bool permitted: false
                        property bool observing: false
                        keys: ["text/uri-list"]
                        onExited: observing = false
                        onEntered: drag => {
                            observing = drag.hasUrls && !drag.formats.includes("application/x-xdzip-export");
                            permitted = drag.hasUrls && !drag.formats.includes("application/x-xdzip-export") && reader.canAddDroppedUrls(drag.urls);
                            if (observing) drag.accept(permitted ? Qt.CopyAction : Qt.IgnoreAction);
                            else drag.accepted = false;
                        }
                        onPositionChanged: drag => {
                            permitted = observing && reader.canAddDroppedUrls(drag.urls);
                            drag.accept(permitted ? Qt.CopyAction : Qt.IgnoreAction);
                        }
                        onDropped: drop => {
                            observing = false;
                            if (permitted && drop.hasUrls && reader.canAddDroppedUrls(drop.urls)) {
                                drop.accept(Qt.CopyAction);
                                reader.addDroppedUrls(drop.urls);
                            } else drop.accepted = false;
                        }
                        Rectangle {
                            anchors.fill: parent
                            objectName: "archiveAddDropFeedback"
                            visible: archiveDrop.containsDrag && archiveDrop.observing
                            color: Theme.selection
                            opacity: 0.9
                            border.width: 2
                            border.color: Theme.accent
                            Text { anchors.centerIn: parent; text: archiveDrop.permitted || !reader.canUpdateArchive ? reader.addDropMessage : qsTrId("update.invalidDrop"); font.pixelSize: Theme.mainBodySize; color: Theme.text; wrapMode: Text.Wrap; width: parent.width - Theme.mainGridInset * 2; horizontalAlignment: Text.AlignHCenter }
                        }
                    }
                    TableView {
                        id: table
                        objectName: "archiveTable"
                        visible: root.viewMode === "list"
                        anchors.fill: parent
                        clip: true
                        property int hoveredRow: -1
                        HoverHandler {
                            enabled: root.viewMode === "list"
                            onPointChanged: table.hoveredRow = table.cellAtPosition(point.position, true).y
                            onHoveredChanged: if (!hovered)
                                table.hoveredRow = -1
                        }
                        model: reader.entries
                        TextMetrics { id: originalSizeHeading; font.pixelSize: Theme.mainHeaderSize; text: qsTrId("ui.301a69c74dfc") }
                        TextMetrics { id: packedSizeHeading; font.pixelSize: Theme.mainHeaderSize; text: qsTrId("ui.2582e6a2c9e2") }
                        TextMetrics { id: sortUpWidth; font.pixelSize: Theme.mainHeaderSize; text: "↑" }
                        TextMetrics { id: sortDownWidth; font.pixelSize: Theme.mainHeaderSize; text: "↓" }
                        function columnSize(column: int): real {
                            const scale = Math.max(1, Theme.fontScale);
                            const arrowSpace = Math.max(sortUpWidth.advanceWidth, sortDownWidth.advanceWidth) + Theme.mainSortIndicatorGap + Theme.mainUnits(4);
                            const other = [
                                Math.max(Theme.mainOriginalColumnWidth * scale, Math.ceil(originalSizeHeading.advanceWidth + arrowSpace + Theme.mainOriginalColumnInset)),
                                Math.max(Theme.mainPackedColumnWidth * scale, Math.ceil(packedSizeHeading.advanceWidth + arrowSpace + Theme.mainPackedColumnInset)),
                                Theme.mainTypeColumnWidth * scale, Theme.mainDateColumnWidth * scale
                            ];
                            const custom = BrowserLayout.columnWidths;
                            if (custom[column] > 0)
                                return Math.round(Math.max(minimumColumnWidth(column), custom[column] * scale));
                            return Math.round(column === 0 ? Math.max(Theme.mainUnits(250), width - other.reduce((sum, value, index) => sum + Math.round(custom[index + 1] > 0 ? Math.max(minimumColumnWidth(index + 1), custom[index + 1] * scale) : value), 0)) : other[column - 1]);
                        }
                        function minimumColumnWidth(column: int): real {
                            return (column === 0 ? Theme.mainNameColumnMinimumWidth : Theme.mainColumnMinimumWidth) * Math.max(1, Theme.fontScale);
                        }
                        function columnLeftInset(column, header) {
                            return column === 0 ? (header ? Theme.mainNameHeaderInset : Theme.mainNameTextInset) : column === 3 ? Theme.mainTypeColumnInset : column === 4 ? Theme.mainDateColumnInset : 0;
                        }
                        function columnRightInset(column) {
                            return column === 1 ? Theme.mainOriginalColumnInset : column === 2 ? Theme.mainPackedColumnInset : 0;
                        }
                        columnWidthProvider: column => table.columnSize(column)
                        onWidthChanged: forceLayout()
                        readonly property real densityFactor: AppSettings.density === "compact" ? Theme.settingsCompactListFactor : 1
                        readonly property real scaledRowHeight: Math.max(Theme.mainFolderRowHeight * densityFactor, Theme.mainTableSize * 1.6 + Theme.mainUnits(16) * densityFactor)
                        onScaledRowHeightChanged: forceLayout()
                        Connections {
                            target: Theme
                            function onFontScaleChanged() {
                                table.forceLayout();
                            }
                        }
                        Connections {
                            target: BrowserLayout
                            function onColumnWidthsChanged() {
                                table.forceLayout();
                            }
                        }
                        rowHeightProvider: function (row) {
                            return Math.max(reader.entries.data(reader.entries.index(row, 0), Qt.UserRole + 1) ? Theme.mainFolderRowHeight * table.densityFactor : Theme.mainFileRowHeight * table.densityFactor, table.scaledRowHeight);
                        }
                        ScrollBar.vertical: ScrollBar {}
                        ScrollBar.horizontal: ScrollBar {
                            policy: table.contentWidth > table.width ? ScrollBar.AlwaysOn : ScrollBar.AlwaysOff
                        }
                        delegate: Rectangle {
                            id: cell
                            required property string display
                            required property int row
                            required property int column
                            required property bool isDirectory
                            required property bool chosen
                            required property string entryId
                            required property bool canNavigate
                            required property bool canSelect
                            required property string entryStatus
                            required property bool attention
                            objectName: "entry-" + row + "-" + column
                            ArchiveMoveTarget { id: cellMoveTarget; objectName: "moveTarget-list-" + cell.row + "-" + cell.column; anchors.fill: parent; z: 20; enabled: cell.isDirectory && cell.canNavigate; controller: reader; dragSource: moveDrag; directoryId: reader.directoryForRow(cell.row) }
                            Accessible.name: display
                            Accessible.description: entryStatus
                            implicitWidth: Theme.mainUnits(100)
                            implicitHeight: table.scaledRowHeight
                            color: Theme.surface
                            Rectangle {
                                objectName: "entryBand-" + cell.row + "-" + cell.column
                                anchors.fill: parent
                                anchors.leftMargin: cell.column === 0 ? Theme.mainUnits(8) : 0
                                anchors.rightMargin: cell.column === 4 ? Theme.mainUnits(8) : 0
                                anchors.topMargin: Theme.mainEntryBandTopInset
                                anchors.bottomMargin: Theme.mainEntryBandBottomInset
                                radius: cell.column === 0 || cell.column === 4 ? Theme.mainControlRadius : 0
                                color: (moveDrag.Drag.active && moveDrag.permitted && cell.isDirectory && moveDrag.targetId === cellMoveTarget.directoryId) || cell.chosen ? Theme.selection : table.hoveredRow === cell.row ? Theme.controlHover : Theme.surface
                                // Square the interior edge so adjacent cells form one continuous rounded band.
                                Rectangle {
                                    visible: cell.column === 0 || cell.column === 4
                                    width: parent.radius
                                    height: parent.height
                                    x: cell.column === 0 ? parent.width - width : 0
                                    color: parent.color
                                }
                            }
                            HoverHandler {
                                id: hover
                            }
                            MouseArea {
                                id: cellPointer
                                anchors.fill: parent
                                acceptedButtons: Qt.LeftButton | Qt.RightButton
                                enabled: reader.runtimeAvailable && !reader.browsingBusy
                                property point pressedAt
                                preventStealing: true
                                property bool transferStarted: false
                                onPressed: mouse => { pressedAt = Qt.point(mouse.x, mouse.y); transferStarted = false; }
                                onPositionChanged: mouse => {
                                    if (pressed && (pressedButtons & Qt.LeftButton) && !transferStarted && Math.hypot(mouse.x - pressedAt.x, mouse.y - pressedAt.y) >= Qt.styleHints.startDragDistance) {
                                        transferStarted = true;
                                        root.beginTransfer(this, cell.row, mouse);
                                    }
                                    if (transferStarted) root.continueTransfer(this, mouse);
                                }
                                onReleased: if (transferStarted) root.finishTransfer()
                                onCanceled: root.deferCancelledTransfer()
                                onClicked: function (mouse) {
                                    if (transferStarted) return;
                                    if (mouse.button === Qt.RightButton)
                                        root.entryMenuAt(cell, cell.row, cell.entryId, cell.isDirectory, cell.canNavigate, mouse.x, mouse.y);
                                    else {
                                        root.selectWithModifiers(cell.row, mouse.modifiers);
                                        cell.forceActiveFocus(Qt.MouseFocusReason);
                                    }
                                }
                                onDoubleClicked: reader.enterDirectory(cell.row)
                            }
                            Keys.onPressed: function (event) {
                                if (event.key === Qt.Key_Menu || (event.key === Qt.Key_F10 && (event.modifiers & Qt.ShiftModifier))) {
                                    root.entryMenuAt(cell, cell.row, cell.entryId, cell.isDirectory, cell.canNavigate, 0, cell.height);
                                    event.accepted = true;
                                } else if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter) {
                                    reader.enterDirectory(cell.row);
                                    event.accepted = true;
                                } else if (event.key === Qt.Key_Space) {
                                    reader.selectRow(cell.row);
                                    event.accepted = true;
                                }
                            }
                            Rectangle {
                                objectName: "entrySeparator-" + cell.row + "-" + cell.column
                                anchors.bottom: parent.bottom
                                width: parent.width
                                height: Theme.mainEntrySeparatorHeight
                                color: Theme.border
                                opacity: 0.45
                            }
                            SelectionCheck {
                                id: entryCheck
                                objectName: "entrySelection"
                                visible: cell.column === 0
                                Keys.onReturnPressed: reader.enterDirectory(cell.row)
                                anchors.left: parent.left
                                anchors.leftMargin: Theme.mainCheckboxInset
                                anchors.verticalCenter: parent.verticalCenter
                                anchors.alignWhenCentered: false
                                checked: cell.chosen
                                enabled: cell.canSelect && reader.runtimeAvailable && !reader.browsingBusy
                                Accessible.name: cell.display
                                onClicked: reader.selectRow(cell.row)
                            }
                            Image {
                                objectName: "entryIcon"
                                visible: cell.column === 0
                                anchors.left: parent.left
                                anchors.leftMargin: Theme.mainNameIconInset
                                anchors.verticalCenter: parent.verticalCenter
                                anchors.alignWhenCentered: false
                                width: root.entryIconSize
                                height: root.entryIconSize
                                source: root.fileIcon(cell.display, cell.isDirectory)
                                fillMode: Image.PreserveAspectFit
                            }
                            Text {
                                anchors.fill: parent
                                anchors.leftMargin: table.columnLeftInset(cell.column, false)
                                anchors.rightMargin: cell.column === 0 && cell.attention ? Theme.mainUnits(24) : table.columnRightInset(cell.column)
                                text: cell.column === 1 || cell.column === 2 ? (cell.isDirectory ? "—" : root.formatSize(cell.display)) : cell.display
                                horizontalAlignment: cell.column === 1 || cell.column === 2 ? Text.AlignRight : Text.AlignLeft
                                textFormat: Text.PlainText
                                verticalAlignment: Text.AlignVCenter
                                elide: Text.ElideRight
                                font.pixelSize: Theme.mainTableSize
                                color: Theme.text
                            }
                            Text {
                                anchors.right: parent.right
                                anchors.rightMargin: Theme.mainUnits(8)
                                anchors.verticalCenter: parent.verticalCenter
                                visible: cell.column === 0 && cell.attention
                                text: "!"
                                font.pixelSize: Theme.mainTableSize
                                font.bold: true
                                color: Theme.menuDanger
                                Accessible.name: cell.entryStatus
                            }
                            CommandTooltip {
                                objectName: "entryTooltip-" + cell.row + "-" + cell.column
                                requested: hover.hovered && cell.column === 0 && !cellPointer.pressed
                                suppressed: root.archiveTransferActive
                                text: cell.display + "\n" + cell.entryStatus
                            }
                        }
                    }
                    GridView {
                        id: grid
                        objectName: "archiveGrid"
                        anchors.fill: parent
                        anchors.leftMargin: Theme.mainGridInset
                        anchors.rightMargin: Theme.mainGridInset
                        anchors.topMargin: Theme.mainUnits(8)
                        visible: root.viewMode === "grid"
                        model: root.viewMode === "grid" ? reader.entries : null
                        clip: true
                        cellWidth: Math.max(Theme.mainGridCellWidth, Theme.mainBodySize * 10 + Theme.mainUnits(16))
                        cellHeight: Math.max(Theme.mainGridCellHeight, Theme.mainBodySize * 3 + Theme.mainGridIconSize * Theme.fontScale + Theme.mainUnits(32))
                        ScrollBar.vertical: ScrollBar {}
                        delegate: Rectangle {
                            id: gridEntry
                            required property int index
                            required property string display
                            required property bool isDirectory
                            required property bool chosen
                            required property string entryId
                            required property bool canNavigate
                            required property bool canSelect
                            objectName: "grid-entry-" + index
                            ArchiveMoveTarget { id: gridMoveTarget; objectName: "moveTarget-grid-" + gridEntry.index; anchors.fill: parent; z: 20; enabled: gridEntry.isDirectory && gridEntry.canNavigate; controller: reader; dragSource: moveDrag; directoryId: reader.directoryForRow(gridEntry.index) }
                            required property string entryStatus
                            Accessible.name: display
                            Accessible.description: entryStatus
                            width: grid.cellWidth - Theme.mainGridGap
                            height: grid.cellHeight - Theme.mainGridGap
                            radius: Theme.mainControlRadius
                            color: (moveDrag.Drag.active && moveDrag.permitted && gridEntry.isDirectory && moveDrag.targetId === gridMoveTarget.directoryId) || gridEntry.chosen ? Theme.selection : gridHover.hovered ? Theme.controlHover : Theme.surface
                            border.color: Theme.border
                            HoverHandler {
                                id: gridHover
                            }
                            MouseArea {
                                id: gridPointer
                                anchors.fill: parent
                                acceptedButtons: Qt.LeftButton | Qt.RightButton
                                enabled: reader.runtimeAvailable && !reader.browsingBusy
                                property point pressedAt
                                preventStealing: true
                                property bool transferStarted: false
                                onPressed: mouse => { pressedAt = Qt.point(mouse.x, mouse.y); transferStarted = false; }
                                onPositionChanged: mouse => {
                                    if (pressed && (pressedButtons & Qt.LeftButton) && !transferStarted && Math.hypot(mouse.x - pressedAt.x, mouse.y - pressedAt.y) >= Qt.styleHints.startDragDistance) {
                                        transferStarted = true;
                                        root.beginTransfer(this, gridEntry.index, mouse);
                                    }
                                    if (transferStarted) root.continueTransfer(this, mouse);
                                }
                                onReleased: if (transferStarted) root.finishTransfer()
                                onCanceled: root.deferCancelledTransfer()
                                onClicked: function (mouse) {
                                    if (transferStarted) return;
                                    if (mouse.button === Qt.RightButton)
                                        root.entryMenuAt(gridEntry, gridEntry.index, gridEntry.entryId, gridEntry.isDirectory, gridEntry.canNavigate, mouse.x, mouse.y);
                                    else {
                                        root.selectWithModifiers(gridEntry.index, mouse.modifiers);
                                        gridEntry.forceActiveFocus(Qt.MouseFocusReason);
                                    }
                                }
                                onDoubleClicked: reader.enterDirectory(gridEntry.index)
                            }
                            SelectionCheck {
                                objectName: "gridSelection-" + gridEntry.index
                                anchors.left: parent.left
                                anchors.top: parent.top
                                anchors.margins: Theme.mainGridCheckboxInset
                                z: 2
                                visible: gridEntry.canSelect && (gridHover.hovered || gridEntry.chosen)
                                checked: gridEntry.chosen
                                enabled: reader.runtimeAvailable && !reader.browsingBusy
                                Accessible.name: qsTrId("ui.98fb7e184544").arg(gridEntry.display)
                                onClicked: reader.selectRow(gridEntry.index)
                            }
                            ColumnLayout {
                                anchors.fill: parent
                                anchors.margins: Theme.mainUnits(12)
                                spacing: Theme.mainUnits(8)
                                Image {
                                    Layout.alignment: Qt.AlignHCenter
                                    Layout.preferredWidth: Theme.mainGridIconSize * Theme.fontScale
                                    Layout.preferredHeight: Theme.mainGridIconSize * Theme.fontScale
                                    source: root.fileIcon(gridEntry.display, gridEntry.isDirectory)
                                    fillMode: Image.PreserveAspectFit
                                }
                                Text {
                                    Layout.fillWidth: true
                                    text: gridEntry.display
                                    textFormat: Text.PlainText
                                    horizontalAlignment: Text.AlignHCenter
                                    font.pixelSize: Theme.mainBodySize
                                    color: Theme.text
                                    elide: Text.ElideRight
                                }
                            }
                            Keys.onPressed: function (event) {
                                if (event.key === Qt.Key_Menu || (event.key === Qt.Key_F10 && (event.modifiers & Qt.ShiftModifier))) {
                                    root.entryMenuAt(gridEntry, gridEntry.index, gridEntry.entryId, gridEntry.isDirectory, gridEntry.canNavigate, 0, gridEntry.height);
                                    event.accepted = true;
                                } else if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter) {
                                    reader.enterDirectory(gridEntry.index);
                                    event.accepted = true;
                                } else if (event.key === Qt.Key_Space) {
                                    reader.selectRow(gridEntry.index);
                                    event.accepted = true;
                                } else if ([Qt.Key_Left, Qt.Key_Right, Qt.Key_Up, Qt.Key_Down].includes(event.key)) {
                                    const columns = Math.max(1, Math.floor(grid.width / grid.cellWidth));
                                    const delta = event.key === Qt.Key_Left ? -1 : event.key === Qt.Key_Right ? 1 : event.key === Qt.Key_Up ? -columns : columns;
                                    grid.currentIndex = Math.max(0, Math.min(grid.count - 1, gridEntry.index + delta));
                                    if (grid.currentItem)
                                        grid.currentItem.forceActiveFocus(Qt.TabFocusReason);
                                    event.accepted = true;
                                }
                            }
                            CommandTooltip {
                                objectName: "gridEntryTooltip-" + gridEntry.index
                                requested: gridHover.hovered && !gridPointer.pressed
                                suppressed: root.archiveTransferActive
                                text: gridEntry.display + "\n" + gridEntry.entryStatus
                            }
                        }
                    }
                    Rectangle {
                        objectName: "archiveLoadingOverlay"
                        anchors.fill: parent
                        color: Theme.surface
                        visible: !root.archiveOperationVisible && (reader.browsingBusy || !reader.ready || reader.total === 0)
                        opacity: 0.95
                    }
                    ColumnLayout {
                        anchors.centerIn: parent
                        width: Math.min(parent.width - 60, 460)
                        visible: !root.archiveOperationVisible && (reader.browsingBusy || !reader.ready || reader.total === 0)
                        spacing: Theme.mainUnits(16)
                        Text {
                            Layout.fillWidth: true
                            text: reader.browsingBusy ? qsTrId("ui.86b6d0d63062") : reader.ready ? qsTrId("ui.4031b1da2311") : qsTrId("ui.ced325d6e7d0")
                            color: Theme.text
                            font.pixelSize: Theme.mainPanelTitleSize
                            horizontalAlignment: Text.AlignHCenter
                        }
                        Text {
                            Layout.fillWidth: true
                            text: reader.message.length ? reader.message : qsTrId("ui.6a9e92783d20")
                            color: Theme.textSecondary
                            font.pixelSize: Theme.mainBodySize
                            wrapMode: Text.Wrap
                            horizontalAlignment: Text.AlignHCenter
                        }
                        AppButton {
                            Layout.alignment: Qt.AlignHCenter
                            visible: reader.browsingBusy
                            text: qsTrId("ui.caf06adff4f1")
                            onClicked: reader.cancel()
                        }
                    }
                }
            }
            InspectorPanel {
                objectName: "archiveInspector"
                visible: root.inspectorVisible
                Layout.preferredWidth: Math.round(Theme.inspectorWidth)
                Layout.minimumWidth: Math.round(Theme.inspectorWidth)
                Layout.maximumWidth: Math.round(Theme.inspectorWidth)
                Layout.fillHeight: true
                detail: reader.detail
                selectionCount: reader.selectedCount
                sizeFormatter: root.formatSize
                iconForFile: root.fileIcon
                canExtract: reader.ready && !reader.browsingBusy
                onExtractRequested: id => reader.prepareExtraction(false, [id])
                onCloseRequested: root.inspectorRequested = false
            }
        }
        Rectangle {
            objectName: "browserStatus"
            Layout.fillWidth: true
            implicitHeight: Math.ceil(Math.max(Theme.mainStatusbarHeight, statusActions.implicitHeight + 4))
            color: Theme.surface
            Rectangle {
                width: parent.width
                height: 1
                color: Theme.border
            }
            RowLayout {
                id: statusActions
                anchors.fill: parent
                anchors.leftMargin: root.welcomeVisible ? 14 : Theme.mainContentInset
                anchors.rightMargin: root.welcomeVisible ? 14 : Theme.mainContentInset
                spacing: Theme.mainUnits(8)
                Rectangle {
                    visible: root.welcomeVisible
                    Layout.preferredWidth: 6
                    Layout.preferredHeight: 6
                    radius: 3
                    color: Theme.accentInk
                }
                Text {
                    Layout.fillWidth: true
                    text: root.welcomeVisible ? qsTrId("ui.d43d21c6f28f") : qsTrId("ui.aeb126abb448", Number(reader.selectedCount)) + (reader.selectedCount ? qsTrId("common.separator") + root.formatSize(reader.selectedBytes) : "")
                    color: Theme.textSecondary
                    font.pixelSize: root.welcomeVisible ? 11 * Theme.fontScale : Theme.mainStatusSize
                    elide: Text.ElideRight
                }
                ArchiveAction {
                    objectName: "browserTasks"
                    taskDisclosure: true
                    labelSize: root.welcomeVisible ? 12 * Theme.fontScale : Theme.mainHeaderSize
                    glyphSize: Theme.mainTaskIconSize
                    asset: "task-list"
                    text: qsTrId("ui.76bbe94afbc9", Number(root.globalTasks.length))
                    explanation: qsTrId("ui.bba8ad73d43c").arg(root.globalActiveTaskCount)
                    onClicked: taskMenu.open()
                    CommandMenu {
                        id: taskMenu
                        objectName: "browserTaskMenu"
                        x: parent.width - width
                        height: 1
                        readonly property real maximumViewportHeight: Math.min(Theme.taskMenuMaximumHeight,
                            root.height - Theme.menuViewportInset * 2)
                        function fitViewport() {
                            // Popup layout can adjust row geometry during its own size pass.
                            // Apply the measured result after that pass, without a height binding.
                            height = Math.min(naturalContentHeight + topPadding + bottomPadding, maximumViewportHeight);
                        }
                        onNaturalContentHeightChanged: Qt.callLater(taskMenu.fitViewport)
                        onFooterHeightChanged: Qt.callLater(taskMenu.fitViewport)
                        onMaximumViewportHeightChanged: Qt.callLater(taskMenu.fitViewport)
                        onTopPaddingChanged: Qt.callLater(taskMenu.fitViewport)
                        onBottomPaddingChanged: Qt.callLater(taskMenu.fitViewport)
                        onVisibleChanged: if (visible) fitViewport()
                        y: -height - Theme.mainPopupGap
                        footerComponent: Component {
                            FocusScope {
                                implicitHeight: clearButton.implicitHeight + Theme.taskMenuFooterInset + 1
                                enabled: root.sessionManager ? root.sessionManager.hasFinishedTasks : reader.hasFinishedTasks
                                onActiveFocusChanged: if (activeFocus) clearButton.forceActiveFocus(Qt.TabFocusReason)
                                Rectangle {
                                    objectName: "taskMenuSeparator"
                                    y: Theme.taskMenuFooterInset / 2
                                    width: parent.width; height: 1; color: Theme.menuBorder
                                }
                                OperationButton {
                                    id: clearButton
                                    objectName: "clearFinishedTasks"
                                    x: 0; y: Theme.taskMenuFooterInset + 1
                                    width: parent.width
                                    text: qsTrId("tasks.clearFinished")
                                    enabled: parent.enabled
                                    Accessible.description: qsTrId("tasks.clearFinishedHint")
                                    Keys.onPressed: function (event) {
                                        if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter) {
                                            click();
                                            event.accepted = true;
                                        }
                                    }
                                    Keys.onBacktabPressed: taskMenu.focusBoundary(true)
                                    Keys.onUpPressed: taskMenu.focusBoundary(true)
                                    onClicked: {
                                        taskMenu.close();
                                        if (root.sessionManager) root.sessionManager.clearFinishedTasks();
                                        else reader.clearFinishedTasks();
                                    }
                                }
                            }
                        }
                        function restoreCurrentItem(item) {
                            if (!item)
                                return;
                            for (let index = 0; index < count; ++index) {
                                if (itemAt(index) === item) {
                                    currentIndex = index;
                                    return;
                                }
                            }
                        }
                        CommandMenuItem {
                            text: qsTrId("ui.da9688984cdc")
                            commandVisible: root.globalTasks.length === 0
                            enabled: false
                            leftPadding: topPadding
                            rightPadding: topPadding + taskMenu.viewportScrollBarWidth
                        }
                        Instantiator {
                            model: root.globalTaskModel
                            delegate: CommandMenuItem {
                                required property var taskData
                                objectName: "taskMenuItem-" + taskData.id
                                leftPadding: topPadding
                                rightPadding: topPadding + taskMenu.viewportScrollBarWidth
                                KeyNavigation.priority: KeyNavigation.BeforeItem
                                KeyNavigation.tab: taskMenu.footerItem && taskMenu.footerItem.enabled ? taskMenu.footerItem : null
                                text: taskData.archive || ""
                                accessibleLabel: root.taskLabel(taskData) + qsTrId("common.separator") + text
                                asset: taskData.kind === "check" ? "task-check" : taskData.kind === "update" ? "task-add" : taskData.kind === "move" ? "task-move" : ["export", "export-copy"].includes(taskData.kind) ? "task-export" : taskData.creating ? "task-create" : "task-extract"
                                sequenceLabel: taskData.backendId || taskData.id
                                statusLabel: root.taskStateLabel(root.taskStatus(taskData))
                                statusInk: taskData.status === "failed" ? Theme.menuDanger : ["waiting", "waiting_password"].includes(taskData.status) ? Theme.resultWarningInk : taskData.status === "cancelled" ? Theme.textSecondary : Theme.accentInk
                                onTriggered: {
                                    const identity = taskData.id;
                                    taskMenu.close();
                                    if (root.sessionManager) root.sessionManager.showTask(identity);
                                    else root.showTask(identity);
                                }
                            }
                            onObjectAdded: (index, object) => {
                                const selected = taskMenu.itemAt(taskMenu.currentIndex);
                                taskMenu.insertItem(index + 1, object);
                                taskMenu.restoreCurrentItem(selected);
                            }
                            onObjectRemoved: (index, object) => {
                                const selected = taskMenu.itemAt(taskMenu.currentIndex);
                                taskMenu.removeItem(object);
                                taskMenu.restoreCurrentItem(selected);
                            }
                        }
                        CommandMenuSeparator {
                            commandVisible: reader.browsingBusy
                        }
                        CommandMenuItem {
                            text: qsTrId("ui.caf06adff4f1")
                            commandVisible: reader.browsingBusy
                            onTriggered: reader.cancel()
                        }
                    }
                }
            }
            RowLayout {
                visible: !root.welcomeVisible
                x: parent.width / 2 - Theme.mainUnits(50)
                anchors.verticalCenter: parent.verticalCenter
                spacing: Theme.mainUnits(9)
                Image {
                    Layout.preferredWidth: Theme.mainUnits(22)
                    Layout.preferredHeight: Theme.mainUnits(22)
                    source: "qrc:/xdzip/assets/" + (Theme.dark ? "dark/" : "light/") + (reader.browsingBusy ? "activity.png" : "completed.png")
                }
                Text {
                    text: root.archiveOperationVisible ? root.archiveOperationTitle : reader.browsingBusy ? qsTrId("ui.86b6d0d63062") : reader.ready ? qsTrId("ui.332e65565b33") : qsTrId("ui.654a6e182ac7")
                    color: Theme.textSecondary
                    font.pixelSize: Theme.mainStatusSize
                }
            }
        }
    }
    BrowserBusyOverlay {
        id: archiveBusyOverlay
        namePrefix: "archiveOperation"
        x: pageColumn.x + archiveWorkspace.x
        y: pageColumn.y + archiveWorkspace.y
        width: archiveWorkspace.width
        height: archiveWorkspace.height
        busy: root.archiveOperationVisible && !root.closedPage
        cancellationPending: reader.archiveCancelling
        backdropColor: "transparent"
        archiveName: reader.title
        titleLabel: root.archiveOperationTitle
        noteLabel: reader.archiveOperation === "move" ? qsTrId("archive.operation.moveNote") : qsTrId("archive.operation.addNote")
        progressLabel: reader.archiveOperation === "move" ? qsTrId("archive.operation.moveProgress") : qsTrId("archive.operation.addProgress")
        cancelLabel: reader.archiveOperation === "move" ? qsTrId("archive.operation.cancelMove") : qsTrId("archive.operation.cancelAdd")
        onCancelRequested: reader.cancel()
        onDismissed: Qt.callLater(function () {
            if (!root.welcomeVisible && reader.ready && !reader.browsingBusy && root.visible)
                (root.viewMode === "grid" ? grid : table).forceActiveFocus(Qt.OtherFocusReason);
        })
    }
}
