pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

MenuItem {
    id: root
    property string asset: ""
    property string shortcutLabel: ""
    property string statusLabel: ""
    property color statusInk: Theme.accentInk
    property string disabledReason: ""
    property bool dangerous: false
    property bool commandVisible: true
    visible: commandVisible
    implicitWidth: Theme.menuWidth - Theme.menuPadding * 2
    // Do not use effective visible: closing the popup also hides every normal item.
    // ListView still lays out omitted commands unless their height is zero.
    implicitHeight: commandVisible ? Math.max(Theme.menuRowHeight, contentItem.implicitHeight + 8) : 0
    leftPadding: Theme.menuLabelInset
    rightPadding: Theme.menuLabelInset
    topPadding: 4
    bottomPadding: 4
    property real labelSize: Theme.menuLabelSize
    font.pixelSize: labelSize
    Accessible.name: statusLabel.length ? text + qsTrId("common.separator") + statusLabel : text
    Accessible.description: enabled ? "" : disabledReason
    indicator: null
    arrow: null
    contentItem: RowLayout {
        spacing: Theme.menuLabelInset
        Item {
            Layout.preferredWidth: Theme.menuIconSize
            Layout.preferredHeight: Theme.menuIconSize
            Image {
                anchors.fill: parent
                visible: root.checkable && root.checked
                source: "qrc:/xdzip/assets/" + (Theme.dark ? "dark/" : "light/") + "check.png"
                opacity: root.enabled ? 1 : 0.45
                fillMode: Image.PreserveAspectFit
            }
            Image {
                anchors.fill: parent
                visible: !root.checkable && root.asset.length > 0
                source: root.asset.length ? "qrc:/xdzip/assets/" + (Theme.dark ? "dark/" : "light/") + root.asset + ".png" : ""
                opacity: root.enabled ? 1 : 0.45
                fillMode: Image.PreserveAspectFit
            }
        }
        ColumnLayout {
            Layout.fillWidth: true
            spacing: 2
            Text {
                Layout.fillWidth: true
                text: root.text
                textFormat: Text.PlainText
                color: !root.enabled ? Theme.menuDisabled : root.dangerous ? Theme.menuDanger : Theme.menuText
                font.pixelSize: root.labelSize
                elide: Text.ElideRight
            }
            Text {
                Layout.fillWidth: true
                visible: !root.enabled && root.disabledReason.length > 0
                text: root.disabledReason
                textFormat: Text.PlainText
                wrapMode: Text.Wrap
                color: Theme.menuSecondary
                font.pixelSize: Theme.menuShortcutSize
            }
        }
        Rectangle {
            objectName: root.objectName + "Status"
            visible: root.statusLabel.length > 0
            implicitWidth: statusText.implicitWidth + 14
            implicitHeight: statusText.implicitHeight + 6
            radius: 4
            color: Qt.rgba(root.statusInk.r, root.statusInk.g, root.statusInk.b, 0.10)
            Text {
                id: statusText
                anchors.centerIn: parent
                text: root.statusLabel; font.pixelSize: Theme.menuShortcutSize
                color: root.statusInk
            }
        }
        Text {
            text: root.shortcutLabel
            color: Theme.menuSecondary
            font.pixelSize: Theme.menuShortcutSize
        }
        Image {
            Layout.preferredWidth: Theme.menuIconSize
            Layout.preferredHeight: Theme.menuIconSize
            visible: root.subMenu !== null
            source: "qrc:/xdzip/assets/" + (Theme.dark ? "dark/" : "light/") + "chevron-right.png"
        }
    }
    background: Rectangle {
        radius: Theme.menuRadius * 0.6
        color: root.enabled && (root.highlighted || root.hovered || root.activeFocus) ? Theme.menuHover : "transparent"
    }
}
