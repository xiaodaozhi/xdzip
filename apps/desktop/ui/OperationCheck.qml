pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls

CheckBox {
    id: root
    property bool radioStyle: false
    autoExclusive: radioStyle
    hoverEnabled: true
    padding: 0
    spacing: Theme.operationControlGap
    implicitHeight: Math.max(28, contentItem.implicitHeight + 4)
    indicator: Rectangle {
        x: root.leftPadding
        y: (root.height - height) / 2
        width: Theme.operationControlIndicatorSize
        height: width
        radius: root.radioStyle ? width / 2 : 4
        color: root.checked && !root.radioStyle ? Theme.accent : Theme.surface
        border.width: root.activeFocus ? 2 : 1
        border.color: root.checked || root.hovered || root.activeFocus ? Theme.accent : Theme.textSecondary
        Rectangle {
            visible: root.radioStyle && root.checked
            anchors.centerIn: parent
            width: parent.width * .55; height: width; radius: width / 2
            color: Theme.accent
        }
        Image {
            visible: !root.radioStyle && root.checked
            anchors.fill: parent
            source: "qrc:/xdzip/assets/dark/check.png"
        }
    }
    contentItem: Text {
        text: root.text
        textFormat: Text.PlainText
        font.pixelSize: Theme.operationBodySize
        color: root.enabled ? Theme.text : Theme.textSecondary
        leftPadding: root.indicator.width + root.spacing
        verticalAlignment: Text.AlignVCenter
        wrapMode: Text.Wrap
    }
}
