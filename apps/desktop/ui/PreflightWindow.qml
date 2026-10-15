pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

OperationWindow {
    id: root
    objectName: "preflightWindow"
    title: qsTrId("ui.ff6f3ae7c4fb")
    property var report: ({})
    width: Theme.operationDefaultWidth
    requestedHeight: Math.min(620, Screen.desktopAvailableHeight - 80)
    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 24
        spacing: 18
        Text {
            text: root.report.error ? qsTrId("ui.716a0b6591ca") : root.report.findingCount !== "0" ? qsTrId("ui.c668803dc295") : qsTrId("ui.9cb79c4bee66")
            font.pixelSize: Theme.panelTitleSize; font.bold: true; color: Theme.text
        }
        Text {
            Layout.fillWidth: true
            text: qsTrId("ui.db661e297081")
            wrapMode: Text.Wrap; color: Theme.textSecondary; font.pixelSize: Theme.bodySize
        }
        Rectangle {
            Layout.fillWidth: true; implicitHeight: 92
            color: Theme.taskPanel; radius: 12; border.color: Theme.taskBorder
            RowLayout {
                anchors.fill: parent; anchors.margins: 18; spacing: 24
                Repeater {
                    model: [qsTrId("ui.0790c26b1e36"), qsTrId("ui.377532e93480"), qsTrId("ui.9d959257ce7e")]
                    ColumnLayout {
                        required property int index
                        required property string modelData
                        Layout.fillWidth: true
                        Text { text: parent.modelData; color: Theme.textSecondary; font.pixelSize: Theme.secondarySize }
                        Text {
                            text: [root.report.entries || "—", root.report.bytes === "unknown" ? qsTrId("ui.aa524057ccca") : root.report.bytes || "—", root.report.findingCount || "—"][parent.index]
                            color: Theme.text; font.pixelSize: 18; font.bold: true
                        }
                    }
                }
            }
        }
        ScrollView {
            Layout.fillWidth: true; Layout.fillHeight: true
            contentWidth: availableWidth
            ColumnLayout {
                width: parent.width; spacing: 12
                Text {
                    Layout.fillWidth: true
                    visible: !!root.report.error
                    text: root.report.error || ""
                    wrapMode: Text.Wrap; color: Theme.text; font.pixelSize: Theme.bodySize
                }
                Repeater {
                    model: root.report.findings || []
                    Text {
                        required property string modelData
                        Layout.fillWidth: true
                        text: modelData; textFormat: Text.PlainText; wrapMode: Text.Wrap
                        color: Theme.text; font.pixelSize: Theme.bodySize
                    }
                }
                Text {
                    Layout.fillWidth: true
                    text: root.report.findingCount === "0" ? qsTrId("ui.f644f6ec6fad") : qsTrId("ui.780102188429")
                    color: Theme.textSecondary; wrapMode: Text.Wrap; font.pixelSize: Theme.bodySize
                    visible: !root.report.error
                }
                Text {
                    Layout.fillWidth: true
                    visible: !!root.report.unicodeReview
                    text: qsTrId("ui.b8b0671912e5")
                    wrapMode: Text.Wrap; color: Theme.textSecondary; font.pixelSize: Theme.bodySize
                }
            }
        }
        OperationFooter {
            Layout.fillWidth: true
            contentInset: 0
            note: qsTrId("ui.0d92f3d4cae3")
            OperationFooterButton { text: qsTrId("ui.26867436321f"); primary: true; onClicked: root.close() }
        }
    }
}
