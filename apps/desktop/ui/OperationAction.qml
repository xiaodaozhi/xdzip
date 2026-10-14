pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Layouts

AppButton {
    id: root
    required property url actionIcon
    property real iconSize: 16
    labelSize: Theme.operationFooterFontSize
    baseHeight: 28
    cornerRadius: 5
    horizontalInset: 12
    verticalInset: 5
    contentItem: Item {
        implicitWidth: group.implicitWidth
        implicitHeight: group.implicitHeight
        RowLayout {
            id: group
            anchors.centerIn: parent
            spacing: 6
            Image {
                source: root.actionIcon
                Layout.preferredWidth: root.iconSize
                Layout.preferredHeight: root.iconSize
                opacity: root.enabled ? 1 : .5
            }
            Text {
                text: root.text
                textFormat: Text.PlainText
                font.pixelSize: root.labelSize
                color: !root.enabled ? Theme.textSecondary : root.primary ? Theme.accentText : Theme.text
            }
        }
    }
}
