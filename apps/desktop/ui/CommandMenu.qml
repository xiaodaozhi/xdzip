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
        // Rasterize one blurred rounded silhouette. Stacked expanded rectangles
        // produce visible bands and disproportionately dark corners.
        Canvas {
            id: shadow
            objectName: "menuShadowLayer-0"
            readonly property real inset: Theme.menuShadowRadius * 2
            x: -inset; y: -inset
            width: parent.width + inset * 2
            height: parent.height + inset * 2
            renderTarget: Canvas.Image
            onWidthChanged: requestPaint()
            onHeightChanged: requestPaint()
            Connections {
                target: Theme
                function onDarkChanged() { shadow.requestPaint(); }
                function onFontScaleChanged() { shadow.requestPaint(); }
            }
            onPaint: {
                const ctx = getContext("2d");
                ctx.reset();
                ctx.clearRect(0, 0, width, height);
                ctx.shadowColor = Theme.menuShadow;
                ctx.shadowBlur = Theme.menuShadowRadius * 2;
                ctx.shadowOffsetY = Theme.menuShadowOffsetY;
                ctx.fillStyle = Theme.menuSurface;
                const left = inset, top = inset;
                const right = width - inset, bottom = height - inset;
                const radius = Theme.menuRadius;
                ctx.beginPath();
                ctx.moveTo(left + radius, top);
                ctx.lineTo(right - radius, top);
                ctx.quadraticCurveTo(right, top, right, top + radius);
                ctx.lineTo(right, bottom - radius);
                ctx.quadraticCurveTo(right, bottom, right - radius, bottom);
                ctx.lineTo(left + radius, bottom);
                ctx.quadraticCurveTo(left, bottom, left, bottom - radius);
                ctx.lineTo(left, top + radius);
                ctx.quadraticCurveTo(left, top, left + radius, top);
                ctx.closePath();
                ctx.fill();
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
