pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Layouts

OperationConfirmationWindow {
    id: root
    objectName: "archiveEntryEditWindow"
    required property var controller
    property var intent: ({})
    readonly property string token: intent.token || ""
    readonly property bool renaming: Boolean(intent.rename)
    readonly property string originalName: intent.name || ""
    readonly property ArchiveNameEditor nameEditor: extraItem as ArchiveNameEditor
    readonly property string proposedName: renaming && nameEditor ? nameEditor.name : ""
    readonly property bool changedName: renaming && proposedName !== originalName
    readonly property bool validName: changedName && controller.validEntryName(proposedName)
    title: renaming ? qsTrId("entry.edit.rename") : qsTrId("entry.edit.delete")
    heading: renaming ? qsTrId("entry.edit.renameHeading") : qsTrId("entry.edit.deleteHeading")
    message: {
        if (renaming || !token)
            return "";
        const count = Number(intent.count);
        const question = count === 1 ? qsTrId("entry.edit.deleteOne").arg(originalName)
                                     : qsTrId("entry.edit.deleteMany", count);
        return intent.includesDirectory ? question + "\n" + qsTrId("entry.edit.deleteDescendants")
                                        : question;
    }
    messageUserData: !renaming && intent.count === 1
    extraContent: renaming ? nameContent : null

    function configure(value: var) {
        if (!value.token)
            return;
        intent = value;
        present();
        if (renaming && nameEditor)
            nameEditor.editName(originalName, Boolean(intent.directory));
        else
            cancelButton.forceActiveFocus();
    }
    function submit() {
        if (token && (!renaming || validName) && controller.startEntryEdit(token, proposedName))
            close();
    }
    onVisibleChanged: if (!visible && token) {
        controller.discardEntryEdit(token);
        intent = ({});
    }

    Component {
        id: nameContent
        ArchiveNameEditor {}
    }
    component ArchiveNameEditor: ColumnLayout {
        readonly property alias name: nameField.text
        spacing: Theme.confirmationSectionGap
        function editName(value: string, directory: bool) {
            nameField.text = value;
            nameField.forceActiveFocus();
            const extension = directory ? -1 : value.lastIndexOf(".");
            nameField.select(0, extension > 0 ? extension : value.length);
        }
        OperationField {
            id: nameField
            objectName: "archiveEntryNameField"
            Layout.fillWidth: true
            font.family: Theme.userDataFontFamily
            Accessible.name: qsTrId("entry.edit.newName")
            Keys.onEscapePressed: root.close()
            onAccepted: root.submit()
        }
        Text {
            objectName: "archiveEntryNameError"
            Layout.fillWidth: true
            visible: root.changedName && !root.validName
            text: qsTrId("entry.edit.invalidName")
            textFormat: Text.PlainText
            wrapMode: Text.Wrap
            color: Theme.menuDanger
            font: root.messageItem.font
        }
    }

    OperationFooterButton {
        id: cancelButton
        objectName: "archiveEntryEditCancel"
        text: qsTrId("entry.edit.cancel")
        Keys.onEscapePressed: root.close()
        onClicked: root.close()
    }
    OperationFooterButton {
        objectName: "archiveEntryEditConfirm"
        text: root.title
        primary: true
        enabled: root.token.length > 0 && (!root.renaming || root.validName)
        Keys.onEscapePressed: root.close()
        onClicked: root.submit()
    }
}
