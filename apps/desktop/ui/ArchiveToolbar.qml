pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Rectangle {
    id: root
    required property bool canOpen
    required property bool canSearch
    required property bool canInspect
    required property bool hasSelection
    signal openRequested
    signal resetSortRequested
    signal inspectRequested
    signal searchRequested(string query)
    function clearSearch() {
        searchField.clear();
    }
    color: Theme.surface
    // An even row height keeps the 24 px icons centered on whole pixels on every host.
    readonly property real controlHeight: 2 * Math.ceil(Math.max(openAction.implicitHeight, searchField.implicitHeight) / 2)
    implicitHeight: controlHeight + 24
    RowLayout {
        id: actions
        height: root.controlHeight
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.verticalCenter: parent.verticalCenter
        anchors.margins: 16
        spacing: 12
        RowLayout {
            Layout.preferredHeight: root.controlHeight
            spacing: 20
            ArchiveAction {
                id: openAction
                toolbarStyle: true
                Layout.minimumHeight: root.controlHeight
                Layout.maximumHeight: root.controlHeight
                objectName: "archiveOpen"
                text: qsTr("打开")
                asset: "open"
                enabled: root.canOpen
                onClicked: root.openRequested()
            }
            ArchiveAction {
                toolbarStyle: true
                Layout.minimumHeight: root.controlHeight
                Layout.maximumHeight: root.controlHeight
                objectName: "archiveNew"
                text: qsTr("新建")
                asset: "new"
                enabled: false
            }
            Rectangle {
                Layout.preferredWidth: 1
                Layout.preferredHeight: 24
                color: Theme.border
                Layout.leftMargin: 4
                Layout.rightMargin: 0
            }
            ArchiveAction {
                toolbarStyle: true
                Layout.minimumHeight: root.controlHeight
                Layout.maximumHeight: root.controlHeight
                objectName: "archiveTest"
                leftPadding: 0
                text: qsTr("测试")
                asset: "archive-test"
                enabled: false
            }
            ArchiveAction {
                toolbarStyle: true
                Layout.minimumHeight: root.controlHeight
                Layout.maximumHeight: root.controlHeight
                objectName: "archiveMore"
                asset: "more"
                Accessible.name: qsTr("更多归档操作")
                explanation: qsTr("更多归档操作")
                onClicked: more.open()
                Menu {
                    id: more
                    y: parent.height
                    MenuItem {
                        text: qsTr("按归档顺序排列")
                        enabled: root.canSearch
                        onTriggered: root.resetSortRequested()
                    }
                    MenuItem {
                        text: qsTr("解压元数据预检")
                        enabled: root.canInspect
                        onTriggered: root.inspectRequested()
                    }
                }
            }
        }
        Item {
            Layout.fillWidth: true
        }
        TextField {
            id: searchField
            Layout.minimumHeight: root.controlHeight
            Layout.maximumHeight: root.controlHeight
            objectName: "archiveSearch"
            Layout.preferredWidth: Math.max(220, Math.min(360, root.width * 0.3))
            Layout.minimumWidth: 180
            Layout.fillWidth: true
            Layout.maximumWidth: 360
            implicitHeight: Math.max(Theme.toolbarButtonHeight, contentHeight + 16)
            leftPadding: 42
            rightPadding: 12
            font.pixelSize: Theme.toolbarSize
            placeholderText: qsTr("搜索压缩包中的文件")
            maximumLength: 256
            enabled: root.canSearch
            onAccepted: root.searchRequested(text)
            Keys.onEscapePressed: {
                clear();
                root.searchRequested("");
            }
            color: Theme.text
            placeholderTextColor: Theme.textSecondary
            Accessible.name: placeholderText
            background: Rectangle {
                color: Theme.canvas
                radius: Theme.toolbarRadius
                border.color: searchField.activeFocus ? Theme.accent : Theme.border
            }
            Image {
                anchors.left: parent.left
                anchors.leftMargin: 12
                anchors.verticalCenter: parent.verticalCenter
                width: Theme.toolbarIconSize
                height: Theme.toolbarIconSize
                source: "qrc:/xdzip/assets/" + (Theme.dark ? "dark/" : "light/") + "search.png"
            }
        }
        ArchiveAction {
            toolbarStyle: true
            Layout.minimumHeight: root.controlHeight
            Layout.maximumHeight: root.controlHeight
            text: root.hasSelection ? qsTr("解压所选") : qsTr("全部解压")
            objectName: "archiveExtract"
            primary: true
            splitIndicator: true
            explanation: qsTr("解压功能尚未实现")
            enabled: false
        }
    }
    Rectangle {
        anchors.bottom: parent.bottom
        width: parent.width
        height: 1
        color: Theme.border
    }
}
