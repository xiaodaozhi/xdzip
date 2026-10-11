pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Xdzip.Native

ApplicationWindow {
    id: root
    width: 1120
    height: 740
    minimumWidth: 960
    minimumHeight: 600
    visible: true
    title: reader.title.length ? reader.title + " — xdzip" : "xdzip"
    color: Theme.canvas
    onClosing: function(close) { close.accepted = reader.requestClose(); }

    BrowserController {
        id: reader
        objectName: "browserController"
        onReadyToClose: root.close()
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
                AppButton {
                    text: Theme.dark ? qsTr("浅色") : qsTr("深色")
                    onClicked: Theme.dark = !Theme.dark
                }
            }
        }
        Rectangle { Layout.fillWidth: true; implicitHeight: 1; color: Theme.border }
        RowLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: 0
            Rectangle {
                Layout.preferredWidth: 180
                Layout.fillHeight: true
                color: Theme.sidebar
                ColumnLayout {
                    anchors.fill: parent
                    anchors.margins: 20
                    spacing: 20
                    Text {
                        text: qsTr("归档内容")
                        font.pixelSize: Theme.bodySize
                        font.bold: true
                        color: Theme.text
                    }
                    Text {
                        Layout.fillWidth: true
                        text: qsTr("全部条目")
                        font.pixelSize: Theme.bodySize
                        color: Theme.accentInk
                    }
                    Text {
                        Layout.fillWidth: true
                        text: qsTr("保留原始记录顺序\n同名文件分别显示")
                        wrapMode: Text.Wrap
                        font.pixelSize: Theme.secondarySize
                        color: Theme.textSecondary
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
                    text: reader.title.length ? reader.title : qsTr("打开 ZIP，先浏览内容")
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
                            model: [qsTr("名称"), qsTr("大小 / bytes"), qsTr("压缩后 / bytes"), qsTr("方法"), qsTr("标记")]
                            Text {
                                required property string modelData
                                required property int index
                                width: table.columnSize(index)
                                height: 36
                                leftPadding: 14
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
                            return column === 0 ? Math.max(240, width - 450) : column === 4 ? 150 : 100;
                        }
                        columnWidthProvider: columnSize
                        onWidthChanged: forceLayout()
                        rowHeightProvider: function(row) { return 40; }
                        ScrollBar.vertical: ScrollBar {}
                        ScrollBar.horizontal: ScrollBar {}
                        delegate: Rectangle {
                            required property string display
                            required property int row
                            required property int column
                            implicitWidth: 100
                            implicitHeight: 40
                            color: row % 2 === 0 ? Theme.surface : Theme.canvas
                            Text {
                                anchors.fill: parent
                                anchors.leftMargin: 14
                                anchors.rightMargin: 10
                                text: parent.display
                                textFormat: Text.PlainText
                                verticalAlignment: Text.AlignVCenter
                                elide: Text.ElideRight
                                font.pixelSize: Theme.tableSize
                                color: Theme.text
                            }
                        }
                    }
                    ColumnLayout {
                        anchors.centerIn: parent
                        width: Math.min(parent.width - 60, 460)
                        visible: reader.busy || !reader.ready || reader.total === 0
                        spacing: 18
                        Text {
                            Layout.fillWidth: true
                            text: reader.busy ? qsTr("正在读取…") : reader.ready ? qsTr("这是一个空归档") : qsTr("浏览归档，无需先解压")
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
        }
        Rectangle { Layout.fillWidth: true; implicitHeight: 1; color: Theme.border }
        Text {
            Layout.fillWidth: true
            Layout.margins: 12
            text: qsTr("开发版本 · ZIP 元数据浏览 · 文件内容与 CRC 尚未校验 · 解压、新建尚未启用")
            color: Theme.textSecondary
            font.pixelSize: Theme.secondarySize
        }
    }
}
