pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Layouts

Rectangle {
    id: root
    objectName: "archiveTitlebar"
    required property string archiveTitle
    property bool integratedTitlebar: false
    property bool canOpen: true
    property bool canClose: false
    signal themeRequested
    signal dragRequested
    signal openRequested
    signal closeRequested
    color: Theme.canvas
    readonly property real capsuleHeight: Math.max(Theme.mainTabHeight, Math.ceil(Theme.mainTabSize * 1.6 + 8))
    implicitHeight: Math.max(Theme.mainTitleHeight, capsuleHeight + 2 * Theme.mainTabTopInset)
    DragHandler {
        target: null
        onActiveChanged: if (active)
            root.dragRequested()
    }
    RowLayout {
        id: brand
        x: root.integratedTitlebar ? Theme.mainBrandMacX : Theme.mainUnits(20)
        height: root.height
        spacing: Theme.mainUnits(8)
        Image {
            source: "qrc:/xdzip/assets/" + (Theme.dark ? "dark/" : "light/") + "app-mark.png"
            Layout.preferredWidth: Theme.mainUnits(30)
            Layout.preferredHeight: Theme.mainUnits(30)
            fillMode: Image.PreserveAspectFit
        }
        Text {
            text: "xdzip"
            font.pixelSize: Theme.mainBrandSize
            font.weight: Font.DemiBold
            color: Theme.text
        }
    }
    Rectangle {
        anchors.bottom: parent.bottom
        width: parent.width
        height: 1
        color: Theme.border
    }
    Rectangle {
        id: tab
        objectName: "activeArchiveTab"
        x: root.integratedTitlebar ? Theme.mainTabFirstMacX : Math.max(Theme.mainTabFirstOtherX, brand.x + brand.width + Theme.mainUnits(24))
        y: (root.height - height) / 2
        width: Math.max(Theme.mainTabWidth, Theme.mainTabWidth * Math.min(1.5, Theme.fontScale))
        height: root.capsuleHeight
        color: Theme.surface
        radius: height / 2
        border.color: Theme.border
        Accessible.role: Accessible.PageTab
        Accessible.name: root.archiveTitle.length ? root.archiveTitle : qsTr("起始页")
        RowLayout {
            anchors.fill: parent
            anchors.leftMargin: Theme.mainUnits(15)
            anchors.rightMargin: Theme.mainUnits(9)
            spacing: Theme.mainTabTextGap
            Image {
                source: "qrc:/xdzip/assets/" + (Theme.dark ? "dark/" : "light/") + "archive.png"
                Layout.preferredWidth: Theme.mainTabIconSize
                Layout.preferredHeight: Theme.mainTabIconSize
                fillMode: Image.PreserveAspectFit
            }
            Text {
                Layout.fillWidth: true
                text: root.archiveTitle.length ? root.archiveTitle : qsTr("起始页")
                textFormat: Text.PlainText
                elide: Text.ElideMiddle
                font.pixelSize: Theme.mainTabSize
                font.weight: Font.DemiBold
                color: Theme.text
            }
            ArchiveAction {
                objectName: "archiveTabClose"
                visible: root.archiveTitle.length > 0
                Layout.preferredWidth: Theme.mainTabCloseSize
                Layout.preferredHeight: Theme.mainTabCloseSize
                leftPadding: 0
                rightPadding: 0
                glyphSize: Theme.mainUnits(20)
                asset: "close"
                enabled: root.canClose
                Accessible.name: qsTr("关闭当前归档")
                explanation: Accessible.name
                onClicked: root.closeRequested()
            }
        }
    }
    ArchiveAction {
        objectName: "archiveNewTab"
        x: tab.x + tab.width + Theme.mainUnits(10)
        y: tab.y + (tab.height - height) / 2
        width: Theme.mainUnits(36)
        height: Theme.mainUnits(32)
        glyphSize: Theme.mainUnits(26)
        asset: "plus"
        enabled: root.canOpen
        Accessible.name: qsTr("打开归档")
        explanation: qsTr("打开归档；当前版本替换现有会话")
        onClicked: root.openRequested()
    }
    ArchiveAction {
        objectName: "archiveSettings"
        anchors.right: parent.right
        anchors.rightMargin: Theme.mainSettingsRightInset
        anchors.verticalCenter: parent.verticalCenter
        width: Theme.mainSettingsSize
        height: Theme.mainSettingsSize
        glyphSize: Theme.mainToolbarIconSize
        asset: "settings"
        Accessible.name: qsTr("外观设置")
        explanation: Accessible.name
        onClicked: appearance.open()
        CommandMenu {
            id: appearance
            x: parent.width - width
            y: parent.height
            CommandMenuItem {
                text: Theme.dark ? qsTr("切换浅色外观") : qsTr("切换深色外观")
                onTriggered: root.themeRequested()
            }
        }
    }
}
