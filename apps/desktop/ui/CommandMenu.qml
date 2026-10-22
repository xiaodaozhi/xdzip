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
    implicitWidth: Math.max(Theme.menuMinWidth, Theme.menuWidth)
    padding: Theme.menuPadding
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
    contentItem: ListView {
        objectName: "xdzipCommandMenuContent"
        focus: true
        implicitHeight: contentHeight
        model: root.contentModel
        currentIndex: root.currentIndex
        clip: true
        boundsBehavior: Flickable.StopAtBounds
        interactive: contentHeight > height
        ScrollBar.vertical: ScrollBar {}
        Keys.onPressed: function (event) {
            if (event.key === Qt.Key_Home || event.key === Qt.Key_End || event.key === Qt.Key_Up && root.currentIndex < 0) {
                root.focusBoundary(event.key !== Qt.Key_Home);
                event.accepted = true;
            }
        }
    }
}
