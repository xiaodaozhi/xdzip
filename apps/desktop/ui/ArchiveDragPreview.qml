pragma ComponentBehavior: Bound
import QtQuick

Rectangle {
    id: root
    property string fileName: ""
    property bool directory: false
    property int itemCount: 0
    property string message: ""
    property bool permitted: false
    property real availableWidth: Theme.dragPreviewMaximumWidth
    width: Math.max(0, Math.min(Theme.dragPreviewMaximumWidth * Theme.fontScale, availableWidth,
        Math.max(Theme.dragPreviewMinimumWidth,
            message.length ? Theme.dragPreviewMaximumWidth * Theme.fontScale
            : nameMetrics.advanceWidth + Theme.dragPreviewIconSize + Theme.dragPreviewPadding * 2
                + Theme.dragPreviewGap + (itemCount > 1 ? countMetrics.advanceWidth + Theme.dragPreviewGap : 0))))
    height: body.implicitHeight + Theme.dragPreviewPadding * 2
    radius: Theme.dragPreviewRadius
    color: Theme.tooltipSurface
    // This overlay has no input handlers: the destination keeps the drag.
    Accessible.role: Accessible.StaticText
    Accessible.name: fileName + (itemCount > 1 ? ", " + itemCount : "") + (message.length ? ", " + message : "")
    TextMetrics { id: nameMetrics; text: root.fileName; font.pixelSize: Theme.menuLabelSize }
    TextMetrics { id: countMetrics; text: String(root.itemCount); font.pixelSize: Theme.menuLabelSize; font.weight: Font.DemiBold }
    Column {
        id: body
        x: Theme.dragPreviewPadding
        y: Theme.dragPreviewPadding
        width: parent.width - Theme.dragPreviewPadding * 2
        spacing: Theme.dragPreviewStatusGap
        Row {
            id: summary
            width: parent.width
            height: Math.max(Theme.dragPreviewIconSize, name.implicitHeight, count.implicitHeight)
            spacing: Theme.dragPreviewGap
            Image {
                source: AssetLibrary.fileIcon(root.fileName, root.directory, false)
                width: Theme.dragPreviewIconSize
                height: Theme.dragPreviewIconSize
                anchors.verticalCenter: parent.verticalCenter
                fillMode: Image.PreserveAspectFit
            }
            Text {
                id: name
                objectName: "dragPreviewName"
                width: Math.max(0, summary.width - Theme.dragPreviewIconSize - Theme.dragPreviewGap
                    - (count.visible ? count.width + Theme.dragPreviewGap : 0))
                anchors.verticalCenter: parent.verticalCenter
                text: root.fileName
                textFormat: Text.PlainText
                elide: Text.ElideMiddle
                color: Theme.tooltipText
                font.pixelSize: Theme.menuLabelSize
            }
            Text {
                id: count
                objectName: "dragPreviewCount"
                anchors.verticalCenter: parent.verticalCenter
                visible: root.itemCount > 1
                text: String(root.itemCount)
                color: Theme.tooltipText
                font.pixelSize: Theme.menuLabelSize
                font.weight: Font.DemiBold
            }
        }
        Text {
            objectName: "dragPreviewStatus"
            width: parent.width
            visible: text.length > 0
            text: root.message
            textFormat: Text.PlainText
            wrapMode: Text.Wrap
            maximumLineCount: 4
            elide: Text.ElideRight
            color: Theme.tooltipText
            font.pixelSize: Theme.menuShortcutSize
        }
    }
}
