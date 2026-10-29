// apps/desktop/ui/CommandMenuItem.qml

pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls

MenuItem {
    id: root
    property string asset: ""
    property string shortcutLabel: ""
    property string statusLabel: ""
    property string sequenceLabel: ""
    property string accessibleLabel: ""
    property bool userData: false
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
    Accessible.name: (accessibleLabel.length ? accessibleLabel : text + (sequenceLabel.length ? qsTrId("common.separator") + sequenceLabel : "")) + (statusLabel.length ? qsTrId("common.separator") + statusLabel : "")
    Accessible.description: enabled ? "" : disabledReason
    indicator: null
    arrow: null
    contentItem: Item {
        id: content
        readonly property real gap: Theme.menuLabelInset
        // These describe content even while the popup is closed and its children are hidden.
        readonly property bool hasStatus: root.statusLabel.length > 0
        readonly property bool hasSequence: root.sequenceLabel.length > 0
        readonly property bool hasShortcut: root.shortcutLabel.length > 0
        readonly property bool hasSubmenu: root.subMenu !== null
        readonly property bool hasTrailing: hasSequence || hasStatus || hasShortcut || hasSubmenu
        implicitWidth: iconSlot.width + gap + label.implicitWidth
            + (hasSequence ? gap + sequenceBadge.width : 0)
            + (hasStatus ? gap + statusBadge.width : 0)
            + (hasShortcut ? gap + shortcut.width : 0)
            + (hasSubmenu ? gap + submenuArrow.width : 0)
        implicitHeight: Math.max(iconSlot.height, label.implicitHeight,
            hasSequence ? sequenceBadge.height : 0,
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
            anchors.right: content.hasSequence ? sequenceBadge.left : content.hasStatus ? statusBadge.left : content.hasShortcut ? shortcut.left : content.hasSubmenu ? submenuArrow.left : parent.right
            anchors.rightMargin: content.hasTrailing ? content.gap : 0
            anchors.verticalCenter: parent.verticalCenter
            height: implicitHeight
            text: root.text
            textFormat: Text.PlainText
            color: !root.enabled ? Theme.menuDisabled : root.dangerous ? Theme.menuDanger : Theme.menuText
            font.pixelSize: root.labelSize
            font.family: root.userData ? Theme.userDataFontFamily : Theme.uiFontFamily
            elide: Text.ElideRight
        }
        CommandMenuTag {
            id: sequenceBadge
            objectName: root.objectName + "Sequence"
            visible: content.hasSequence
            width: implicitWidth
            height: implicitHeight
            anchors.right: content.hasStatus ? statusBadge.left : content.hasShortcut ? shortcut.left : content.hasSubmenu ? submenuArrow.left : parent.right
            anchors.rightMargin: content.hasStatus || content.hasShortcut || content.hasSubmenu ? content.gap : 0
            anchors.verticalCenter: parent.verticalCenter
            text: root.sequenceLabel
            ink: Theme.menuSecondary
        }
        CommandMenuTag {
            id: statusBadge
            objectName: root.objectName + "Status"
            visible: content.hasStatus
            width: implicitWidth
            height: implicitHeight
            anchors.right: content.hasShortcut ? shortcut.left : content.hasSubmenu ? submenuArrow.left : parent.right
            anchors.rightMargin: content.hasShortcut || content.hasSubmenu ? content.gap : 0
            anchors.verticalCenter: parent.verticalCenter
            text: root.statusLabel
            ink: root.statusInk
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
