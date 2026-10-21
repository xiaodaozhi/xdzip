// apps/desktop/ui/ArchiveTabs.qml

pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQml.Models

Rectangle {
    id: root
    objectName: "archiveTitlebar"
    required property string archiveTitle
    property var tabs: []
    property bool sessionControls: false
    property bool integratedTitlebar: false
    property bool windowsControls: false
    property bool maximized: false
    property bool canOpen: true
    property bool canClose: false
    signal tabSelected(string identity)
    signal tabClosed(string identity)
    signal tabMoved(string identity, int index)
    signal settingsRequested()
    signal dragRequested()
    signal openRequested()
    signal closeRequested()
    signal minimizeRequested()
    signal maximizeRequested()
    signal windowCloseRequested()
    readonly property var displayedTabs: tabs.length ? tabs : [{id: "", title: archiveTitle, selected: true}]
    readonly property int tabCount: displayedTabs.length
    readonly property real capsuleHeight: Math.max(Theme.mainTabHeight, Math.ceil(Theme.mainTabSize * 1.6 + 8))
    readonly property real preferredTabWidth: Theme.mainTabWidth * Math.min(1.5, Theme.fontScale)
    readonly property real minimumTabWidth: Theme.mainTabMinimumWidth * Math.min(1.5, Theme.fontScale)
    readonly property real tabStartX: integratedTitlebar ? Theme.mainTabFirstMacX : Math.max(Theme.mainTabFirstOtherX, brand.x + brand.width + Theme.mainUnits(24))
    readonly property real availableTabWidth: Math.max(0, settingsButton.x - tabStartX - newTab.width - Theme.mainTabGap * 2)
    readonly property bool overflowing: tabCount * minimumTabWidth + (tabCount - 1) * Theme.mainTabGap > availableTabWidth
    readonly property real viewportWidth: Math.max(0, availableTabWidth - (overflowing ? overflowControls.width + Theme.mainTabGap : 0))
    readonly property real capsuleWidth: Math.max(minimumTabWidth, Math.min(preferredTabWidth, (viewportWidth - (tabCount - 1) * Theme.mainTabGap) / tabCount))
    readonly property real totalTabWidth: tabCount * capsuleWidth + (tabCount - 1) * Theme.mainTabGap
    property string draggedId: ""
    property string draggedTitle: ""
    property bool draggedSelected: false
    property int dragFrom: -1
    property int dropIndex: -1
    property real dragOffset: 0
    property real dragPointerX: 0
    property real dragPointerY: 0
    property real dragX: 0
    readonly property bool reordering: draggedId.length > 0
    ListModel { id: tabModel }
    function syncTabs() {
        if (!tabModel) return;
        for (let index = tabModel.count - 1; index >= 0; --index)
            if (!displayedTabs.some(tab => tab.id === tabModel.get(index).identity)) tabModel.remove(index);
        for (let index = 0; index < displayedTabs.length; ++index) {
            const tab = displayedTabs[index];
            let existing = -1;
            for (let row = 0; row < tabModel.count; ++row)
                if (tabModel.get(row).identity === tab.id) { existing = row; break; }
            if (existing < 0) tabModel.insert(index, {identity:tab.id,title:tab.title,selected:tab.selected});
            else {
                if (existing !== index) tabModel.move(existing,index,1);
                tabModel.setProperty(index,"title",tab.title);
                tabModel.setProperty(index,"selected",tab.selected);
            }
        }
        revealTimer.restart();
    }
    function labelFor(tab) { return tab.title.length ? tab.title : qsTrId("ui.8f1009f1369b"); }
    function selectTab(identity) { if (root.sessionControls) root.tabSelected(identity); }
    function closeTab(identity) { if (root.sessionControls) root.tabClosed(identity); else if (root.canClose) root.closeRequested(); }
    function selectRelative(step) {
        const index = tabs.findIndex(t => t.selected);
        if (tabs.length > 1) tabSelected(tabs[(index + step + tabs.length) % tabs.length].id);
    }
    function moveRelative(step) {
        const index = tabs.findIndex(t => t.selected);
        if (index >= 0 && index + step >= 0 && index + step < tabs.length)
            tabMoved(tabs[index].id, index + step);
    }
    function revealActive() {
        if (!tabStrip || reordering) return;
        const index = displayedTabs.findIndex(t => t.selected);
        if (index >= 0) tabStrip.positionViewAtIndex(index, ListView.Contain);
    }
    function scrollTabs(delta) {
        revealTimer.stop();
        tabStrip.cancelFlick();
        const origin = tabStrip.originX;
        tabStrip.contentX = Math.max(origin, Math.min(origin + Math.max(0, totalTabWidth - tabStrip.width), tabStrip.contentX + delta));
    }
    function beginReorder(tab, index, offset, x, y) {
        if (!sessionControls || tabs.length < 2) return;
        draggedTitle = labelFor(tab);
        draggedSelected = tab.selected;
        dragFrom = index;
        dropIndex = index;
        dragOffset = offset;
        draggedId = tab.id;
        updateReorder(x, y);
    }
    function updateReorder(x, y) {
        if (!reordering) return;
        dragPointerX = x;
        dragPointerY = y;
        dragX = Math.max(tabStrip.x, Math.min(tabStrip.x + tabStrip.width - capsuleWidth, x - dragOffset));
        const center = x - dragOffset - tabStrip.x + tabStrip.contentX - tabStrip.originX + capsuleWidth / 2;
        dropIndex = Math.max(0, Math.min(tabCount - 1, Math.floor(center / (capsuleWidth + Theme.mainTabGap))));
    }
    function cancelReorder() { draggedId = ""; dragFrom = -1; dropIndex = -1; }
    function finishReorder() {
        const identity = draggedId, destination = dropIndex, source = dragFrom;
        const inside = dragPointerY >= 0 && dragPointerY <= height && dragPointerX >= 0 && dragPointerX <= width;
        cancelReorder();
        if (inside && identity.length && destination !== source) tabMoved(identity, destination);
    }
    onDisplayedTabsChanged: { cancelReorder(); if (tabList) tabList.close(); syncTabs(); }
    Component.onCompleted: syncTabs()
    onWidthChanged: { cancelReorder(); if (tabList) tabList.close(); revealTimer.restart(); }
    onCapsuleWidthChanged: revealTimer.restart()
    Timer { id: revealTimer; interval: 0; onTriggered: root.revealActive() }
    Shortcut { sequences: [StandardKey.NextChild]; enabled: root.visible && root.tabs.length > 1; onActivated: root.selectRelative(1) }
    Shortcut { sequences: [StandardKey.PreviousChild]; enabled: root.visible && root.tabs.length > 1; onActivated: root.selectRelative(-1) }
    Shortcut { sequence: "Ctrl+Shift+PgUp"; enabled: root.visible && root.tabs.length > 1; onActivated: root.moveRelative(-1) }
    Shortcut { sequence: "Ctrl+Shift+PgDown"; enabled: root.visible && root.tabs.length > 1; onActivated: root.moveRelative(1) }
    Shortcut { sequence: "Escape"; enabled: root.visible && root.reordering; onActivated: root.cancelReorder() }
    color: Theme.canvas
    implicitHeight: Math.max(Theme.mainTitleHeight, capsuleHeight + 2 * Theme.mainTabTopInset)
    Item {
        anchors.fill: parent
        visible: !root.windowsControls
        DragHandler { target: null; onActiveChanged: if (active) root.dragRequested() }
    }
    MouseArea {
        anchors.fill: parent
        enabled: root.windowsControls
        acceptedButtons: Qt.LeftButton
        DragHandler { target: null; onActiveChanged: if (active) root.dragRequested() }
        onDoubleClicked: if (root.windowsControls) root.maximizeRequested()
    }
    RowLayout {
        id: brand
        x: root.integratedTitlebar ? Theme.mainBrandMacX : Theme.mainUnits(20)
        height: root.height
        spacing: Theme.mainUnits(8)
        Image {
            source: "qrc:/xdzip/assets/" + (Theme.dark ? "dark/" : "light/") + "app-mark.png"
            Layout.preferredWidth: Theme.mainUnits(30); Layout.preferredHeight: Theme.mainUnits(30)
            fillMode: Image.PreserveAspectFit
        }
        Text { text: "xdzip"; font.pixelSize: Theme.mainBrandSize; font.weight: Font.DemiBold; color: Theme.text }
    }
    Rectangle { anchors.bottom: parent.bottom; z: 1; width: parent.width; height: 1; color: Theme.border }
    ListView {
        id: tabStrip
        objectName: "archiveTabStrip"
        x: root.tabStartX
        y: (root.height - height) / 2
        height: root.capsuleHeight
        width: Math.min(root.totalTabWidth, root.viewportWidth)
        contentWidth: root.totalTabWidth
        clip: true
        orientation: ListView.Horizontal
        boundsBehavior: Flickable.StopAtBounds
        flickableDirection: Flickable.HorizontalFlick
        spacing: Theme.mainTabGap
        model: tabModel
        cacheBuffer: 0
        Accessible.role: Accessible.PageTabList
        Accessible.name: qsTrId("tabs.list")
        WheelHandler {
            target: null
            onWheel: event => {
                if (!root.overflowing) { event.accepted = false; return; }
                const pixels = event.pixelDelta.x || event.pixelDelta.y;
                const angle = event.angleDelta.x || event.angleDelta.y;
                root.scrollTabs(pixels ? -pixels : -angle / 120 * root.capsuleWidth);
                event.accepted = true;
            }
        }
        delegate: Rectangle {
            id: capsule
            required property string identity
            required property string title
            required property bool selected
            readonly property var modelData: ({id:identity,title:title,selected:selected})
            required property int index
            objectName: modelData.selected ? "activeArchiveTab" : "archiveTab-" + index
            width: root.capsuleWidth; height: root.capsuleHeight
            opacity: root.draggedId === modelData.id && root.reordering ? 0.35 : 1
            radius: height / 2
            color: pointer.pressed && !pointer.reordering ? Theme.controlPressed : modelData.selected ? Theme.surface : hover.hovered ? Theme.controlHover : "transparent"
            border.color: modelData.selected ? Theme.border : "transparent"
            Accessible.role: Accessible.PageTab
            Accessible.name: root.labelFor(modelData)
            Accessible.description: qsTrId("tabs.reorderHint")
            Accessible.selected: modelData.selected
            Accessible.onPressAction: root.selectTab(capsule.modelData.id)
            activeFocusOnTab: true
            Keys.onReturnPressed: root.selectTab(capsule.modelData.id)
            Keys.onSpacePressed: root.selectTab(capsule.modelData.id)
            Keys.onLeftPressed: if (root.tabs.length > 1) root.tabSelected(root.tabs[(index - 1 + root.tabs.length) % root.tabs.length].id)
            Keys.onRightPressed: if (root.tabs.length > 1) root.tabSelected(root.tabs[(index + 1) % root.tabs.length].id)
            HoverHandler { id: hover }
            MouseArea {
                id: pointer
                anchors.fill: parent
                acceptedButtons: Qt.LeftButton | Qt.MiddleButton
                preventStealing: true
                property real pressX: 0
                property real pressOffset: 0
                property bool reordering: false
                onPressed: mouse => { pressX = mapToItem(root, mouse.x, mouse.y).x; pressOffset = mouse.x; reordering = false; }
                onPositionChanged: mouse => {
                    if (!pressed || pressedButtons !== Qt.LeftButton) return;
                    const point = mapToItem(root, mouse.x, mouse.y);
                    if (!reordering && Math.abs(point.x - pressX) >= Qt.styleHints.startDragDistance && root.sessionControls && root.tabs.length > 1) {
                        reordering = true;
                        root.beginReorder(capsule.modelData, capsule.index, pressOffset, point.x, point.y);
                    }
                    if (reordering) root.updateReorder(point.x, point.y);
                }
                onReleased: if (reordering && root.reordering) root.finishReorder()
                onCanceled: root.cancelReorder()
                onClicked: mouse => {
                    if (reordering) return;
                    if (mouse.button === Qt.MiddleButton) root.closeTab(capsule.modelData.id);
                    else root.selectTab(capsule.modelData.id);
                }
            }
            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: Theme.mainUnits(15); anchors.rightMargin: Theme.mainUnits(9)
                spacing: Theme.mainTabTextGap
                Image {
                    source: "qrc:/xdzip/assets/" + (Theme.dark ? "dark/" : "light/") + (capsule.modelData.selected ? "archive.png" : "archive-grey.png")
                    Layout.preferredWidth: Theme.mainTabIconSize; Layout.preferredHeight: Theme.mainTabIconSize
                    fillMode: Image.PreserveAspectFit
                }
                Text {
                    objectName: "archiveTabLabel-" + capsule.index
                    Layout.fillWidth: true
                    text: root.labelFor(capsule.modelData)
                    textFormat: Text.PlainText
                    elide: Text.ElideRight
                    font.pixelSize: Theme.mainTabSize
                    font.weight: capsule.modelData.selected ? Theme.mainTabActiveWeight : Theme.mainTabInactiveWeight
                    color: capsule.modelData.selected ? Theme.text : Theme.textSecondary
                }
                Item {
                    Layout.preferredWidth: Theme.mainTabCloseSize; Layout.preferredHeight: Theme.mainTabCloseSize
                    ArchiveAction {
                        objectName: "archiveTabClose"
                        anchors.fill: parent
                        visible: (root.sessionControls || root.archiveTitle.length > 0) && (capsule.modelData.selected || hover.hovered || capsule.activeFocus)
                        leftPadding: 0; rightPadding: 0
                        glyphSize: Theme.mainUnits(20); asset: "close"
                        enabled: root.sessionControls || root.canClose
                        Accessible.name: qsTrId("tabs.close").arg(root.labelFor(capsule.modelData))
                        explanation: Accessible.name
                        onClicked: root.closeTab(capsule.modelData.id)
                    }
                }
            }
        }
    }
    ArchiveAction {
        id: newTab
        objectName: "archiveNewTab"
        x: tabStrip.x + tabStrip.width + Theme.mainTabGap
        anchors.verticalCenter: parent.verticalCenter
        width: Theme.mainUnits(36); height: Theme.mainUnits(32)
        glyphSize: Theme.mainUnits(26); asset: "plus"
        enabled: root.canOpen
        Accessible.name: root.sessionControls ? qsTrId("ui.e5ab37895edd") : qsTrId("ui.1b59353f1874")
        explanation: qsTrId("ui.e5ab37895edd")
        onClicked: root.openRequested()
    }
    Row {
        id: overflowControls
        x: newTab.x + newTab.width + Theme.mainTabGap
        anchors.verticalCenter: parent.verticalCenter
        visible: root.overflowing
        spacing: Theme.mainTabControlGap
        ArchiveAction {
            objectName: "archiveTabsPrevious"
            width: Theme.mainTabControlSize; height: width
            asset: "chevron-right"; glyphMirrored: true; enabled: tabStrip.contentX > tabStrip.originX + 0.5
            autoRepeat: true
            Accessible.name: qsTrId("tabs.previous")
            explanation: Accessible.name
            onClicked: root.scrollTabs(-root.capsuleWidth - Theme.mainTabGap)
        }
        ArchiveAction {
            objectName: "archiveTabsNext"
            width: Theme.mainTabControlSize; height: width
            asset: "chevron-right"; enabled: tabStrip.contentX + tabStrip.width < tabStrip.originX + root.totalTabWidth - 0.5
            autoRepeat: true
            Accessible.name: qsTrId("tabs.next")
            explanation: Accessible.name
            onClicked: root.scrollTabs(root.capsuleWidth + Theme.mainTabGap)
        }
        ArchiveAction {
            id: listButton
            objectName: "archiveTabList"
            width: Theme.mainTabControlSize; height: width
            asset: "chevron-down"
            Accessible.name: qsTrId("tabs.list")
            explanation: Accessible.name
            onClicked: tabList.open()
            CommandMenu {
                id: tabList
                objectName: "archiveTabListMenu"
                x: listButton.width - width
                y: listButton.height + Theme.mainPopupGap
                width: Math.min(Theme.menuWidth, root.width - Theme.menuViewportInset * 2)
                height: Math.min(contentItem.implicitHeight + padding * 2, root.Window.height - root.height - Theme.menuViewportInset * 2)
                Instantiator {
                    model: root.tabs
                    delegate: CommandMenuItem {
                        required property var modelData
                        objectName: "archiveTabListItem-" + modelData.id
                        text: root.labelFor(modelData)
                        checkable: true
                        checked: modelData.selected
                        onTriggered: root.tabSelected(modelData.id)
                    }
                    onObjectAdded: (index, object) => tabList.insertItem(index, object)
                    onObjectRemoved: (index, object) => tabList.removeItem(object)
                }
            }
        }
    }
    Rectangle {
        objectName: "archiveTabDragPreview"
        visible: root.reordering
        z: 10
        x: root.dragX; y: tabStrip.y
        width: root.capsuleWidth; height: root.capsuleHeight
        radius: height / 2; color: Theme.surface; border.color: Theme.accent
        Accessible.ignored: true
        RowLayout {
            anchors.fill: parent; anchors.leftMargin: Theme.mainUnits(15); anchors.rightMargin: Theme.mainUnits(15)
            spacing: Theme.mainTabTextGap
            Image {
                source: "qrc:/xdzip/assets/" + (Theme.dark ? "dark/" : "light/") + "archive.png"
                Layout.preferredWidth: Theme.mainTabIconSize; Layout.preferredHeight: Theme.mainTabIconSize
                fillMode: Image.PreserveAspectFit
            }
            Text {
                Layout.fillWidth: true; text: root.draggedTitle; textFormat: Text.PlainText; elide: Text.ElideRight
                font.pixelSize: Theme.mainTabSize; font.weight: root.draggedSelected ? Theme.mainTabActiveWeight : Theme.mainTabInactiveWeight
                color: Theme.text
            }
        }
    }
    Rectangle {
        objectName: "archiveTabDropIndicator"
        visible: root.reordering && root.dropIndex !== root.dragFrom
        z: 11
        x: Math.max(tabStrip.x, Math.min(tabStrip.x + tabStrip.width - width,
            tabStrip.x + (root.dropIndex + (root.dropIndex > root.dragFrom ? 1 : 0)) * (root.capsuleWidth + Theme.mainTabGap) - tabStrip.contentX + tabStrip.originX - Theme.mainTabGap / 2))
        y: tabStrip.y
        width: Theme.mainTabDropIndicatorWidth; height: root.capsuleHeight
        radius: width / 2; color: Theme.accent
    }
    Timer {
        interval: Theme.mainTabDragScrollInterval; repeat: true; running: root.reordering && root.overflowing
        onTriggered: {
            const left = tabStrip.x, right = left + tabStrip.width;
            if (root.dragPointerX < left + Theme.mainTabDragEdge) root.scrollTabs(-Theme.mainTabDragScrollStep);
            else if (root.dragPointerX > right - Theme.mainTabDragEdge) root.scrollTabs(Theme.mainTabDragScrollStep);
            root.updateReorder(root.dragPointerX, root.dragPointerY);
        }
    }
    ArchiveAction {
        id: settingsButton
        objectName: "archiveSettings"
        anchors.right: parent.right
        anchors.rightMargin: Theme.mainSettingsRightInset + (root.windowsControls ? captionControls.width : 0)
        anchors.verticalCenter: parent.verticalCenter
        width: Theme.mainSettingsSize; height: Theme.mainSettingsSize
        glyphSize: Theme.mainToolbarIconSize; asset: "settings"
        Accessible.name: qsTrId("settings.title"); explanation: Accessible.name
        onClicked: root.settingsRequested()
    }
    Row {
        id: captionControls
        objectName: "windowCaptionControls"
        anchors.right: parent.right
        height: root.height
        visible: root.windowsControls
        WindowCaptionButton {
            objectName: "windowMinimize"; height: captionControls.height; captionAction: "minimize"
            text: qsTrId("window.minimize"); onClicked: root.minimizeRequested()
        }
        WindowCaptionButton {
            objectName: "windowMaximize"; height: captionControls.height; captionAction: root.maximized ? "restore" : "maximize"
            text: root.maximized ? qsTrId("window.restore") : qsTrId("window.maximize"); onClicked: root.maximizeRequested()
        }
        WindowCaptionButton {
            objectName: "windowClose"; height: captionControls.height; captionAction: "close"
            text: qsTrId("window.close"); onClicked: root.windowCloseRequested()
        }
    }
}
