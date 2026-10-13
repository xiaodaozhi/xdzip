import QtQuick
import QtQuick.Controls

Button {
    id: root
    property bool primary: false
    property string explanation: ""
    hoverEnabled: true
    implicitHeight: Math.max(Theme.buttonHeight, contentItem.implicitHeight + 12)
    implicitWidth: contentItem.implicitWidth + Theme.gutter * 2
    leftPadding: Theme.gap
    rightPadding: Theme.gap
    topPadding: 6
    bottomPadding: 6
    focusPolicy: Qt.StrongFocus
    Accessible.name: text
    Accessible.description: explanation
    CommandTooltip {
        requested: root.hovered && !root.down
        text: root.explanation
    }
    contentItem: Text {
        text: root.text
        textFormat: Text.PlainText
        horizontalAlignment: Text.AlignHCenter
        verticalAlignment: Text.AlignVCenter
        color: !root.enabled ? Theme.textSecondary : root.primary ? Theme.accentText : Theme.text
        font.pixelSize: Theme.bodySize
    }
    background: Rectangle {
        radius: Theme.controlRadius
        color: !root.enabled ? (root.primary ? Theme.accentDisabled : Theme.surface) : root.primary ? (root.down ? Qt.darker(Theme.accent, 1.22) : root.hovered ? Qt.darker(Theme.accent, 1.12) : Theme.accent) : root.down ? Theme.controlPressed : root.hovered ? Theme.controlHover : Theme.surface
        border.color: root.activeFocus ? Theme.accent : Theme.border
        border.width: root.activeFocus ? 2 : 1
    }
}
