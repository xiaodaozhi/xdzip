pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls

ComboBox {
    id: root
    property real density: 1
    implicitHeight: Math.max(Theme.operationFieldHeight * density, contentItem.implicitHeight + 16 * density)
    font.pixelSize: Math.round(Theme.operationBodySize * density)
    leftPadding: 12
    rightPadding: 38
    contentItem: Text {
        text: root.displayText
        textFormat: Text.PlainText
        font: root.font; color: root.enabled ? Theme.text : Theme.textSecondary
        verticalAlignment: Text.AlignVCenter
        elide: Text.ElideRight
    }
    indicator: Image {
        x: root.width - width - 12
        y: (root.height - height) / 2
        width: 18; height: 18
        opacity: root.enabled ? 1 : 0.45
        source: "qrc:/xdzip/assets/" + (Theme.dark ? "dark/" : "light/") + "chevron-down.png"
    }
    background: Rectangle {
        radius: 6
        color: !root.enabled ? Theme.operationCard : root.down ? Theme.controlPressed : root.hovered ? Theme.controlHover : Theme.surface
        border.color: root.activeFocus ? Theme.accent : Theme.operationFieldBorder
        border.width: root.activeFocus ? 2 : 1
    }
    delegate: CommandMenuItem {
        required property var modelData
        required property int index
        width: root.popup.availableWidth
        labelSize: Math.round(Theme.operationBodySize * root.density)
        text: String(modelData)
        checkable: true
        checked: root.currentIndex === index
        highlighted: root.highlightedIndex === index
        onTriggered: { root.currentIndex = index; root.popup.close(); }
    }
    popup: CommandMenu {
        // Keep the normal gap on either side of the field, including short windows.
        y: {
            const expectedHeight = Math.min(Math.max(root.count * Theme.menuRowHeight, contentItem.implicitHeight) + padding * 2, root.Window.height - 40);
            const fieldTop = root.mapToItem(null, 0, 0).y;
            return fieldTop + root.height + Theme.mainPopupGap + expectedHeight > root.Window.height - Theme.menuViewportInset
                ? -expectedHeight - Theme.mainPopupGap : root.height + Theme.mainPopupGap;
        }
        width: root.width
        implicitHeight: Math.min(contentItem.implicitHeight + topPadding + bottomPadding, root.Window.height - 40)
        contentItem: ListView {
            clip: true
            implicitHeight: contentHeight
            model: root.popup.visible ? root.delegateModel : null
            currentIndex: root.highlightedIndex
            ScrollIndicator.vertical: ScrollIndicator {}
        }
    }
}
