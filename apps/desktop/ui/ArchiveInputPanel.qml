// apps/desktop/ui/ArchiveInputPanel.qml

pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Rectangle {
    id: root
    required property var controller
    required property Window dialogOwner
    property string namePrefix: "create"
    property string heading: qsTrId("ui.ad9daebe00dc")
    property bool active: dialogOwner.visible
    property var inputs: []
    property string requestedKey: ""
    readonly property var inputPaths: inputs.map(item => item.path)
    readonly property real rowHeight: Math.max(Theme.operationCreateInputRowHeight,
        Theme.operationCreateBodySize * Theme.operationAddingRowLineFactor + Theme.operationAddingRowVerticalInset)
    function baseName(path: string): string {
        return path.replace(/\\/g, "/").split("/").pop();
    }
    function formatInputSize(value: var): string {
        const bytes = Number(value || 0);
        return bytes >= 1000000 ? (bytes / 1000000).toFixed(1) + " MB" : bytes >= 1000 ? Math.round(bytes / 1000) + " KB" : bytes + " B";
    }
    readonly property string inputSummary: {
        const files = inputs.filter(item => !item.directory);
        const directories = inputs.length - files.length;
        const parts = [];
        if (files.length)
            parts.push(qsTrId("ui.2e84a56c5f19", Number(files.length)));
        if (directories)
            parts.push(qsTrId("ui.43a28a01147e", Number(directories)));
        if (files.length && files.every(item => item.known))
            parts.push(formatInputSize(files.reduce((total, item) => total + Number(item.sizeBytes || 0), 0)));
        else if (files.length)
            parts.push(qsTrId("ui.db436a950681"));
        if (directories)
            parts.push(qsTrId("ui.2ad1d92b2858"));
        return parts.length ? parts.join(qsTrId("common.separator")) : qsTrId("ui.02256d747caf");
    }
    function addInputPaths(paths: var, directory: bool) {
        const next = inputs.slice();
        let added = false;
        // Native multi-selection can be large; deduplicate in one pass.
        const selected = new Set(next.map(item => item.path));
        for (let i = 0; i < paths.length; ++i) {
            const path = String(paths[i]);
            if (!selected.has(path)) {
                next.push({
                    path,
                    name: baseName(path),
                    directory
                });
                selected.add(path);
                added = true;
            }
        }
        if (added) inputs = next;
    }
    function setInputPaths(paths: var) {
        inputs = [];
        addInputPaths(paths, false);
    }
    function requestPreview(force: bool) {
        if (!active) return;
        const key = JSON.stringify(inputPaths.slice(0, 256));
        if (!force && requestedKey === key) return;
        requestedKey = key;
        controller.inspectCreationInputs(inputPaths);
        applyMetadata();
    }
    function applyMetadata() {
        if (!active) return;
        const metadata = new Map();
        for (const item of controller.creationInputs) metadata.set(item.path, item);
        let changed = false;
        const next = inputs.map(item => {
            if (!metadata.has(item.path)) return item;
            const merged = Object.assign({}, item, metadata.get(item.path));
            if (JSON.stringify(merged) === JSON.stringify(item)) return item;
            changed = true;
            return merged;
        });
        if (changed) inputs = next;
    }
    onInputPathsChanged: requestPreview(false)
    onActiveChanged: if (active) requestPreview(true)
    Connections {
        target: root.controller
        function onCreationInputsChanged() { root.applyMetadata(); }
    }
    implicitHeight: sourceContent.implicitHeight + Theme.operationCreateCardInset * 2
    color: Theme.operationCard
    border.color: Theme.operationCardBorder
    radius: Theme.operationAddingCardRadius
    ColumnLayout {
        id: sourceContent
        anchors.fill: parent
        anchors.margins: Theme.operationCreateCardInset
        spacing: Theme.operationAddingRowGap
        Text {
            objectName: root.namePrefix + "InputHeading"
            text: root.heading
            font.pixelSize: Theme.operationCreateBodySize
            font.bold: true
            color: Theme.text
        }
        Rectangle {
            objectName: root.namePrefix + "InputSurface"
            Layout.fillWidth: true
            implicitHeight: sourceList.implicitHeight + Theme.operationAddingSourceListInset * 2
            color: Theme.surface
            border.color: Theme.operationCardBorder
            radius: Theme.operationAddingCardRadius
            ColumnLayout {
                id: sourceList
                anchors.fill: parent
                anchors.margins: Theme.operationAddingSourceListInset
                spacing: Theme.operationAddingSourceContentGap
                RowLayout {
                    Layout.fillWidth: true
                    spacing: Theme.operationAddingRowGap
                    OperationAction {
                        objectName: root.namePrefix + "AddFiles"
                        text: qsTrId("ui.ad9daebe00dc")
                        primary: true
                        actionIcon: AssetLibrary.fileIcon("", false, false)
                        enabled: !root.controller.busy
                        onClicked: root.addInputPaths(root.controller.chooseCreationInputs(false, root.dialogOwner), false)
                    }
                    OperationAction {
                        objectName: root.namePrefix + "AddDirectories"
                        text: qsTrId("ui.45da2d9c85d1")
                        actionIcon: AssetLibrary.fileIcon("", true, false)
                        enabled: !root.controller.busy
                        onClicked: root.addInputPaths(root.controller.chooseCreationInputs(true, root.dialogOwner), true)
                    }
                    Item {
                        Layout.fillWidth: true
                    }
                }
                ListView {
                    id: inputList
                    objectName: root.namePrefix + "InputList"
                    Layout.fillWidth: true
                    Layout.preferredHeight: root.inputs.length ? Math.min(Theme.operationAddingSourceListMaximumHeight, root.inputs.length * root.rowHeight) : Theme.operationAddingSourceListMinimumHeight
                    implicitHeight: Layout.preferredHeight
                    clip: true
                    model: root.inputs
                    ScrollBar.vertical: ScrollBar {
                        policy: ScrollBar.AsNeeded
                    }
                    delegate: Rectangle {
                        id: inputRow
                        objectName: root.namePrefix + "InputRow-" + index
                        required property var modelData
                        required property int index
                        width: inputList.width
                        height: root.rowHeight
                        color: rowHover.hovered ? Theme.controlHover : Theme.surface
                        Rectangle {
                            anchors.left: parent.left
                            anchors.right: parent.right
                            anchors.bottom: parent.bottom
                            height: 1
                            color: Theme.operationCardBorder
                        }
                        HoverHandler {
                            id: rowHover
                        }
                        RowLayout {
                            anchors.fill: parent
                            spacing: Theme.operationAddingRowGap
                            Image {
                                source: AssetLibrary.fileIcon(inputRow.modelData.name, inputRow.modelData.directory, false)
                                Layout.preferredWidth: Theme.operationAddingSourceIconSize
                                Layout.preferredHeight: Theme.operationAddingSourceIconSize
                            }
                            Text {
                                Layout.fillWidth: true
                                text: inputRow.modelData.name
                                textFormat: Text.PlainText
                                elide: Text.ElideMiddle
                                font.pixelSize: Theme.operationCreateBodySize
                                color: Theme.text
                                Accessible.description: inputRow.modelData.path
                            }
                            Text {
                                objectName: root.namePrefix + "InputSize-" + inputRow.index
                                visible: inputRow.modelData.known === true
                                text: root.formatInputSize(inputRow.modelData.sizeBytes)
                                font.pixelSize: Theme.operationCreateNoteSize
                                color: Theme.textSecondary
                                horizontalAlignment: Text.AlignRight
                            }
                            OperationRemoveButton {
                                objectName: root.namePrefix + "RemoveInput-" + inputRow.index
                                entryName: inputRow.modelData.name
                                enabled: !root.controller.busy
                                onClicked: root.inputs = root.inputs.filter((_, i) => i !== inputRow.index)
                            }
                        }
                    }
                }
                Text {
                    objectName: root.namePrefix + "InputSummary"
                    Layout.fillWidth: true
                    wrapMode: Text.Wrap
                    text: root.inputSummary
                    font.pixelSize: Theme.operationCreateNoteSize
                    color: Theme.textSecondary
                }
            }
        }
    }
}
