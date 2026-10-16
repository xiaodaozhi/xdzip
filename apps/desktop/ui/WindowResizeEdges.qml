pragma ComponentBehavior: Bound
import QtQuick

Item {
    id: root
    required property Window window
    anchors.fill: parent
    z: 100
    Repeater {
        model: [Qt.LeftEdge, Qt.RightEdge, Qt.TopEdge, Qt.BottomEdge,
                Qt.LeftEdge | Qt.TopEdge, Qt.RightEdge | Qt.TopEdge,
                Qt.LeftEdge | Qt.BottomEdge, Qt.RightEdge | Qt.BottomEdge]
        MouseArea {
            required property int modelData
            readonly property bool nearLeft: (modelData & Qt.LeftEdge) !== 0
            readonly property bool nearRight: (modelData & Qt.RightEdge) !== 0
            readonly property bool nearTop: (modelData & Qt.TopEdge) !== 0
            readonly property bool nearBottom: (modelData & Qt.BottomEdge) !== 0
            readonly property real edge: Theme.mainResizeBorderWidth
            x: nearRight ? root.width - width : 0
            y: nearBottom ? root.height - height : 0
            width: nearLeft || nearRight ? edge : root.width
            height: nearTop || nearBottom ? edge : root.height
            cursorShape: (nearLeft || nearRight) && (nearTop || nearBottom) ? (nearLeft === nearTop ? Qt.SizeFDiagCursor : Qt.SizeBDiagCursor)
                                                                        : nearLeft || nearRight ? Qt.SizeHorCursor : Qt.SizeVerCursor
            onPressed: mouse => { mouse.accepted = root.window.startSystemResize(modelData); }
        }
    }
}
