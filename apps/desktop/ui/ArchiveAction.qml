import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Button {
    id: root
    property string asset: ""
    property bool primary: false
    property string explanation: text
    implicitHeight: Math.max(36, label.implicitHeight + 12)
    implicitWidth: contentItem.implicitWidth + 24
    leftPadding: 12
    rightPadding: 12
    topPadding: 6
    bottomPadding: 6
    hoverEnabled: true
    Accessible.name: text
    ToolTip.visible: hovered && explanation.length > 0
    ToolTip.text: explanation
    contentItem: RowLayout {
        spacing: 8
        opacity: root.enabled ? 1 : 0.45
        Image {
            visible: root.asset.length > 0
            source: root.asset.length ? "qrc:/xdzip/assets/" + (Theme.dark ? "dark/" : "light/") + root.asset + ".png" : ""
            Layout.preferredWidth: 18
            Layout.preferredHeight: 18
        }
        Text {
            id: label
            visible: text.length > 0
            text: root.text
            textFormat: Text.PlainText
            font.pixelSize: Theme.bodySize
            color: root.primary ? Theme.accentText : Theme.text
            verticalAlignment: Text.AlignVCenter
        }
    }
    background: Rectangle {
        radius: Theme.controlRadius
        color: root.primary ? Theme.accent : root.down || root.hovered ? Theme.sidebar : "transparent"
        opacity: root.enabled ? 1 : 0.55
        border.width: root.activeFocus ? 2 : 0
        border.color: Theme.accent
    }
}
