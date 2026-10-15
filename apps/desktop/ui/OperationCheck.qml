pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

CheckBox {
    id: root
    focusPolicy: Qt.StrongFocus
    property bool radioStyle: false
    property string trailingText: ""
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
            border.width: root.checked || root.radioStyle ? 0 : 1
            border.color: root.checked || root.hovered ? Theme.accent : Theme.textSecondary
            Image {
                visible: root.radioStyle || root.checked
                anchors.centerIn: parent
                width: parent.width * 24 / 19; height: width
                source: "qrc:/xdzip/assets/" + (Theme.dark ? "dark/" : "light/") + (root.radioStyle ? (root.checked ? "radio-selected.png" : "radio-unselected.png") : "check-selected.png")
            }
        }
    }
    contentItem: RowLayout {
        spacing: 0
        Item { implicitWidth: root.indicator.width + root.spacing; implicitHeight: 1 }
        Text {
            objectName: root.objectName ? root.objectName + "Label" : ""
            Layout.fillWidth: true
            text: root.text; textFormat: Text.PlainText
            font.pixelSize: Math.round(Theme.operationBodySize * root.density)
            color: root.enabled ? Theme.text : Theme.textSecondary
            verticalAlignment: Text.AlignVCenter; wrapMode: Text.Wrap
        }
        Text {
            objectName: root.objectName ? root.objectName + "Count" : ""
            visible: root.trailingText.length > 0
            Layout.leftMargin: 12
            text: root.trailingText; textFormat: Text.PlainText
            font.pixelSize: Math.round(Theme.operationBodySize * root.density)
            color: Theme.textSecondary
            horizontalAlignment: Text.AlignRight
        }
    }
}
