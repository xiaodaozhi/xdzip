import QtQuick
import QtQuick.Controls

Button {
    id: root
    property bool primary: false
    implicitHeight: Math.max(Theme.buttonHeight, contentItem.implicitHeight + Theme.gap * 2)
    implicitWidth: contentItem.implicitWidth + Theme.gutter * 2
    padding: Theme.gap
    focusPolicy: Qt.StrongFocus
    Accessible.name: text
    contentItem: Text {
        text: root.text
        textFormat: Text.PlainText
        horizontalAlignment: Text.AlignHCenter
        verticalAlignment: Text.AlignVCenter
        color: root.primary ? Theme.accentText : Theme.text
        opacity: root.enabled ? 1 : 0.45
        font.pixelSize: Theme.bodySize
    }
    background: Rectangle {
        radius: Theme.controlRadius
        color: root.primary ? Theme.accent : (root.down || root.hovered ? Theme.sidebar : Theme.surface)
        border.color: root.activeFocus ? Theme.accent : Theme.border
        border.width: root.activeFocus ? 2 : 1
    }
}
