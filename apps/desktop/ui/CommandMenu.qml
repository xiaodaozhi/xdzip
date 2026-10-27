pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls

Menu {
    id: root
    // Keep the shared delegates on every platform; AppKit-native menus ignore
    // custom backgrounds and disabled-reason rows. Operation windows are separate.
    popupType: Popup.Item
    focus: true
    property Item returnFocusItem: null
    property Component footerComponent: null
    readonly property Item footerItem: footerLoader.item as Item
    property real measuredFooterHeight: 0
    readonly property real footerHeight: footerComponent ? measuredFooterHeight : 0
    readonly property real viewportScrollBarWidth: menuList.contentHeight > menuList.height && verticalScrollBar.visible ? verticalScrollBar.width : 0
    readonly property real naturalContentHeight: {
        let total = 0;
        for (let index = 0; index < count; ++index) {
            const item = itemAt(index);
            if (item) total += item.implicitHeight;
        }
        return total;
    }
    function measureFooter() {
        measuredFooterHeight = footerItem ? Math.ceil(footerItem.implicitHeight) : 0;
    }
    implicitWidth: Math.max(Theme.menuMinWidth, Theme.menuWidth)
    padding: Theme.menuPadding
    bottomPadding: padding + footerHeight
    margins: Theme.menuViewportInset
    font.pixelSize: Theme.menuLabelSize
    delegate: CommandMenuItem {}
    function focusBoundary(last) {
        for (let index = last ? count - 1 : 0; last ? index >= 0 : index < count; index += last ? -1 : 1) {
            const item = itemAt(index) as MenuItem;
            if (item && item.enabled && item.visible) {
                currentIndex = index;
                item.forceActiveFocus(Qt.TabFocusReason);
                return;
            }
        }
    }
    // Qt's menu owns focus, navigation, submenu delays and viewport placement.
    // Keep the shared presentation independent of the platform's default style.
    onAboutToShow: {
        if (parent && parent.Window.window)
            returnFocusItem = parent.Window.window.activeFocusItem;
    }
    onClosed: {
        if (returnFocusItem && returnFocusItem.Window.window && returnFocusItem.Window.window.active && returnFocusItem.visible && returnFocusItem.enabled)
            returnFocusItem.forceActiveFocus(Qt.PopupFocusReason);
    }
    background: Item {
        // Shared nine-slice pixels are ready with the menu's first frame.
        // Stretch only the straight edges, preserving the authored corner blur.
        BorderImage {
            id: shadow
            objectName: "menuShadowLayer-0"
            readonly property real inset: Theme.menuShadowRadius * 2
            readonly property int fontPercent: Math.round(Theme.fontScale * 100)
            x: -inset; y: -inset
            width: parent.width + inset * 2
            height: parent.height + inset * 2
            source: "qrc:/xdzip/assets/menu/" + (Theme.dark ? "dark" : "light") + "-" + fontPercent + ".png"
            border.left: inset + Math.round(Theme.menuRadius)
            border.top: border.left
            border.right: border.left
            border.bottom: border.left
            horizontalTileMode: BorderImage.Stretch
            verticalTileMode: BorderImage.Stretch
            asynchronous: false
            cache: true
        }
        Rectangle {
            anchors.fill: parent
            radius: Theme.menuRadius
            color: Theme.menuSurface
            border.color: Theme.menuBorder
        }
    }
    Connections {
        target: root.footerItem
        function onImplicitHeightChanged() { root.measureFooter(); }
    }
    contentItem: ListView {
        id: menuList
        objectName: "xdzipCommandMenuContent"
        focus: true
        implicitHeight: contentHeight
        model: root.contentModel
        currentIndex: root.currentIndex
        clip: true
        boundsBehavior: Flickable.StopAtBounds
        interactive: contentHeight > height
        ScrollBar.vertical: ScrollBar {
            id: verticalScrollBar
            objectName: "commandMenuScrollBar"
            policy: root.footerComponent ? (menuList.contentHeight > menuList.height ? ScrollBar.AlwaysOn : ScrollBar.AlwaysOff) : ScrollBar.AsNeeded
        }
        Loader {
            id: footerLoader
            // Keep the fixed action in the interactive popup layer, outside the clipped list.
            parent: menuList.parent
            active: root.footerComponent !== null
            sourceComponent: root.footerComponent
            x: root.leftPadding
            y: parent ? parent.height - root.padding - root.footerHeight : 0
            width: parent ? parent.width - root.leftPadding - root.rightPadding : 0
            height: root.footerHeight
            onLoaded: root.measureFooter()
        }
        Keys.onPressed: function (event) {
            if (event.key === Qt.Key_Tab && root.footerItem && root.footerItem.enabled) {
                if (root.footerItem.activeFocus && !(event.modifiers & Qt.ShiftModifier)) root.close();
                else root.footerItem.forceActiveFocus(Qt.TabFocusReason);
                event.accepted = true;
                return;
            }
            if (event.key === Qt.Key_Home || event.key === Qt.Key_End || event.key === Qt.Key_Up && root.currentIndex < 0) {
                root.focusBoundary(event.key !== Qt.Key_Home);
                event.accepted = true;
            }
        }
    }
}
