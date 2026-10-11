import QtQuick

Window {
    id: root
    required property Window owner
    property bool allowMinimize: false
    transientParent: owner
    modality: Qt.NonModal
    // Keep native decorations on every platform; never draw macOS traffic lights.
    flags: Qt.Window | Qt.WindowTitleHint | Qt.WindowSystemMenuHint | Qt.WindowCloseButtonHint
           | (allowMinimize ? Qt.WindowMinimizeButtonHint : 0)
    color: Theme.surface
    visible: false
    width: 760
    height: Math.min(720, Screen.desktopAvailableHeight - 80)
    minimumWidth: width
    maximumWidth: width
    minimumHeight: height
    maximumHeight: height

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
    Shortcut {
        sequence: "Escape"
        enabled: root.visible && root.active
        onActivated: root.close()
    }
}
