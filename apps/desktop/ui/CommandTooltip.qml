pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls

ToolTip {
    id: root
    property bool requested: false
    readonly property Item focusItem: parent && parent.Window.window ? parent.Window.window.activeFocusItem : null
    visible: requested && text.length > 0 && !(focusItem as MenuItem) && (!focusItem || focusItem.objectName !== "xdzipCommandMenuContent")
    popupType: Popup.Item
    delay: Theme.hoverDelayMs
    timeout: Theme.tooltipDurationMs
    margins: Theme.menuViewportInset
    padding: Theme.tooltipPadding
    implicitWidth: Math.min(Theme.tooltipMaxWidth, contentItem.implicitWidth + padding * 2)
    implicitHeight: Math.min(Theme.treeTooltipMaxHeight * Theme.fontScale, contentItem.implicitHeight + padding * 2)
    contentItem: Text {
        text: root.text
        // Archive display names must remain text, including angle brackets.
        textFormat: Text.PlainText
        font.pixelSize: Theme.menuShortcutSize
        color: Theme.tooltipText
        wrapMode: Text.WrapAnywhere
        clip: true
    }
    background: Rectangle {
        color: Theme.tooltipSurface
        radius: Theme.menuRadius * 0.65
    }
}
