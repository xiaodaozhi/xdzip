pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Layouts

IntentCheckBox {
    id: root
    property bool value: false
    property string description: ""
    signal selectionRequested(bool value)
    Component.onCompleted: checked = value
    onValueChanged: if (checked !== value) checked = value
    focusPolicy: Qt.StrongFocus
    hoverEnabled: true
    padding: 0
    spacing: 12
    implicitHeight: Math.max(36, contentItem.implicitHeight)
    Accessible.name: description.length ? text + ", " + description : text
    onClicked: {
        root.selectionRequested(root.checked);
        // System confirmation and failed writes can leave the model unchanged.
        if (root.checked !== root.value) root.checked = root.value;
    }
    indicator: null
    background: Item {}
    contentItem: RowLayout {
        spacing: root.spacing
        ColumnLayout {
            Layout.fillWidth: true
            spacing: 4
            Text {
                Layout.fillWidth: true
                text: root.text
                textFormat: Text.PlainText
                font.pixelSize: Math.round(Theme.settingsBodySize * Theme.fontScale)
                color: root.enabled ? Theme.text : Theme.textSecondary
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
        Rectangle {
            Layout.alignment: Qt.AlignVCenter
            implicitWidth: Theme.settingsSwitchWidth
            implicitHeight: Theme.settingsSwitchHeight
            radius: height / 2
            color: root.checked ? Theme.accent : Theme.taskTrack
            opacity: root.enabled ? 1 : 0.5
            Rectangle {
                width: Theme.settingsSwitchThumb
                height: width
                radius: width / 2
                anchors.verticalCenter: parent.verticalCenter
                x: root.checked ? parent.width - width - 2 : 2
                color: "white"
            }
        }
    }
}
