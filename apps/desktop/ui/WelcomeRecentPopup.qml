pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import Xdzip.Settings 1.0

Popup {
    id: root
    objectName: "welcomeRecentMenu"
    required property Item focusReturn
    property var paths: []
    signal recentSelected(string path)
    signal clearRequested

    readonly property int pageSize: Theme.welcomeRecentPageSize
    readonly property bool hasMore: shownArchives.count < paths.length
    readonly property real rowHeight: Theme.menuRowHeight
    readonly property int contentRowCount: Math.max(1, shownArchives.count + (hasMore ? 1 : 0))
    readonly property real footerGap: Theme.welcomeRecentFooterGap
    readonly property real availableInset: Theme.menuViewportInset
        + (parent?.Window.window?.WindowInsets.frameInset ?? 0)
    property real viewportHeight: rowHeight
    property bool placeAbove: false
    property int focusedIndex: -1

    popupType: Popup.Item
    focus: true
    closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside
    padding: Theme.menuPadding
    margins: availableInset
    width: Math.max(parent.width, Theme.menuMinWidth, Theme.menuWidth)
    height: topPadding + viewportHeight + footerGap + rowHeight + bottomPadding
    x: parent.width - width
    y: placeAbove ? -height - Theme.mainPopupGap : parent.height + Theme.mainPopupGap

    ListModel { id: shownArchives }

    function appendPage() {
        const first = shownArchives.count;
        const end = Math.min(first + pageSize, paths.length);
        for (let index = first; index < end; ++index)
            shownArchives.append({archivePath: paths[index]});
    }

    function fitViewport() {
        const overlay = parent.Overlay.overlay;
        if (!overlay)
            return;
        const top = parent.mapToItem(overlay, 0, 0).y;
        const spaceAbove = top - availableInset - Theme.mainPopupGap;
        const spaceBelow = overlay.height - top - parent.height - availableInset - Theme.mainPopupGap;
        const fixedHeight = topPadding + footerGap + rowHeight + bottomPadding;
        const preferredHeight = Math.min(Theme.welcomeRecentPopupMaximumHeight * Theme.fontScale, fixedHeight + contentRowCount * rowHeight);
        placeAbove = spaceBelow < preferredHeight && spaceAbove > spaceBelow;
        const available = Math.min(preferredHeight, placeAbove ? spaceAbove : spaceBelow) - fixedHeight;
        viewportHeight = rowHeight * Math.max(1, Math.min(contentRowCount, Math.floor(available / rowHeight)));
    }

    function focusEntry(index) {
        const row = index === shownArchives.count && hasMore ? moreRow : entries.itemAt(index);
        if (!row || !row.enabled)
            return;
        const top = index * rowHeight;
        viewport.contentY = Math.min(Math.max(viewport.contentY, top + rowHeight - viewport.height), top);
        focusedIndex = index;
        row.forceActiveFocus(Qt.TabFocusReason);
    }

    function handleNavigation(event) {
        if (!paths.length)
            return;
        switch (event.key) {
        case Qt.Key_Home:
            focusEntry(0);
            break;
        case Qt.Key_End:
            focusEntry(contentRowCount - 1);
            break;
        case Qt.Key_Up:
            focusEntry(clearRow.activeFocus || focusedIndex <= 0 ? contentRowCount - 1 : focusedIndex - 1);
            break;
        case Qt.Key_Down:
            focusEntry(clearRow.activeFocus || focusedIndex >= contentRowCount - 1 ? 0 : focusedIndex + 1);
            break;
        case Qt.Key_Backtab:
            focusEntry(contentRowCount - 1);
            break;
        case Qt.Key_Tab:
            if (clearRow.activeFocus)
                close();
            else
                clearRow.forceActiveFocus(Qt.TabFocusReason);
            break;
        default:
            return;
        }
        event.accepted = true;
    }

    onAboutToShow: {
        shownArchives.clear();
        appendPage();
        fitViewport();
        focusedIndex = -1;
        viewport.cancelFlick();
        viewport.contentY = 0;
    }
    onOpened: navigationScope.forceActiveFocus(Qt.PopupFocusReason)
    onClosed: if (focusReturn.visible && focusReturn.enabled && focusReturn.Window.window?.active)
        focusReturn.forceActiveFocus(Qt.PopupFocusReason)
    onRowHeightChanged: if (visible)
        fitViewport()
    Connections {
        target: root.parent?.Window.window ?? null
        function onWidthChanged() { if (root.visible) root.fitViewport(); }
        function onHeightChanged() { if (root.visible) root.fitViewport(); }
    }

    background: CommandMenuBackground {}

    contentItem: FocusScope {
        id: navigationScope
        Accessible.role: Accessible.PopupMenu
        Accessible.name: qsTrId("desktop.recent")
        Keys.onPressed: event => root.handleNavigation(event)

        Flickable {
            id: viewport
            objectName: "welcomeRecentList"
            readonly property int count: root.contentRowCount
            width: parent.width
            height: root.viewportHeight
            contentWidth: width
            contentHeight: root.contentRowCount * root.rowHeight
            clip: true
            boundsBehavior: Flickable.StopAtBounds
            flickableDirection: Flickable.VerticalFlick
            interactive: contentHeight > height

            ScrollBar.vertical: ScrollBar {
                id: scrollBar
                objectName: "commandMenuScrollBar"
                policy: viewport.interactive ? ScrollBar.AlwaysOn : ScrollBar.AlwaysOff
            }

            Column {
                id: rows
                width: viewport.width
                CommandMenuItem {
                    objectName: "welcomeEmptyRecent"
                    width: rows.width
                    height: root.rowHeight
                    visible: !root.paths.length
                    enabled: false
                    text: qsTrId("desktop.emptyRecent")
                }
                Repeater {
                    id: entries
                    model: shownArchives
                    delegate: CommandMenuItem {
                        id: entry
                        required property string archivePath
                        required property int index
                        objectName: "welcomeRecentEntry-" + index
                        width: rows.width
                        height: root.rowHeight
                        text: archivePath.replace(/\\/g, "/").split("/").pop()
                        accessibleLabel: archivePath
                        userData: true
                        asset: "file-archive"
                        rightPadding: Theme.menuLabelInset + (viewport.interactive ? scrollBar.width : 0)
                        onActiveFocusChanged: if (activeFocus) root.focusedIndex = index
                        Keys.onReturnPressed: click()
                        Keys.onEnterPressed: click()
                        onClicked: {
                            root.close();
                            root.recentSelected(archivePath);
                        }
                        CommandTooltip {
                            requested: entry.enabled && entry.hovered && !entry.down
                            text: entry.archivePath
                            userData: true
                        }
                    }
                }
                CommandMenuItem {
                    id: moreRow
                    objectName: "welcomeLoadMore"
                    width: rows.width
                    height: root.rowHeight
                    visible: root.hasMore
                    text: qsTrId("welcome.recentLoadMore")
                    asset: "chevron-down"
                    onActiveFocusChanged: if (activeFocus) root.focusedIndex = shownArchives.count
                    Keys.onReturnPressed: click()
                    Keys.onEnterPressed: click()
                    onClicked: {
                        const first = shownArchives.count;
                        root.appendPage();
                        root.focusEntry(first);
                    }
                }
            }
        }

        MouseArea {
            anchors.fill: viewport
            acceptedButtons: Qt.NoButton
            onWheel: event => {
                event.accepted = viewport.interactive;
                if (!event.accepted)
                    return;
                const travel = event.pixelDelta.y || event.angleDelta.y * root.rowHeight / 40;
                viewport.cancelFlick();
                viewport.contentY = Math.max(0, Math.min(viewport.contentY - travel,
                    viewport.contentHeight - viewport.height));
            }
        }

        Rectangle {
            x: 4
            y: viewport.height + 2
            width: parent.width - 8
            height: 1
            color: Theme.menuBorder
        }

        CommandMenuItem {
            id: clearRow
            objectName: "welcomeClearRecent"
            y: viewport.height + root.footerGap
            height: root.rowHeight
            width: parent.width
            enabled: root.paths.length > 0
            text: qsTrId("welcome.recentClear")
            asset: "remove"
            Keys.onReturnPressed: click()
            Keys.onEnterPressed: click()
            onClicked: {
                root.clearRequested();
                root.close();
            }
        }
    }
}
