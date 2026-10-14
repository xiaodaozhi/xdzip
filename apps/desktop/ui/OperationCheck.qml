pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls

CheckBox {
    id: root
    property bool radioStyle: false
    property real density: 1
    readonly property real indicatorSize: Theme.operationControlIndicatorSize * (radioStyle ? 1 : density)
    property real indicatorSlotSize: indicatorSize
    autoExclusive: radioStyle
    hoverEnabled: true
    padding: 0
    spacing: Theme.operationControlGap * density
    implicitHeight: Math.max(indicatorSlotSize, 28 * density, contentItem.implicitHeight + 4 * density)
    indicator: Item {
        objectName: root.objectName ? root.objectName + "Indicator" : ""
        x: root.leftPadding
        y: (root.height - height) / 2
        width: root.indicatorSlotSize
        height: width
        Rectangle {
            anchors.centerIn: parent
            width: root.indicatorSize; height: width
            radius: root.radioStyle ? width / 2 : 4 * root.density
            color: root.checked || root.radioStyle ? "transparent" : Theme.surface
            border.width: root.activeFocus ? 2 : root.checked || root.radioStyle ? 0 : 1
            border.color: root.checked || root.hovered || root.activeFocus ? Theme.accent : Theme.textSecondary
            Image {
                visible: root.radioStyle || root.checked
                anchors.centerIn: parent
                width: parent.width * 24 / 19; height: width
                source: "qrc:/xdzip/assets/" + (Theme.dark ? "dark/" : "light/") + (root.radioStyle ? (root.checked ? "radio-selected.png" : "radio-unselected.png") : "check-selected.png")
            }
        }
    }
    contentItem: Text {
        objectName: root.objectName ? root.objectName + "Label" : ""
        text: root.text
        textFormat: Text.PlainText
        font.pixelSize: Math.round(Theme.operationBodySize * root.density)
        color: root.enabled ? Theme.text : Theme.textSecondary
        leftPadding: root.indicator.width + root.spacing
        verticalAlignment: Text.AlignVCenter
        wrapMode: Text.Wrap
    }
}
