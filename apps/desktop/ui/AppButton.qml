import QtQuick
import QtQuick.Controls

Button {
    id: root
    property bool primary: false
    property string explanation: ""
    property real density: 1
    hoverEnabled: true
    implicitHeight: Math.max(Theme.buttonHeight * density, contentItem.implicitHeight + 12 * density)
    implicitWidth: contentItem.implicitWidth + Theme.gutter * 2 * density
    leftPadding: Theme.gap * density
    rightPadding: Theme.gap * density
    topPadding: 6 * density
    bottomPadding: 6 * density
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
        font.pixelSize: Math.round(Theme.bodySize / Theme.fontScale * root.density) * Theme.fontScale
    }
    background: Rectangle {
        radius: Theme.controlRadius * root.density
        color: !root.enabled ? (root.primary ? Theme.accentDisabled : Theme.surface) : root.primary ? (root.down ? Qt.darker(Theme.accent, 1.22) : root.hovered ? Qt.darker(Theme.accent, 1.12) : Theme.accent) : root.down ? Theme.controlPressed : root.hovered ? Theme.controlHover : Theme.surface
        border.color: root.activeFocus ? Theme.accent : Theme.border
        border.width: root.activeFocus ? 2 : 1
    }
}
