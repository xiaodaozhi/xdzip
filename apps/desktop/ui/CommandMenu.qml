pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls

Menu {
    id: root
    // Keep the shared delegates on every platform; AppKit-native menus ignore
    // custom backgrounds and disabled-reason rows. Operation windows are separate.
    popupType: Popup.Item
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
        // Soft rounded layers work on the software renderer as well as the GPU.
        Repeater {
            model: Theme.menuShadowLayers
            Rectangle {
                required property int index
                readonly property real spread: Theme.menuShadowRadius * (1 - index / Theme.menuShadowLayers)
                objectName: "menuShadowLayer-" + index
                x: -spread
                y: Theme.menuShadowOffsetY - spread
                width: parent.width + spread * 2
                height: parent.height + spread * 2
                radius: Theme.menuRadius + spread
                color: Theme.menuShadow
                opacity: 1 / Theme.menuShadowLayers
            }
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
