pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Xdzip.Settings 1.0

Rectangle {
    id: root
    required property bool canOpen
    property bool canSelect: canSearch
    required property bool canSearch
    required property bool canInspect
    required property bool hasSelection
    signal extractRequested(bool all)
    signal newRequested
    signal openRequested
    property bool canTest: false
    signal testRequested
    signal resetSortRequested
    signal inspectRequested
    signal searchRequested(string query)
    signal selectAllRequested
    signal clearSelectionRequested
    signal detailsRequested
    signal settingsRequested
    signal archiveCommentRequested
    property bool archiveCommentSupported: false
    property bool canOpenArchiveComment: false
    property bool preserveActionAppearance: false
    function focusMore() { moreButton.forceActiveFocus(Qt.OtherFocusReason); }
    signal recentArchiveRequested(string path)
    signal revealRequested
    signal quickExtractRequested(string destination)
    property string archiveFolder: ""
    property bool canQuickExtract: false
    property bool canReveal: false
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
            FocusScope {
                id: openGroup
                objectName: "toolbarOpenSplit"
                readonly property real disclosureWidth: Math.round(Theme.welcomeRecentDisclosureWidth * Theme.fontScale) * Theme.mainToolbarRecentWidthFactor
                readonly property bool splitVisible: enabled && (openAction.hovered || recentButton.hovered
                    || recentButton.visualFocus || history.visible)
                enabled: root.canOpen
                implicitWidth: Math.max(Theme.mainToolbarActionWidth, openAction.implicitWidth + disclosureWidth)
                Layout.preferredWidth: implicitWidth
                Layout.minimumHeight: root.controlHeight
                Layout.maximumHeight: root.controlHeight
                Layout.rightMargin: Math.floor(Theme.mainToolbarOpenGap)
                onVisibleChanged: if (!visible) history.close()
                onEnabledChanged: if (!enabled) history.close()

                function showHistory() {
                    recentButton.forceActiveFocus(Qt.OtherFocusReason);
                    history.paths = AppSettings.recentArchives();
                    history.open();
                }
                ArchiveAction {
                    id: openAction
                    objectName: "archiveOpen"
                    anchors.left: parent.left
                    anchors.top: parent.top
                    anchors.bottom: parent.bottom
                    anchors.right: recentButton.left
                    toolbarStyle: true
                    tooltipEnabled: false
                    appearanceEnabled: root.preserveActionAppearance || enabled
                    leftPadding: Theme.mainToolbarActionInset
                    rightPadding: Math.round(Theme.mainToolbarContentGap * Theme.fontScale) * Theme.mainToolbarRecentLabelGapFactor
                    text: qsTrId("ui.c771248e511f")
                    asset: "open"
                    onClicked: root.openRequested()
                    Keys.onDownPressed: event => {
                        if (event.modifiers & Qt.AltModifier) {
                            openGroup.showHistory();
                            event.accepted = true;
                        }
                    }
                    background: Rectangle {
                        objectName: "toolbarOpenBackground"
                        width: openGroup.splitVisible ? openAction.width : openGroup.width
                        height: openAction.height
                        radius: Theme.mainToolbarRadius
                        topRightRadius: openGroup.splitVisible ? 0 : radius
                        bottomRightRadius: topRightRadius
                        color: !openAction.enabled ? "transparent" : openAction.down ? Theme.controlPressed
                            : openAction.hovered ? Theme.controlHover : "transparent"
                        border.color: Theme.accentInk
                        border.width: openAction.visualFocus ? 1 : 0
                    }
                }
                Button {
                    id: recentButton
                    objectName: "toolbarRecentToggle"
                    anchors.right: parent.right
                    anchors.top: parent.top
                    anchors.bottom: parent.bottom
                    width: openGroup.disclosureWidth
                    hoverEnabled: true
                    focusPolicy: Qt.StrongFocus
                    Accessible.name: qsTrId("desktop.recent")
                    onClicked: openGroup.showHistory()
                    Keys.onDownPressed: openGroup.showHistory()
                    Keys.onReturnPressed: openGroup.showHistory()
                    padding: 0
                    contentItem: Item {
                        Image {
                            anchors.centerIn: parent
                            width: Theme.welcomeRecentDisclosureIconSize
                            height: width
                            visible: openGroup.splitVisible
                            source: "qrc:/xdzip/assets/" + (Theme.dark ? "dark/" : "light/") + "chevron-down.png"
                            fillMode: Image.PreserveAspectFit
                        }
                    }
                    background: Rectangle {
                        objectName: "toolbarRecentBackground"
                        visible: openGroup.splitVisible
                        radius: Theme.mainToolbarRadius
                        topLeftRadius: 0
                        bottomLeftRadius: 0
                        color: recentButton.down || history.visible ? Theme.controlPressed
                            : recentButton.hovered ? Theme.controlHover : "transparent"
                        border.color: Theme.accentInk
                        border.width: recentButton.visualFocus ? 1 : 0
                        Rectangle {
                            width: 1
                            height: parent.height
                            color: Theme.border
                            opacity: 0.8
                        }
                    }
                }
                WelcomeRecentPopup {
                    id: history
                    parent: openGroup
                    focusReturn: recentButton
                    x: 0
                    onRecentSelected: path => root.recentArchiveRequested(path)
                    onClearRequested: AppSettings.clearRecentArchives()
                }
            }
            ArchiveAction {
                toolbarStyle: true
                tooltipEnabled: false
                Layout.minimumHeight: root.controlHeight
                Layout.maximumHeight: root.controlHeight
                objectName: "archiveNew"
                Layout.preferredWidth: Math.max(Theme.mainToolbarActionWidth, implicitWidth)
                Layout.rightMargin: Math.floor(Theme.mainToolbarNewGap)
                leftPadding: Theme.mainToolbarActionInset
                rightPadding: 0
                text: qsTrId("ui.50ef2f4cf6a4")
                asset: "new"
                enabled: root.canOpen
                appearanceEnabled: root.preserveActionAppearance || enabled
                onClicked: root.newRequested()
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
                tooltipEnabled: false
                Layout.minimumHeight: root.controlHeight
                Layout.maximumHeight: root.controlHeight
                objectName: "archiveTest"
                Layout.preferredWidth: Math.max(Theme.mainToolbarActionWidth, implicitWidth)
                Layout.rightMargin: Theme.mainToolbarTestGap
                leftPadding: Theme.mainToolbarActionInset
                rightPadding: 0
                text: qsTrId("ui.6aa8f49cc992")
                asset: "archive-test"
                enabled: root.canTest
                appearanceEnabled: root.preserveActionAppearance || enabled
                explanation: qsTrId("desktop.testArchive")
                onClicked: root.testRequested()
            }
            ArchiveAction {
                toolbarStyle: true
                tooltipEnabled: false
                Layout.minimumHeight: root.controlHeight
                Layout.maximumHeight: root.controlHeight
                id: moreButton
                objectName: "archiveMore"
                Layout.preferredWidth: Theme.mainToolbarMoreWidth
                leftPadding: Theme.mainToolbarActionInset
                rightPadding: Theme.mainToolbarActionInset
                asset: "more"
                Accessible.name: qsTrId("ui.124e56e8a623")
                explanation: qsTrId("ui.124e56e8a623")
                onClicked: more.open()
                CommandMenu {
                    id: more
                    y: parent.height + Theme.mainPopupGap
                    objectName: "archiveMoreMenu"
                    CommandMenuItem {
                        objectName: "toolbarRevealArchive"
                        text: qsTrId("desktop.revealArchive")
                        enabled: root.canReveal
                        onTriggered: root.revealRequested()
                    }
                    CommandMenuItem {
                        objectName: "toolbarArchiveComment"
                        text: qsTrId("archive.comment.view")
                        commandVisible: root.archiveCommentSupported
                        enabled: root.canOpenArchiveComment
                        onTriggered: root.archiveCommentRequested()
                    }
                    CommandMenuItem {
                        text: qsTrId("settings.title")
                        onTriggered: root.settingsRequested()
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
            font.family: text.length || inputMethodComposing ? Theme.userDataFontFamily : Theme.uiFontFamily
            placeholderText: qsTrId("ui.1cd2322e6ab6")
            maximumLength: 256
            enabled: root.canSearch
            onAccepted: root.searchRequested(text)
            Keys.onEscapePressed: {
                clear();
                root.searchRequested("");
            }
            color: Theme.text
            placeholderTextColor: Theme.textSecondary
            selectionColor: Theme.accent
            selectedTextColor: Theme.accentText
            Accessible.name: placeholderText
            background: Rectangle {
                color: Theme.canvas
                radius: Theme.mainToolbarRadius
                border.color: Theme.border
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
                Accessible.name: qsTrId("ui.b6f00afa6537")
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
                tooltipEnabled: false
                text: root.hasSelection ? qsTrId("ui.a1fca7bde5d2") : qsTrId("ui.0af5cbabf545")
                objectName: "archiveExtract"
                primary: true
                splitIndicator: true
                explanation: qsTrId("ui.96eaa0499c11")
                enabled: root.canSearch && root.canSelect
                appearanceEnabled: root.preserveActionAppearance || enabled
                onClicked: root.extractRequested(!root.hasSelection)
            }
            ToolButton {
                id: scopeButton
                objectName: "archiveExtractMenuButton"
                anchors.right: parent.right
                width: Theme.mainSplitWidth
                height: parent.height
                enabled: root.canSearch
                Accessible.name: qsTrId("ui.509e71a51467")
                contentItem: Item {}
                background: Rectangle {
                    radius: Theme.mainToolbarRadius
                    color: scopeButton.down ? "#30FFFFFF" : scopeButton.hovered ? "#18FFFFFF" : "transparent"
                    border.width: 0
                    border.color: Theme.accentInk
                }
                onClicked: scopes.open()
                CommandMenu {
                    id: scopes
                    objectName: "archiveExtractMenu"
                    x: parent.width - width
                    y: parent.height + Theme.mainPopupGap
                    CommandMenuItem {
                        text: qsTrId("ui.0af5cbabf545")
                        enabled: root.canSearch && root.canSelect
                        onTriggered: root.extractRequested(true)
                    }
                    CommandMenuItem {
                        text: qsTrId("ui.a1fca7bde5d2")
                        enabled: root.canSearch && root.canSelect && root.hasSelection
                        onTriggered: root.extractRequested(false)
                    }
                    CommandMenu {
                        objectName: "toolbarQuickMenu"
                        title: qsTrId("desktop.quickExtract")
                        enabled: root.canQuickExtract
                        CommandMenuItem {
                            objectName: "toolbarQuickCurrent"
                            text: qsTrId("desktop.quickExtractHere")
                            onTriggered: root.quickExtractRequested("here")
                        }
                        CommandMenuItem {
                            objectName: "toolbarQuickSameName"
                            text: qsTrId("desktop.quickExtractSameName").arg(root.archiveFolder)
                            userData: true
                            onTriggered: root.quickExtractRequested("sameName")
                        }
                        CommandMenuItem {
                            objectName: "toolbarQuickDesktop"
                            text: qsTrId("desktop.quickExtractDesktop")
                            onTriggered: root.quickExtractRequested("desktop")
                        }
                        CommandMenuItem {
                            objectName: "toolbarQuickDocuments"
                            text: qsTrId("desktop.quickExtractDocuments")
                            onTriggered: root.quickExtractRequested("documents")
                        }
                        CommandMenuItem {
                            objectName: "toolbarQuickDownloads"
                            text: qsTrId("desktop.quickExtractDownloads")
                            onTriggered: root.quickExtractRequested("downloads")
                        }
                    }
                    CommandMenuSeparator {}
                    CommandMenuItem {
                        text: qsTrId("ui.994e3891e9d5")
                        enabled: root.canInspect
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
