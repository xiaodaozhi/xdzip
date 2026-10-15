import QtQuick
import QtQuick.Controls

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
    implicitHeight: Math.max(toolbarStyle ? Theme.mainToolbarControlHeight : Theme.mainViewButtonHeight, label.implicitHeight + (toolbarStyle ? Theme.mainUnits(12) : Theme.mainUnits(8)))
    implicitWidth: Math.max(toolbarStyle && !text.length ? Theme.toolbarButtonHeight : 0, contentItem.implicitWidth + leftPadding + rightPadding)
    leftPadding: !text.length ? 0 : primary && toolbarStyle ? Theme.mainUnits(20) : toolbarStyle ? Theme.mainUnits(8) : Theme.mainUnits(12)
    rightPadding: splitIndicator ? Theme.mainSplitWidth + Theme.mainUnits(12) : leftPadding
    topPadding: 0
    bottomPadding: 0
    hoverEnabled: true
    Accessible.name: text
    Accessible.description: explanation
    CommandTooltip {
        requested: root.hovered && !root.down
        text: root.explanation
    }
    property real labelSize: toolbarStyle ? Theme.mainToolbarSize : Theme.mainHeaderSize
    contentItem: Item {
        implicitWidth: actionRow.implicitWidth
        implicitHeight: Math.max(root.glyphSize, label.implicitHeight)
        Row {
            id: actionRow
            anchors.verticalCenter: parent.verticalCenter
            x: root.text.length ? 0 : (parent.width - width) / 2
            height: parent.height
            spacing: root.toolbarStyle ? Theme.mainToolbarContentGap : Theme.mainUnits(8)
            Image {
                objectName: "actionIcon"
                visible: root.asset.length > 0
                source: visible ? "qrc:/xdzip/assets/" + (Theme.dark ? "dark/" : "light/") + root.asset + ".png" : ""
                width: root.glyphSize
                height: root.glyphSize
                anchors.verticalCenter: parent.verticalCenter
                anchors.alignWhenCentered: false
                fillMode: Image.PreserveAspectFit
                opacity: root.enabled ? 1 : 0.6
            }
            Text {
                id: label
                objectName: "actionLabel"
                visible: text.length > 0
                height: parent.height
                text: root.text
                textFormat: Text.PlainText
                font.pixelSize: root.labelSize
                font.weight: root.toolbarStyle ? Font.Medium : Font.Normal
                color: !root.enabled ? (root.primary ? Theme.accentDisabledText : Theme.textSecondary) : root.primary ? Theme.accentText : Theme.text
                verticalAlignment: Text.AlignVCenter
            }
        }
    }
    background: Rectangle {
        objectName: "actionBackground"
        radius: root.toolbarStyle ? Theme.mainToolbarRadius : Theme.mainControlRadius
        color: root.primary && !root.enabled ? Theme.accentDisabled : root.primary ? (root.down ? Theme.accentPressed : root.hovered ? Theme.accentHover : Theme.accent) : !root.enabled ? "transparent" : root.down ? Theme.controlPressed : root.selected ? Theme.selection : root.hovered ? Theme.controlHover : "transparent"
        border.width: root.activeFocus ? 2 : 0
        border.color: Theme.accent
    }
    Rectangle {
        visible: root.splitIndicator
        anchors.right: parent.right
        anchors.rightMargin: Theme.mainSplitWidth
        width: 1
        height: parent.height
        objectName: "actionSplitDivider"
        color: Theme.dark ? Theme.border : Theme.accentText
        opacity: Theme.dark ? 1 : 0.3
    }
    Image {
        objectName: "actionSplitIndicator"
        visible: root.splitIndicator
        anchors.right: parent.right
        anchors.rightMargin: Theme.mainUnits(11)
        anchors.verticalCenter: parent.verticalCenter
        width: Theme.mainUnits(18)
        height: Theme.mainUnits(18)
        source: visible ? "qrc:/xdzip/assets/primary-chevron.png" : ""
    }
}
