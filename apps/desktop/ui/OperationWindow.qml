import QtQuick

Window {
    id: root
    required property Window owner
    property bool allowMinimize: false
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
    height: requestedHeight
    minimumWidth: width
    maximumWidth: width
    minimumHeight: requestedHeight
    maximumHeight: requestedHeight

    function present() {
        // Reopening preserves view state and uses the owner's current display.
        screen = owner.screen;
        x = Math.max(Screen.virtualX, Math.min(owner.x + (owner.width - width) / 2,
                     Screen.virtualX + Screen.width - width));
        y = Math.max(Screen.virtualY, Math.min(owner.y + (owner.height - height) / 2,
                     Screen.virtualY + Screen.height - height));
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
        enabled: root.visible && root.active
        onActivated: root.close()
    }
}
