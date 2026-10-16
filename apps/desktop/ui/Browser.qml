pragma ComponentBehavior: Bound
import QtQuick
import Xdzip.Settings 1.0
import QtQuick.Controls
import QtQuick.Layouts
import QtQml.Models
import Xdzip.Native

ApplicationWindow {
    id: root
    width: Theme.mainWindowWidth
    height: Theme.mainWindowHeight
    minimumWidth: Theme.mainMinimumWidth
    minimumHeight: Theme.mainMinimumHeight
    WindowResizeEdges {
        window: root
        visible: root.windowsTitlebar && root.visibility === Window.Windowed
    }
    readonly property bool windowsTitlebar: Qt.platform.os === "windows"
    flags: windowsTitlebar ? Qt.Window | Qt.FramelessWindowHint | Qt.WindowSystemMenuHint | Qt.WindowMinMaxButtonsHint | Qt.WindowCloseButtonHint : Qt.Window
    function toggleMaximized() {
        if (visibility === Window.Maximized) showNormal();
        else showMaximized();
    }
    readonly property color accentTextColor: Theme.accentText
    property bool uiReady: false
    Component.onCompleted: {
        uiReady = true;
        Theme.dark = AppSettings.dark;
        Theme.fontScale = AppSettings.textScale;
    }
    readonly property bool welcomeVisible: !reader.ready
    property bool welcomeWasBusy: false
    onWelcomeVisibleChanged: if (uiReady && welcomeVisible) {
        inspectorWindow.close();
        Qt.callLater(() => { if (root.welcomeVisible && welcomePage.canOpen) welcomePage.focusOpen(); });
    }
    property bool inspectorRequested: true
    readonly property bool inspectorVisible: inspectorRequested && width >= Theme.inspectorCollapseWidth
    property bool integratedTitlebar: false
    property string searchSession: ""
    property bool darkMode: AppSettings.dark
    property string activeTaskId: ""
    property var taskSnapshot: ({})
    property bool waitBeforeExit: false
    property bool allowExit: false
    function phaseLabel(status: string): string {
        return ({queued: qsTrId("ui.03f07c12d5fa"), preparing: qsTrId("ui.a1b9ba49c8a2"), running: taskSnapshot.creating ? qsTrId("ui.f066ff44699e") : qsTrId("ui.8865f5103610"), verifying: qsTrId("ui.b87ef7da7959"),
                 waiting: qsTrId("ui.9a6cfa6abc53"), committing: qsTrId("ui.f5312f1e339a"), cancelling: qsTrId("ui.684398ae6c51"),
                 cleaning: qsTrId("ui.b4966391542e"), succeeded: qsTrId("ui.c0b3fbff51cc"), failed: qsTrId("ui.28384d7afd2e"), cancelled: qsTrId("ui.a37778f17c5f")})[status] || qsTrId("ui.b53be09930f4");
    }
    function taskStateLabel(status: string): string {
        return ({queued: qsTrId("ui.d6f766f2adf5"), preparing: qsTrId("ui.b53be09930f4"), running: qsTrId("ui.dc9591e56d50"),
                 verifying: qsTrId("ui.784f9b54ae2b"), waiting: qsTrId("ui.8fb3a3261b43"), committing: qsTrId("ui.a1a1720d3eb5"),
                 cancelling: qsTrId("ui.72eea71385de"), cleaning: qsTrId("ui.718b6372d7fb"), succeeded: qsTrId("ui.f28461bb49c8"),
                 failed: qsTrId("ui.6707de42c29d"), cancelled: qsTrId("ui.a37778f17c5f")})[status] || qsTrId("ui.b53be09930f4");
    }
    function showTask(id: string) {
        activeTaskId = id;
        taskSnapshot = reader.tasks.find(t => t.id === id) || {};
        taskWindow.present();
        if (taskSnapshot.promptId) conflictWindow.present();
    }
    function taskFraction(): real {
        const t = taskSnapshot;
        if (t.status === "succeeded") return 1;
        const bytes = Number(t.declaredBytes || 0);
        const files = Number(t.files || 0);
        return Math.min(.99, bytes ? Number(t.decodedBytes || 0) / bytes : files ?
            (Number(t.committedFiles || 0) + Number(t.skippedFiles || 0)) / files : 0);
    }
    function averageSpeed(): string {
        const seconds = Number(taskSnapshot.elapsedMs || 0) / 1000;
        if (taskSnapshot.terminal && seconds < 2) return "—";
        return seconds >= 2 && Number(taskSnapshot.writtenBytes || 0) > 0 ?
            formatSize(String(Math.round(Number(taskSnapshot.writtenBytes) / seconds))) + "/s" : qsTrId("ui.f4199761edc9");
    }
    function remainingTime(): string {
        const t = taskSnapshot;
        const seconds = Number(t.elapsedMs || 0) / 1000;
        const written = Number(t.creating ? t.decodedBytes || 0 : t.writtenBytes || 0);
        if (seconds < 2 || !written || t.status !== "running") return "—";
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
            extractWindow.close(); taskWindow.close(); conflictWindow.close(); quitWindow.close();
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
    title: reader.title.length ? reader.title : "xdzip"
    color: Theme.surface
    onClosing: function (close) {
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
        if (!/^\d+$/.test(value)) return "—";
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
        onReadyToClose: root.close()
        onArchiveOpened: path => AppSettings.rememberArchive(path)
        onPreflightReady: preflightWindow.present()
        onExtractionReady: extractWindow.configure()
        onCreationStarted: id => { createWindow.close(); root.showTask(id); }
        onExtractionStarted: id => { extractWindow.close(); root.showTask(id); }
        onTasksChanged: {
            root.taskSnapshot = tasks.find(t => t.id === root.activeTaskId) || {};
            if (root.taskSnapshot.promptId && !conflictWindow.visible) conflictWindow.present();
            if (!root.taskSnapshot.promptId) conflictWindow.close();
            if (root.waitBeforeExit && activeTaskCount === 0) { root.allowExit = true; root.close(); }
        }
        onChanged: {
            if (root.welcomeVisible && !busy && root.welcomeWasBusy) Qt.callLater(() => { if (root.welcomeVisible && welcomePage.canOpen) welcomePage.focusOpen(); });
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
    CreateWindow { id: createWindow; owner: root; controller: reader }
    ExtractWindow { id: extractWindow; owner: root; controller: reader }
    TaskWindow {
        id: taskWindow
        owner: root
        archiveName: root.taskSnapshot.archive || reader.extractionIntent.archive || ""
        destination: root.taskSnapshot.target || reader.extractionIntent.target || ""
        currentFile: root.taskSnapshot.current || qsTrId("ui.f07335163e36")
        fraction: root.taskFraction()
        creating: root.taskSnapshot.creating || false
        totalKnown: root.taskSnapshot.totalKnown || false
        stopping: ["cancelling", "cleaning"].includes(root.taskSnapshot.status)
        taskStatus: root.taskSnapshot.status || "queued"
        phaseSummary: root.phaseLabel(taskStatus)
        errorSummary: root.taskSnapshot.error || ""
        cleanupWarning: root.taskSnapshot.cleanupWarning || false
        outcomeUncertain: root.taskSnapshot.outcomeUncertain || false
        processedSummary: taskWindow.creating ? qsTrId("ui.a1c9c21cfbef").arg(root.formatSize(root.taskSnapshot.decodedBytes || "0")).arg(root.formatSize(root.taskSnapshot.writtenBytes || "0")) : qsTrId("ui.8bce13df4816").arg(root.formatSize(root.taskSnapshot.writtenBytes || "0")).arg(root.formatSize(root.taskSnapshot.committedBytes || "0"))
        fileCountSummary: taskWindow.creating ? qsTrId("ui.1f43814c240b").arg(root.taskSnapshot.files || "0").arg(root.taskSnapshot.terminal ? qsTrId("ui.b61825c80ed6") : qsTrId("ui.483f8dd01321")) : qsTrId("ui.ed52971e9256").arg(root.taskSnapshot.committedFiles || "0").arg(root.taskSnapshot.files || "0").arg(root.taskSnapshot.skippedFiles || "0")
        resultSummary: qsTrId("ui.51fe7c80c764").arg(root.taskSnapshot.committedFiles || "0").arg(root.taskSnapshot.skippedFiles || "0").arg(root.taskSnapshot.createdDirectories || "0")
        resultTask: root.taskSnapshot
        speedSummary: root.averageSpeed()
        remainingSummary: root.remainingTime()
        onCancelRequested: reader.cancelTask(root.activeTaskId)
        onOpenOutputRequested: reader.openOutput(root.activeTaskId)
        onConflictRequested: conflictWindow.present()
    }
    ConflictWindow {
        id: conflictWindow
        owner: root; task: root.taskSnapshot; sizeFormatter: root.formatSize
        onDecision: (policy, all) => { if (reader.decideTask(root.activeTaskId, root.taskSnapshot.promptId || "", policy, all)) conflictWindow.close(); }
    }
    QuitWindow {
        id: quitWindow
        owner: root
        onWaitRequested: { root.waitBeforeExit = true; close(); }
        onCancelAndQuitRequested: { stopping = true; root.allowExit = true; reader.requestClose(); }
    }
    PreflightWindow {
        id: preflightWindow
        owner: root
        report: reader.preflightReport
    }
    Window {
        id: inspectorWindow
        objectName: "inspectorWindow"
        transientParent: root
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
            if (!createWindow.visible && root.welcomeVisible) welcomePage.focusCreate();
        }
    }
    SettingsWindow { id: settingsWindow; owner: root }
    Connections {
        target: AppSettings
        function onAppearanceChanged() {
            root.darkMode = AppSettings.dark;
            Theme.fontScale = AppSettings.textScale;
        }
    }
    Shortcut {
        sequence: StandardKey.Preferences
        onActivated: settingsWindow.configure()
    }
    Shortcut {
        sequence: StandardKey.Open
        enabled: reader.runtimeAvailable && !reader.browsingBusy
        onActivated: reader.chooseArchive(root)
    }
    Shortcut {
        sequence: StandardKey.New
        enabled: reader.runtimeAvailable && !reader.browsingBusy
        onActivated: createWindow.configure()
    }
    Shortcut {
        sequence: StandardKey.Find
        enabled: reader.browserQueryAvailable
        onActivated: toolbar.focusSearch()
    }
    Shortcut {
        sequences: [StandardKey.Close]
        enabled: reader.ready && !reader.browsingBusy
        onActivated: reader.closeArchive()
    }
    Shortcut {
        sequence: StandardKey.SelectAll
        enabled: reader.ready && !reader.browsingBusy
        onActivated: reader.selectAll(true)
    }
    Shortcut {
        sequences: [StandardKey.Copy]
        enabled: reader.ready && !reader.browsingBusy && reader.selectedCount > 0
        onActivated: {
            entryMenu.close();
            reader.copySelection(false);
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
            text: root.contextDirectoryExpanded ? qsTrId("ui.9c9c21cc25c0") : qsTrId("ui.521dbd520244")
            enabled: reader.ready && !reader.browsingBusy && root.contextDirectoryHasChildren
            disabledReason: qsTrId("ui.fa58108c4823")
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
        anchors.fill: parent
        spacing: 0
        ArchiveTabs {
            Layout.fillWidth: true
            archiveTitle: reader.title
            integratedTitlebar: root.integratedTitlebar
            windowsControls: root.windowsTitlebar
            maximized: root.visibility === Window.Maximized
            canOpen: reader.runtimeAvailable && !reader.browsingBusy
            canClose: reader.ready && !reader.browsingBusy
            onOpenRequested: reader.chooseArchive(root)
            onCloseRequested: reader.closeArchive()
            onSettingsRequested: settingsWindow.configure()
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
            canInspect: reader.ready && !reader.browsingBusy && reader.canPreflight
            hasSelection: reader.selectedCount > 0
            onNewRequested: createWindow.configure()
            onOpenRequested: reader.chooseArchive(root)
            onInspectRequested: reader.inspectSelection()
            onExtractRequested: all => reader.prepareExtraction(all)
            onResetSortRequested: reader.sortByColumn(-1)
            onSearchRequested: query => reader.search(query)
            onSelectAllRequested: reader.selectAll(true)
            onClearSelectionRequested: reader.selectAll(false)
            onDetailsRequested: root.toggleInspector()
            onSettingsRequested: settingsWindow.configure()
        }
        WelcomePage {
            id: welcomePage
            visible: root.welcomeVisible
            Layout.fillWidth: true
            Layout.fillHeight: true
            canOpen: reader.runtimeAvailable && !reader.browsingBusy
            canCreate: reader.runtimeAvailable && !reader.browsingBusy
            busy: reader.browsingBusy && reader.title.length > 0
            openingName: reader.title
            message: reader.browsingBusy ? "" : reader.message || AppSettings.error
            onOpenRequested: reader.chooseArchive(root)
            onCreateRequested: createWindow.configure()
            onCancelRequested: reader.cancel()
            onDragEntered: urls => {
                dropSupported = reader.canOpenDroppedUrls(urls);
                dropMessage = dropSupported ? qsTrId("ui.dc3e93fbf81a") : qsTrId("ui.a3c9bc4e63f7");
            }
            onArchiveDropped: urls => {
                if (reader.openDroppedUrls(urls)) {
                    // Activate after DropArea has accepted the drag and Finder has ended it.
                    Qt.callLater(function () { root.raise(); root.requestActivate(); });
                }
            }
        }
        RowLayout {
            objectName: "archiveWorkspace"
            visible: !root.welcomeVisible
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
                                text: reader.ready ? qsTrId("ui.569488175a99", Number(reader.archiveTotal)) : qsTrId("ui.7fe646004f2a")
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
                            color: reader.location.length === 0 ? Theme.treeSelectionColor : allFiles.hovered ? Theme.treeHoverColor : "transparent"
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
                                root.contextDirectoryExpanded = false;
                                const position = allFiles.mapToItem(root.contentItem, eventPoint.position.x, eventPoint.position.y);
                                directoryMenu.popup(position.x, position.y);
                            }
                        }
                        Keys.onPressed: function (event) {
                            if (event.key === Qt.Key_Menu || event.key === Qt.Key_F10 && event.modifiers & Qt.ShiftModifier) {
                                root.contextDirectoryId = "";
                                root.contextDirectoryRow = -1;
                                root.contextDirectoryHasChildren = false;
                                root.contextDirectoryExpanded = false;
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
                                color: reader.directoryId === directoryNode.nodeId ? Theme.treeSelectionColor : directoryNode.hovered ? Theme.treeHoverColor : "transparent"
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
                    Text {
                        text: qsTrId("ui.7fe646004f2a")
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
                    actionInset: root.inspectorVisible ? 0 : Theme.mainIconButtonSize + Theme.mainUnits(10)
                    objectName: "filePaneHeader"
                    Layout.fillWidth: true
                    Breadcrumb {
                        Layout.fillWidth: true
                        Layout.fillHeight: true
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
                    Text {
                        text: reader.ready ? qsTrId("ui.569488175a99", Number(reader.total)) : ""
                        color: Theme.textSecondary
                        font.pixelSize: Theme.mainSummarySize
                    }
                    ArchiveAction {
                        id: viewButton
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
                        parent: fileHeader
                        anchors.right: parent.right
                        anchors.rightMargin: fileHeader.rightInset
                        anchors.verticalCenter: parent.verticalCenter
                        width: Theme.mainIconButtonSize
                        height: Theme.mainIconButtonSize
                        visible: !root.inspectorVisible
                        asset: "panel"
                        Accessible.name: qsTrId("ui.749ca86aef0e")
                        explanation: Accessible.name
                        onClicked: root.toggleInspector()
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
                                height: parent.parent.height
                                leftPadding: table.columnLeftInset(index, true)
                                rightPadding: table.columnRightInset(index)
                                horizontalAlignment: index === 1 || index === 2 ? Text.AlignRight : Text.AlignLeft
                                verticalAlignment: Text.AlignVCenter
                                objectName: "sort-column-" + index
                                text: modelData
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
                                    onTapped: reader.sortByColumn(columnHeader.index)
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
                                    anchors.verticalCenter: parent.verticalCenter
                                    x: columnHeader.index === 0 ? Theme.mainNameHeaderInset + Theme.mainUnits(44) * Theme.fontScale : Theme.mainUnits(4)
                                    text: reader.sortOrder % 2 === 1 ? "↑" : "↓"
                                    visible: reader.sortOrder > 0 && (reader.sortOrder < 9 ? Math.floor((reader.sortOrder - 1) / 2) : Math.floor((reader.sortOrder - 3) / 2)) === columnHeader.index
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
                    TableView {
                        id: table
                        visible: root.viewMode === "list"
                        anchors.fill: parent
                        clip: true
                        property int hoveredRow: -1
                        HoverHandler {
                            enabled: root.viewMode === "list"
                            onPointChanged: table.hoveredRow = table.cellAtPosition(point.position, true).y
                            onHoveredChanged: if (!hovered) table.hoveredRow = -1
                        }
                        model: reader.entries
                        function columnSize(column: int): real {
                            const scale = Math.max(1, Theme.fontScale);
                            const other = [Theme.mainOriginalColumnWidth, Theme.mainPackedColumnWidth, Theme.mainTypeColumnWidth, Theme.mainDateColumnWidth];
                            return column === 0 ? Math.max(Theme.mainUnits(250), width - other.reduce((a, b) => a + b, 0) * scale) : other[column - 1] * scale;
                        }
                        function columnLeftInset(column, header) {
                            return column === 0 ? (header ? Theme.mainNameHeaderInset : Theme.mainNameTextInset) : column === 3 ? Theme.mainTypeColumnInset : column === 4 ? Theme.mainDateColumnInset : 0;
                        }
                        function columnRightInset(column) {
                            return column === 1 ? Theme.mainOriginalColumnInset : column === 2 ? Theme.mainPackedColumnInset : 0;
                        }
                        columnWidthProvider: columnSize
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
                                color: cell.chosen ? Theme.selection : table.hoveredRow === cell.row ? Theme.controlHover : Theme.surface
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
                                anchors.fill: parent
                                acceptedButtons: Qt.LeftButton | Qt.RightButton
                                enabled: reader.runtimeAvailable && !reader.browsingBusy
                                onClicked: function (mouse) {
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
                                requested: hover.hovered && cell.column === 0
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
                            required property string entryStatus
                            Accessible.name: display
                            Accessible.description: entryStatus
                            width: grid.cellWidth - Theme.mainGridGap
                            height: grid.cellHeight - Theme.mainGridGap
                            radius: Theme.mainControlRadius
                            color: gridEntry.chosen ? Theme.selection : gridHover.hovered ? Theme.controlHover : Theme.surface
                            border.color: Theme.border
                            HoverHandler {
                                id: gridHover
                            }
                            MouseArea {
                                anchors.fill: parent
                                acceptedButtons: Qt.LeftButton | Qt.RightButton
                                enabled: reader.runtimeAvailable && !reader.browsingBusy
                                onClicked: function (mouse) {
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
                                requested: gridHover.hovered
                                text: gridEntry.display + "\n" + gridEntry.entryStatus
                            }
                        }
                    }
                    Rectangle {
                        objectName: "archiveLoadingOverlay"
                        anchors.fill: parent
                        color: Theme.surface
                        visible: reader.browsingBusy || !reader.ready || reader.total === 0
                        opacity: 0.95
                    }
                    ColumnLayout {
                        anchors.centerIn: parent
                        width: Math.min(parent.width - 60, 460)
                        visible: reader.browsingBusy || !reader.ready || reader.total === 0
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
            implicitHeight: root.welcomeVisible ? Math.max(Theme.welcomeStatusbarHeight, statusActions.implicitHeight + 4) : Math.round(Math.max(Theme.mainStatusHeight, Theme.mainStatusSize * 1.6 + Theme.mainUnits(16)))
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
                    labelSize: root.welcomeVisible ? 12 * Theme.fontScale : Theme.mainHeaderSize
                    glyphSize: root.welcomeVisible ? 16 : Theme.mainUnits(24)
                    asset: "task-list"
                    text: qsTrId("ui.76bbe94afbc9", Number(reader.tasks.length))
                    explanation: qsTrId("ui.bba8ad73d43c").arg(reader.activeTaskCount)
                    onClicked: taskMenu.open()
                    CommandMenu {
                        id: taskMenu
                        objectName: "browserTaskMenu"
                        x: parent.width - width
                        y: -implicitHeight - Theme.mainPopupGap
                        CommandMenuItem {
                            text: qsTrId("ui.da9688984cdc")
                            commandVisible: reader.tasks.length === 0
                            enabled: false
                        }
                        Instantiator {
                            model: reader.tasks
                            delegate: CommandMenuItem {
                                required property var modelData
                                objectName: "taskMenuItem-" + modelData.id
                                text: (modelData.creating ? qsTrId("ui.9a63e2cb4339") : qsTrId("ui.fb4b66729449")).arg(modelData.id)
                                asset: "task-list"
                                statusLabel: root.taskStateLabel(modelData.status)
                                statusInk: modelData.status === "failed" ? Theme.menuDanger : modelData.status === "waiting" ? Theme.resultWarningInk : modelData.status === "cancelled" ? Theme.textSecondary : Theme.accentInk
                                onTriggered: root.showTask(modelData.id)
                            }
                            onObjectAdded: (index, object) => taskMenu.insertItem(index + 1, object)
                            onObjectRemoved: (index, object) => taskMenu.removeItem(object)
                        }
                        CommandMenuSeparator { commandVisible: reader.browsingBusy }
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
                    text: reader.browsingBusy ? qsTrId("ui.86b6d0d63062") : reader.ready ? qsTrId("ui.332e65565b33") : qsTrId("ui.654a6e182ac7")
                    color: Theme.textSecondary
                    font.pixelSize: Theme.mainStatusSize
                }
            }
        }
    }
}
