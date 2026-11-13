pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

OperationWindow {
    id: root
    objectName: "shellActionsWindow"
    required property var batch
    signal cancelRequested()
    signal dismissed()
    width: Theme.operationProgressWidth
    requestedHeight: Math.min(Math.ceil(content.implicitHeight + footer.implicitHeight), Theme.operationResultClientHeight, Screen.desktopAvailableHeight - 80)
    title: qsTrId("shell.title")
    onClosing: root.dismissed()
    ColumnLayout {
        anchors.fill: parent
        spacing: 0
        ScrollView {
            id: scroll
            Layout.fillWidth: true
            Layout.fillHeight: true
            contentWidth: availableWidth
            clip: true
            ColumnLayout {
                id: content
                width: scroll.availableWidth
                spacing: Theme.operationHeaderBodyGap
                OperationHeader {
                    Layout.fillWidth: true
                    source: "qrc:/xdzip/assets/archive-add-large.png"
                    title: root.title
                    subtitle: root.batch.summary
                }
                ColumnLayout {
                    objectName: "operationHeaderBody"
                    Layout.fillWidth: true
                    Layout.leftMargin: Theme.confirmationContentInset
                    Layout.rightMargin: Theme.confirmationContentInset
                    Layout.bottomMargin: Theme.confirmationContentInset
                    spacing: Theme.confirmationSectionGap
                    Repeater {
                        model: root.batch.choosing ? root.batch.actions : []
                        OperationButton {
                            required property var modelData
                            Layout.fillWidth: true
                            text: modelData.label
                            wrapLabel: true
                            objectName: "shellAction-" + modelData.action
                            onClicked: root.batch.choose(modelData.action)
                        }
                    }
                    BusyIndicator { visible: root.batch.active; running: visible; Layout.alignment: Qt.AlignHCenter }
                }
            }
        }
        OperationFooter {
            id: footer
            Layout.fillWidth: true
            OperationFooterButton {
                text: root.batch.active ? qsTrId("common.cancel") : qsTrId("shell.close")
                onClicked: { if (root.batch.active) root.cancelRequested(); else root.close(); }
            }
        }
    }
}
