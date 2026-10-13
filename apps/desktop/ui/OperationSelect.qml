pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls

ComboBox {
    id: root
    implicitHeight: Math.max(Theme.operationFieldHeight, contentItem.implicitHeight + 16)
    font.pixelSize: Theme.operationBodySize
    leftPadding: 12
    rightPadding: 38
    contentItem: Text {
        text: root.displayText
        textFormat: Text.PlainText
        font: root.font; color: Theme.text
        verticalAlignment: Text.AlignVCenter
        elide: Text.ElideRight
    }
    indicator: Image {
        x: root.width - width - 12
        y: (root.height - height) / 2
        width: 18; height: 18
        source: "qrc:/xdzip/assets/" + (Theme.dark ? "dark/" : "light/") + "chevron-down.png"
    }
    background: Rectangle {
        radius: 6
        color: root.down ? Theme.controlPressed : root.hovered ? Theme.controlHover : Theme.surface
        border.color: root.activeFocus ? Theme.accent : Theme.taskBorder
        border.width: root.activeFocus ? 2 : 1
    }
    delegate: CommandMenuItem {
        required property var modelData
        required property int index
        width: root.popup.availableWidth
        labelSize: Theme.operationBodySize
        text: String(modelData)
        highlighted: root.highlightedIndex === index
        onTriggered: { root.currentIndex = index; root.popup.close(); }
    }
    popup: CommandMenu {
        y: root.height + Theme.mainPopupGap
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
