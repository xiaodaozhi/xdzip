// apps/desktop/ui/UpdateWindow.qml

pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

OperationWindow {
    id: root
    objectName: "updateWindow"
    required property var controller
    property var inputs: []
    property string sourceSession: ""
    property string targetDirectory: ""
    property string targetLabel: ""
    property string archiveLabel: ""
    width: Theme.operationCreateWidth
    requestedHeight: Math.min(Math.ceil(updateContent.implicitHeight + footer.implicitHeight), Theme.operationCreateClientHeight, Screen.desktopAvailableHeight - 60)
    title: qsTrId("update.title")
    function configure(paths) {
        sourceSession = controller.sessionId;
        targetDirectory = controller.directoryId;
        targetLabel = controller.location || qsTrId("update.root");
        archiveLabel = controller.title;
        inputs = paths;
        conflict.currentIndex = 0;
        controller.clearCreationError();
        present();
    }
    Connections {
        target: root
        function onActiveFocusItemChanged() { root.revealFocus(scroll); }
    }
    ColumnLayout {
        anchors.fill: parent
        spacing: 0
        ScrollView {
            id: scroll
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            contentWidth: availableWidth
            ColumnLayout {
                id: updateContent
                width: scroll.availableWidth
                spacing: Theme.operationAddingSectionGap
                ColumnLayout {
                    Layout.fillWidth: true
                    Layout.margins: Theme.operationCreateContentInset
                    spacing: Theme.operationAddingSectionGap
                    Text { Layout.fillWidth: true; text: qsTrId("update.title"); font.pixelSize: Theme.operationCreateTitleSize; font.bold: true; color: Theme.text; wrapMode: Text.Wrap }
                    Text { Layout.fillWidth: true; text: root.archiveLabel; font.pixelSize: Theme.operationCreateBodySize; color: Theme.textSecondary; wrapMode: Text.Wrap; textFormat: Text.PlainText }
                    Text { text: qsTrId("update.destination"); font.pixelSize: Theme.operationCreateBodySize; color: Theme.text }
                    Text { Layout.fillWidth: true; text: root.targetLabel; font.pixelSize: Theme.operationCreateBodySize; color: Theme.textSecondary; wrapMode: Text.Wrap; textFormat: Text.PlainText }
                    Text { text: qsTrId("update.inputs"); font.pixelSize: Theme.operationCreateBodySize; color: Theme.text }
                    OperationFrame {
                        id: inputFrame
                        control: inputFrame
                        Layout.fillWidth: true
                        implicitHeight: Math.min(Theme.operationAddingSourceListMaximumHeight, Math.max(Theme.operationAddingSourceListMinimumHeight, root.inputs.length * (Theme.operationControlBodyButtonHeight + Theme.operationAddingSourceRowGap) + Theme.operationAddingSourceListInset * 2))
                        ListView {
                            id: inputList
                            objectName: "updateInputList"
                            anchors.fill: parent
                            anchors.margins: Theme.operationAddingSourceListInset
                            clip: true
                            spacing: Theme.operationAddingSourceRowGap
                            model: root.inputs
                            ScrollBar.vertical: ScrollBar {}
                            delegate: RowLayout {
                                id: inputRow
                                required property string modelData
                                required property int index
                                width: inputList.width
                                Text { Layout.fillWidth: true; text: inputRow.modelData.replace(/\\/g, "/").split("/").pop(); Accessible.name: inputRow.modelData; elide: Text.ElideMiddle; font.pixelSize: Theme.operationCreateBodySize; color: Theme.text; textFormat: Text.PlainText }
                                OperationButton { text: qsTrId("update.remove"); onClicked: root.inputs = root.inputs.filter((_, i) => i !== inputRow.index) }
                            }
                        }
                    }
                    Text { text: qsTrId("update.conflicts"); font.pixelSize: Theme.operationCreateBodySize; color: Theme.text }
                    OperationSelect { id: conflict; objectName: "updateConflictPolicy"; Layout.fillWidth: true; density: Theme.operationCreateDensity; model: [qsTrId("update.ask"), qsTrId("update.skip"), qsTrId("update.rename"), qsTrId("update.replace")] }
                    Text { Layout.fillWidth: true; text: qsTrId("update.hint"); font.pixelSize: Theme.operationCreateNoteSize; color: Theme.textSecondary; wrapMode: Text.Wrap }
                    Text { Layout.fillWidth: true; visible: text.length > 0; text: root.controller.creationError; font.pixelSize: Theme.operationCreateBodySize; color: Theme.menuDanger; wrapMode: Text.Wrap; textFormat: Text.PlainText }
                }
            }
        }
        OperationFooter {
            id: footer
            Layout.fillWidth: true
            contentInset: Theme.operationCreateContentInset
            OperationFooterButton { objectName: "updateCancel"; text: qsTrId("update.cancel"); onClicked: root.close() }
            OperationFooterButton {
                objectName: "startUpdate"
                text: qsTrId("update.add")
                primary: true
                enabled: root.inputs.length > 0 && root.controller.canUpdateArchive && !root.controller.busy && root.controller.sessionId === root.sourceSession
                onClicked: root.controller.startUpdate(root.inputs, root.sourceSession, root.targetDirectory, conflict.currentIndex)
            }
        }
    }
}
