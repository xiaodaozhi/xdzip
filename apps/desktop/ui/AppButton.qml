import QtQuick
import QtQuick.Controls

Button {
    id: root
    property real baseHeight: Theme.buttonHeight
    property real minimumWidth: 0
    property real horizontalInset: Theme.gutter
    property real labelSize: Theme.bodySize
    property real verticalInset: 6
    property color surfaceColor: Theme.surface
    property color outlineColor: Theme.border
    property bool wrapLabel: false
    property bool primary: false
    property bool dangerous: false
    property real cornerRadius: Theme.controlRadius
    property string explanation: ""
    property real density: 1
    hoverEnabled: true
    implicitHeight: Math.max(root.baseHeight * density, contentItem.implicitHeight + root.verticalInset * 2 * density)
    implicitWidth: Math.max(root.minimumWidth, contentItem.implicitWidth + root.horizontalInset * 2 * density)
    leftPadding: Theme.gap * density
    rightPadding: Theme.gap * density
    topPadding: root.verticalInset * density
    bottomPadding: root.verticalInset * density
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
        wrapMode: root.wrapLabel ? Text.Wrap : Text.NoWrap
        horizontalAlignment: Text.AlignHCenter
        verticalAlignment: Text.AlignVCenter
        color: !root.enabled ? (root.primary ? Theme.accentDisabledText : Theme.textSecondary) : root.primary ? Theme.accentText : root.dangerous ? Theme.menuDanger : Theme.text
        font.pixelSize: Math.round(root.labelSize / Theme.fontScale * root.density) * Theme.fontScale
    }
    background: Rectangle {
        objectName: root.objectName ? root.objectName + "Background" : ""
        radius: root.cornerRadius * root.density
        color: !root.enabled ? (root.primary ? Theme.accentDisabled : root.surfaceColor) : root.dangerous ? Qt.rgba(Theme.menuDanger.r, Theme.menuDanger.g, Theme.menuDanger.b, root.down ? .2 : root.hovered ? .12 : .06) : root.primary ? (root.down ? Theme.accentPressed : root.hovered ? Theme.accentHover : Theme.accent) : root.down ? Theme.controlPressed : root.hovered ? Theme.controlHover : root.surfaceColor
        border.color: root.dangerous ? Theme.menuDanger : root.outlineColor
        border.width: 1
    }
}
