pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls

ToolTip {
    id: root
    property bool requested: false
    readonly property Item focusItem: parent && parent.Window.window ? parent.Window.window.activeFocusItem : null
    readonly property bool eligible: parent && parent.visible && requested && text.length > 0 && press && !press.pressed && !(focusItem as MenuItem) && (!focusItem || focusItem.objectName !== "xdzipCommandMenuContent")
    property bool dwellReady: false
    property bool initialized: false
    property point pointerPosition: Qt.point(0, 0)
    property bool pointerPositionKnown: false
    visible: eligible && dwellReady
    // Built-in delay starts on entry; restart our dwell timer on movement within the owner.
    function restartDwell() {
        if (!initialized || !dwell) return;
        dwellReady = false;
        dwell.stop();
        if (eligible) dwell.start();
    }
    onEligibleChanged: restartDwell()
    onTextChanged: restartDwell()
    Component.onCompleted: { initialized = true; restartDwell(); }
    Component.onDestruction: initialized = false
    Timer {
        id: dwell
        interval: Theme.hoverDelayMs
        onTriggered: root.dwellReady = root.eligible
    }
    HoverHandler {
        id: pointer
        parent: root.parent
        target: null
        onPointChanged: {
            const position = point.scenePosition;
            // Layout/hover refreshes can repeat a point without physical movement.
            if (hovered && (!root.pointerPositionKnown || position.x !== root.pointerPosition.x || position.y !== root.pointerPosition.y)) {
                root.pointerPosition = position;
                root.pointerPositionKnown = true;
                root.restartDwell();
            }
        }
    }
    TapHandler {
        id: press
        parent: root.parent
        target: null
        acceptedButtons: Qt.AllButtons
        gesturePolicy: TapHandler.DragThreshold
    }
    onAboutToShow: {
        if (!parent) return;
        // A tooltip clamped over its owner causes a hover-leave/show loop near the top edge.
        const aboveFits = parent.mapToItem(null, 0, 0).y >= implicitHeight + Theme.menuGap + margins;
        y = aboveFits ? -implicitHeight - Theme.menuGap : parent.height + Theme.menuGap;
    }
    popupType: Popup.Item
    delay: 0
    timeout: Theme.tooltipDurationMs
    margins: Theme.menuViewportInset
    padding: Theme.tooltipPadding
    implicitWidth: Math.min(Theme.tooltipMaxWidth, contentItem.implicitWidth + padding * 2)
    implicitHeight: Math.min(Theme.treeTooltipMaxHeight * Theme.fontScale, contentItem.implicitHeight + padding * 2)
    contentItem: Text {
        text: root.text
        // Archive display names must remain text, including angle brackets.
        textFormat: Text.PlainText
        font.pixelSize: Theme.menuShortcutSize
        color: Theme.tooltipText
        wrapMode: Text.WrapAnywhere
        clip: true
    }
    background: Rectangle {
        color: Theme.tooltipSurface
        radius: Theme.menuRadius * 0.65
    }
}
