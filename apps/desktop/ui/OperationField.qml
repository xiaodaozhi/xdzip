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
    leftPadding: root.framed ? Theme.operationControlFieldTextInset : 0
    rightPadding: root.framed ? Theme.operationControlFieldTextInset : 0
    implicitHeight: root.framed ? Math.max(Theme.operationFieldHeight * root.density, contentHeight + 16 * root.density) : contentHeight
    Text {
        visible: !root.activeFocus && root.previewText.length > 0
        x: root.leftPadding
        width: root.width - root.leftPadding - root.rightPadding
        anchors.verticalCenter: parent.verticalCenter
        text: root.previewText
        textFormat: Text.PlainText
        font: root.font
        color: Theme.text
        elide: Text.ElideMiddle
    }
    background: OperationFrame {
        objectName: root.objectName + "Background"
        control: root
        framed: root.framed
        hovered: root.hovered
    }
}
