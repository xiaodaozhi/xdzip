pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Xdzip.Native

ApplicationWindow {
    id: root
    width: 1360
    height: 740
    minimumWidth: 1160
    minimumHeight: 600
    readonly property color accentTextColor: Theme.accentText
    property string searchSession: ""
    property bool darkMode: false
    onDarkModeChanged: Theme.dark = darkMode
    visible: true
    title: reader.title.length ? reader.title + " — xdzip" : "xdzip"
    color: Theme.surface
    onClosing: function(close) { close.accepted = reader.requestClose(); }

    // Human-readable display only; exact integer strings remain in the model.
    function formatSize(value: string): string {
        const size = Number(value);
        if (size < 1024) return value + " B";
        if (size < 1048576) return (size / 1024).toFixed(1) + " KiB";
        if (size < 1073741824) return (size / 1048576).toFixed(1) + " MiB";
        return (size / 1073741824).toFixed(1) + " GiB";
    }
    BrowserController {
        id: reader
        objectName: "browserController"
        onReadyToClose: root.close()
        onChanged: {
            if (sessionId !== root.searchSession) {
                root.searchSession = sessionId;
                searchField.clear();
            }
        }
    }
    Shortcut {
        sequence: StandardKey.Open
        enabled: !reader.busy
        onActivated: reader.chooseArchive()
    }
    ColumnLayout {
        anchors.fill: parent
        spacing: 0
        Rectangle {
            Layout.fillWidth: true
            implicitHeight: 84
            color: Theme.surface
            RowLayout {
                anchors.fill: parent
                anchors.margins: 20
                spacing: 12
                Text {
                    text: "xdzip"
                    font.pixelSize: 25
                    font.bold: true
                    color: Theme.text
                    Layout.rightMargin: 20
                }
                AppButton {
                    text: qsTr("打开归档")
                    primary: true
                    enabled: !reader.busy
                    onClicked: reader.chooseArchive()
                }
                AppButton { text: qsTr("解压"); enabled: false }
                AppButton { text: qsTr("新建"); enabled: false }
                Item { Layout.fillWidth: true }
                TextField {
                    id: searchField
                    objectName: "archiveSearch"
                    Layout.preferredWidth: 280
                    placeholderText: qsTr("搜索归档中的文件 · 回车确认")
                    implicitHeight: 42
                    maximumLength: 256
                    enabled: reader.ready && !reader.busy
                    onAccepted: reader.search(text)
                    color: Theme.text
                    placeholderTextColor: Theme.textSecondary
                    background: Rectangle { color: Theme.canvas; radius: 8; border.color: Theme.border }
                }

                AppButton {
                    text: Theme.dark ? qsTr("浅色") : qsTr("深色")
                    onClicked: root.darkMode = !root.darkMode
                }
            }
        }
        Rectangle { Layout.fillWidth: true; implicitHeight: 1; color: Theme.border }
        RowLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: 0
            Rectangle {
                Layout.preferredWidth: 220
                Layout.fillHeight: true
                color: Theme.sidebar
                ColumnLayout {
                    anchors.fill: parent
                    anchors.margins: 20
                    spacing: 20
                    Text { text: "ARCHIVE"; color: Theme.textSecondary; font.pixelSize: 11; font.letterSpacing: 1.5 }
                    Image { source: "qrc:/xdzip/assets/archive.png"; sourceSize.width: 48; sourceSize.height: 48; Layout.preferredWidth: 48; Layout.preferredHeight: 48 }
                    Text {
                        Layout.fillWidth: true
                        text: reader.title.length ? reader.title : qsTr("尚未打开归档")
                        textFormat: Text.PlainText
                        elide: Text.ElideMiddle
                        font.pixelSize: 18; font.bold: true; color: Theme.text
                    }
                    Text { text: "ZIP  ·  " + qsTr("只读浏览"); color: Theme.accentInk; font.pixelSize: Theme.secondarySize }
                    Rectangle { Layout.fillWidth: true; implicitHeight: 1; color: Theme.border }
                    AppButton {
                        Layout.fillWidth: true
                        text: qsTr("全部文件")
                        enabled: reader.ready && !reader.busy
                        onClicked: reader.showAll()
                    }
                    Text {
                        Layout.fillWidth: true
                        text: reader.location.length ? reader.location : qsTr("双击目录进入浏览")
                        textFormat: Text.PlainText
                        wrapMode: Text.WrapAnywhere
                        color: Theme.textSecondary; font.pixelSize: Theme.bodySize
                    }
                    Item { Layout.fillHeight: true }
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
                Text {
                    Layout.fillWidth: true
                    Layout.margins: 20
                    text: reader.location.length ? reader.location : reader.title.length ? qsTr("全部文件") : qsTr("打开 ZIP，先浏览内容")
                    textFormat: Text.PlainText
                    font.pixelSize: Theme.panelTitleSize
                    font.bold: true
                    color: Theme.text
                    elide: Text.ElideMiddle
                }
                Rectangle {
                    Layout.fillWidth: true
                    implicitHeight: 36
                    color: Theme.sidebar
                    Row {
                        Repeater {
                            model: [qsTr("名称"), qsTr("原始大小"), qsTr("压缩后"), qsTr("方法"), qsTr("标记")]
                            Text {
                                required property string modelData
                                required property int index
                                width: table.columnSize(index)
                                height: 36
                                leftPadding: index === 0 ? 76 : 14
                                verticalAlignment: Text.AlignVCenter
                                text: modelData
                                color: Theme.textSecondary
                                font.pixelSize: Theme.secondarySize
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
                            return column === 0 ? Math.max(220, width - 340) : column === 4 ? 100 : 80;
                        }
                        columnWidthProvider: columnSize
                        onWidthChanged: forceLayout()
                        rowHeightProvider: function(row) { return 48; }
                        ScrollBar.vertical: ScrollBar {}
                        ScrollBar.horizontal: ScrollBar {}
                        delegate: Rectangle {
                            required property string display
                            required property int row
                            required property int column
                            required property bool isDirectory
                            required property bool chosen
                            id: cell
                            objectName: "entry-" + row + "-" + column
                            implicitWidth: 100
                            implicitHeight: 48
                            color: chosen ? Theme.selection : hover.hovered ? Theme.canvas : Theme.surface
                            HoverHandler { id: hover }
                            TapHandler {
                                enabled: !reader.busy
                                onSingleTapped: reader.selectRow(cell.row)
                                onDoubleTapped: reader.enterDirectory(cell.row)
                            }
                            Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 1; color: Theme.border; opacity: 0.45 }
                            CheckBox {
                                id: entryCheck
                                visible: cell.column === 0
                                width: 40; height: 48
                                indicator: Rectangle {
                                    x: 10; y: 14; width: 20; height: 20; radius: 4
                                    color: entryCheck.checked ? Theme.accent : Theme.surface
                                    border.color: entryCheck.activeFocus ? Theme.accent : Theme.border
                                    Text { anchors.centerIn: parent; text: "✓"; color: Theme.accentText; visible: entryCheck.checked }
                                }
                                Keys.onReturnPressed: reader.enterDirectory(cell.row)
                                anchors.left: parent.left; anchors.verticalCenter: parent.verticalCenter
                                checked: cell.chosen
                                enabled: !reader.busy
                                Accessible.name: cell.display
                                onClicked: reader.selectRow(cell.row)
                            }
                            Image {
                                visible: cell.column === 0
                                anchors.left: parent.left; anchors.leftMargin: 42; anchors.verticalCenter: parent.verticalCenter
                                width: 24; height: 24
                                source: cell.isDirectory ? "qrc:/xdzip/assets/folder.png" : "qrc:/xdzip/assets/file-text.png"
                            }
                            Text {
                                anchors.fill: parent
                                anchors.leftMargin: cell.column === 0 ? 76 : 14
                                anchors.rightMargin: 10
                                text: cell.column === 1 || cell.column === 2 ? root.formatSize(cell.display) : cell.display
                                textFormat: Text.PlainText
                                verticalAlignment: Text.AlignVCenter
                                elide: Text.ElideRight
                                font.pixelSize: Theme.tableSize
                                color: Theme.text
                            }
                        }
                    }
                    Rectangle { anchors.fill: parent; color: Theme.surface; visible: reader.busy || !reader.ready || reader.total === 0; opacity: 0.95 }
                    ColumnLayout {
                        anchors.centerIn: parent
                        width: Math.min(parent.width - 60, 460)
                        visible: reader.busy || !reader.ready || reader.total === 0
                        spacing: 18
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
                    Text {
                        Layout.fillWidth: true
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
            Rectangle { Layout.fillHeight: true; implicitWidth: 1; color: Theme.border }
            Rectangle {
                Layout.preferredWidth: 250
                Layout.fillHeight: true
                color: Theme.surface
                ScrollView {
                    anchors.fill: parent
                    anchors.margins: 20
                    contentWidth: availableWidth
                    ColumnLayout {
                        width: parent.width
                        spacing: 22
                        Text { text: qsTr("条目详情"); font.pixelSize: 18; font.bold: true; color: Theme.text }
                        Rectangle {
                            Layout.fillWidth: true; implicitHeight: 144; radius: 12; color: Theme.canvas
                            Image {
                                anchors.centerIn: parent; width: 64; height: 64
                                source: reader.detail.directory ? "qrc:/xdzip/assets/folder.png" : "qrc:/xdzip/assets/file-text.png"
                            }
                        }
                        Text {
                            Layout.fillWidth: true
                            text: reader.detail.name || qsTr("选择条目查看详情")
                            textFormat: Text.PlainText; wrapMode: Text.WrapAnywhere
                            font.pixelSize: 17; font.bold: true; color: Theme.text
                        }
                        Rectangle { Layout.fillWidth: true; implicitHeight: 1; color: Theme.border }
                        Repeater {
                            model: [qsTr("原始大小"), qsTr("压缩后"), qsTr("压缩方法"), qsTr("条目标记"), qsTr("记录 ID")]
                            RowLayout {
                                required property string modelData
                                required property int index
                                Layout.fillWidth: true
                                Text { text: parent.modelData; color: Theme.textSecondary; font.pixelSize: Theme.secondarySize }
                                Text {
                                    Layout.fillWidth: true
                                    text: [reader.detail.size ? root.formatSize(reader.detail.size) : "—", reader.detail.packed ? root.formatSize(reader.detail.packed) : "—", reader.detail.method || "—", reader.detail.status || "—", reader.detail.id || "—"][parent.index]
                                    textFormat: Text.PlainText; wrapMode: Text.WrapAnywhere; horizontalAlignment: Text.AlignRight
                                    color: Theme.text; font.pixelSize: Theme.secondarySize
                                }
                            }
                        }
                        Text {
                            Layout.fillWidth: true; text: qsTr("此处显示目录元数据。文件内容尚未读取或校验。")
                            wrapMode: Text.Wrap; color: Theme.textSecondary; font.pixelSize: Theme.secondarySize
                        }
                    }
                }
            }
        }
        Rectangle { Layout.fillWidth: true; implicitHeight: 1; color: Theme.border }
        Text {
            Layout.fillWidth: true
            Layout.margins: 12
            text: qsTr("本页已选择 %1 项").arg(reader.selectedCount) + "     ·     " + (reader.ready ? qsTr("目录已读取") : qsTr("等待打开归档")) + "     ·     " + qsTr("开发版 · 解压与新建暂不可用")
            color: Theme.textSecondary
            font.pixelSize: Theme.secondarySize
        }
    }
}
