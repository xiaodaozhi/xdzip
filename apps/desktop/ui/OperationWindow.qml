import QtQuick
import Xdzip.Settings 1.0

Window {
    id: root
    property Window owner: null
    property bool allowMinimize: false
    property bool escapeClosesWindow: true
    property bool centerOnScreen: false
    // Constraints depend on requested geometry, never the constrained height.
    property int requestedHeight: Math.min(720, Screen.desktopAvailableHeight - 80)
    transientParent: owner
    modality: Qt.NonModal
    // Keep native decorations on every platform; never draw macOS traffic lights.
    flags: Qt.Window | Qt.WindowTitleHint | Qt.WindowSystemMenuHint | Qt.WindowCloseButtonHint
           | (allowMinimize ? Qt.WindowMinimizeButtonHint : 0)
    color: Theme.surface
    visible: false
    width: Theme.operationDefaultWidth
    minimumWidth: width
    maximumWidth: width

    // Widen the range before resizing so the native window never receives
    // inverted limits or a target outside its current constraints.
    function applyRequestedHeight() {
        const desired = requestedHeight;
        if (minimumHeight > desired) minimumHeight = desired;
        if (maximumHeight < desired) maximumHeight = desired;
        if (height !== desired) height = desired;
        if (minimumHeight !== desired) minimumHeight = desired;
        if (maximumHeight !== desired) maximumHeight = desired;
    }
    Component.onCompleted: applyRequestedHeight()
    onRequestedHeightChanged: Qt.callLater(applyRequestedHeight)
    // Cocoa can deliver an older resize after the content limits have changed.
    onHeightChanged: if (visible) {
        if (height !== requestedHeight) Qt.callLater(applyRequestedHeight);
        if (centerOnScreen) Qt.callLater(() => { if (root.visible) BuildInfo.centerOnScreen(root, root.owner || root); });
    }

    function present() {
        // Reopening preserves view state and uses the owner's current display.
        if (owner) screen = owner.screen;
        applyRequestedHeight();
        if (centerOnScreen) {
            BuildInfo.centerOnScreen(root, owner);
            show();
            BuildInfo.centerOnScreen(root, owner);
            raise();
            requestActivate();
            return;
        }
        const area = { x: Screen.virtualX, y: Screen.virtualY, width: Screen.desktopAvailableWidth, height: Screen.desktopAvailableHeight };
        const centerX = owner ? owner.x + owner.width / 2 : area.x + area.width / 2;
        const centerY = owner ? owner.y + owner.height / 2 : area.y + area.height / 2;
        x = Math.max(area.x, Math.min(centerX - width / 2, area.x + area.width - width));
        y = Math.max(area.y, Math.min(centerY - height / 2, area.y + area.height - height));
        show();
        raise();
        requestActivate();
    }
    function revealFocus(scroll: var) {
        const item = root.activeFocusItem;
        const flick = scroll.contentItem as Flickable;
        if (!item || !flick) return;
        let ancestor = item.parent;
        while (ancestor && ancestor !== scroll) ancestor = ancestor.parent;
        if (ancestor !== scroll) return;
        const position = item.mapToItem(flick.contentItem,0,0);
        const top = position.y - 4;
        const bottom = position.y + item.height + 4;
        if (top < flick.contentY) flick.contentY = Math.max(0,top);
        else if (bottom > flick.contentY + flick.height)
            flick.contentY = Math.min(flick.contentHeight - flick.height,bottom - flick.height);
    }
    Shortcut {
        sequence: "Escape"
        enabled: root.visible && root.active && root.escapeClosesWindow
        onActivated: root.close()
    }
}
