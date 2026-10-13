pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQml.Models
import Xdzip.Native

ApplicationWindow {
    id: root
    width: 1280
    height: 777
    minimumWidth: 1040
    minimumHeight: 680
    readonly property color accentTextColor: Theme.accentText
    property bool inspectorRequested: true
    readonly property bool inspectorVisible: inspectorRequested && width >= Theme.inspectorCollapseWidth
    property bool integratedTitlebar: false
    property string searchSession: ""
    property bool darkMode: false
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
        close.accepted = reader.requestClose();
        if (close.accepted) {
            preflightWindow.close();
            inspectorWindow.close();
        }
    }

    function fileIcon(name: string, directory: bool): string {
        const suffix = name.toLowerCase().split(".").pop();
        // Extension hints only; content detection belongs to the archive service.
        let kind = "file-text";
        if (directory)
            kind = "folder";
        else if (suffix === "pdf")
            kind = "file-pdf";
        else if (suffix === "md")
            kind = "file-markdown";
        else if (["jpg", "jpeg", "png", "gif", "webp", "avif", "heic"].includes(suffix))
            kind = "image-file";
        else if (["js", "ts", "rs", "cpp", "h", "json", "xml", "gpx", "py"].includes(suffix))
            kind = "file-code";
        return "qrc:/xdzip/assets/" + (Theme.dark ? "dark/" : "light/") + kind + ".png";
    }
    // Human-readable display only; exact integer strings remain in the model.
    function formatSize(value: string): string {
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
        onChanged: {
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
            onCloseRequested: inspectorWindow.close()
        }
    }
    Shortcut {
        sequence: StandardKey.Open
        enabled: reader.runtimeAvailable && !reader.busy
        onActivated: reader.chooseArchive()
    }
    Shortcut {
        sequence: StandardKey.Find
        enabled: reader.ready && !reader.busy
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
        onActivated: reader.selectAllOnPage(true)
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
            text: root.contextDirectoryId.length ? qsTr("进入目录") : qsTr("全部文件")
            enabled: reader.ready && !reader.busy
            onTriggered: {
                if (root.contextDirectoryId.length)
                    reader.navigateDirectory(root.contextDirectoryId);
                else
                    reader.showAll();
            }
        }
        CommandMenuItem {
            text: root.contextDirectoryExpanded ? qsTr("收起子目录") : qsTr("展开子目录")
            enabled: reader.ready && !reader.busy && root.contextDirectoryHasChildren
            disabledReason: qsTr("该节点没有子目录")
            onTriggered: directoryTree.toggleExpanded(root.contextDirectoryRow)
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
                    reader.showAll();
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
            text: qsTr("压缩方法")
            checkable: true
            checked: reader.sortOrder === 7 || reader.sortOrder === 8
            enabled: reader.ready && !reader.busy
            onTriggered: reader.sortByColumn(3)
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
            visible: Boolean(root.contextEntry.directory)
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
            enabled: root.contextAvailable && reader.selectedCount > 0
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
            enabled: false
            disabledReason: qsTr("安全写入层尚未实现；可先执行元数据预检")
        }
    }
    CommandMenu {
        id: emptyMenu
        objectName: "browserBackgroundMenu"
        CommandMenuItem {
            text: qsTr("全选当前页")
            enabled: reader.ready && !reader.busy
            onTriggered: reader.selectAllOnPage(true)
        }
        CommandMenuItem {
            text: qsTr("取消选择")
            enabled: reader.ready && !reader.busy && reader.selectedCount > 0
            onTriggered: reader.selectAllOnPage(false)
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
            archiveTitle: reader.title
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
            Layout.fillWidth: true
            canOpen: reader.runtimeAvailable && !reader.busy
            canSearch: reader.ready && !reader.busy
            canInspect: reader.ready && !reader.busy && reader.selectedCount > 0
            hasSelection: reader.selectedCount > 0
            onOpenRequested: reader.chooseArchive()
            onInspectRequested: reader.inspectSelection()
            onResetSortRequested: reader.sortByColumn(-1)
            onSearchRequested: query => reader.search(query)
            onSelectAllRequested: reader.selectAllOnPage(true)
            onClearSelectionRequested: reader.selectAllOnPage(false)
            onDetailsRequested: root.toggleInspector()
            onThemeRequested: root.darkMode = !root.darkMode
        }
        RowLayout {
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
                        implicitHeight: Math.max(Theme.mainArchiveCardHeight, archiveSummary.y + archiveSummary.height + 32)
                        Text {
                            id: archiveCaption
                            x: 8
                            y: 26
                            text: "ARCHIVE"
                            color: Theme.textSecondary
                            font.pixelSize: Theme.mainArchiveCaptionSize
                        }
                        Text {
                            id: archiveTitle
                            objectName: "sidebarArchiveName"
                            x: 8
                            y: archiveCaption.y + archiveCaption.height + 7
                            width: parent.width - 16
                            text: reader.title.length ? reader.title : qsTr("尚未打开归档")
                            textFormat: Text.PlainText
                            elide: Text.ElideMiddle
                            font.pixelSize: Theme.mainArchiveTitleSize
                            font.weight: Font.DemiBold
                            color: Theme.text
                        }
                        RowLayout {
                            id: archiveSummary
                            x: 8
                            y: archiveTitle.y + archiveTitle.height + 8
                            spacing: 12
                            Rectangle {
                                implicitWidth: 40
                                implicitHeight: Math.max(24, Theme.mainSummarySize * 1.6)
                                radius: 5
                                color: Theme.selection
                                Text {
                                    anchors.centerIn: parent
                                    text: "ZIP"
                                    font.pixelSize: Theme.mainSummarySize
                                    color: Theme.accentInk
                                }
                            }
                            Text {
                                text: reader.ready ? qsTr("%1 项").arg(reader.archiveTotal) : qsTr("只读浏览")
                                color: Theme.textSecondary
                                font.pixelSize: Theme.mainSummarySize
                            }
                        }
                        Rectangle {
                            x: 8
                            width: parent.width - 16
                            y: parent.height - 16
                            height: 1
                            color: Theme.border
                        }
                    }
                    Button {
                        id: allFiles
                        objectName: "directoryRoot"
                        Layout.fillWidth: true
                        implicitHeight: Math.ceil(Math.max(Theme.treeRowMinHeight, Theme.treeBodyFontSize * Theme.fontScale * 1.6 + 16, Theme.treeDisclosureHitMinHeight * Theme.fontScale + 16))
                        leftPadding: 12
                        rightPadding: 12
                        enabled: reader.ready && !reader.busy
                        onClicked: reader.showAll()
                        Accessible.name: qsTr("全部文件")
                        background: Rectangle {
                            radius: Theme.controlRadius
                            color: reader.location.length === 0 ? Theme.treeSelectionColor : allFiles.hovered ? Theme.treeHoverColor : "transparent"
                            border.width: allFiles.activeFocus ? 2 : 0
                            border.color: Theme.accent
                            Rectangle {
                                visible: reader.location.length === 0
                                width: 3
                                height: parent.height - 6
                                y: 3
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
                                text: qsTr("全部文件")
                                elide: Text.ElideRight
                                color: Theme.text
                                font.pixelSize: Theme.bodySize
                                font.bold: true
                            }
                            Text {
                                text: reader.ready ? reader.archiveTotal : "—"
                                color: Theme.textSecondary
                                font.pixelSize: Theme.secondarySize
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
                        enabled: reader.ready && !reader.busy
                        columnWidthProvider: function (column) {
                            return Math.max(width, reader.navigationWidth * Math.max(1, Theme.fontScale));
                        }
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
                            readonly property bool hiddenCurrentBranch: reader.directoryId !== nodeId && !expanded && reader.directoryAncestors(reader.directoryId).includes(nodeId)
                            objectName: "directory-" + nodeId
                            implicitHeight: Math.ceil(Math.max(Theme.treeRowMinHeight, Theme.treeBodyFontSize * Theme.fontScale * 1.6 + 16, Theme.treeDisclosureHitMinHeight * Theme.fontScale + 16))
                            implicitWidth: directoryTree.width
                            indentation: Theme.treeDepthIndent
                            leftMargin: 0
                            leftPadding: Theme.treeContentLeftInset + nodeDepth * Theme.treeDepthIndent + Theme.treeDisclosureHitWidth * Theme.fontScale
                            topPadding: 0
                            bottomPadding: 0
                            rightPadding: 8
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
                                    radius: 4
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
                            ToolTip {
                                visible: directoryNode.hovered
                                width: Math.min(360, implicitWidth)
                                contentItem: Text {
                                    text: directoryNode.fullPath
                                    textFormat: Text.PlainText
                                    color: Theme.text
                                    font.pixelSize: Theme.secondarySize
                                    wrapMode: Text.WrapAnywhere
                                    maximumLineCount: 8
                                    elide: Text.ElideMiddle
                                }
                                background: Rectangle {
                                    color: Theme.surface
                                    border.color: Theme.border
                                    radius: Theme.controlRadius
                                }
                            }
                            background: Rectangle {
                                radius: Theme.controlRadius
                                color: reader.directoryId === directoryNode.nodeId ? Theme.treeSelectionColor : directoryNode.hovered ? Theme.treeHoverColor : "transparent"
                                border.width: directoryNode.activeFocus || (directoryTree.activeFocus && directoryNode.current) ? 2 : 0
                                border.color: Theme.accent
                                Rectangle {
                                    visible: reader.directoryId === directoryNode.nodeId
                                    width: 3
                                    height: parent.height - 6
                                    y: 3
                                    radius: 2
                                    color: Theme.accent
                                }
                                Repeater {
                                    model: directoryNode.continuationFlags
                                    Rectangle {
                                        required property bool modelData
                                        required property int index
                                        visible: modelData
                                        x: Theme.treeContentLeftInset + Theme.treeDisclosureHitWidth / 2 + index * Theme.treeDepthIndent
                                        width: 1
                                        height: directoryNode.height
                                        color: Theme.treeGuideColor
                                    }
                                }
                                Rectangle {
                                    visible: directoryNode.nodeDepth > 0
                                    x: Theme.treeContentLeftInset + Theme.treeDisclosureHitWidth / 2 + (directoryNode.nodeDepth - 1) * Theme.treeDepthIndent
                                    width: 1
                                    height: directoryNode.lastSibling ? directoryNode.height / 2 : directoryNode.height
                                    color: Theme.treeGuideColor
                                    Rectangle {
                                        y: directoryNode.lastSibling ? parent.height : directoryNode.height / 2
                                        width: 10
                                        height: 1
                                        color: Theme.treeGuideColor
                                    }
                                }
                            }
                            contentItem: RowLayout {
                                spacing: 8
                                Image {
                                    objectName: "directoryFolder-" + directoryNode.nodeId
                                    source: root.fileIcon("", true)
                                    Layout.preferredWidth: Theme.treeFolderCanvas * Theme.fontScale
                                    Layout.preferredHeight: Theme.treeFolderCanvas * Theme.fontScale
                                    fillMode: Image.PreserveAspectFit
                                }
                                Text {
                                    Layout.fillWidth: true
                                    Layout.rightMargin: countOverlay.width
                                    text: directoryNode.nodeName
                                    textFormat: Text.PlainText
                                    elide: Text.ElideRight
                                    color: Theme.text
                                    font.pixelSize: Theme.bodySize
                                }
                            }
                            Rectangle {
                                id: countOverlay
                                x: Math.max(0, Math.min(directoryNode.width - width, directoryTree.contentX + directoryTree.width - width))
                                width: Math.max(32 * Theme.fontScale, countText.implicitWidth + 12) + (directoryNode.hiddenCurrentBranch ? 24 : 0)
                                height: directoryNode.height
                                color: reader.directoryId === directoryNode.nodeId ? Theme.treeSelectionColor : directoryNode.hovered ? Theme.treeHoverColor : Theme.treeSurfaceColor
                                Image {
                                    objectName: "directoryBranchMarker-" + directoryNode.nodeId
                                    anchors.left: parent.left
                                    anchors.verticalCenter: parent.verticalCenter
                                    width: 24
                                    height: 24
                                    visible: directoryNode.hiddenCurrentBranch
                                    source: "qrc:/xdzip/assets/" + (Theme.dark ? "dark/" : "light/") + "tree-active-branch.png"
                                }
                                Text {
                                    id: countText
                                    anchors.right: parent.right
                                    anchors.rightMargin: 8
                                    anchors.verticalCenter: parent.verticalCenter
                                    text: directoryNode.entryCount
                                    color: Theme.textSecondary
                                    font.pixelSize: Theme.secondarySize
                                }
                            }
                        }
                    }
                    Text {
                        Layout.fillWidth: true
                        visible: reader.navigationLimited
                        text: qsTr("目录数量或层级超出显示限额，仍可使用全部文件和搜索。")
                        wrapMode: Text.Wrap
                        color: Theme.textSecondary
                        font.pixelSize: Theme.secondarySize
                    }
                    Text {
                        text: qsTr("只读浏览")
                        color: Theme.textSecondary
                        font.pixelSize: Theme.secondarySize
                    }
                }
            }
            ColumnLayout {
                Layout.fillWidth: true
                Layout.fillHeight: true
                spacing: 0
                PaneHeader {
                    objectName: "filePaneHeader"
                    Layout.fillWidth: true
                    Breadcrumb {
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        available: reader.ready && !reader.busy
                        entries: reader.title.length ? [
                            {
                                id: "",
                                label: reader.title,
                                current: false
                            },
                            ...(reader.breadcrumbs.length ? reader.breadcrumbs : [
                                    {
                                        id: "",
                                        label: qsTr("全部文件"),
                                        current: true
                                    }
                                ])] : [
                            {
                                id: "",
                                label: qsTr("打开 ZIP，先浏览内容"),
                                current: true
                            }
                        ]
                        onActivated: function (nodeId) {
                            if (nodeId.length)
                                reader.navigateDirectory(nodeId);
                            else
                                reader.showAll();
                        }
                    }
                    Text {
                        text: reader.ready ? qsTr("%1 项").arg(reader.total) : ""
                        color: Theme.textSecondary
                        font.pixelSize: Theme.mainSummarySize
                    }
                    ArchiveAction {
                        objectName: "browserViewMenuButton"
                        asset: root.viewMode === "list" ? "view-list-selected" : "view-list"
                        Layout.preferredWidth: Theme.mainViewButtonWidth
                        leftPadding: 8
                        rightPadding: 8
                        selected: root.viewMode === "list"
                        Accessible.name: qsTr("列表视图")
                        explanation: Accessible.name
                        onClicked: root.viewMode = "list"
                        TapHandler {
                            acceptedButtons: Qt.RightButton
                            onTapped: viewMenu.popup()
                        }
                    }
                    ArchiveAction {
                        objectName: "browserGridButton"
                        asset: root.viewMode === "grid" ? "view-grid-selected" : "view-grid"
                        leftPadding: 5
                        rightPadding: 5
                        selected: root.viewMode === "grid"
                        Accessible.name: qsTr("网格视图")
                        explanation: Accessible.name
                        onClicked: root.viewMode = "grid"
                    }
                    ArchiveAction {
                        objectName: "browserSortMenuButton"
                        asset: "sort"
                        leftPadding: 2
                        rightPadding: 2
                        glyphSize: 28
                        Accessible.name: qsTr("排序方式")
                        explanation: Accessible.name
                        enabled: reader.ready && !reader.busy
                        onClicked: sortMenu.popup()
                    }
                    ArchiveAction {
                        objectName: "inspectorToggle"
                        visible: !root.inspectorVisible
                        asset: "panel"
                        Accessible.name: qsTr("显示文件预览与属性")
                        explanation: Accessible.name
                        onClicked: root.toggleInspector()
                    }
                }
                Rectangle {
                    objectName: "fileTableHeader"
                    Layout.fillWidth: true
                    implicitHeight: Math.round(Math.max(Theme.mainTableHeaderHeight, Theme.mainHeaderSize * 1.6 + 12))
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
                                activeFocusOnTab: index < 3
                                enabled: index >= 3 || (reader.ready && !reader.busy)
                                Accessible.role: index < 3 ? Accessible.Button : Accessible.StaticText
                                Accessible.name: modelData + (reader.sortOrder === index * 2 + 1 ? qsTr("，升序") : reader.sortOrder === index * 2 + 2 ? qsTr("，降序") : "")
                                Accessible.onPressAction: if (index < 3)
                                    reader.sortByColumn(index)
                                Keys.onReturnPressed: if (index < 3)
                                    reader.sortByColumn(index)
                                Keys.onEnterPressed: if (index < 3)
                                    reader.sortByColumn(index)
                                Keys.onSpacePressed: if (index < 3)
                                    reader.sortByColumn(index)
                                TapHandler {
                                    enabled: columnHeader.index < 3 && reader.ready && !reader.busy
                                    onTapped: reader.sortByColumn(columnHeader.index)
                                }
                                TapHandler {
                                    acceptedButtons: Qt.RightButton
                                    enabled: reader.ready && !reader.busy
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
                                    x: columnHeader.index === 0 ? Theme.mainNameHeaderInset + 44 * Theme.fontScale : 4
                                    text: reader.sortOrder % 2 === 1 ? "↑" : "↓"
                                    visible: columnHeader.index < 3 && reader.sortOrder > 0 && Math.floor((reader.sortOrder - 1) / 2) === columnHeader.index
                                    color: Theme.accentInk
                                    font.pixelSize: Theme.mainHeaderSize
                                }
                                Rectangle {
                                    anchors.right: parent.right
                                    anchors.verticalCenter: parent.verticalCenter
                                    width: 1
                                    height: 22
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
                                    Accessible.name: qsTr("选择当前页全部文件")
                                    onClicked: reader.selectAllOnPage(checkState === Qt.Checked)
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
                        model: reader.entries
                        function columnSize(column: int): real {
                            const scale = Math.max(1, Theme.fontScale);
                            const other = [Theme.mainOriginalColumnWidth, Theme.mainPackedColumnWidth, Theme.mainTypeColumnWidth, Theme.mainDateColumnWidth];
                            return column === 0 ? Math.max(250, width - other.reduce((a, b) => a + b, 0) * scale) : other[column - 1] * scale;
                        }
                        function columnLeftInset(column, header) {
                            return column === 0 ? (header ? Theme.mainNameHeaderInset : Theme.mainNameTextInset) : column === 3 ? Theme.mainTypeColumnInset : column === 4 ? Theme.mainDateColumnInset : 0;
                        }
                        function columnRightInset(column) {
                            return column === 1 ? Theme.mainOriginalColumnInset : column === 2 ? Theme.mainPackedColumnInset : 0;
                        }
                        columnWidthProvider: columnSize
                        onWidthChanged: forceLayout()
                        readonly property real scaledRowHeight: Math.max(Theme.mainFolderRowHeight, Theme.mainTableSize * 1.6 + 16)
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
                            required property string entryStatus
                            required property bool attention
                            objectName: "entry-" + row + "-" + column
                            Accessible.name: display
                            Accessible.description: entryStatus
                            implicitWidth: 100
                            implicitHeight: table.scaledRowHeight
                            color: Theme.surface
                            Rectangle {
                                anchors.fill: parent
                                anchors.leftMargin: cell.column === 0 ? 8 : 0
                                anchors.rightMargin: cell.column === 4 ? 8 : 0
                                anchors.topMargin: 2
                                anchors.bottomMargin: 2
                                radius: cell.column === 0 || cell.column === 4 ? Theme.controlRadius : 0
                                color: cell.chosen ? Theme.selection : hover.hovered ? Theme.canvas : Theme.surface
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
                                checked: cell.chosen
                                enabled: reader.runtimeAvailable && !reader.busy
                                Accessible.name: cell.display
                                onClicked: reader.selectRow(cell.row)
                            }
                            Image {
                                visible: cell.column === 0
                                anchors.left: parent.left
                                anchors.leftMargin: Theme.mainNameIconInset
                                anchors.verticalCenter: parent.verticalCenter
                                width: root.entryIconSize
                                height: root.entryIconSize
                                source: root.fileIcon(cell.display, cell.isDirectory)
                                fillMode: Image.PreserveAspectFit
                            }
                            Text {
                                anchors.fill: parent
                                anchors.leftMargin: table.columnLeftInset(cell.column, false)
                                anchors.rightMargin: cell.column === 0 && cell.attention ? 24 : table.columnRightInset(cell.column)
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
                                anchors.rightMargin: 8
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
                        visible: root.viewMode === "grid"
                        model: root.viewMode === "grid" ? reader.entries : null
                        clip: true
                        cellWidth: Math.max(150, Theme.bodySize * 10 + 16)
                        cellHeight: Math.max(146, Theme.bodySize * 3 + 104)
                        ScrollBar.vertical: ScrollBar {}
                        delegate: Rectangle {
                            id: gridEntry
                            required property int index
                            required property string display
                            required property bool isDirectory
                            required property bool chosen
                            required property string entryId
                            required property bool canNavigate
                            objectName: "grid-entry-" + index
                            required property string entryStatus
                            Accessible.name: display
                            Accessible.description: entryStatus
                            width: grid.cellWidth - 12
                            height: grid.cellHeight - 12
                            radius: Theme.controlRadius
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
                            ColumnLayout {
                                anchors.fill: parent
                                anchors.margins: 12
                                spacing: 8
                                Image {
                                    Layout.alignment: Qt.AlignHCenter
                                    Layout.preferredWidth: 48
                                    Layout.preferredHeight: 48
                                    source: root.fileIcon(gridEntry.display, gridEntry.isDirectory)
                                    fillMode: Image.PreserveAspectFit
                                }
                                Text {
                                    Layout.fillWidth: true
                                    text: gridEntry.display
                                    textFormat: Text.PlainText
                                    horizontalAlignment: Text.AlignHCenter
                                    font.pixelSize: Theme.bodySize
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
                        spacing: 16
                        Text {
                            Layout.fillWidth: true
                            text: reader.busy ? qsTr("正在读取…") : reader.ready ? qsTr("没有匹配的条目") : qsTr("浏览归档，无需先解压")
                            color: Theme.text
                            font.pixelSize: Theme.panelTitleSize
                            horizontalAlignment: Text.AlignHCenter
                        }
                        Text {
                            Layout.fillWidth: true
                            text: reader.message.length ? reader.message : qsTr("选择 ZIP 文件查看真实目录。当前支持只读浏览，不写出文件。")
                            color: Theme.textSecondary
                            font.pixelSize: Theme.bodySize
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
                RowLayout {
                    Layout.fillWidth: true
                    visible: reader.ready && reader.total > 128
                    Layout.margins: 16
                    spacing: 8
                    Text {
                        Layout.fillWidth: true
                        wrapMode: Text.Wrap
                        text: reader.ready ? qsTr("共 %1 项 · 当前 %2–%3").arg(reader.total).arg(reader.total ? reader.offset + 1 : 0).arg(Math.min(reader.offset + 128, reader.total)) : qsTr("尚未打开归档")
                        color: Theme.textSecondary
                        font.pixelSize: Theme.secondarySize
                    }
                    AppButton {
                        text: qsTr("上一页")
                        enabled: reader.ready && !reader.busy && reader.offset > 0
                        onClicked: reader.fetchPage(Math.max(0, reader.offset - 128))
                    }
                    AppButton {
                        text: qsTr("下一页")
                        enabled: reader.ready && !reader.busy && reader.offset + 128 < reader.total
                        onClicked: reader.fetchPage(reader.offset + 128)
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
                onCloseRequested: root.inspectorRequested = false
            }
        }
        Rectangle {
            objectName: "browserStatus"
            Layout.fillWidth: true
            implicitHeight: Math.round(Math.max(Theme.mainStatusHeight, Theme.mainStatusSize * 1.6 + 16))
            color: Theme.surface
            Rectangle {
                width: parent.width
                height: 1
                color: Theme.border
            }
            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: Theme.mainContentInset
                anchors.rightMargin: Theme.mainContentInset
                spacing: 8
                Text {
                    Layout.fillWidth: true
                    text: qsTr("已选择 %1 项").arg(reader.selectedCount) + (reader.selectedCount ? " · " + root.formatSize(reader.selectedBytes) : "")
                    color: Theme.textSecondary
                    font.pixelSize: Theme.mainStatusSize
                    elide: Text.ElideRight
                }
                ArchiveAction {
                    objectName: "browserTasks"
                    asset: "task-list"
                    text: qsTr("任务 %1").arg(reader.busy ? 1 : 0)
                    explanation: qsTr("查看当前读取任务")
                    onClicked: taskMenu.popup()
                    CommandMenu {
                        id: taskMenu
                        x: parent.width - width
                        y: -height
                        CommandMenuItem {
                            text: reader.busy ? qsTr("正在读取归档元数据") : qsTr("暂无后台任务")
                            enabled: false
                        }
                        CommandMenuItem {
                            text: qsTr("取消读取")
                            enabled: reader.busy
                            onTriggered: reader.cancel()
                        }
                    }
                }
            }
            RowLayout {
                x: parent.width / 2 - 50
                anchors.verticalCenter: parent.verticalCenter
                spacing: 9
                Image {
                    Layout.preferredWidth: 22
                    Layout.preferredHeight: 22
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
