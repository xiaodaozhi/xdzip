pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Rectangle {
    id: panel
    required property var detail
    required property var sizeFormatter
    required property var iconForFile
    property int selectionCount: 0
    property bool showCloseAction: true
    signal closeRequested
    color: Theme.inspector
    Rectangle {
        width: 1
        height: parent.height
        color: Theme.border
    }
    PaneHeader {
        id: header
        objectName: "inspectorPaneHeader"
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        height: implicitHeight
        leftInset: Theme.mainInspectorLeftInset
        rightInset: Theme.mainInspectorRightInset
        Text {
            objectName: "inspectorPaneTitle"
            Layout.fillWidth: true
            Layout.fillHeight: true
            text: qsTr("文件预览")
            font.pixelSize: Theme.mainPaneTitleSize
            font.weight: Font.DemiBold
            color: Theme.text
            elide: Text.ElideRight
            verticalAlignment: Text.AlignVCenter
        }
        ArchiveAction {
            objectName: "inspectorClose"
            visible: panel.showCloseAction
            leftPadding: 0
            rightPadding: 0
            asset: "close"
            Accessible.name: qsTr("关闭文件预览与属性")
            explanation: Accessible.name
            onClicked: panel.closeRequested()
        }
    }
    ScrollView {
        id: scroll
        anchors.top: header.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        anchors.leftMargin: Theme.mainInspectorLeftInset
        anchors.rightMargin: Theme.mainInspectorRightInset
        anchors.bottomMargin: Theme.mainUnits(20)
        contentWidth: availableWidth
        ColumnLayout {
            width: scroll.availableWidth
            spacing: 0
            Rectangle {
                objectName: "previewContent"
                Layout.fillWidth: true
                implicitHeight: Math.round(Theme.mainPreviewHeight)
                radius: Theme.mainPreviewRadius
                color: Theme.canvas
                border.color: Theme.border
                ColumnLayout {
                    anchors.centerIn: parent
                    width: parent.width - Theme.mainUnits(32)
                    spacing: Theme.mainUnits(14)
                    Image {
                        Layout.alignment: Qt.AlignHCenter
                        Layout.preferredWidth: Theme.mainUnits(64)
                        Layout.preferredHeight: Theme.mainUnits(64)
                        source: panel.iconForFile(panel.detail.name || "", panel.detail.directory || false)
                        fillMode: Image.PreserveAspectFit
                    }
                    Text {
                        Layout.fillWidth: true
                        text: panel.detail.id ? qsTr("此类型暂不支持内容预览") : qsTr("选择文件查看属性")
                        font.pixelSize: Theme.mainPropertySize
                        color: Theme.textSecondary
                        horizontalAlignment: Text.AlignHCenter
                        wrapMode: Text.Wrap
                    }
                }
            }
            Text {
                Layout.fillWidth: true
                Layout.topMargin: Theme.mainUnits(14)
                text: panel.selectionCount > 1 ? qsTr("已选择 %1 项").arg(panel.selectionCount) : panel.detail.name || qsTr("未选择文件")
                textFormat: Text.PlainText
                wrapMode: Text.WrapAnywhere
                font.pixelSize: Theme.mainDetailTitleSize
                font.weight: Font.DemiBold
                color: Theme.text
            }
            Text {
                Layout.fillWidth: true
                Layout.topMargin: Theme.mainUnits(6)
                text: panel.selectionCount > 1 ? qsTr("以下为当前焦点条目的属性") : panel.detail.type ? panel.detail.type + (panel.detail.directory ? "" : qsTr(" 文件")) : qsTr("归档内条目")
                color: Theme.textSecondary
                font.pixelSize: Theme.mainDetailTypeSize
                wrapMode: Text.Wrap
            }
            Rectangle {
                Layout.fillWidth: true
                Layout.topMargin: Theme.mainUnits(16)
                Layout.bottomMargin: Theme.mainUnits(10)
                implicitHeight: 1
                color: Theme.border
            }
            Text {
                visible: panel.detail.attention || false
                Layout.fillWidth: true
                Layout.bottomMargin: visible ? Theme.mainUnits(8) : 0
                text: panel.detail.status || ""
                textFormat: Text.PlainText
                wrapMode: Text.Wrap
                font.pixelSize: Theme.mainPropertySize
                color: Theme.menuDanger
            }
            Repeater {
                model: [
                    {
                        label: qsTr("原始大小"),
                        value: panel.detail.size ? panel.sizeFormatter(panel.detail.size) : "—"
                    },
                    {
                        label: qsTr("压缩后"),
                        value: panel.detail.packed ? panel.sizeFormatter(panel.detail.packed) : "—"
                    },
                    {
                        label: qsTr("修改日期"),
                        value: panel.detail.modified || "—"
                    },
                    {
                        label: qsTr("压缩方法"),
                        value: panel.detail.method || "—"
                    }
                ]
                GridLayout {
                    required property var modelData
                    columns: Theme.fontScale > 1.3 ? 1 : 2
                    rowSpacing: Theme.mainUnits(4)
                    columnSpacing: Theme.mainUnits(10)
                    Layout.fillWidth: true
                    Layout.minimumHeight: Math.max(Theme.mainUnits(32), Theme.mainPropertySize * 1.6 + Theme.mainUnits(8))
                    Text {
                        text: parent.modelData.label
                        color: Theme.textSecondary
                        font.pixelSize: Theme.mainPropertySize
                    }
                    Text {
                        Layout.fillWidth: true
                        text: parent.modelData.value
                        textFormat: Text.PlainText
                        wrapMode: Text.WrapAnywhere
                        horizontalAlignment: Text.AlignRight
                        color: Theme.text
                        font.pixelSize: Theme.mainPropertySize
                    }
                }
            }
            ArchiveAction {
                objectName: "inspectorExtract"
                Layout.fillWidth: true
                Layout.topMargin: Theme.mainUnits(12)
                implicitHeight: Math.max(Theme.mainUnits(47), Theme.mainHeaderSize * 1.6 + Theme.mainUnits(16))
                asset: "external-open"
                text: qsTr("解压此文件")
                enabled: false
                explanation: qsTr("安全解压写入尚未实现")
                background: Rectangle {
                    radius: Theme.mainUnits(6)
                    color: Theme.surface
                    border.color: Theme.border
                }
            }
        }
    }
}
