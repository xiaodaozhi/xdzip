pragma ComponentBehavior: Bound
import QtQuick

OperationConfirmationWindow {
    id: root
    objectName: "restartWindow"
    signal restartRequested()
    title: qsTrId("settings.title")
    heading: qsTrId("settings.restartQuestion")
    OperationFooterButton { objectName: "restartLater"; text: qsTrId("settings.restartLater"); onClicked: root.close() }
    OperationFooterButton { objectName: "restartNow"; text: qsTrId("settings.restartNow"); primary: true; onClicked: root.restartRequested() }
}
