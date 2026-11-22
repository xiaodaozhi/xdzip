pragma ComponentBehavior: Bound
import QtQuick

MouseArea {
    id: root
    acceptedButtons: Qt.LeftButton | Qt.RightButton
    preventStealing: true
    property point pressedAt
    property bool transferStarted: false
    signal transferStartedAt(point position)
    signal transferMovedTo(point position)
    signal transferReleased()
    signal transferCancelled()
    signal contextRequested(point position)
    signal selectionRequested(int modifiers)
    signal activationRequested()

    onPressed: mouse => {
        root.pressedAt = Qt.point(mouse.x, mouse.y);
        root.transferStarted = false;
    }
    onPositionChanged: mouse => {
        const position = Qt.point(mouse.x, mouse.y);
        if (root.pressed && (root.pressedButtons & Qt.LeftButton) && !root.transferStarted
            && Math.hypot(mouse.x - root.pressedAt.x, mouse.y - root.pressedAt.y) >= Application.styleHints.startDragDistance) {
            root.transferStarted = true;
            root.transferStartedAt(position);
        }
        if (root.transferStarted)
            root.transferMovedTo(position);
    }
    onReleased: if (root.transferStarted) root.transferReleased()
    onCanceled: root.transferCancelled()
    onClicked: mouse => {
        if (root.transferStarted)
            return;
        if (mouse.button === Qt.RightButton)
            root.contextRequested(Qt.point(mouse.x, mouse.y));
        else
            root.selectionRequested(mouse.modifiers);
    }
    onDoubleClicked: root.activationRequested()
}
