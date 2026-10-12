pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Rectangle {
    id: panel
    required property var detail
    required property var sizeFormatter
    required property var iconForFile
    property bool showCloseAction: true
    signal closeRequested
    color: Theme.inspector
    PaneHeader {
        id: header
        objectName: "inspectorPaneHeader"
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        height: implicitHeight
        Text {
            objectName: "inspectorPaneTitle"
            Layout.fillWidth: true
            Layout.fillHeight: true
            text: qsTr("条目详情")
            font.pixelSize: Theme.bodySize
            font.bold: true
            color: Theme.text
            elide: Text.ElideRight
            verticalAlignment: Text.AlignVCenter
        }
        ArchiveAction {
            objectName: "inspectorClose"
            visible: panel.showCloseAction
            asset: "close"
            Accessible.name: qsTr("关闭条目详情")
            explanation: Accessible.name
            onClicked: panel.closeRequested()
        }
    }
    ScrollView {
        anchors.top: header.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        anchors.leftMargin: 16
        anchors.rightMargin: 16
        anchors.bottomMargin: 16
        contentWidth: availableWidth
        ColumnLayout {
            width: parent.width
            spacing: 16
            Rectangle {
                Layout.fillWidth: true
                implicitHeight: width * 0.8
                radius: 12
                color: Theme.canvas
                Image {
                    anchors.centerIn: parent
                    width: 64
                    height: 64
                    source: panel.iconForFile(panel.detail.name || "", panel.detail.directory || false)
                    fillMode: Image.PreserveAspectFit
                }
            }
            Text {
                Layout.fillWidth: true
                text: panel.detail.name || qsTr("选择条目查看详情")
                textFormat: Text.PlainText
                wrapMode: Text.WrapAnywhere
                font.pixelSize: Theme.panelTitleSize
                font.bold: true
                color: Theme.text
            }
            Rectangle {
                Layout.fillWidth: true
                implicitHeight: 1
                color: Theme.border
            }
            Repeater {
                model: [qsTr("原始大小"), qsTr("压缩后"), qsTr("压缩方法"), qsTr("条目标记"), qsTr("记录 ID")]
                GridLayout {
                    columns: Theme.fontScale > 1.3 ? 1 : 2
                    rowSpacing: 4
                    columnSpacing: 12
                    required property string modelData
                    required property int index
                    Layout.fillWidth: true
                    Text {
                        text: parent.modelData
                        color: Theme.textSecondary
                        font.pixelSize: Theme.secondarySize
                    }
                    Text {
                        Layout.fillWidth: true
                        text: [panel.detail.size ? panel.sizeFormatter(panel.detail.size) : "—", panel.detail.packed ? panel.sizeFormatter(panel.detail.packed) : "—", panel.detail.method || "—", panel.detail.status || "—", panel.detail.id || "—"][parent.index]
                        textFormat: Text.PlainText
                        wrapMode: Text.WrapAnywhere
                        horizontalAlignment: Text.AlignRight
                        color: Theme.text
                        font.pixelSize: Theme.secondarySize
                    }
                }
            }
            Text {
                Layout.fillWidth: true
                text: qsTr("此处显示目录元数据。文件内容尚未读取或校验。")
                wrapMode: Text.Wrap
                color: Theme.textSecondary
                font.pixelSize: Theme.secondarySize
            }
        }
    }
}
