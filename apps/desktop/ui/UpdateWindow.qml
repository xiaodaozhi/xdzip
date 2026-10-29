// apps/desktop/ui/UpdateWindow.qml

pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

OperationWindow {
    id: root
    objectName: "updateWindow"
    required property var controller
    readonly property var inputs: inputPanel.inputPaths
    property string sourceSession: ""
    property string targetDirectory: ""
    property string targetLabel: ""
    property string archiveLabel: ""
    readonly property real density: Theme.operationCreateDensity
    width: Theme.operationCreateWidth
    requestedHeight: Math.min(Math.ceil(updateContent.implicitHeight + footer.implicitHeight), Theme.operationCreateClientHeight, Screen.desktopAvailableHeight - Theme.operationAddingScreenMargin)
    title: qsTrId("update.title")
    function configure(paths) {
        sourceSession = controller.sessionId;
        targetDirectory = controller.directoryId;
        targetLabel = controller.location || qsTrId("update.root");
        archiveLabel = controller.title;
        inputPanel.setInputPaths(paths);
        conflict.currentIndex = 0;
        controller.clearCreationError();
        present();
    }
    function addInputPaths(paths: var, directory: bool) {
        inputPanel.addInputPaths(paths, directory);
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
            objectName: "updateScroll"
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            contentWidth: availableWidth
            ColumnLayout {
                id: updateContent
                objectName: "updateContent"
                width: scroll.availableWidth
                spacing: Theme.operationHeaderBodyGap
                OperationHeader {
                    Layout.fillWidth: true
                    source: Theme.dark ? "qrc:/xdzip/assets/dark/archive-add-large.png" : "qrc:/xdzip/assets/archive-add-large.png"
                    title: qsTrId("update.title")
                    subtitle: qsTrId("update.subtitle")
                    objectName: "updateHero"
                    titleObjectName: "updateHeading"
                }
                ColumnLayout {
                    objectName: "operationHeaderBody"
                    Layout.fillWidth: true
                    Layout.leftMargin: Theme.operationCreateContentInset
                    Layout.rightMargin: Theme.operationCreateContentInset
                    Layout.bottomMargin: Theme.operationAddingBodyBottomInset
                    spacing: Theme.operationAddingCardGap
                    ArchiveInputPanel {
                        id: inputPanel
                        objectName: "updateInputPanel"
                        Layout.fillWidth: true
                        controller: root.controller
                        dialogOwner: root
                        namePrefix: "update"
                        heading: qsTrId("update.inputs")
                    }
                    Rectangle {
                        objectName: "updateDestinationCard"
                        Layout.fillWidth: true
                        implicitHeight: destinationContent.implicitHeight + Theme.operationCreateCardInset * 2
                        color: Theme.operationCard
                        border.color: Theme.operationCardBorder
                        radius: Theme.operationAddingCardRadius
                        ColumnLayout {
                            id: destinationContent
                            anchors.fill: parent
                            anchors.margins: Theme.operationCreateCardInset
                            spacing: Theme.operationAddingCardContentGap
                            Text {
                                Layout.fillWidth: true
                                text: qsTrId("update.location")
                                font.pixelSize: Theme.operationCreateBodySize
                                font.bold: true
                                color: Theme.text
                                wrapMode: Text.Wrap
                            }
                            Rectangle {
                                Layout.fillWidth: true
                                implicitHeight: destinationFields.implicitHeight + Theme.operationAddingSourceListInset * 2
                                color: Theme.surface
                                border.color: Theme.operationCardBorder
                                radius: Theme.operationAddingCardRadius
                                GridLayout {
                                    id: destinationFields
                                    anchors.fill: parent
                                    anchors.margins: Theme.operationAddingSourceListInset
                                    columns: 2
                                    columnSpacing: Theme.operationAddingRowGap
                                    rowSpacing: Theme.operationAddingRowGap
                                    Text {
                                        text: qsTrId("update.archive")
                                        color: Theme.text
                                        font.pixelSize: Theme.operationCreateBodySize
                                    }
                                    OperationFrame {
                                        id: archiveFrame
                                        objectName: "updateArchiveFrame"
                                        control: archiveFrame
                                        Layout.fillWidth: true
                                        implicitHeight: Math.max(Theme.operationCreateFieldHeight, archiveName.implicitHeight + Theme.operationControlVerticalInset * 2)
                                        RowLayout {
                                            anchors.fill: parent
                                            anchors.margins: Theme.operationControlVerticalInset
                                            spacing: Theme.operationAddingRowGap
                                            Image {
                                                source: AssetLibrary.fileIcon(root.archiveLabel, false, false)
                                                Layout.preferredWidth: Theme.operationControlSelectIndicatorSize
                                                Layout.preferredHeight: Theme.operationControlSelectIndicatorSize
                                            }
                                            Text {
                                                id: archiveName
                                                objectName: "updateArchiveName"
                                                Layout.fillWidth: true
                                                text: root.archiveLabel
                                                font.family: Theme.userDataFontFamily
                                                textFormat: Text.PlainText
                                                wrapMode: Text.WrapAnywhere
                                                font.pixelSize: Theme.operationCreateBodySize
                                                color: Theme.text
                                                Accessible.name: qsTrId("update.archive") + ": " + text
                                            }
                                        }
                                    }
                                    Text {
                                        text: qsTrId("update.destination")
                                        color: Theme.text
                                        font.pixelSize: Theme.operationCreateBodySize
                                    }
                                    OperationFrame {
                                        id: directoryFrame
                                        objectName: "updateDirectoryFrame"
                                        control: directoryFrame
                                        Layout.fillWidth: true
                                        implicitHeight: Math.max(Theme.operationCreateFieldHeight, directoryName.implicitHeight + Theme.operationControlVerticalInset * 2)
                                        RowLayout {
                                            anchors.fill: parent
                                            anchors.margins: Theme.operationControlVerticalInset
                                            spacing: Theme.operationAddingRowGap
                                            Image {
                                                source: AssetLibrary.fileIcon("", true, false)
                                                Layout.preferredWidth: Theme.operationControlSelectIndicatorSize
                                                Layout.preferredHeight: Theme.operationControlSelectIndicatorSize
                                            }
                                            Text {
                                                id: directoryName
                                                objectName: "updateDirectoryName"
                                                Layout.fillWidth: true
                                                text: root.targetLabel
                                                font.family: Theme.userDataFontFamily
                                                textFormat: Text.PlainText
                                                wrapMode: Text.WrapAnywhere
                                                font.pixelSize: Theme.operationCreateBodySize
                                                color: Theme.text
                                                Accessible.name: qsTrId("update.destination") + ": " + text
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Rectangle {
                        objectName: "updateOptionsCard"
                        Layout.fillWidth: true
                        implicitHeight: optionsContent.implicitHeight + Theme.operationCreateCardInset * 2
                        color: Theme.operationCard
                        border.color: Theme.operationCardBorder
                        radius: Theme.operationAddingCardRadius
                        ColumnLayout {
                            id: optionsContent
                            anchors.fill: parent
                            anchors.margins: Theme.operationCreateCardInset
                            spacing: Theme.operationAddingCardContentGap
                            Text {
                                text: qsTrId("update.options")
                                font.pixelSize: Theme.operationCreateBodySize
                                font.bold: true
                                color: Theme.text
                            }
                            Rectangle {
                                Layout.fillWidth: true
                                implicitHeight: optionsFields.implicitHeight + Theme.operationAddingSourceListInset * 2
                                color: Theme.surface
                                border.color: Theme.operationCardBorder
                                radius: Theme.operationAddingCardRadius
                                ColumnLayout {
                                    id: optionsFields
                                    anchors.fill: parent
                                    anchors.margins: Theme.operationAddingSourceListInset
                                    spacing: Theme.operationAddingCardContentGap
                                    Text { text: qsTrId("update.conflicts"); font.pixelSize: Theme.operationCreateBodySize; color: Theme.text }
                                    OperationSelect {
                                        id: conflict
                                        objectName: "updateConflictPolicy"
                                        Layout.fillWidth: true
                                        density: root.density
                                        model: [qsTrId("update.ask"), qsTrId("update.skip"), qsTrId("update.rename"), qsTrId("update.replace")]
                                    }
                                    Text {
                                        Layout.fillWidth: true
                                        text: qsTrId("update.hint")
                                        font.pixelSize: Theme.operationCreateNoteSize
                                        color: Theme.textSecondary
                                        wrapMode: Text.Wrap
                                    }
                                }
                            }
                        }
                    }
                    Text {
                        objectName: "updateError"
                        Layout.fillWidth: true
                        visible: text.length > 0
                        text: root.controller.creationError
                        font.pixelSize: Theme.operationCreateBodySize
                        color: Theme.menuDanger
                        wrapMode: Text.Wrap
                        textFormat: Text.PlainText
                    }
                }
            }
        }
        OperationFooter {
            id: footer
            objectName: "updateFooter"
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
