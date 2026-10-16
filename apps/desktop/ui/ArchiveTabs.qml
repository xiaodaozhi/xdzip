pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Layouts

Rectangle {
    id: root
    objectName: "archiveTitlebar"
    required property string archiveTitle
    property bool integratedTitlebar: false
    property bool windowsControls: false
    property bool maximized: false
    property bool canOpen: true
    property bool canClose: false
    signal settingsRequested
    signal dragRequested
    signal openRequested
    signal closeRequested
    signal minimizeRequested
    signal maximizeRequested
    signal windowCloseRequested
    color: Theme.canvas
    readonly property real capsuleHeight: Math.max(Theme.mainTabHeight, Math.ceil(Theme.mainTabSize * 1.6 + 8))
    implicitHeight: Math.max(Theme.mainTitleHeight, capsuleHeight + 2 * Theme.mainTabTopInset)
    DragHandler {
        enabled: !root.windowsControls
        target: null
        onActiveChanged: if (active) root.dragRequested()
    }
    MouseArea {
        anchors.fill: parent
        enabled: root.windowsControls
        DragHandler {
            target: null
            onActiveChanged: if (active) root.dragRequested()
        }
        onDoubleClicked: if (root.windowsControls) root.maximizeRequested()
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
        z: 1
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
        Accessible.name: root.archiveTitle.length ? root.archiveTitle : qsTrId("ui.8f1009f1369b")
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
                text: root.archiveTitle.length ? root.archiveTitle : qsTrId("ui.8f1009f1369b")
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
                Accessible.name: qsTrId("ui.6eff18378890")
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
        Accessible.name: qsTrId("ui.1b59353f1874")
        explanation: qsTrId("ui.e5ab37895edd")
        onClicked: root.openRequested()
    }
    ArchiveAction {
        objectName: "archiveSettings"
        anchors.right: parent.right
        anchors.rightMargin: Theme.mainSettingsRightInset + (root.windowsControls ? captionControls.width : 0)
        anchors.verticalCenter: parent.verticalCenter
        width: Theme.mainSettingsSize
        height: Theme.mainSettingsSize
        glyphSize: Theme.mainToolbarIconSize
        asset: "settings"
        Accessible.name: qsTrId("settings.title")
        explanation: Accessible.name
        onClicked: root.settingsRequested()
    }
    Row {
        id: captionControls
        objectName: "windowCaptionControls"
        anchors.right: parent.right
        height: root.height
        visible: root.windowsControls
        WindowCaptionButton {
            objectName: "windowMinimize"
            height: captionControls.height
            captionAction: "minimize"
            text: qsTrId("window.minimize")
            onClicked: root.minimizeRequested()
        }
        WindowCaptionButton {
            objectName: "windowMaximize"
            height: captionControls.height
            captionAction: root.maximized ? "restore" : "maximize"
            text: root.maximized ? qsTrId("window.restore") : qsTrId("window.maximize")
            onClicked: root.maximizeRequested()
        }
        WindowCaptionButton {
            objectName: "windowClose"
            height: captionControls.height
            captionAction: "close"
            text: qsTrId("window.close")
            onClicked: root.windowCloseRequested()
        }
    }
}
