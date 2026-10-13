import QtQuick
import QtQuick.Controls

TextField {
    id: root
    font.pixelSize: Theme.operationBodySize
    color: Theme.text
    selectByMouse: true
    leftPadding: 12
    rightPadding: 12
    implicitHeight: Math.max(Theme.operationFieldHeight, contentHeight + 16)
    background: Rectangle {
        radius: 6
        color: Theme.surface
        border.color: root.activeFocus ? Theme.accent : Theme.taskBorder
        border.width: root.activeFocus ? 2 : 1
    }
}
