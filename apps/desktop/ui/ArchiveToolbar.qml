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
    signal inspectRequested
    signal searchRequested(string query)
    function clearSearch() {
        searchField.clear();
    }
    color: Theme.surface
    // Round once for the entire row: platform font metrics can be fractional.
    readonly property real controlHeight: Math.ceil(Math.max(openAction.implicitHeight, searchField.implicitHeight))
    implicitHeight: controlHeight + 24
    RowLayout {
        id: actions
        height: root.controlHeight
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.verticalCenter: parent.verticalCenter
        anchors.margins: 16
        spacing: 8
        ArchiveAction {
            id: openAction
            Layout.minimumHeight: root.controlHeight
            Layout.maximumHeight: root.controlHeight
            objectName: "archiveOpen"
            text: qsTr("打开")
            asset: "open"
            enabled: root.canOpen
            onClicked: root.openRequested()
        }
        ArchiveAction {
            Layout.minimumHeight: root.controlHeight
            Layout.maximumHeight: root.controlHeight
            text: qsTr("新建")
            asset: "new"
            enabled: false
        }
        Rectangle {
            Layout.preferredWidth: 1
            Layout.preferredHeight: 24
            color: Theme.border
            Layout.leftMargin: 8
            Layout.rightMargin: 8
        }
        ArchiveAction {
            Layout.minimumHeight: root.controlHeight
            Layout.maximumHeight: root.controlHeight
            text: qsTr("测试")
            asset: "archive-test"
            enabled: false
        }
        ArchiveAction {
            Layout.minimumHeight: root.controlHeight
            Layout.maximumHeight: root.controlHeight
            asset: "more"
            Accessible.name: qsTr("更多归档操作")
            explanation: qsTr("更多归档操作")
            onClicked: more.open()
            Menu {
                id: more
                y: parent.height
                MenuItem {
                    text: qsTr("解压元数据预检")
                    enabled: root.canInspect
                    onTriggered: root.inspectRequested()
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
            implicitHeight: Math.max(36, contentHeight + 12)
            leftPadding: 36
            rightPadding: 12
            font.pixelSize: Theme.bodySize
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
                radius: Theme.controlRadius
                border.color: searchField.activeFocus ? Theme.accent : Theme.border
            }
            Image {
                anchors.left: parent.left
                anchors.leftMargin: 12
                anchors.verticalCenter: parent.verticalCenter
                width: 18
                height: 18
                source: "qrc:/xdzip/assets/" + (Theme.dark ? "dark/" : "light/") + "search.png"
            }
        }
        ArchiveAction {
            Layout.minimumHeight: root.controlHeight
            Layout.maximumHeight: root.controlHeight
            text: root.hasSelection ? qsTr("解压所选") : qsTr("全部解压")
            primary: true
            enabled: false
            Layout.leftMargin: 8
        }
    }
    Rectangle {
        anchors.bottom: parent.bottom
        width: parent.width
        height: 1
        color: Theme.border
    }
}
