pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Button {
    id: root
    required property string title
    required property string description
    required property string asset
    property string shortcutText: ""
    property bool primary: false
    property bool busy: false
    hoverEnabled: true
    padding: Theme.welcomeActionInset
    implicitHeight: Math.max(Theme.welcomeActionHeight, labels.implicitHeight + 2 * padding)
    implicitWidth: (Theme.welcomeContentWidth - Theme.welcomeActionGap) / 2
    Accessible.name: title
    Accessible.description: description + (shortcutText.length ? ", " + shortcutText : "")
    contentItem: RowLayout {
        spacing: 14
        BusyIndicator {
            objectName: "welcomeBusyIndicator"
            visible: root.busy
            running: root.busy
            Layout.alignment: Qt.AlignTop
            Layout.preferredWidth: Theme.welcomeActionIcon
            Layout.preferredHeight: Theme.welcomeActionIcon
        }
        Image {
            visible: !root.busy
            Layout.alignment: Qt.AlignTop
            Layout.topMargin: 2
            Layout.preferredWidth: Theme.welcomeActionIcon
            Layout.preferredHeight: Theme.welcomeActionIcon
            source: "qrc:/xdzip/assets/" + (root.primary || Theme.dark ? "dark/" : "light/") + root.asset + ".png"
            fillMode: Image.PreserveAspectFit
        }
        ColumnLayout {
            id: labels
            Layout.fillWidth: true
            spacing: 0
            Text {
                Layout.fillWidth: true
                text: root.busy ? qsTr("正在打开…") : root.title
                textFormat: Text.PlainText
                font.pixelSize: Theme.welcomeActionTitleSize * Theme.fontScale
                font.weight: Font.DemiBold
                color: !root.enabled ? Theme.accentDisabledText : root.primary ? Theme.accentText : Theme.text
                wrapMode: Text.Wrap
            }
            Text {
                Layout.topMargin: 7
                Layout.fillWidth: true
                text: root.description
                textFormat: Text.PlainText
                font.pixelSize: Theme.welcomeActionBodySize * Theme.fontScale
                lineHeightMode: Text.FixedHeight
                lineHeight: 20 * Theme.fontScale
                color: !root.enabled ? Theme.accentDisabledText : root.primary ? Theme.accentText : Theme.textSecondary
                opacity: root.enabled && root.primary ? .86 : 1
                wrapMode: Text.Wrap
            }
            Text {
                Layout.topMargin: 13
                text: root.shortcutText
                font.pixelSize: 11 * Theme.fontScale
                color: !root.enabled ? Theme.accentDisabledText : root.primary ? Theme.accentText : Theme.textSecondary
                opacity: .7
            }
        }
    }
    background: Rectangle {
        radius: Theme.welcomeActionRadius
        color: !root.enabled ? Theme.accentDisabled : root.primary ? (root.down ? Theme.accentPressed : root.hovered ? Theme.accentHover : Theme.accent) : root.down ? Theme.selection : root.hovered ? Theme.canvas : Theme.surface
        border.color: !root.enabled ? Theme.accentDisabled : root.primary ? color : root.hovered ? Theme.operationFieldBorder : Theme.border
        Rectangle {
            anchors.fill: parent
            anchors.margins: -5
            radius: 11
            color: "transparent"
            border.width: 2
            border.color: Theme.accentInk
            visible: root.visualFocus
        }
    }
}
