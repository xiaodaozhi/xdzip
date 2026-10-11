pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Rectangle {
    id: root
    required property string archiveTitle
    property bool integratedTitlebar: false
    signal themeRequested
    signal dragRequested
    color: Theme.canvas
    implicitHeight: Math.max(48, Theme.bodySize + 24)
    // Only the background starts native window movement; controls retain their own input.
    DragHandler {
        target: null
        onActiveChanged: if (active)
            root.dragRequested()
    }
    RowLayout {
        anchors.fill: parent
        anchors.leftMargin: root.integratedTitlebar ? 96 : 24
        anchors.rightMargin: 16
        spacing: 16
        Image {
            source: "qrc:/xdzip/assets/archive.png"
            Layout.preferredWidth: 24
            Layout.preferredHeight: 24
        }
        Text {
            text: "xdzip"
            font.pixelSize: Theme.panelTitleSize
            font.bold: true
            color: Theme.text
            Layout.rightMargin: 24
        }
        Rectangle {
            objectName: "activeArchiveTab"
            Layout.preferredWidth: 240
            Layout.fillHeight: true
            Layout.topMargin: 8
            color: Theme.surface
            radius: 8
            border.color: Theme.border
            Accessible.role: Accessible.PageTab
            Accessible.name: root.archiveTitle.length ? root.archiveTitle : qsTr("起始页")
            Rectangle {
                anchors.bottom: parent.bottom
                width: parent.width
                height: 8
                color: Theme.surface
            }
            RowLayout {
                anchors.fill: parent
                anchors.margins: 12
                spacing: 8
                Image {
                    source: "qrc:/xdzip/assets/archive.png"
                    Layout.preferredWidth: 18
                    Layout.preferredHeight: 18
                }
                Text {
                    Layout.fillWidth: true
                    text: root.archiveTitle.length ? root.archiveTitle : qsTr("起始页")
                    textFormat: Text.PlainText
                    elide: Text.ElideMiddle
                    font.pixelSize: Theme.bodySize
                    font.bold: true
                    color: Theme.text
                }
            }
        }
        // Multiple sessions and closing a session require runtime support; no decorative fake tabs.
        Item {
            Layout.fillWidth: true
        }
        ArchiveAction {
            asset: "settings"
            Accessible.name: qsTr("外观设置")
            explanation: qsTr("外观设置")
            onClicked: appearance.open()
            Menu {
                id: appearance
                y: parent.height
                MenuItem {
                    text: Theme.dark ? qsTr("切换浅色外观") : qsTr("切换深色外观")
                    onTriggered: root.themeRequested()
                }
            }
        }
    }
    Rectangle {
        anchors.bottom: parent.bottom
        width: parent.width
        height: 1
        color: Theme.border
    }
}
