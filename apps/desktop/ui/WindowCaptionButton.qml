import QtQuick
import QtQuick.Controls

Button {
    id: root
    required property string captionAction
    property bool linuxStyle: false
    readonly property color faceColor: down ? (captionAction === "close" ? Theme.captionClosePressed : Theme.controlPressed)
                                           : hovered ? (captionAction === "close" ? Theme.captionCloseHover : Theme.controlHover) : Theme.canvas
    readonly property string iconTheme: captionAction === "close" && (hovered || down) ? "dark/" : Theme.dark ? "dark/" : "light/"
    width: linuxStyle ? Theme.mainLinuxCaptionWidth : Theme.mainCaptionButtonWidth
    hoverEnabled: true
    focusPolicy: Qt.StrongFocus
    padding: 0
    Accessible.name: text
    CommandTooltip { requested: root.hovered && !root.down; text: root.text }
    background: Rectangle {
        anchors.centerIn: parent
        width: root.linuxStyle ? Theme.mainLinuxCaptionSize : root.width
        height: root.linuxStyle ? Theme.mainLinuxCaptionSize : root.height
        radius: root.linuxStyle ? width / 2 : 0
        color: root.linuxStyle && !root.hovered && !root.down ? Theme.controlHover : root.faceColor
    }
    contentItem: Item {
        Item {
            anchors.centerIn: parent
            width: Theme.mainCaptionIconSize
            height: width
            Image {
                visible: root.captionAction !== "restore"
                anchors.fill: parent
                source: "qrc:/xdzip/assets/" + root.iconTheme + (root.captionAction === "restore" ? "maximize" : root.captionAction) + ".png"
            }
            Item {
                visible: root.captionAction === "restore"
                anchors.fill: parent
                Image {
                    x: Theme.mainCaptionRestoreOffset
                    width: parent.width - x
                    height: width
                    source: "qrc:/xdzip/assets/" + root.iconTheme + "maximize.png"
                }
                Rectangle {
                    y: Theme.mainCaptionRestoreOffset
                    width: parent.width - y
                    height: width
                    color: root.linuxStyle && !root.hovered && !root.down ? Theme.controlHover : root.faceColor
                    Image {
                        anchors.fill: parent
                        source: "qrc:/xdzip/assets/" + root.iconTheme + "maximize.png"
                    }
                }
            }
        }
    }
}
