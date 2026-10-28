pragma ComponentBehavior: Bound
import QtQuick

Rectangle {
    id: root
    property string text: ""
    property color ink: Theme.accentInk
    implicitWidth: label.implicitWidth + Theme.menuTagHorizontalInset * 2
    implicitHeight: label.implicitHeight + Theme.menuTagVerticalInset * 2
    radius: Theme.menuTagRadius
    color: Qt.rgba(ink.r, ink.g, ink.b, 0.10)
    Text {
        id: label
        objectName: root.objectName + "Text"
        width: implicitWidth
        height: implicitHeight
        x: (root.width - width) / 2
        y: (root.height - height) / 2
        text: root.text
        textFormat: Text.PlainText
        font.pixelSize: Theme.menuShortcutSize
        color: root.ink
    }
}
