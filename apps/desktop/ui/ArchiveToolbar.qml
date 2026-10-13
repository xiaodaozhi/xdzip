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
    // Keep source-measured logical geometry at 100%; grow for accessible text sizes.
    readonly property real controlHeight: Math.max(Theme.mainToolbarControlHeight, openAction.implicitHeight, searchField.implicitHeight)
    implicitHeight: Math.round(controlHeight + Theme.mainToolbarHeight - Theme.mainToolbarControlHeight)
    RowLayout {
        id: actions
        height: root.controlHeight
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.verticalCenter: parent.verticalCenter
        anchors.leftMargin: Theme.mainToolbarLeftInset
        // Enlarged text can round intrinsic widths upward; preserve the outer inset.
        anchors.rightMargin: Theme.fontScale > 1 ? Math.ceil(Theme.mainToolbarRightInset) : Theme.mainToolbarRightInset
        spacing: Math.round(Theme.mainToolbarGroupGap)
        RowLayout {
            Layout.preferredHeight: root.controlHeight
            spacing: 0
            ArchiveAction {
                id: openAction
                toolbarStyle: true
                Layout.minimumHeight: root.controlHeight
                Layout.maximumHeight: root.controlHeight
                objectName: "archiveOpen"
                Layout.preferredWidth: Math.max(Theme.mainToolbarActionWidth, implicitWidth)
                Layout.rightMargin: Math.floor(Theme.mainToolbarOpenGap)
                leftPadding: Theme.mainToolbarActionInset
                rightPadding: 0
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
                Layout.preferredWidth: Math.max(Theme.mainToolbarActionWidth, implicitWidth)
                Layout.rightMargin: Math.floor(Theme.mainToolbarNewGap)
                leftPadding: Theme.mainToolbarActionInset
                rightPadding: 0
                text: qsTr("新建")
                asset: "new"
                enabled: false
                explanation: qsTr("创建归档尚未实现")
            }
            Rectangle {
                Layout.preferredWidth: 1
                Layout.preferredHeight: Theme.mainToolbarIconSize
                color: Theme.border
                Layout.leftMargin: Theme.mainToolbarActionInset
                Layout.rightMargin: Theme.mainToolbarSeparatorGap
            }
            ArchiveAction {
                toolbarStyle: true
                Layout.minimumHeight: root.controlHeight
                Layout.maximumHeight: root.controlHeight
                objectName: "archiveTest"
                Layout.preferredWidth: Math.max(Theme.mainToolbarActionWidth, implicitWidth)
                Layout.rightMargin: Theme.mainToolbarTestGap
                leftPadding: Theme.mainToolbarActionInset
                rightPadding: 0
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
                Layout.preferredWidth: Theme.mainToolbarMoreWidth
                leftPadding: Theme.mainToolbarActionInset
                rightPadding: Theme.mainToolbarActionInset
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
                        text: qsTr("全选当前目录")
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
            Layout.preferredWidth: Theme.mainSearchWidth
            Layout.minimumWidth: Theme.mainUnits(180)
            Layout.fillWidth: true
            Layout.maximumWidth: Theme.mainSearchWidth
            implicitHeight: Math.max(Theme.mainToolbarControlHeight, contentHeight + Theme.mainUnits(12))
            leftPadding: Theme.mainSearchTextInset
            rightPadding: text.length ? Theme.mainUnits(42) : Theme.mainUnits(12)
            font.pixelSize: Theme.mainToolbarSize
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
                radius: Theme.mainToolbarRadius
                border.color: searchField.activeFocus ? Theme.accent : Theme.border
            }
            Image {
                anchors.left: parent.left
                anchors.leftMargin: Theme.mainSearchIconInset
                anchors.verticalCenter: parent.verticalCenter
                width: Theme.mainSearchIconSize
                height: Theme.mainSearchIconSize
                source: "qrc:/xdzip/assets/" + (Theme.dark ? "dark/" : "light/") + "search.png"
            }
            ToolButton {
                id: searchClear
                objectName: "archiveSearchClear"
                visible: searchField.text.length > 0
                anchors.right: parent.right
                anchors.rightMargin: Theme.mainUnits(8)
                anchors.verticalCenter: parent.verticalCenter
                width: Theme.mainUnits(28)
                height: Theme.mainUnits(28)
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
                    radius: Theme.mainUnits(4)
                    color: searchClear.down ? Theme.controlPressed : searchClear.hovered ? Theme.controlHover : "transparent"
                }
            }
        }
        Item {
            Layout.preferredWidth: Math.max(Theme.mainExtractWidth, extractAction.implicitWidth)
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
                width: Theme.mainSplitWidth
                height: parent.height
                enabled: root.canSearch
                Accessible.name: qsTr("解压范围与可用操作")
                contentItem: Item {}
                background: Rectangle {
                    radius: Theme.mainToolbarRadius
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
