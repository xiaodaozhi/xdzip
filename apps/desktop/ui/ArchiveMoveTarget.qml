// apps/desktop/ui/ArchiveMoveTarget.qml

pragma ComponentBehavior: Bound
import QtQuick

DropArea {
    id: target
    required property var controller
    required property var dragSource
    required property string directoryId
    property var dropState: ({permitted: false, message: ""})
    keys: ["xdzip-internal-move"]
    HoverHandler { enabled: target.containsDrag; cursorShape: target.dropState.permitted ? Qt.DragMoveCursor : Qt.ForbiddenCursor }
    function inspect(drag) {
        if (drag.source !== dragSource) { drag.accepted = false; return; }
        dropState = controller.moveDropState(dragSource.token, directoryId);
        dragSource.feedback(target, directoryId, dropState);
        drag.accept(dropState.permitted ? Qt.MoveAction : Qt.IgnoreAction);
    }
    onEntered: drag => inspect(drag)
    onPositionChanged: drag => inspect(drag)
    onExited: dragSource.clearFeedback(target)
    onDropped: drop => {
        dropState = controller.moveDropState(dragSource.token, directoryId);
        if (drop.source === dragSource && dropState.permitted) {
            const token = dragSource.token;
            drop.accept(Qt.MoveAction);
            controller.moveInternalDrag(token, directoryId);
        } else drop.accepted = false;
        dragSource.clearFeedback(target);
    }
}
