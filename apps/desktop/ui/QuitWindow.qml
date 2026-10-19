pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Layouts

OperationWindow {
    id: root
    objectName: "quitWindow"
    property bool stopping: false
    signal waitRequested()
    signal cancelAndQuitRequested()
    width: Theme.operationDefaultWidth
    requestedHeight: Theme.fontScale > 1.35 ? 540 : 270
    title: qsTrId("ui.a0cdc5abc463")
    ColumnLayout {
        anchors.fill: parent; spacing: 0
        ColumnLayout {
            Layout.fillWidth: true; Layout.fillHeight: true; Layout.margins: 24; spacing: 16
            Text { text: root.stopping ? qsTrId("ui.25c162aaed5d") : qsTrId("ui.f4de8f93cc94"); font.pixelSize: Theme.operationTitleSize; font.bold: true; color: Theme.text }
            Text { Layout.fillWidth: true; text: root.stopping ? qsTrId("ui.7cb66eedb43c") : qsTrId("ui.7af06ed37de3"); font.pixelSize: Theme.operationBodySize; color: Theme.textSecondary; wrapMode: Text.Wrap }
        }
        OperationFooter {
            Layout.fillWidth: true
            OperationFooterButton { text: qsTrId("ui.a3bba31ccfb9"); enabled: !root.stopping; onClicked: root.close() }
            OperationFooterButton { text: qsTrId("ui.5b42b09f40df"); enabled: !root.stopping; onClicked: root.waitRequested() }
            OperationFooterButton { text: qsTrId("ui.b0921da970f8"); enabled: !root.stopping; primary: true; onClicked: root.cancelAndQuitRequested() }
        }
    }
}
