pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQml.Models
import Xdzip.Native

ApplicationWindow {
    id: root
    width: 1280
    height: 820
    minimumWidth: 1040
    minimumHeight: 680
    readonly property color accentTextColor: Theme.accentText
    property bool inspectorRequested: true
    readonly property bool inspectorVisible: inspectorRequested && width >= Theme.inspectorCollapseWidth
    property bool integratedTitlebar: false
    property string searchSession: ""
    property bool darkMode: false
    // File and directory icons share one logical slot; artwork retains its aspect ratio.
    readonly property int entryIconSize: 18
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
    ColumnLayout {
        anchors.fill: parent
        spacing: 0
        ArchiveTabs {
            Layout.fillWidth: true
            archiveTitle: reader.title
            integratedTitlebar: root.integratedTitlebar
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
        }
        RowLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: 0
            Rectangle {
                objectName: "archiveSidebar"
                Layout.preferredWidth: Theme.sidebarWidth
                Layout.minimumWidth: Theme.sidebarWidth
                Layout.maximumWidth: Theme.sidebarWidth
                Layout.fillHeight: true
                color: Theme.sidebar
                ColumnLayout {
                    anchors.fill: parent
                    anchors.margins: 16
                    spacing: 16
                    Text {
                        text: "ARCHIVE"
                        color: Theme.textSecondary
                        font.pixelSize: Theme.secondarySize
                        font.letterSpacing: 1.5
                    }
                    Text {
                        Layout.fillWidth: true
                        text: reader.title.length ? reader.title : qsTr("尚未打开归档")
                        textFormat: Text.PlainText
                        elide: Text.ElideMiddle
                        font.pixelSize: Theme.panelTitleSize
                        font.bold: true
                        color: Theme.text
                    }
                    Text {
                        text: "ZIP  ·  " + qsTr("只读浏览")
                        color: Theme.accentInk
                        font.pixelSize: Theme.secondarySize
                    }
                    Rectangle {
                        Layout.fillWidth: true
                        implicitHeight: 1
                        color: Theme.border
                    }
                    Button {
                        id: allFiles
                        Layout.fillWidth: true
                        implicitHeight: Math.max(40, Theme.bodySize * 1.6 + 16)
                        leftPadding: 12
                        rightPadding: 12
                        enabled: reader.ready && !reader.busy
                        onClicked: reader.showAll()
                        Accessible.name: qsTr("全部文件")
                        background: Rectangle {
                            radius: Theme.controlRadius
                            color: reader.location.length === 0 ? Theme.selection : allFiles.hovered ? Theme.canvas : "transparent"
                            border.width: allFiles.activeFocus ? 2 : 0
                            border.color: Theme.accent
                        }
                        contentItem: RowLayout {
                            spacing: 12
                            Image {
                                source: root.fileIcon("", true)
                                Layout.preferredWidth: root.entryIconSize
                                Layout.preferredHeight: root.entryIconSize
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
                            objectName: "directory-" + nodeId
                            implicitHeight: Math.ceil(Math.max(40, Theme.bodySize * 1.6 + 16))
                            implicitWidth: directoryTree.width
                            indentation: 16
                            leftMargin: 0
                            topPadding: 0
                            bottomPadding: 0
                            rightPadding: 8
                            palette.windowText: Theme.textSecondary
                            onClicked: reader.navigateDirectory(nodeId)
                            Keys.onReturnPressed: reader.navigateDirectory(nodeId)
                            Keys.onEnterPressed: reader.navigateDirectory(nodeId)
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
                                color: reader.directoryId === directoryNode.nodeId ? Theme.selection : directoryNode.hovered ? Theme.canvas : "transparent"
                                border.width: directoryNode.activeFocus || (directoryTree.activeFocus && directoryNode.current) ? 2 : 0
                                border.color: Theme.accent
                            }
                            contentItem: RowLayout {
                                spacing: 8
                                Image {
                                    source: root.fileIcon("", true)
                                    Layout.preferredWidth: root.entryIconSize
                                    Layout.preferredHeight: root.entryIconSize
                                    fillMode: Image.PreserveAspectFit
                                }
                                Text {
                                    Layout.fillWidth: true
                                    text: directoryNode.nodeName
                                    textFormat: Text.PlainText
                                    elide: Text.ElideRight
                                    color: Theme.text
                                    font.pixelSize: Theme.bodySize
                                }
                                Text {
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
                    Text {
                        objectName: "filePaneTitle"
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        text: reader.title.length ? reader.title + "  /  " + (reader.location.length ? reader.location : qsTr("全部文件")) : qsTr("打开 ZIP，先浏览内容")
                        textFormat: Text.PlainText
                        font.pixelSize: Theme.bodySize
                        color: Theme.text
                        elide: Text.ElideMiddle
                        verticalAlignment: Text.AlignVCenter
                    }
                    ArchiveAction {
                        objectName: "inspectorToggle"
                        text: qsTr("详情")
                        Accessible.name: root.inspectorVisible ? qsTr("隐藏条目详情") : qsTr("显示条目详情")
                        explanation: Accessible.name
                        onClicked: {
                            if (root.width < Theme.inspectorCollapseWidth) {
                                inspectorWindow.show();
                                inspectorWindow.raise();
                                inspectorWindow.requestActivate();
                            } else {
                                inspectorWindow.close();
                                root.inspectorRequested = !root.inspectorVisible;
                            }
                        }
                    }
                }
                Rectangle {
                    Layout.fillWidth: true
                    implicitHeight: Math.max(36, Theme.secondarySize * 1.6 + 12)
                    color: Theme.canvas
                    clip: true
                    Row {
                        x: -table.contentX
                        Repeater {
                            model: [qsTr("名称"), qsTr("原始大小"), qsTr("压缩后"), qsTr("方法"), qsTr("标记")]
                            Text {
                                id: columnHeader
                                required property string modelData
                                required property int index
                                width: table.columnSize(index)
                                height: parent.parent.height
                                leftPadding: index === 0 ? 72 : 12
                                rightPadding: 12
                                horizontalAlignment: index === 1 || index === 2 ? Text.AlignRight : Text.AlignLeft
                                verticalAlignment: Text.AlignVCenter
                                objectName: "sort-column-" + index
                                text: modelData
                                color: Theme.textSecondary
                                font.pixelSize: Theme.secondarySize
                                activeFocusOnTab: index < 4
                                enabled: index === 4 || (reader.ready && !reader.busy)
                                Accessible.role: index < 4 ? Accessible.Button : Accessible.StaticText
                                Accessible.name: modelData + (reader.sortOrder === index * 2 + 1 ? qsTr("，升序") : reader.sortOrder === index * 2 + 2 ? qsTr("，降序") : "")
                                Accessible.onPressAction: reader.sortByColumn(index)
                                Keys.onReturnPressed: reader.sortByColumn(index)
                                Keys.onEnterPressed: reader.sortByColumn(index)
                                Keys.onSpacePressed: reader.sortByColumn(index)
                                TapHandler {
                                    enabled: columnHeader.index < 4 && reader.ready && !reader.busy
                                    onTapped: reader.sortByColumn(columnHeader.index)
                                }
                                Rectangle {
                                    anchors.fill: parent
                                    color: "transparent"
                                    border.width: columnHeader.activeFocus ? 2 : 0
                                    border.color: Theme.accent
                                }
                                Text {
                                    anchors.verticalCenter: parent.verticalCenter
                                    x: columnHeader.index === 0 ? 52 : columnHeader.index < 3 ? 4 : parent.width - 20
                                    text: reader.sortOrder % 2 === 1 ? "↑" : "↓"
                                    visible: reader.sortOrder > 0 && Math.floor((reader.sortOrder - 1) / 2) === columnHeader.index
                                    color: Theme.accentInk
                                    font.pixelSize: Theme.secondarySize
                                }
                            }
                        }
                    }
                }
                Item {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    TableView {
                        id: table
                        anchors.fill: parent
                        clip: true
                        model: reader.entries
                        function columnSize(column: int): real {
                            const scale = Math.max(1, Theme.fontScale);
                            return column === 0 ? Math.max(240, width - 400 * scale) : 100 * scale;
                        }
                        columnWidthProvider: columnSize
                        onWidthChanged: forceLayout()
                        readonly property real scaledRowHeight: Math.ceil(Math.max(Theme.rowHeight, Theme.tableSize * 1.6 + 16))
                        onScaledRowHeightChanged: forceLayout()
                        Connections {
                            target: Theme
                            function onFontScaleChanged() {
                                table.forceLayout();
                            }
                        }
                        rowHeightProvider: function (row) {
                            return table.scaledRowHeight;
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
                            objectName: "entry-" + row + "-" + column
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
                            TapHandler {
                                enabled: reader.runtimeAvailable && !reader.busy
                                onSingleTapped: reader.selectRow(cell.row)
                                onDoubleTapped: reader.enterDirectory(cell.row)
                            }
                            Rectangle {
                                anchors.bottom: parent.bottom
                                width: parent.width
                                height: 1
                                color: Theme.border
                                opacity: 0.45
                            }
                            CheckBox {
                                id: entryCheck
                                visible: cell.column === 0
                                width: 40
                                height: parent.height
                                indicator: Rectangle {
                                    x: 12
                                    y: (entryCheck.height - 18) / 2
                                    width: 18
                                    height: 18
                                    radius: 4
                                    color: entryCheck.checked ? Theme.accent : Theme.surface
                                    border.color: entryCheck.activeFocus ? Theme.accent : Theme.border
                                    Text {
                                        anchors.centerIn: parent
                                        text: "✓"
                                        color: Theme.accentText
                                        visible: entryCheck.checked
                                    }
                                }
                                Keys.onReturnPressed: reader.enterDirectory(cell.row)
                                anchors.left: parent.left
                                anchors.verticalCenter: parent.verticalCenter
                                checked: cell.chosen
                                enabled: reader.runtimeAvailable && !reader.busy
                                Accessible.name: cell.display
                                onClicked: reader.selectRow(cell.row)
                            }
                            Image {
                                visible: cell.column === 0
                                anchors.left: parent.left
                                anchors.leftMargin: 42
                                anchors.verticalCenter: parent.verticalCenter
                                width: root.entryIconSize
                                height: root.entryIconSize
                                source: root.fileIcon(cell.display, cell.isDirectory)
                                fillMode: Image.PreserveAspectFit
                            }
                            Text {
                                anchors.fill: parent
                                anchors.leftMargin: cell.column === 0 ? 72 : 12
                                anchors.rightMargin: 12
                                text: cell.column === 1 || cell.column === 2 ? (cell.isDirectory ? "—" : root.formatSize(cell.display)) : cell.display
                                horizontalAlignment: cell.column === 1 || cell.column === 2 ? Text.AlignRight : Text.AlignLeft
                                textFormat: Text.PlainText
                                verticalAlignment: Text.AlignVCenter
                                elide: Text.ElideRight
                                font.pixelSize: Theme.tableSize
                                color: Theme.text
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
            Rectangle {
                visible: root.inspectorVisible
                Layout.fillHeight: true
                implicitWidth: 1
                color: Theme.border
            }
            InspectorPanel {
                objectName: "archiveInspector"
                visible: root.inspectorVisible
                Layout.preferredWidth: Theme.inspectorWidth
                Layout.minimumWidth: Theme.inspectorWidth
                Layout.maximumWidth: Theme.inspectorWidth
                Layout.fillHeight: true
                detail: reader.detail
                sizeFormatter: root.formatSize
                iconForFile: root.fileIcon
                onCloseRequested: root.inspectorRequested = false
            }
        }
        Rectangle {
            Layout.fillWidth: true
            implicitHeight: 1
            color: Theme.border
        }
        GridLayout {
            Layout.fillWidth: true
            Layout.margins: 12
            columns: 3
            uniformCellWidths: true
            columnSpacing: 24
            rowSpacing: 8
            Text {
                Layout.fillWidth: true
                text: qsTr("本页已选择 %1 项").arg(reader.selectedCount)
                color: Theme.textSecondary
                font.pixelSize: Theme.secondarySize
            }
            Text {
                Layout.fillWidth: true
                text: reader.busy ? qsTr("正在处理…") : reader.ready ? qsTr("目录已读取") : qsTr("等待打开归档")
                horizontalAlignment: parent.columns === 3 ? Text.AlignHCenter : Text.AlignLeft
                color: Theme.textSecondary
                font.pixelSize: Theme.secondarySize
            }
            Text {
                Layout.fillWidth: true
                text: qsTr("只读浏览")
                horizontalAlignment: parent.columns === 3 ? Text.AlignRight : Text.AlignLeft
                color: Theme.textSecondary
                font.pixelSize: Theme.secondarySize
                HoverHandler {
                    id: readOnlyHover
                }
                ToolTip.visible: readOnlyHover.hovered
                ToolTip.text: qsTr("开发版：解压、新建与校验尚未实现")
            }
        }
    }
}
