pragma ComponentBehavior: Bound
import QtQuick

OperationConfirmationWindow {
    id: root
    objectName: "quitWindow"
    property bool stopping: false
    signal waitRequested()
    signal cancelAndQuitRequested()
    title: qsTrId("ui.a0cdc5abc463")
    heading: root.stopping ? qsTrId("ui.25c162aaed5d") : qsTrId("ui.f4de8f93cc94")
    message: root.stopping ? qsTrId("ui.7cb66eedb43c") : qsTrId("ui.7af06ed37de3")
    escapeClosesWindow: !stopping
    OperationFooterButton { objectName: "quitReturn"; text: qsTrId("ui.a3bba31ccfb9"); enabled: !root.stopping; onClicked: root.close() }
    OperationFooterButton { objectName: "quitWait"; text: qsTrId("ui.5b42b09f40df"); enabled: !root.stopping; onClicked: root.waitRequested() }
    OperationFooterButton { objectName: "quitCancel"; text: qsTrId("ui.b0921da970f8"); enabled: !root.stopping; primary: true; onClicked: root.cancelAndQuitRequested() }
}
