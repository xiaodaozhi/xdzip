pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQml.Models

RowLayout {
    id: root
    property var entries: []
    property bool available: true
    signal activated(string nodeId)
    spacing: 8
    onEntriesChanged: hiddenMenu.close()
    function visibleEntries() {
        return entries.length <= 3 ? entries : [entries[0],
            {
                label: "…",
                more: true
            },
            ...entries.slice(-2)];
    }
    Repeater {
        model: root.visibleEntries()
        RowLayout {
            id: segment
            required property var modelData
            required property int index
            Layout.fillWidth: modelData.current
            Layout.preferredWidth: Math.min(240 * Theme.fontScale, Math.ceil(button.implicitWidth) + (index > 0 ? 18 : 0))
            Layout.maximumWidth: modelData.current ? 500 : modelData.more ? 32 : 240 * Theme.fontScale
            spacing: 8
            Text {
                visible: segment.index > 0
                text: "/"
                font.pixelSize: Theme.mainPaneTitleSize
                color: Theme.textSecondary
            }
            ToolButton {
                id: button
                Layout.fillWidth: true
                // Layout rounding must not elide an otherwise fitting archive name on Cocoa.
                implicitWidth: Math.ceil(contentItem.implicitWidth) + leftPadding + rightPadding
                Layout.preferredHeight: Math.max(Theme.mainBreadcrumbHeight, Theme.mainPaneTitleSize * 1.6 + 6)
                Layout.alignment: Qt.AlignVCenter
                padding: 2
                enabled: root.available && !segment.modelData.current
                Accessible.name: segment.modelData.more ? qsTr("显示上级目录") : segment.modelData.label
                onClicked: segment.modelData.more ? hiddenMenu.popup(button, 0, button.height + Theme.mainPopupGap) : root.activated(segment.modelData.id)
                contentItem: Text {
                    objectName: segment.modelData.current ? "filePaneTitle" : ""
                    text: segment.modelData.label
                    textFormat: Text.PlainText
                    color: segment.modelData.current ? Theme.text : Theme.textSecondary
                    font.pixelSize: Theme.mainPaneTitleSize
                    font.bold: segment.modelData.current
                    elide: Text.ElideMiddle
                    verticalAlignment: Text.AlignVCenter
                }
                background: Rectangle {
                    radius: 4
                    color: button.down ? Theme.controlPressed : button.hovered && button.enabled ? Theme.controlHover : "transparent"
                    border.width: 0
                    border.color: Theme.accent
                }
                CommandTooltip {
                    requested: button.hovered && !button.down
                    text: button.Accessible.name
                }
            }
        }
    }
    CommandMenu {
        id: hiddenMenu
        title: qsTr("上级目录")
        Instantiator {
            model: root.entries.length > 3 ? root.entries.slice(1, -2) : []
            delegate: CommandMenuItem {
                required property var modelData
                text: modelData.label
                asset: "folder"
                enabled: root.available
                onTriggered: root.activated(modelData.id)
            }
            onObjectAdded: function (index, object) {
                hiddenMenu.insertItem(index, object);
            }
            onObjectRemoved: function (index, object) {
                hiddenMenu.removeItem(object);
            }
        }
    }
}
