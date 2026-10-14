import QtQuick
import QtQuick.Controls

TextField {
    id: root
    property real density: 1
    property bool framed: true
    // Show the resolved destination while retaining an editable base directory.
    property string previewText: ""
    font.pixelSize: Math.round(Theme.operationBodySize * density)
    color: !root.activeFocus && root.previewText ? "transparent" : Theme.text
    selectByMouse: true
    leftPadding: root.framed ? 12 * root.density : 0
    rightPadding: root.framed ? 12 * root.density : 0
    implicitHeight: root.framed ? Math.max(Theme.operationFieldHeight * root.density, contentHeight + 16 * root.density) : contentHeight
    Text {
        visible: !root.activeFocus && root.previewText.length > 0
        x: root.leftPadding; width: root.width - root.leftPadding - root.rightPadding
        anchors.verticalCenter: parent.verticalCenter
        text: root.previewText; textFormat: Text.PlainText
        font: root.font; color: Theme.text; elide: Text.ElideMiddle
    }
    background: Rectangle {
        radius: 6
        color: root.framed ? Theme.surface : "transparent"
        border.color: root.framed ? (root.activeFocus ? Theme.accent : Theme.operationFieldBorder) : "transparent"
        border.width: root.activeFocus ? 2 : 1
    }
}
