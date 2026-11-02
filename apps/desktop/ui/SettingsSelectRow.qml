pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Layouts

RowLayout {
    id: root
    property string label: ""
    property string description: ""
    property var options: []
    property int selectedIndex: 0
    property string controlName: ""
    signal selectionRequested(int index)
    spacing: 16
    ColumnLayout {
        Layout.fillWidth: true
        spacing: 4
        Text {
            Layout.fillWidth: true
            text: root.label
            textFormat: Text.PlainText
            font.pixelSize: Math.round(Theme.settingsBodySize * Theme.fontScale)
            color: Theme.text
            wrapMode: Text.Wrap
        }
        Text {
            visible: root.description.length > 0
            Layout.fillWidth: true
            text: root.description
            textFormat: Text.PlainText
            font.pixelSize: Math.round(Theme.settingsNoteSize * Theme.fontScale)
            color: Theme.textSecondary
            wrapMode: Text.Wrap
        }
    }
    OperationSelect {
        id: select
        objectName: root.controlName
        Accessible.name: root.label
        Layout.preferredWidth: Math.max(160 * Theme.fontScale,
            select.widestOption + select.leftPadding + select.rightPadding + 4)
        Layout.minimumWidth: 80
        density: Theme.settingsBodySize * Theme.fontScale / Theme.operationBodySize
        implicitHeight: Math.max(Theme.operationControlBodyButtonHeight, contentItem.implicitHeight + 8)
        model: root.options
        currentIndex: root.selectedIndex
        onActivated: index => {
            root.selectionRequested(index);
            currentIndex = root.selectedIndex;
        }
        Connections {
            target: root
            function onSelectedIndexChanged() { select.currentIndex = root.selectedIndex; }
        }
    }
}
