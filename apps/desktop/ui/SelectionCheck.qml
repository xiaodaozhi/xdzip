import QtQuick
import QtQuick.Controls

CheckBox {
    id: control
    padding: 0
    implicitWidth: Theme.mainCheckboxSize
    implicitHeight: Theme.mainCheckboxSize
    hoverEnabled: true
    contentItem: Item {}
    indicator: Rectangle {
        width: Theme.mainCheckboxSize
        height: Theme.mainCheckboxSize
        anchors.centerIn: parent
        radius: 3.3
        color: control.checkState === Qt.Unchecked ? (control.down ? Theme.controlPressed : control.hovered ? Theme.controlHover : Theme.surface) : Theme.accent
        border.width: control.activeFocus ? 2 : 1
        border.color: control.activeFocus || control.hovered || control.checkState !== Qt.Unchecked ? Theme.accent : Theme.textSecondary
        Image {
            anchors.fill: parent
            visible: control.checkState === Qt.Checked
            source: "qrc:/xdzip/assets/dark/check.png"
        }
        Rectangle {
            anchors.centerIn: parent
            width: parent.width * 0.55
            height: 2.5
            color: Theme.accentText
            visible: control.checkState === Qt.PartiallyChecked
        }
    }
}
