pragma ComponentBehavior: Bound
import QtQuick
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
    readonly property color accentTextColor: Theme.accentText
    property bool uiReady: false
    Component.onCompleted: uiReady = true
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
    property bool darkMode: false
    property string activeTaskId: ""
    property var taskSnapshot: ({})
    property bool waitBeforeExit: false
    property bool allowExit: false
    function phaseLabel(status: string): string {
        return ({queued: qsTr("等待开始"), preparing: qsTr("检查目标与归档"), running: taskSnapshot.creating ? qsTr("读取与压缩") : qsTr("解码与写入"), verifying: qsTr("校验完整归档"),
                 waiting: qsTr("等待处理重名"), committing: qsTr("提交完整文件"), cancelling: qsTr("停止与清理"),
                 cleaning: qsTr("清理暂存"), succeeded: qsTr("完成"), failed: qsTr("失败"), cancelled: qsTr("已取消")})[status] || qsTr("准备中");
    }
    function taskStateLabel(status: string): string {
        return ({queued: qsTr("排队中"), preparing: qsTr("准备中"), running: qsTr("进行中"),
                 verifying: qsTr("校验中"), waiting: qsTr("等待处理"), committing: qsTr("保存中"),
                 cancelling: qsTr("停止中"), cleaning: qsTr("清理中"), succeeded: qsTr("已完成"),
                 failed: qsTr("未完成"), cancelled: qsTr("已取消")})[status] || qsTr("准备中");
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
            formatSize(String(Math.round(Number(taskSnapshot.writtenBytes) / seconds))) + "/s" : qsTr("等待估算");
    }
    function remainingTime(): string {
        const t = taskSnapshot;
        const seconds = Number(t.elapsedMs || 0) / 1000;
        const written = Number(t.creating ? t.decodedBytes || 0 : t.writtenBytes || 0);
        if (seconds < 2 || !written || t.status !== "running") return "—";
        return qsTr("约 %1 秒").arg(Math.ceil(Math.max(0, Number(t.declaredBytes || 0) - Number(t.decodedBytes || 0)) / (written / seconds)));
    }
    property string viewMode: "list"
    property var contextEntry: ({})
    property string contextDirectoryId: ""
    property int contextDirectoryRow: -1
    property bool contextDirectoryHasChildren: false
    property bool contextDirectoryExpanded: false
    property bool pendingDirectorySearchFocus: false
    readonly property bool contextAvailable: reader.ready && !reader.busy && contextEntry.session === reader.sessionId
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
        if (!reader.ready || reader.busy)
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
    title: reader.title.length ? reader.title + " — xdzip" : "xdzip"
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
            return (size / 1024).toFixed(1) + " KiB";
        if (size < 1073741824)
            return (size / 1048576).toFixed(1) + " MiB";
        return (size / 1073741824).toFixed(1) + " GiB";
    }
    BrowserController {
        id: reader
        objectName: "browserController"
        onReadyToClose: root.close()
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
            if (busy || root.contextEntry.session !== sessionId) {
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
        currentFile: root.taskSnapshot.current || qsTr("正在准备…")
        fraction: root.taskFraction()
        creating: root.taskSnapshot.creating || false
        totalKnown: root.taskSnapshot.totalKnown || false
        stopping: ["cancelling", "cleaning"].includes(root.taskSnapshot.status)
        taskStatus: root.taskSnapshot.status || "queued"
        phaseSummary: root.phaseLabel(taskStatus)
        errorSummary: root.taskSnapshot.error || ""
        cleanupWarning: root.taskSnapshot.cleanupWarning || false
        outcomeUncertain: root.taskSnapshot.outcomeUncertain || false
        processedSummary: taskWindow.creating ? qsTr("已读取 %1，压缩后 %2").arg(root.formatSize(root.taskSnapshot.decodedBytes || "0")).arg(root.formatSize(root.taskSnapshot.writtenBytes || "0")) : qsTr("已写入 %1，已提交 %2").arg(root.formatSize(root.taskSnapshot.writtenBytes || "0")).arg(root.formatSize(root.taskSnapshot.committedBytes || "0"))
        fileCountSummary: taskWindow.creating ? qsTr("%1 个文件，%2").arg(root.taskSnapshot.files || "0").arg(root.taskSnapshot.terminal ? qsTr("整体保存已结束") : qsTr("校验后整体保存")) : qsTr("%1 / %2 个文件，跳过 %3").arg(root.taskSnapshot.committedFiles || "0").arg(root.taskSnapshot.files || "0").arg(root.taskSnapshot.skippedFiles || "0")
        resultSummary: qsTr("完成 %1 个文件，跳过 %2，创建 %3 个目录").arg(root.taskSnapshot.committedFiles || "0").arg(root.taskSnapshot.skippedFiles || "0").arg(root.taskSnapshot.createdDirectories || "0")
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
        title: qsTr("条目详情 — xdzip")
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
            canExtract: reader.ready && !reader.busy
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
    Shortcut {
        sequence: StandardKey.Open
        enabled: reader.runtimeAvailable && !reader.busy
        onActivated: reader.chooseArchive()
    }
    Shortcut {
        sequence: StandardKey.New
        enabled: reader.runtimeAvailable && !reader.busy
        onActivated: createWindow.configure()
    }
    Shortcut {
        sequence: StandardKey.Find
        enabled: reader.queryAvailable
        onActivated: toolbar.focusSearch()
    }
    Shortcut {
        sequences: [StandardKey.Close]
        enabled: reader.ready && !reader.busy
        onActivated: reader.closeArchive()
    }
    Shortcut {
        sequence: StandardKey.SelectAll
        enabled: reader.ready && !reader.busy
        onActivated: reader.selectAll(true)
    }
    Shortcut {
        sequences: [StandardKey.Copy]
        enabled: reader.ready && !reader.busy && reader.selectedCount > 0
        onActivated: {
            entryMenu.close();
            reader.copySelection(false);
        }
    }
    CommandMenu {
        id: directoryMenu
        objectName: "browserDirectoryMenu"
        CommandMenuItem {
            text: root.contextDirectoryId.length ? qsTr("进入目录") : qsTr("根目录")
            enabled: reader.ready && !reader.busy
            onTriggered: {
                if (root.contextDirectoryId.length)
                    reader.navigateDirectory(root.contextDirectoryId);
                else
                    reader.showRoot();
            }
        }
        CommandMenuItem {
            text: root.contextDirectoryExpanded ? qsTr("收起子目录") : qsTr("展开子目录")
            enabled: reader.ready && !reader.busy && root.contextDirectoryHasChildren
            disabledReason: qsTr("该节点没有子目录")
            onTriggered: directoryTree.toggleExpanded(root.contextDirectoryRow)
        }
        CommandMenuItem {
            text: qsTr("解压此目录")
            enabled: reader.ready && !reader.busy
            onTriggered: reader.prepareExtraction(!root.contextDirectoryId.length, root.contextDirectoryId.length ? [root.contextDirectoryId] : [])
        }
        CommandMenuSeparator {}
        CommandMenuItem {
            text: qsTr("在此目录中搜索")
            asset: "search"
            enabled: reader.ready && !reader.busy
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
            text: qsTr("名称")
            checkable: true
            checked: reader.sortOrder === 1 || reader.sortOrder === 2
            enabled: reader.ready && !reader.busy
            onTriggered: reader.sortByColumn(0)
        }
        CommandMenuItem {
            text: qsTr("原始大小")
            checkable: true
            checked: reader.sortOrder === 3 || reader.sortOrder === 4
            enabled: reader.ready && !reader.busy
            onTriggered: reader.sortByColumn(1)
        }
        CommandMenuItem {
            text: qsTr("压缩后")
            checkable: true
            checked: reader.sortOrder === 5 || reader.sortOrder === 6
            enabled: reader.ready && !reader.busy
            onTriggered: reader.sortByColumn(2)
        }
        CommandMenuItem {
            text: qsTr("类型")
            checkable: true
            checked: reader.sortOrder === 9 || reader.sortOrder === 10
            enabled: reader.ready && !reader.busy
            onTriggered: reader.sortByColumn(3)
        }
        CommandMenuItem {
            text: qsTr("修改日期")
            checkable: true
            checked: reader.sortOrder === 11 || reader.sortOrder === 12
            enabled: reader.ready && !reader.busy
            onTriggered: reader.sortByColumn(4)
        }
        CommandMenuSeparator {}
        CommandMenuItem {
            text: qsTr("按归档顺序排列")
            checkable: true
            checked: reader.sortOrder === 0
            enabled: reader.ready && !reader.busy
            onTriggered: reader.sortByColumn(-1)
        }
    }
    CommandMenu {
        id: viewMenu
        objectName: "browserViewMenu"
        title: qsTr("视图")
        CommandMenuItem {
            text: qsTr("列表视图")
            checkable: true
            checked: root.viewMode === "list"
            onTriggered: root.viewMode = "list"
        }
        CommandMenuItem {
            text: qsTr("网格视图")
            checkable: true
            checked: root.viewMode === "grid"
            onTriggered: root.viewMode = "grid"
        }
        CommandMenuSeparator {}
        CommandMenuItem {
            text: qsTr("文件详情")
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
            text: qsTr("进入目录")
            asset: "folder"
            commandVisible: Boolean(root.contextEntry.directory)
            enabled: root.contextAvailable && Boolean(root.contextEntry.navigable)
            disabledReason: qsTr("该条目不具有可导航的安全目录身份")
            onTriggered: reader.enterDirectory(root.contextEntry.row)
        }
        CommandMenuItem {
            objectName: "contextProperties"
            text: qsTr("文件属性")
            enabled: root.contextAvailable
            onTriggered: {
                reader.selectForContext(root.contextEntry.row);
                if (!root.inspectorVisible)
                    root.toggleInspector();
            }
        }
        CommandMenuItem {
            text: qsTr("解压元数据预检")
            enabled: root.contextAvailable && reader.canPreflight
            onTriggered: reader.inspectSelection()
        }
        CommandMenuSeparator {}
        CommandMenuItem {
            objectName: "contextCopyName"
            text: qsTr("复制名称")
            shortcutLabel: Qt.platform.os === "osx" ? "⌘C" : "Ctrl+C"
            enabled: root.contextAvailable && reader.selectedCount > 0
            onTriggered: reader.copySelection(false)
        }
        CommandMenuItem {
            text: qsTr("复制归档内路径")
            enabled: root.contextAvailable && reader.selectedCount > 0
            onTriggered: reader.copySelection(true)
        }
        CommandMenuSeparator {}
        CommandMenuItem {
            text: qsTr("解压所选")
            enabled: root.contextAvailable && reader.selectedCount > 0
            onTriggered: reader.prepareExtraction(false)
        }
    }
    CommandMenu {
        id: emptyMenu
        objectName: "browserBackgroundMenu"
        CommandMenuItem {
            text: qsTr("全选当前目录")
            enabled: reader.ready && !reader.busy
            onTriggered: reader.selectAll(true)
        }
        CommandMenuItem {
            text: qsTr("取消选择")
            enabled: reader.ready && !reader.busy && reader.selectedCount > 0
            onTriggered: reader.selectAll(false)
        }
        CommandMenuSeparator {}
        CommandMenu {
            title: qsTr("视图")
            CommandMenuItem {
                text: qsTr("列表视图")
                checkable: true
                checked: root.viewMode === "list"
                onTriggered: root.viewMode = "list"
            }
            CommandMenuItem {
                text: qsTr("网格视图")
                checkable: true
                checked: root.viewMode === "grid"
                onTriggered: root.viewMode = "grid"
            }
        }
        CommandMenuItem {
            text: qsTr("文件详情")
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
            archiveTitle: root.welcomeVisible ? "" : reader.title
            welcome: root.welcomeVisible
            integratedTitlebar: root.integratedTitlebar
            canOpen: reader.runtimeAvailable && !reader.busy
            canClose: reader.ready && !reader.busy
            onOpenRequested: reader.chooseArchive()
            onCloseRequested: reader.closeArchive()
            onThemeRequested: root.darkMode = !root.darkMode
            onDragRequested: root.startSystemMove()
        }
        ArchiveToolbar {
            id: toolbar
            visible: !root.welcomeVisible
            Layout.fillWidth: true
            canOpen: reader.runtimeAvailable && !reader.busy
            canSelect: reader.ready && !reader.busy
            canSearch: reader.queryAvailable
            canInspect: reader.ready && !reader.busy && reader.canPreflight
            hasSelection: reader.selectedCount > 0
            onNewRequested: createWindow.configure()
            onOpenRequested: reader.chooseArchive()
            onInspectRequested: reader.inspectSelection()
            onExtractRequested: all => reader.prepareExtraction(all)
            onResetSortRequested: reader.sortByColumn(-1)
            onSearchRequested: query => reader.search(query)
            onSelectAllRequested: reader.selectAll(true)
            onClearSelectionRequested: reader.selectAll(false)
            onDetailsRequested: root.toggleInspector()
            onThemeRequested: root.darkMode = !root.darkMode
        }
        WelcomePage {
            id: welcomePage
            visible: root.welcomeVisible
            Layout.fillWidth: true
            Layout.fillHeight: true
            canOpen: reader.runtimeAvailable && !reader.busy
            canCreate: reader.runtimeAvailable && !reader.busy
            busy: reader.busy && reader.title.length > 0
            message: reader.busy ? "" : reader.message
            onOpenRequested: reader.chooseArchive()
            onCreateRequested: createWindow.configure()
            onCancelRequested: reader.cancel()
            onDragEntered: urls => {
                dropSupported = reader.canOpenDroppedUrls(urls);
                dropMessage = dropSupported ? qsTr("松开即可打开压缩文件") : qsTr("请拖入单个本地 ZIP、TAR 或 TAR.GZ 压缩文件");
            }
            onArchiveDropped: urls => reader.openDroppedUrls(urls)
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
                            text: qsTr("档案文件")
                            color: Theme.textSecondary
                            font.pixelSize: Theme.mainArchiveCaptionSize
                        }
                        Text {
                            id: archiveTitle
                            objectName: "sidebarArchiveName"
                            x: Theme.mainUnits(8)
                            y: archiveCaption.y + archiveCaption.height + Theme.mainUnits(7)
                            width: parent.width - Theme.mainUnits(16)
                            text: reader.title.length ? reader.title : qsTr("尚未打开归档")
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
                                text: reader.ready ? qsTr("%1 项").arg(reader.archiveTotal) : qsTr("安全归档解压")
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
                        enabled: reader.queryAvailable
                        onClicked: reader.showRoot()
                        Accessible.name: qsTr("根目录")
                        background: Rectangle {
                            radius: Theme.mainControlRadius
                            topLeftRadius: 0
                            bottomLeftRadius: 0
                            color: reader.location.length === 0 ? Theme.treeSelectionColor : allFiles.hovered ? Theme.treeHoverColor : "transparent"
                            border.width: 0
                            border.color: Theme.accent
                            Rectangle {
                                visible: reader.location.length === 0 || allFiles.activeFocus
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
                                text: qsTr("根目录")
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
                                if (row < 0 || !reader.ready || reader.busy)
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
                        enabled: reader.queryAvailable
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
                                Accessible.name: directoryNode.expanded ? qsTr("收起目录") : qsTr("展开目录")
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
                            Accessible.name: nodeName + qsTr("，%1 条记录").arg(entryCount)
                            Accessible.description: directoryNode.fullPath
                            background: Rectangle {
                                radius: Theme.mainControlRadius
                                topLeftRadius: 0
                                bottomLeftRadius: 0
                                color: reader.directoryId === directoryNode.nodeId ? Theme.treeSelectionColor : directoryNode.hovered ? Theme.treeHoverColor : "transparent"
                                border.width: 0
                                border.color: Theme.accent
                                Rectangle {
                                    visible: reader.directoryId === directoryNode.nodeId || (directoryNode.activeFocus && directoryNode.current)
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
                        text: qsTr("目录数量或层级超出显示限额，仍可浏览根目录或搜索后代。")
                        wrapMode: Text.Wrap
                        color: Theme.textSecondary
                        font.pixelSize: Theme.mainSecondarySize
                    }
                    Text {
                        text: qsTr("安全归档解压")
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
                    objectName: "filePaneHeader"
                    Layout.fillWidth: true
                    Breadcrumb {
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        available: reader.queryAvailable
                        entries: reader.title.length ? [
                            {
                                id: "",
                                label: reader.title,
                                current: false
                            },
                            ...(reader.breadcrumbs.length ? reader.breadcrumbs : [
                                    {
                                        id: "",
                                        label: qsTr("根目录"),
                                        current: true
                                    }
                                ])] : [
                            {
                                id: "",
                                label: qsTr("打开归档，先浏览内容"),
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
                        text: reader.ready ? qsTr("%1 项").arg(reader.total) : ""
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
                        Accessible.name: qsTr("列表视图")
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
                        Accessible.name: qsTr("网格视图")
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
                        Accessible.name: qsTr("排序方式")
                        explanation: Accessible.name
                        enabled: reader.queryAvailable
                        onClicked: {
                            const p = mapToItem(root.contentItem, width, height);
                            sortMenu.popup(root.contentItem, p.x - sortMenu.width, p.y + Theme.mainPopupGap);
                        }
                    }
                    ArchiveAction {
                        objectName: "inspectorToggle"
                        Layout.minimumWidth: Theme.mainIconButtonSize
                        Layout.preferredWidth: Theme.mainIconButtonSize
                        Layout.maximumWidth: Theme.mainIconButtonSize
                        Layout.minimumHeight: Theme.mainIconButtonSize
                        Layout.preferredHeight: Theme.mainIconButtonSize
                        Layout.maximumHeight: Theme.mainIconButtonSize
                        visible: !root.inspectorVisible
                        asset: "panel"
                        Accessible.name: qsTr("显示文件预览与属性")
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
                            model: [qsTr("名称"), qsTr("原始大小"), qsTr("压缩后"), qsTr("类型"), qsTr("修改日期")]
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
                                enabled: reader.queryAvailable
                                Accessible.role: Accessible.Button
                                Accessible.name: modelData + (reader.sortOrder === (index < 3 ? index * 2 + 1 : index * 2 + 3) ? qsTr("，升序") : reader.sortOrder === (index < 3 ? index * 2 + 2 : index * 2 + 4) ? qsTr("，降序") : "")
                                Accessible.onPressAction: if (reader.queryAvailable)
                                    reader.sortByColumn(index)
                                Keys.onReturnPressed: if (reader.queryAvailable)
                                    reader.sortByColumn(index)
                                Keys.onEnterPressed: if (reader.queryAvailable)
                                    reader.sortByColumn(index)
                                Keys.onSpacePressed: if (reader.queryAvailable)
                                    reader.sortByColumn(index)
                                TapHandler {
                                    enabled: reader.queryAvailable
                                    onTapped: reader.sortByColumn(columnHeader.index)
                                }
                                TapHandler {
                                    acceptedButtons: Qt.RightButton
                                    enabled: reader.queryAvailable
                                    onTapped: function (eventPoint) {
                                        const position = columnHeader.mapToItem(root.contentItem, eventPoint.position.x, eventPoint.position.y);
                                        sortMenu.popup(position.x, position.y);
                                    }
                                }
                                Rectangle {
                                    anchors.fill: parent
                                    color: "transparent"
                                    border.width: columnHeader.activeFocus ? 2 : 0
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
                                    enabled: reader.ready && !reader.busy && reader.entries.rowCount() > 0
                                    checkState: reader.selectedCount === 0 ? Qt.Unchecked : reader.selectedCount === reader.entries.rowCount() ? Qt.Checked : Qt.PartiallyChecked
                                    nextCheckState: () => checkState === Qt.Checked ? Qt.Unchecked : Qt.Checked
                                    Accessible.name: qsTr("选择当前目录全部文件")
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
                        readonly property real scaledRowHeight: Math.max(Theme.mainFolderRowHeight, Theme.mainTableSize * 1.6 + Theme.mainUnits(16))
                        onScaledRowHeightChanged: forceLayout()
                        Connections {
                            target: Theme
                            function onFontScaleChanged() {
                                table.forceLayout();
                            }
                        }
                        rowHeightProvider: function (row) {
                            return Math.max(reader.entries.data(reader.entries.index(row, 0), Qt.UserRole + 1) ? Theme.mainFolderRowHeight : Theme.mainFileRowHeight, table.scaledRowHeight);
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
                                anchors.topMargin: 2
                                anchors.bottomMargin: 2
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
                                enabled: reader.runtimeAvailable && !reader.busy
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
                                anchors.bottom: parent.bottom
                                width: parent.width
                                height: 1
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
                                enabled: cell.canSelect && reader.runtimeAvailable && !reader.busy
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
                            border.color: gridEntry.activeFocus ? Theme.accent : Theme.border
                            HoverHandler {
                                id: gridHover
                            }
                            MouseArea {
                                anchors.fill: parent
                                acceptedButtons: Qt.LeftButton | Qt.RightButton
                                enabled: reader.runtimeAvailable && !reader.busy
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
                                enabled: reader.runtimeAvailable && !reader.busy
                                Accessible.name: qsTr("选择 %1").arg(gridEntry.display)
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
                        anchors.fill: parent
                        color: Theme.surface
                        visible: reader.busy || !reader.ready || reader.total === 0
                        opacity: 0.95
                    }
                    ColumnLayout {
                        anchors.centerIn: parent
                        width: Math.min(parent.width - 60, 460)
                        visible: reader.busy || !reader.ready || reader.total === 0
                        spacing: Theme.mainUnits(16)
                        Text {
                            Layout.fillWidth: true
                            text: reader.busy ? qsTr("正在读取…") : reader.ready ? qsTr("没有匹配的条目") : qsTr("浏览归档，无需先解压")
                            color: Theme.text
                            font.pixelSize: Theme.mainPanelTitleSize
                            horizontalAlignment: Text.AlignHCenter
                        }
                        Text {
                            Layout.fillWidth: true
                            text: reader.message.length ? reader.message : qsTr("选择 ZIP、TAR 或 TAR.GZ 浏览内容，并解压到指定目录。")
                            color: Theme.textSecondary
                            font.pixelSize: Theme.mainBodySize
                            wrapMode: Text.Wrap
                            horizontalAlignment: Text.AlignHCenter
                        }
                        AppButton {
                            Layout.alignment: Qt.AlignHCenter
                            visible: reader.busy
                            text: qsTr("取消读取")
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
                canExtract: reader.ready && !reader.busy
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
                    text: root.welcomeVisible ? qsTr("文件在本机处理") : qsTr("已选择 %1 项").arg(reader.selectedCount) + (reader.selectedCount ? "，" + root.formatSize(reader.selectedBytes) : "")
                    color: Theme.textSecondary
                    font.pixelSize: root.welcomeVisible ? 11 * Theme.fontScale : Theme.mainStatusSize
                    elide: Text.ElideRight
                }
                ArchiveAction {
                    objectName: "browserTasks"
                    labelSize: root.welcomeVisible ? 12 * Theme.fontScale : Theme.mainHeaderSize
                    glyphSize: root.welcomeVisible ? 16 : Theme.mainUnits(24)
                    asset: "task-list"
                    text: qsTr("任务 %1").arg(reader.tasks.length)
                    explanation: qsTr("查看创建和解压任务，进行中 %1 项").arg(reader.activeTaskCount)
                    onClicked: taskMenu.open()
                    CommandMenu {
                        id: taskMenu
                        objectName: "browserTaskMenu"
                        x: parent.width - width
                        y: -implicitHeight - Theme.mainPopupGap
                        CommandMenuItem {
                            text: qsTr("暂无任务")
                            commandVisible: reader.tasks.length === 0
                            enabled: false
                        }
                        Instantiator {
                            model: reader.tasks
                            delegate: CommandMenuItem {
                                required property var modelData
                                objectName: "taskMenuItem-" + modelData.id
                                text: (modelData.creating ? qsTr("创建任务 %1") : qsTr("解压任务 %1")).arg(modelData.id)
                                asset: "task-list"
                                statusLabel: root.taskStateLabel(modelData.status)
                                statusInk: modelData.status === "failed" ? Theme.menuDanger : modelData.status === "waiting" ? Theme.resultWarningInk : modelData.status === "cancelled" ? Theme.textSecondary : Theme.accentInk
                                onTriggered: root.showTask(modelData.id)
                            }
                            onObjectAdded: (index, object) => taskMenu.insertItem(index + 1, object)
                            onObjectRemoved: (index, object) => taskMenu.removeItem(object)
                        }
                        CommandMenuSeparator { commandVisible: reader.busy }
                        CommandMenuItem {
                            text: qsTr("取消读取")
                            commandVisible: reader.busy
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
                    source: "qrc:/xdzip/assets/" + (Theme.dark ? "dark/" : "light/") + (reader.busy ? "activity.png" : "completed.png")
                }
                Text {
                    text: reader.busy ? qsTr("正在读取…") : reader.ready ? qsTr("目录已读取") : qsTr("等待打开归档")
                    color: Theme.textSecondary
                    font.pixelSize: Theme.mainStatusSize
                }
            }
        }
    }
}
