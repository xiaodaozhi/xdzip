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
    signal selectAllRequested
    signal clearSelectionRequested
    signal detailsRequested
    signal themeRequested
    function focusSearch() {
        searchField.forceActiveFocus(Qt.ShortcutFocusReason);
        searchField.selectAll();
    }
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
                explanation: qsTr("创建归档尚未实现")
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
                explanation: qsTr("CRC 校验尚未实现")
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
                CommandMenu {
                    id: more
                    y: parent.height
                    objectName: "archiveMoreMenu"
                    CommandMenuItem {
                        text: qsTr("按归档顺序排列")
                        enabled: root.canSearch
                        onTriggered: root.resetSortRequested()
                    }
                    CommandMenuItem {
                        text: qsTr("解压元数据预检")
                        enabled: root.canInspect
                        onTriggered: root.inspectRequested()
                    }
                    CommandMenuSeparator {}
                    CommandMenuItem {
                        text: qsTr("全选当前页")
                        shortcutLabel: Qt.platform.os === "osx" ? "⌘A" : "Ctrl+A"
                        enabled: root.canSearch
                        onTriggered: root.selectAllRequested()
                    }
                    CommandMenuItem {
                        text: qsTr("取消选择")
                        enabled: root.hasSelection && root.canSearch
                        onTriggered: root.clearSelectionRequested()
                    }
                    CommandMenuSeparator {}
                    CommandMenuItem {
                        text: qsTr("文件详情")
                        enabled: root.canSearch
                        onTriggered: root.detailsRequested()
                    }
                    CommandMenuItem {
                        text: Theme.dark ? qsTr("浅色外观") : qsTr("深色外观")
                        onTriggered: root.themeRequested()
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
            rightPadding: text.length ? 42 : 12
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
            ToolButton {
                id: searchClear
                objectName: "archiveSearchClear"
                visible: searchField.text.length > 0
                anchors.right: parent.right
                anchors.rightMargin: 8
                anchors.verticalCenter: parent.verticalCenter
                width: 28
                height: 28
                Accessible.name: qsTr("清空搜索")
                onClicked: {
                    searchField.clear();
                    root.searchRequested("");
                    searchField.forceActiveFocus();
                }
                contentItem: Image {
                    source: "qrc:/xdzip/assets/" + (Theme.dark ? "dark/" : "light/") + "close.png"
                    fillMode: Image.PreserveAspectFit
                }
                padding: 5
                background: Rectangle {
                    radius: 4
                    color: searchClear.down ? Theme.controlPressed : searchClear.hovered ? Theme.controlHover : "transparent"
                }
            }
        }
        Item {
            Layout.preferredWidth: extractAction.implicitWidth
            Layout.minimumHeight: root.controlHeight
            Layout.maximumHeight: root.controlHeight
            implicitHeight: root.controlHeight
            ArchiveAction {
                id: extractAction
                anchors.fill: parent
                toolbarStyle: true
                text: root.hasSelection ? qsTr("解压所选") : qsTr("全部解压")
                objectName: "archiveExtract"
                primary: true
                splitIndicator: true
                explanation: qsTr("解压功能尚未实现")
                enabled: false
            }
            ToolButton {
                id: scopeButton
                objectName: "archiveExtractMenuButton"
                anchors.right: parent.right
                width: 40
                height: parent.height
                enabled: root.canSearch
                Accessible.name: qsTr("解压范围与可用操作")
                contentItem: Item {}
                background: Rectangle {
                    radius: Theme.toolbarRadius
                    color: scopeButton.down ? "#30FFFFFF" : scopeButton.hovered ? "#18FFFFFF" : "transparent"
                    border.width: scopeButton.activeFocus ? 2 : 0
                    border.color: Theme.accentInk
                }
                onClicked: scopes.open()
                CommandMenu {
                    id: scopes
                    objectName: "archiveExtractMenu"
                    x: parent.width - width
                    y: parent.height
                    CommandMenuItem {
                        text: qsTr("全部解压")
                        enabled: false
                        disabledReason: qsTr("安全写入层尚未实现")
                    }
                    CommandMenuItem {
                        text: qsTr("解压所选")
                        enabled: false
                        disabledReason: qsTr("安全写入层尚未实现")
                    }
                    CommandMenuSeparator {}
                    CommandMenuItem {
                        text: qsTr("解压元数据预检")
                        enabled: root.canInspect
                        disabledReason: qsTr("先选择条目")
                        onTriggered: root.inspectRequested()
                    }
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
