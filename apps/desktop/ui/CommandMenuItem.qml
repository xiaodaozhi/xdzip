// apps/desktop/ui/CommandMenuItem.qml

pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls

MenuItem {
    id: root
    property string asset: ""
    property string shortcutLabel: ""
    property string statusLabel: ""
    property color statusInk: Theme.accentInk
    // Disabled reasons remain available to assistive technology, without a visible hint or tooltip.
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
    contentItem: Item {
        id: content
        readonly property real gap: Theme.menuLabelInset
        // These describe content even while the popup is closed and its children are hidden.
        readonly property bool hasStatus: root.statusLabel.length > 0
        readonly property bool hasShortcut: root.shortcutLabel.length > 0
        readonly property bool hasSubmenu: root.subMenu !== null
        readonly property bool hasTrailing: hasStatus || hasShortcut || hasSubmenu
        implicitWidth: iconSlot.width + gap + label.implicitWidth
            + (hasStatus ? gap + statusBadge.width : 0)
            + (hasShortcut ? gap + shortcut.width : 0)
            + (hasSubmenu ? gap + submenuArrow.width : 0)
        implicitHeight: Math.max(iconSlot.height, label.implicitHeight,
            hasStatus ? statusBadge.height : 0,
            hasShortcut ? shortcut.height : 0,
            hasSubmenu ? submenuArrow.height : 0)
        Item {
            id: iconSlot
            width: Theme.menuIconSize
            height: Theme.menuIconSize
            anchors.left: parent.left
            anchors.verticalCenter: parent.verticalCenter
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
        Text {
            id: label
            objectName: root.objectName + "Label"
            anchors.left: iconSlot.right
            anchors.leftMargin: content.gap
            anchors.right: content.hasStatus ? statusBadge.left : content.hasShortcut ? shortcut.left : content.hasSubmenu ? submenuArrow.left : parent.right
            anchors.rightMargin: content.hasTrailing ? content.gap : 0
            anchors.verticalCenter: parent.verticalCenter
            height: implicitHeight
            text: root.text
            textFormat: Text.PlainText
            color: !root.enabled ? Theme.menuDisabled : root.dangerous ? Theme.menuDanger : Theme.menuText
            font.pixelSize: root.labelSize
            elide: Text.ElideRight
        }
        Rectangle {
            id: statusBadge
            objectName: root.objectName + "Status"
            visible: content.hasStatus
            implicitWidth: statusText.implicitWidth + 14
            implicitHeight: statusText.implicitHeight + 6
            width: implicitWidth
            height: implicitHeight
            anchors.right: content.hasShortcut ? shortcut.left : content.hasSubmenu ? submenuArrow.left : parent.right
            anchors.rightMargin: content.hasShortcut || content.hasSubmenu ? content.gap : 0
            anchors.verticalCenter: parent.verticalCenter
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
            id: shortcut
            visible: content.hasShortcut
            width: implicitWidth
            height: implicitHeight
            anchors.right: content.hasSubmenu ? submenuArrow.left : parent.right
            anchors.rightMargin: content.hasSubmenu ? content.gap : 0
            anchors.verticalCenter: parent.verticalCenter
            text: root.shortcutLabel
            color: Theme.menuSecondary
            font.pixelSize: Theme.menuShortcutSize
        }
        Image {
            id: submenuArrow
            width: Theme.menuIconSize
            height: Theme.menuIconSize
            anchors.right: parent.right
            anchors.verticalCenter: parent.verticalCenter
            visible: content.hasSubmenu
            source: "qrc:/xdzip/assets/" + (Theme.dark ? "dark/" : "light/") + "chevron-right.png"
        }
    }
    background: Rectangle {
        radius: Theme.menuRadius * 0.6
        color: root.enabled && (root.highlighted || root.hovered || root.activeFocus) ? Theme.menuHover : "transparent"
    }
}
