pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Layouts

OperationWindow {
    id: root
    objectName: "restartWindow"
    signal restartRequested()
    modality: Qt.ApplicationModal
    width: Theme.operationProgressWidth
    title: qsTrId("settings.title")
    requestedHeight: Math.ceil(body.implicitHeight + 48 + footer.implicitHeight)
    ColumnLayout {
        anchors.fill: parent
        spacing: 0
        ColumnLayout {
            id: body
            Layout.fillWidth: true
            Layout.margins: 24
            Text {
                Layout.fillWidth: true
                text: qsTrId("settings.restartQuestion")
                font.pixelSize: Theme.settingsTitleSize * Theme.fontScale
                font.bold: true
                color: Theme.text
                wrapMode: Text.Wrap
            }
        }
        OperationFooter {
            id: footer
            objectName: "restartFooter"
            Layout.fillWidth: true
            OperationFooterButton { objectName: "restartLater"; text: qsTrId("settings.restartLater"); onClicked: root.close() }
            OperationFooterButton { objectName: "restartNow"; text: qsTrId("settings.restartNow"); primary: true; onClicked: root.restartRequested() }
        }
    }
}
