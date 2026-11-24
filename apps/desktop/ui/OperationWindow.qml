import QtQuick
import Xdzip.Settings 1.0

Window {
    id: root
    default property alias operationData: body.data
    property Window owner: null
    property bool allowMinimize: false
    property bool escapeClosesWindow: true
    property bool centerOnScreen: false
    property int screenAvailableHeight: BuildInfo.availableScreenHeight(root)
    // Constraints depend on requested geometry, never the constrained height.
    property int clientWidth: Theme.operationDefaultWidth
    property int requestedHeight: Math.min(720, Screen.desktopAvailableHeight - 80)
    readonly property bool waylandFrame: linuxFrame.wayland
    readonly property int shadowInset: waylandFrame ? Theme.mainLinuxShadowInset : 0
    readonly property int frameCornerRadius: waylandFrame ? Theme.mainLinuxCornerRadius : 0
    readonly property int titlebarHeight: waylandFrame ? Theme.mainLinuxCaptionSize + 2 * Theme.mainLinuxCaptionRightInset : 0
    readonly property int frameHeight: 2 * shadowInset + titlebarHeight
    readonly property int surfaceWidth: clientWidth + 2 * shadowInset
    readonly property int surfaceHeight: waylandFrame
        ? Math.min(requestedHeight + frameHeight, Math.max(frameHeight + 1, screenAvailableHeight))
        : requestedHeight
    WindowInsets.frameInset: shadowInset
    transientParent: owner
    modality: Qt.NonModal
    flags: waylandFrame
           ? Qt.Dialog | Qt.FramelessWindowHint
           : Qt.Window | Qt.WindowTitleHint | Qt.WindowSystemMenuHint | Qt.WindowCloseButtonHint
             | (allowMinimize && Qt.platform.os !== "linux" ? Qt.WindowMinimizeButtonHint : 0)
    color: waylandFrame ? "transparent" : Theme.surface
    visible: false
    width: surfaceWidth
    minimumWidth: surfaceWidth
    maximumWidth: surfaceWidth

    // Widen the range before resizing so the native window never receives
    // inverted limits or a target outside its current constraints.
    function applyRequestedHeight() {
        const desired = surfaceHeight;
        if (minimumHeight > desired) minimumHeight = desired;
        if (maximumHeight < desired) maximumHeight = desired;
        if (height !== desired) height = desired;
        if (minimumHeight !== desired) minimumHeight = desired;
        if (maximumHeight !== desired) maximumHeight = desired;
    }
    Component.onCompleted: applyRequestedHeight()
    onSurfaceHeightChanged: Qt.callLater(applyRequestedHeight)
    onScreenChanged: screenAvailableHeight = BuildInfo.availableScreenHeight(root)
    // Cocoa can deliver an older resize after the content limits have changed.
    onHeightChanged: if (visible) {
        if (height !== surfaceHeight) Qt.callLater(applyRequestedHeight);
    }

    function present() {
        if (visible) {
            applyRequestedHeight();
            raise();
            requestActivate();
            return;
        }
        // Reopening preserves view state and uses the owner's current display.
        if (owner) screen = owner.screen;
        // Select the display before constraining content to its actual work area.
        if (centerOnScreen) BuildInfo.centerOnScreen(root, owner);
        screenAvailableHeight = BuildInfo.availableScreenHeight(root);
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
    data: [
        LinuxFrame {
            id: linuxFrame
            window: root
            inset: root.shadowInset
        },
        Item {
            width: root.width
            height: root.height
            BorderImage {
                anchors.fill: parent
                visible: root.waylandFrame
                source: visible ? "qrc:/xdzip/assets/frame/linux-" + (Theme.dark ? "dark" : "light") + ".png" : ""
                border.left: Theme.mainLinuxShadowInset + Theme.mainLinuxCornerRadius
                border.top: border.left
                border.right: border.left
                border.bottom: border.left
            }
            Item {
                anchors.fill: parent
                anchors.margins: root.shadowInset
                Rectangle {
                    anchors.fill: parent
                    anchors.topMargin: root.titlebarHeight - root.frameCornerRadius
                    visible: root.waylandFrame
                    radius: root.frameCornerRadius
                    color: Theme.surface
                    antialiasing: true
                }
                Item {
                    id: body
                    objectName: "operationClientArea"
                    anchors.fill: parent
                    anchors.topMargin: root.titlebarHeight
                }
                Rectangle {
                    objectName: "operationTitlebar"
                    anchors.left: parent.left
                    anchors.right: parent.right
                    height: root.titlebarHeight
                    visible: root.waylandFrame
                    radius: root.frameCornerRadius
                    color: Theme.canvas
                    antialiasing: true
                    // Match the main window's rounded top and square join to the body.
                    Rectangle {
                        anchors.bottom: parent.bottom
                        width: parent.width
                        height: root.frameCornerRadius
                        color: parent.color
                    }
                    Text {
                        anchors.centerIn: parent
                        width: Math.max(0, parent.width - 2 * (closeButton.width + Theme.mainLinuxCaptionRightInset))
                        horizontalAlignment: Text.AlignHCenter
                        text: root.title
                        textFormat: Text.PlainText
                        elide: Text.ElideRight
                        color: Theme.text
                        font.pixelSize: Math.round(Theme.menuLabelSize)
                        font.weight: Font.DemiBold
                    }
                    DragHandler {
                        target: null
                        acceptedButtons: Qt.LeftButton
                        onActiveChanged: if (active) root.startSystemMove()
                    }
                    WindowCaptionButton {
                        id: closeButton
                        objectName: "operationWindowClose"
                        anchors.right: parent.right
                        anchors.rightMargin: Theme.mainLinuxCaptionRightInset
                        height: parent.height
                        linuxStyle: true
                        captionAction: "close"
                        text: qsTrId("window.close")
                        onClicked: root.close()
                    }
                }
                Rectangle {
                    anchors.fill: parent
                    visible: root.waylandFrame
                    radius: root.frameCornerRadius
                    color: "transparent"
                    border.width: Theme.mainLinuxOutlineWidth
                    border.color: Theme.menuBorder
                    antialiasing: true
                }
            }
        },
        Shortcut {
            sequence: "Escape"
            enabled: root.visible && root.active && root.escapeClosesWindow
            onActivated: root.close()
        }
    ]
}
