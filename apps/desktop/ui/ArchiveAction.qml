import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Button {
    id: root
    property string asset: ""
    property bool primary: false
    property bool toolbarStyle: false
    property bool selected: false
    // Reserved presentation for the unavailable extraction action; no dropdown is activated.
    property bool splitIndicator: false
    property string explanation: text
    property real glyphSize: toolbarStyle ? Theme.mainToolbarIconSize : Theme.mainViewIconSize
    implicitHeight: Math.max(toolbarStyle ? Theme.mainToolbarControlHeight : Theme.mainViewButtonHeight, label.implicitHeight + (toolbarStyle ? 12 : 8))
    implicitWidth: Math.max(toolbarStyle && !text.length ? Theme.toolbarButtonHeight : 0, contentItem.implicitWidth + leftPadding + rightPadding)
    leftPadding: primary && toolbarStyle ? 20 : toolbarStyle ? 8 : 12
    rightPadding: splitIndicator ? Theme.mainSplitWidth + 12 : leftPadding
    topPadding: 0
    bottomPadding: 0
    hoverEnabled: true
    Accessible.name: text
    Accessible.description: explanation
    CommandTooltip {
        requested: root.hovered && !root.down
        text: root.explanation
    }
    contentItem: RowLayout {
        spacing: root.toolbarStyle ? Theme.mainToolbarTextGap : 8
        Image {
            objectName: "actionIcon"
            visible: root.asset.length > 0
            source: root.asset.length ? "qrc:/xdzip/assets/" + (Theme.dark ? "dark/" : "light/") + root.asset + ".png" : ""
            Layout.preferredWidth: root.glyphSize
            Layout.preferredHeight: root.glyphSize
            opacity: root.enabled ? 1 : 0.6
        }
        Text {
            id: label
            objectName: "actionLabel"
            // Keep the text box centered even when native font metrics have odd heights.
            Layout.fillHeight: true
            visible: text.length > 0
            text: root.text
            textFormat: Text.PlainText
            font.pixelSize: root.toolbarStyle ? Theme.mainToolbarSize : Theme.mainHeaderSize
            font.weight: root.toolbarStyle ? Font.Medium : Font.Normal
            color: root.primary ? Theme.accentText : root.enabled ? Theme.text : Theme.textSecondary
            verticalAlignment: Text.AlignVCenter
        }
    }
    background: Rectangle {
        objectName: "actionBackground"
        radius: root.toolbarStyle ? Theme.toolbarRadius : Theme.controlRadius
        color: root.primary ? (!root.enabled ? Theme.accentDisabled : root.down ? Qt.darker(Theme.accent, 1.22) : root.hovered ? Qt.darker(Theme.accent, 1.12) : Theme.accent) : !root.enabled ? "transparent" : root.down ? Theme.controlPressed : root.selected ? Theme.selection : root.hovered ? Theme.controlHover : "transparent"
        border.width: root.activeFocus ? 2 : 0
        border.color: Theme.accent
    }
    Rectangle {
        visible: root.splitIndicator
        anchors.right: parent.right
        anchors.rightMargin: Theme.mainSplitWidth
        width: 1
        height: parent.height
        color: Theme.accentText
        opacity: 0.3
    }
    Image {
        objectName: "actionSplitIndicator"
        visible: root.splitIndicator
        anchors.right: parent.right
        anchors.rightMargin: 11
        anchors.verticalCenter: parent.verticalCenter
        width: 18
        height: 18
        source: visible ? "qrc:/xdzip/assets/primary-chevron.png" : ""
    }
}
